use super::*;

/// Execute one entity's move as part of an enclosing [`execute_move_set`]
/// operation.
///
/// Unlike [`execute_move`], this does not acquire or release the per-entity
/// lock files: the caller already holds the union of lock paths for the
/// whole set for the lifetime of the set operation.
pub(super) fn execute_move_within_set<D: MoveDomain + ?Sized>(
    domain: &D,
    plan: &MovePlan,
    journal_id: Uuid,
    existing: Option<MoveJournal>,
    pre_migrated_board_entries: Option<Vec<BoardEntry>>,
    resumed: bool,
) -> MoveResult<MoveOutcome> {
    if !plan.supported() {
        return Err(MoveError::Domain(
            "move preflight contains blockers".to_string(),
        ));
    }
    execute_or_resume(
        domain,
        plan,
        Some(existing.unwrap_or_else(|| journal_from_plan(plan, journal_id))),
        pre_migrated_board_entries,
        resumed,
        true,
        true,
        true,
    )
}

pub(super) fn validate_move_set_outcomes<D: MoveDomain + ?Sized>(
    domain: &D,
    outcomes: &mut [MoveOutcome],
) -> MoveResult<()> {
    let entity_ids = outcomes
        .iter()
        .map(|outcome| outcome.journal.entity_id)
        .collect::<Vec<_>>();
    let source_seen =
        domain.entity_indexed_in_many(&outcomes[0].journal.source_store_root, &entity_ids)?;
    let target_seen =
        domain.entity_indexed_in_many(&outcomes[0].journal.target_store_root, &entity_ids)?;

    for outcome in outcomes {
        let journal = &mut outcome.journal;
        let source_path_exists = journal.source_entity_path.exists();
        let destination_path_exists = journal.destination_entity_path.exists();
        let source_indexed = source_seen
            .get(&journal.entity_id)
            .copied()
            .unwrap_or(false);
        let target_indexed = target_seen
            .get(&journal.entity_id)
            .copied()
            .unwrap_or(false);
        if source_path_exists || !destination_path_exists {
            return Err(build_post_move_validation_error(
                journal,
                source_path_exists,
                destination_path_exists,
                source_indexed,
                target_indexed,
            ));
        }
        journal.phase = MoveExecutionPhase::Validated;
        journal.updated_at = Utc::now();
        journal
            .steps
            .push("validated move ownership in set".to_string());
        journal.failure = None;
        journal.next_recovery_step = None;
        persist_journal(&journal.source_store_root, journal)?;
    }
    Ok(())
}

/// Execute a supported set of moves, holding the union of source/target store
/// locks for the lifetime of the whole set operation rather than acquiring
/// and releasing them once per entity (finding F1/F7's lock-cycling cost).
///
/// A [`MoveSetJournal`] stores the immutable normalized plan and stable
/// per-entity journal identities before mutation. Recovery therefore resumes
/// only unfinished entries from that durable record, without preflight.
pub fn execute_move_set<D: MoveDomain + ?Sized>(
    domain: &D,
    set_plan: &MoveSetPlan,
) -> MoveResult<MoveSetOutcome> {
    let _span_guard = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_execute_set",
        entity_count = set_plan.entity_ids.len(),
        supported = set_plan.supported(),
    )
    .entered();
    if !set_plan.supported() {
        return Err(MoveError::Domain(
            "move set preflight contains blockers".to_string(),
        ));
    }

    let mut lock_paths = BTreeSet::new();
    for plan in &set_plan.entity_plans {
        lock_paths.extend(collect_lock_paths(
            plan.entity_id,
            &plan.source_store_root,
            &plan.target_store_root,
        ));
    }
    let mut journal = MoveSetJournal {
        id: Uuid::new_v4(),
        entity_ids: set_plan.entity_ids.clone(),
        source_store_root: set_plan.source_store_root.clone(),
        target_store_root: set_plan.target_store_root.clone(),
        entity_plans: set_plan.entity_plans.clone(),
        entity_journal_ids: set_plan
            .entity_ids
            .iter()
            .map(|id| (*id, Uuid::new_v4()))
            .collect(),
        lock_paths: lock_paths.into_iter().collect(),
        phase: MoveSetExecutionPhase::Planned,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        completed_entity_ids: Vec::new(),
        rollback_completed_entity_ids: Vec::new(),
        entity_errors: std::collections::BTreeMap::new(),
        failure: None,
        next_recovery_step: None,
    };
    persist_move_set_journal(&journal.source_store_root, &journal)?;
    execute_move_set_journal(domain, &mut journal, false)
}

/// Resume a set operation from its immutable persisted plan without re-running
/// [`plan_move`] or [`plan_move_set`].
pub fn resume_move_set<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
) -> MoveResult<MoveSetOutcome> {
    let mut journal = load_move_set_journal(&domain.source_store_root(), journal_id)?;
    execute_move_set_journal(domain, &mut journal, true)
}

pub(super) fn execute_move_set_journal<D: MoveDomain + ?Sized>(
    domain: &D,
    journal: &mut MoveSetJournal,
    resumed: bool,
) -> MoveResult<MoveSetOutcome> {
    if !domain.target_store_present(&journal.target_store_root)? {
        domain.initialize_target_store(&journal.target_store_root)?;
    }
    acquire_lock_set(&journal.lock_paths)?;
    journal.phase = MoveSetExecutionPhase::InProgress;
    journal.updated_at = Utc::now();
    persist_move_set_journal(&journal.source_store_root, journal)?;

    let mut entity_outcomes = Vec::with_capacity(journal.entity_plans.len());
    let mut migrated_board_entries_by_entity =
        domain.migrate_board_history_for_set(&journal.target_store_root, &journal.entity_ids)?;
    for plan in &journal.entity_plans {
        if journal.completed_entity_ids.contains(&plan.entity_id) {
            let entity_journal_id = journal.entity_journal_ids[&plan.entity_id];
            let completed = load_journal(&journal.source_store_root, entity_journal_id)?;
            if !matches!(
                completed.phase,
                MoveExecutionPhase::SourceScanned
                    | MoveExecutionPhase::TargetScanned
                    | MoveExecutionPhase::Validated
            ) {
                release_lock_set(&journal.lock_paths);
                return Err(MoveError::Domain(format!(
                    "move set journal marks entity {} complete before its journal validated",
                    plan.entity_id
                )));
            }
            entity_outcomes.push(MoveOutcome {
                journal: completed,
                resumed: true,
                rolled_back: false,
            });
            continue;
        }
        let entity_journal_id = journal.entity_journal_ids[&plan.entity_id];
        let existing = load_journal(&journal.source_store_root, entity_journal_id).ok();
        if let Some(existing) = existing.as_ref() {
            if matches!(
                existing.phase,
                MoveExecutionPhase::TargetScanned | MoveExecutionPhase::Validated
            ) {
                journal.completed_entity_ids.push(plan.entity_id);
                entity_outcomes.push(MoveOutcome {
                    journal: existing.clone(),
                    resumed: true,
                    rolled_back: false,
                });
                continue;
            }
        }
        match execute_move_within_set(
            domain,
            plan,
            entity_journal_id,
            existing,
            Some(
                migrated_board_entries_by_entity
                    .remove(&plan.entity_id)
                    .unwrap_or_default(),
            ),
            resumed,
        ) {
            Ok(outcome) => {
                journal.completed_entity_ids.push(plan.entity_id);
                journal.entity_errors.remove(&plan.entity_id);
                entity_outcomes.push(outcome);
                journal.updated_at = Utc::now();
                persist_move_set_journal(&journal.source_store_root, journal)?;
            }
            Err(error) => {
                journal
                    .entity_errors
                    .insert(plan.entity_id, error.to_string());
                journal.failure = Some(error.to_string());
                journal.next_recovery_step =
                    Some("run resume_move_set or rollback_move_set".to_string());
                journal.updated_at = Utc::now();
                persist_move_set_journal(&journal.source_store_root, journal)?;
                release_lock_set(&journal.lock_paths);
                return Err(error);
            }
        }
    }

    let reconciliation = (|| {
        domain.reconcile_store_set(&journal.source_store_root, &journal.entity_ids)?;
        domain.reconcile_store_set(&journal.target_store_root, &journal.entity_ids)?;
        Ok::<(), MoveError>(())
    })();
    if let Err(error) = reconciliation {
        journal.failure = Some(error.to_string());
        journal.next_recovery_step = Some("run resume_move_set or rollback_move_set".to_string());
        journal.updated_at = Utc::now();
        let _ = persist_move_set_journal(&journal.source_store_root, journal);
        release_lock_set(&journal.lock_paths);
        return Err(error);
    }
    if let Err(error) = validate_move_set_outcomes(domain, &mut entity_outcomes) {
        journal.failure = Some(error.to_string());
        journal.next_recovery_step = Some("run resume_move_set or rollback_move_set".to_string());
        journal.updated_at = Utc::now();
        let _ = persist_move_set_journal(&journal.source_store_root, journal);
        release_lock_set(&journal.lock_paths);
        return Err(error);
    }
    journal.phase = MoveSetExecutionPhase::Validated;
    journal.failure = None;
    journal.next_recovery_step = None;
    journal.updated_at = Utc::now();
    persist_move_set_journal(&journal.source_store_root, journal)?;
    release_lock_set(&journal.lock_paths);
    Ok(MoveSetOutcome {
        journal: journal.clone(),
        entity_ids: journal.entity_ids.clone(),
        entity_outcomes,
    })
}

/// Roll back completed entries of a set operation in reverse normalized order.
/// Repeating the operation skips entries already recorded as restored.
pub fn rollback_move_set<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
) -> MoveResult<MoveSetOutcome> {
    let mut journal = load_move_set_journal(&domain.source_store_root(), journal_id)?;
    acquire_lock_set(&journal.lock_paths)?;
    let mut entity_outcomes = Vec::new();
    for entity_id in journal.entity_ids.iter().rev() {
        if journal.rollback_completed_entity_ids.contains(entity_id) {
            continue;
        }
        let entity_journal_id = journal.entity_journal_ids[entity_id];
        let Ok(entity_journal) = load_journal(&journal.source_store_root, entity_journal_id) else {
            continue;
        };
        if entity_journal.phase == MoveExecutionPhase::Planned {
            continue;
        }
        match rollback_move_within_lock(domain, entity_journal_id, true, true) {
            Ok(outcome) => {
                journal.rollback_completed_entity_ids.push(*entity_id);
                entity_outcomes.push(outcome);
                if journal.rollback_completed_entity_ids.len()
                    % MOVE_SET_JOURNAL_CHECKPOINT_INTERVAL
                    == 0
                {
                    journal.updated_at = Utc::now();
                    persist_move_set_journal(&journal.source_store_root, &journal)?;
                }
            }
            Err(error) => {
                journal.entity_errors.insert(*entity_id, error.to_string());
                journal.failure = Some(error.to_string());
                journal.next_recovery_step =
                    Some("run rollback_move_set again after resolving the error".to_string());
                journal.updated_at = Utc::now();
                persist_move_set_journal(&journal.source_store_root, &journal)?;
                release_lock_set(&journal.lock_paths);
                return Err(error);
            }
        }
    }
    let reconciliation = (|| {
        domain.reconcile_store_set(&journal.source_store_root, &journal.entity_ids)?;
        domain.reconcile_store_set(&journal.target_store_root, &journal.entity_ids)?;
        Ok::<(), MoveError>(())
    })();
    if let Err(error) = reconciliation {
        journal.failure = Some(error.to_string());
        journal.next_recovery_step =
            Some("run rollback_move_set again after resolving the error".to_string());
        journal.updated_at = Utc::now();
        let _ = persist_move_set_journal(&journal.source_store_root, &journal);
        release_lock_set(&journal.lock_paths);
        return Err(error);
    }
    journal.phase = MoveSetExecutionPhase::RolledBack;
    journal.failure = None;
    journal.next_recovery_step = None;
    journal.updated_at = Utc::now();
    persist_move_set_journal(&journal.source_store_root, &journal)?;
    release_lock_set(&journal.lock_paths);
    let entity_ids = journal.entity_ids.clone();
    Ok(MoveSetOutcome {
        journal,
        entity_ids,
        entity_outcomes,
    })
}
