use super::*;

pub(in crate::storage::move_kernel) fn advance_phase_planned(
    journal: &mut MoveJournal,
    journal_root: &Path,
    span: &tracing::Span,
    skip_lock: bool,
    persist_phase: bool,
) -> MoveResult<()> {
    if journal.phase != MoveExecutionPhase::Planned {
        return Ok(());
    }

    let started = Instant::now();
    let step = if skip_lock {
        // A set-level operation (`execute_move_set`) already acquired the
        // union of these lock paths for the whole set before any entity
        // started; re-acquiring the same lock files here would fail with
        // "already held" instead of succeeding.
        "source/target store locks already held by the enclosing move-set operation"
    } else {
        acquire_lock_set(&journal.lock_paths)?;
        "acquired source/target store locks and move entity lock"
    };
    record_phase_timing(journal, "lock_acquisition_ms", started.elapsed());
    journal.phase = MoveExecutionPhase::Locked;
    span.record("phase", phase_name(&journal.phase));
    journal.updated_at = Utc::now();
    journal.steps.push(step.to_string());
    if persist_phase {
        persist_journal(journal_root, journal)?;
    }
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        phase = phase_name(&journal.phase),
        "move_phase_advanced"
    );
    Ok(())
}

pub(in crate::storage::move_kernel) fn advance_phase_locked(
    journal: &mut MoveJournal,
    journal_root: &Path,
    span: &tracing::Span,
) -> MoveResult<()> {
    if journal.phase != MoveExecutionPhase::Locked {
        return Ok(());
    }

    if let Some(parent) = journal.destination_entity_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let started = Instant::now();
    if journal.source_entity_path.exists() {
        fs::rename(
            &journal.source_entity_path,
            &journal.destination_entity_path,
        )?;
    }
    record_phase_timing(journal, "rename_entity_ms", started.elapsed());
    journal.phase = MoveExecutionPhase::Moved;
    span.record("phase", phase_name(&journal.phase));
    journal.updated_at = Utc::now();
    journal.steps.push("moved entity folder".to_string());
    persist_journal(journal_root, journal)?;
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        phase = phase_name(&journal.phase),
        "move_phase_advanced"
    );
    Ok(())
}

pub(in crate::storage::move_kernel) fn advance_phase_moved<D: MoveDomain + ?Sized>(
    domain: &D,
    plan: &MovePlan,
    journal: &mut MoveJournal,
    journal_root: &Path,
    span: &tracing::Span,
    skip_reconciliation: bool,
    pre_migrated_board_entries: Option<Vec<BoardEntry>>,
) -> MoveResult<()> {
    if journal.phase != MoveExecutionPhase::Moved {
        return Ok(());
    }

    if journal.rewritten_path_files.is_empty() && journal.manual_followups.is_empty() {
        let started = Instant::now();
        let (rewritten, followups) = rewrite_path_references(plan)?;
        record_phase_timing(journal, "rewrite_path_refs_ms", started.elapsed());
        if !rewritten.is_empty() {
            journal.steps.push(format!(
                "rewrote {} tracked path reference files",
                rewritten.len()
            ));
        }
        if !followups.is_empty() {
            journal.steps.push(format!(
                "recorded {} manual path-reference follow-ups",
                followups.len()
            ));
        }
        journal.rewritten_path_files = rewritten;
        journal.manual_followups = followups;
    }

    if journal.migrated_board_entries.is_empty() {
        let started = Instant::now();
        journal.migrated_board_entries = match pre_migrated_board_entries {
            Some(entries) => entries,
            None => domain.migrate_board_history(&journal.target_store_root, &journal.entity_id)?,
        };
        record_phase_timing(journal, "migrate_board_history_ms", started.elapsed());
        if !journal.migrated_board_entries.is_empty() {
            journal.steps.push(format!(
                "migrated {} historical board rows",
                journal.migrated_board_entries.len()
            ));
        }
    }

    let started = Instant::now();
    if !skip_reconciliation {
        domain.reconcile_store_touched(&journal.source_store_root, &[journal.entity_id])?;
    }
    record_phase_timing(journal, "scan_source_ms", started.elapsed());
    journal.phase = MoveExecutionPhase::SourceScanned;
    span.record("phase", phase_name(&journal.phase));
    journal.updated_at = Utc::now();
    journal.steps.push("scanned source store".to_string());
    persist_journal(journal_root, journal)?;
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        phase = phase_name(&journal.phase),
        rewritten_path_files = journal.rewritten_path_files.len(),
        manual_followups = journal.manual_followups.len(),
        migrated_board_entries = journal.migrated_board_entries.len(),
        "move_phase_advanced"
    );
    Ok(())
}

pub(in crate::storage::move_kernel) fn advance_phase_source_scanned<D: MoveDomain + ?Sized>(
    domain: &D,
    journal: &mut MoveJournal,
    journal_root: &Path,
    span: &tracing::Span,
    skip_reconciliation: bool,
    persist_phase: bool,
) -> MoveResult<()> {
    if journal.phase != MoveExecutionPhase::SourceScanned {
        return Ok(());
    }

    let started = Instant::now();
    if !skip_reconciliation {
        domain.reconcile_store_touched(&journal.target_store_root, &[journal.entity_id])?;
    }
    record_phase_timing(journal, "scan_target_ms", started.elapsed());
    journal.phase = MoveExecutionPhase::TargetScanned;
    span.record("phase", phase_name(&journal.phase));
    journal.updated_at = Utc::now();
    journal.steps.push("scanned target store".to_string());
    if persist_phase {
        persist_journal(journal_root, journal)?;
    }
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        phase = phase_name(&journal.phase),
        "move_phase_advanced"
    );
    Ok(())
}

pub(in crate::storage::move_kernel) fn advance_phase_target_scanned<D: MoveDomain + ?Sized>(
    domain: &D,
    journal: &mut MoveJournal,
    journal_root: &Path,
    span: &tracing::Span,
    skip_validation: bool,
    persist_phase: bool,
) -> MoveResult<()> {
    if journal.phase != MoveExecutionPhase::TargetScanned {
        return Ok(());
    }

    let started = Instant::now();
    if skip_validation {
        journal.phase = MoveExecutionPhase::TargetScanned;
        span.record("phase", phase_name(&journal.phase));
        journal.updated_at = Utc::now();
        journal
            .steps
            .push("deferred set ownership validation".to_string());
        if persist_phase {
            persist_journal(journal_root, journal)?;
        }
        return Ok(());
    }
    let source_path_exists = journal.source_entity_path.exists();
    let destination_path_exists = journal.destination_entity_path.exists();
    let source_seen = domain.entity_indexed_in(&journal.source_store_root, &journal.entity_id)?;
    let target_seen = domain.entity_indexed_in(&journal.target_store_root, &journal.entity_id)?;
    record_phase_timing(journal, "validate_move_ms", started.elapsed());
    if source_path_exists || !destination_path_exists {
        return Err(build_post_move_validation_error(
            journal,
            source_path_exists,
            destination_path_exists,
            source_seen,
            target_seen,
        ));
    }

    if source_seen {
        journal.steps.push(format!(
            "source index still resolves entity {}; source folder is absent, so ownership is correct (run scan --force later to clear stale index visibility)",
            journal.entity_id
        ));
    }
    if !target_seen {
        journal.steps.push(format!(
            "destination index has not resolved entity {} yet; destination folder exists and ownership is correct",
            journal.entity_id
        ));
    }

    journal.phase = MoveExecutionPhase::Validated;
    span.record("phase", phase_name(&journal.phase));
    journal.updated_at = Utc::now();
    journal.steps.push("validated move ownership".to_string());
    journal.failure = None;
    journal.next_recovery_step = None;
    if persist_phase {
        persist_journal(journal_root, journal)?;
    }
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        phase = phase_name(&journal.phase),
        "move_phase_advanced"
    );
    Ok(())
}

pub(in crate::storage::move_kernel) fn build_post_move_validation_error(
    journal: &MoveJournal,
    source_path_exists: bool,
    destination_path_exists: bool,
    source_seen: bool,
    target_seen: bool,
) -> MoveError {
    let mut problems = Vec::new();
    if source_path_exists {
        problems.push(format!(
            "source entity folder {} still exists after the move",
            normalize_slashes(&journal.source_entity_path),
        ));
    }
    if !destination_path_exists {
        problems.push(format!(
            "destination entity folder {} does not exist after the move",
            normalize_slashes(&journal.destination_entity_path),
        ));
    }
    if source_seen {
        problems.push(format!(
            "source store {} still indexes entity {} after the move (source entity folder {} should no longer exist)",
            normalize_slashes(&journal.source_store_root),
            journal.entity_id,
            normalize_slashes(&journal.source_entity_path),
        ));
    }
    if !target_seen {
        problems.push(format!(
            "destination store {} does not index entity {} after the move (expected entity folder {} — check that the destination store root resolved without a Windows verbatim `\\?\\` prefix)",
            normalize_slashes(&journal.target_store_root),
            journal.entity_id,
            normalize_slashes(&journal.destination_entity_path),
        ));
    }
    MoveError::Domain(format!(
        "post-move validation failed: {}",
        problems.join("; ")
    ))
}
