use super::*;

/// Execute a supported move with a fresh journal.
pub fn execute_move<D: MoveDomain + ?Sized>(
    domain: &D,
    plan: &MovePlan,
) -> MoveResult<MoveOutcome> {
    let _span_guard = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_execute",
        entity_id = %plan.entity_id,
        supported = plan.supported(),
    )
    .entered();
    if !plan.supported() {
        return Err(MoveError::Domain(
            "move preflight contains blockers".to_string(),
        ));
    }
    if !domain.target_store_present(&plan.target_store_root)? {
        domain.initialize_target_store(&plan.target_store_root)?;
    }
    execute_or_resume(domain, plan, None, None, false, false, false, false)
}

/// Resume an interrupted move identified by its journal id.
pub fn resume_move<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
) -> MoveResult<MoveOutcome> {
    let _span_guard = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_resume",
        journal_id = %journal_id,
    )
    .entered();
    let journal = load_journal(&domain.source_store_root(), journal_id)?;
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        entity_id = %journal.entity_id,
        phase = phase_name(&journal.phase),
        "move_resume_journal_loaded"
    );
    let target_workspace_root = crate::workspace::resolve_workspace_root_from_store_root(
        &journal.target_store_root,
        domain.store_index_dir(),
    );
    let plan = plan_move(domain, &journal.entity_id, &target_workspace_root)?;
    execute_or_resume(
        domain,
        &plan,
        Some(journal),
        None,
        true,
        false,
        false,
        false,
    )
}

/// Roll back a move identified by its journal id.
pub fn rollback_move<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
) -> MoveResult<MoveOutcome> {
    rollback_move_within_lock(domain, journal_id, false, false)
}

pub(super) fn rollback_move_within_lock<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
    skip_lock: bool,
    skip_reconciliation: bool,
) -> MoveResult<MoveOutcome> {
    let source_store_root = domain.source_store_root();
    let mut journal = load_journal(&source_store_root, journal_id)?;
    normalize_journal_entity_paths(domain, &mut journal);
    let span = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_rollback",
        journal_id = %journal.id,
        entity_id = %journal.entity_id,
        resumed = false,
        rolled_back = true,
        phase = Empty,
    );
    let _span_guard = span.enter();
    span.record("phase", phase_name(&journal.phase));
    if journal.lock_paths.is_empty() {
        journal.lock_paths = collect_lock_paths(
            journal.entity_id,
            &journal.source_store_root,
            &journal.target_store_root,
        );
    }
    if !skip_lock {
        acquire_lock_set(&journal.lock_paths)?;
    }

    if journal.destination_entity_path.exists() && !journal.source_entity_path.exists() {
        if let Some(parent) = journal.source_entity_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(
            &journal.destination_entity_path,
            &journal.source_entity_path,
        )?;
    }

    for rewrite in &journal.rewritten_path_files {
        restore_rewritten_path(rewrite)?;
    }

    if !journal.migrated_board_entries.is_empty() {
        let started = Instant::now();
        domain
            .restore_board_history(&journal.target_store_root, &journal.migrated_board_entries)?;
        record_phase_timing(
            &mut journal,
            "rollback_restore_board_history_ms",
            started.elapsed(),
        );
    }

    let source_scan_started = Instant::now();
    if !skip_reconciliation {
        domain.reconcile_store_touched(&journal.source_store_root, &[journal.entity_id])?;
    }
    record_phase_timing(
        &mut journal,
        "rollback_scan_source_ms",
        source_scan_started.elapsed(),
    );

    let target_scan_started = Instant::now();
    if !skip_reconciliation {
        domain.reconcile_store_touched(&journal.target_store_root, &[journal.entity_id])?;
    }
    record_phase_timing(
        &mut journal,
        "rollback_scan_target_ms",
        target_scan_started.elapsed(),
    );

    journal.phase = MoveExecutionPhase::RolledBack;
    journal.updated_at = Utc::now();
    journal
        .steps
        .push("rolled back entity folder to source store".to_string());
    journal.failure = None;
    journal.next_recovery_step = None;
    persist_journal(&source_store_root, &journal)?;
    if !skip_lock {
        release_lock_set(&journal.lock_paths);
    }

    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        journal_id = %journal.id,
        entity_id = %journal.entity_id,
        phase = phase_name(&journal.phase),
        migrated_board_entries = journal.migrated_board_entries.len(),
        rewritten_path_files = journal.rewritten_path_files.len(),
        manual_followups = journal.manual_followups.len(),
        "move_rollback_complete"
    );

    Ok(MoveOutcome {
        journal,
        resumed: false,
        rolled_back: true,
    })
}

pub(super) fn execute_or_resume<D: MoveDomain + ?Sized>(
    domain: &D,
    plan: &MovePlan,
    existing: Option<MoveJournal>,
    pre_migrated_board_entries: Option<Vec<BoardEntry>>,
    resumed: bool,
    skip_lock: bool,
    skip_reconciliation: bool,
    skip_validation: bool,
) -> MoveResult<MoveOutcome> {
    let journal_root = domain.source_store_root();
    let mut journal = existing.unwrap_or_else(|| MoveJournal {
        id: Uuid::new_v4(),
        entity_id: plan.entity_id,
        source_store_root: plan.source_store_root.clone(),
        target_store_root: plan.target_store_root.clone(),
        source_entity_path: plan.source_entity_path.clone(),
        destination_entity_path: plan.destination_entity_path.clone(),
        phase: MoveExecutionPhase::Planned,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        steps: vec!["created move journal".to_string()],
        rollback_steps: vec![
            "rename destination entity folder back to source path".to_string(),
            "restore migrated board history rows to source store".to_string(),
            "scan source and target stores".to_string(),
        ],
        lock_paths: collect_lock_paths(
            plan.entity_id,
            &plan.source_store_root,
            &plan.target_store_root,
        ),
        migrated_board_entries: Vec::new(),
        rewritten_path_files: Vec::new(),
        manual_followups: Vec::new(),
        phase_timings_ms: std::collections::BTreeMap::new(),
        failure: None,
        next_recovery_step: None,
    });
    normalize_journal_entity_paths(domain, &mut journal);
    if journal.lock_paths.is_empty() {
        journal.lock_paths = collect_lock_paths(
            journal.entity_id,
            &journal.source_store_root,
            &journal.target_store_root,
        );
    }
    persist_journal(&journal_root, &journal)?;

    let span = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_execute_or_resume",
        journal_id = %journal.id,
        entity_id = %journal.entity_id,
        resumed,
        rolled_back = false,
        phase = Empty,
    );
    let _span_guard = span.enter();
    span.record("phase", phase_name(&journal.phase));
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        phase = phase_name(&journal.phase),
        "move_journal_ready"
    );

    let result: MoveResult<()> = (|| {
        advance_phase_planned(
            &mut journal,
            &journal_root,
            &span,
            skip_lock,
            !skip_reconciliation,
        )?;
        advance_phase_locked(&mut journal, &journal_root, &span)?;
        advance_phase_moved(
            domain,
            plan,
            &mut journal,
            &journal_root,
            &span,
            skip_reconciliation,
            pre_migrated_board_entries,
        )?;
        advance_phase_source_scanned(
            domain,
            &mut journal,
            &journal_root,
            &span,
            skip_reconciliation,
            !skip_reconciliation,
        )?;
        advance_phase_target_scanned(
            domain,
            &mut journal,
            &journal_root,
            &span,
            skip_validation,
            !skip_validation,
        )?;

        Ok(())
    })();

    match result {
        Ok(()) => {
            if !skip_lock {
                release_lock_set(&journal.lock_paths);
            }
            tracing::debug!(
                target: MOVE_TRACE_TARGET,
                journal_id = %journal.id,
                entity_id = %journal.entity_id,
                phase = phase_name(&journal.phase),
                resumed,
                rewritten_path_files = journal.rewritten_path_files.len(),
                manual_followups = journal.manual_followups.len(),
                migrated_board_entries = journal.migrated_board_entries.len(),
                "move_execute_or_resume_complete"
            );
            Ok(MoveOutcome {
                journal,
                resumed,
                rolled_back: false,
            })
        }
        Err(error) => {
            journal.updated_at = Utc::now();
            journal.failure = Some(error.to_string());
            journal.next_recovery_step = Some(recovery_hint_for_phase(&journal.phase).to_string());
            let _ = persist_journal(&journal_root, &journal);
            if !skip_lock {
                release_lock_set(&journal.lock_paths);
            }
            tracing::error!(
                target: MOVE_TRACE_TARGET,
                journal_id = %journal.id,
                entity_id = %journal.entity_id,
                phase = phase_name(&journal.phase),
                resumed,
                error = %error,
                "move_execute_or_resume_failed"
            );
            Err(error)
        }
    }
}

pub(super) fn journal_from_plan(plan: &MovePlan, journal_id: Uuid) -> MoveJournal {
    MoveJournal {
        id: journal_id,
        entity_id: plan.entity_id,
        source_store_root: plan.source_store_root.clone(),
        target_store_root: plan.target_store_root.clone(),
        source_entity_path: plan.source_entity_path.clone(),
        destination_entity_path: plan.destination_entity_path.clone(),
        phase: MoveExecutionPhase::Planned,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        steps: vec!["created move journal".to_string()],
        rollback_steps: vec![
            "rename destination entity folder back to source path".to_string(),
            "restore migrated board history rows to source store".to_string(),
            "scan source and target stores".to_string(),
        ],
        lock_paths: collect_lock_paths(
            plan.entity_id,
            &plan.source_store_root,
            &plan.target_store_root,
        ),
        migrated_board_entries: Vec::new(),
        rewritten_path_files: Vec::new(),
        manual_followups: Vec::new(),
        phase_timings_ms: std::collections::BTreeMap::new(),
        failure: None,
        next_recovery_step: None,
    }
}
