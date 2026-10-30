use super::*;

/// Lock paths the kernel acquires for a move (store + entity locks on both roots).
pub fn collect_lock_paths(
    entity_id: Uuid,
    source_store_root: &Path,
    target_store_root: &Path,
) -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for root in [source_store_root, target_store_root] {
        paths.insert(root.join(MOVE_LOCKS_DIR).join("store.lock"));
        paths.insert(
            root.join(MOVE_LOCKS_DIR)
                .join(format!("entity-{}.lock", entity_id)),
        );
    }
    paths.into_iter().collect()
}

pub(in crate::storage::move_kernel) fn acquire_lock_set(lock_paths: &[PathBuf]) -> MoveResult<()> {
    let mut acquired = Vec::new();
    for path in lock_paths {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(_) => acquired.push(path.clone()),
            Err(error) => {
                release_lock_set(&acquired);
                return Err(MoveError::Domain(format!(
                    "move lock already held at {}: {}",
                    path.display(),
                    error
                )));
            }
        }
    }
    Ok(())
}

pub(in crate::storage::move_kernel) fn release_lock_set(lock_paths: &[PathBuf]) {
    for path in lock_paths {
        let _ = fs::remove_file(path);
    }
}

pub(in crate::storage::move_kernel) fn record_phase_timing(
    journal: &mut MoveJournal,
    key: &str,
    elapsed: std::time::Duration,
) {
    let millis = elapsed.as_millis().min(u64::MAX as u128) as u64;
    journal.phase_timings_ms.insert(key.to_string(), millis);
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        journal_id = %journal.id,
        entity_id = %journal.entity_id,
        phase = phase_name(&journal.phase),
        timing_key = key,
        elapsed_ms = millis,
        "move_phase_complete"
    );
}

pub(in crate::storage::move_kernel) fn phase_name(phase: &MoveExecutionPhase) -> &'static str {
    match phase {
        MoveExecutionPhase::Planned => "planned",
        MoveExecutionPhase::Locked => "locked",
        MoveExecutionPhase::Moved => "moved",
        MoveExecutionPhase::SourceScanned => "source_scanned",
        MoveExecutionPhase::TargetScanned => "target_scanned",
        MoveExecutionPhase::Validated => "validated",
        MoveExecutionPhase::RolledBack => "rolled_back",
    }
}

pub(in crate::storage::move_kernel) fn recovery_hint_for_phase(
    phase: &MoveExecutionPhase,
) -> &'static str {
    match phase {
        MoveExecutionPhase::Planned | MoveExecutionPhase::Locked => {
            "run resume_move to continue execution"
        }
        MoveExecutionPhase::Moved
        | MoveExecutionPhase::SourceScanned
        | MoveExecutionPhase::TargetScanned => {
            "run rollback_move for safety, or resume_move to retry"
        }
        MoveExecutionPhase::Validated | MoveExecutionPhase::RolledBack => {
            "no recovery action needed"
        }
    }
}
