use super::*;

pub(in crate::storage::move_kernel) fn normalize_journal_entity_paths<D: MoveDomain + ?Sized>(
    domain: &D,
    journal: &mut MoveJournal,
) {
    let subdir = domain.entity_subdir();
    let expected_source = journal
        .source_store_root
        .join(subdir)
        .join(journal.entity_id.to_string());
    let expected_destination = journal
        .target_store_root
        .join(subdir)
        .join(journal.entity_id.to_string());

    if journal.source_entity_path != expected_source {
        journal.source_entity_path = expected_source;
    }
    if journal.destination_entity_path != expected_destination {
        journal.destination_entity_path = expected_destination;
    }
}

pub(in crate::storage::move_kernel) fn journal_path(store_root: &Path, id: Uuid) -> PathBuf {
    store_root
        .join(MOVE_JOURNALS_DIR)
        .join(format!("{}.json", id))
}

pub(in crate::storage::move_kernel) fn move_set_journal_path(
    store_root: &Path,
    id: Uuid,
) -> PathBuf {
    store_root
        .join(MOVE_JOURNALS_DIR)
        .join(format!("{}.set.json", id))
}

/// Persist a move journal under the store root's `move-journals/` directory.
///
/// Enforces the journal-backed operation interoperability contract at this
/// persistence boundary: a journal missing authoritative identity, replay/
/// rollback lineage, or deterministic mutation payload ownership is rejected
/// and never written to disk.
pub fn persist_journal(store_root: &Path, journal: &MoveJournal) -> MoveResult<()> {
    // Compile-time check that MoveJournal implements InteroperableArtifact
    fn assert_interoperable<T: InteroperableArtifact>() {}
    assert_interoperable::<MoveJournal>();

    journal.validate_interoperability_contract()?;
    let path = journal_path(store_root, journal.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload =
        serde_json::to_vec_pretty(journal).map_err(|error| MoveError::Domain(error.to_string()))?;
    fs::write(path, payload).map_err(MoveError::Io)
}

/// Load a move journal from the store root's `move-journals/` directory.
pub fn load_journal(store_root: &Path, id: Uuid) -> MoveResult<MoveJournal> {
    let payload = fs::read(journal_path(store_root, id)).map_err(MoveError::Io)?;
    serde_json::from_slice(&payload).map_err(|error| MoveError::Domain(error.to_string()))
}

/// Persist one set-operation journal alongside legacy per-entity journals.
pub fn persist_move_set_journal(store_root: &Path, journal: &MoveSetJournal) -> MoveResult<()> {
    journal.validate()?;
    let path = move_set_journal_path(store_root, journal.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut persisted = journal.clone();
    if matches!(
        persisted.phase,
        MoveSetExecutionPhase::Validated | MoveSetExecutionPhase::RolledBack
    ) {
        persisted.entity_plans.clear();
    }
    let payload = serde_json::to_vec_pretty(&persisted)
        .map_err(|error| MoveError::Domain(error.to_string()))?;
    fs::write(path, payload).map_err(MoveError::Io)
}

/// Load one set-operation journal from the source store.
pub fn load_move_set_journal(store_root: &Path, id: Uuid) -> MoveResult<MoveSetJournal> {
    let payload = fs::read(move_set_journal_path(store_root, id)).map_err(MoveError::Io)?;
    let journal: MoveSetJournal =
        serde_json::from_slice(&payload).map_err(|error| MoveError::Domain(error.to_string()))?;
    journal.validate()?;
    Ok(journal)
}

#[cfg(test)]
mod persist_journal_contract_tests {
    use super::*;

    fn journal_with_id(id: Uuid) -> MoveJournal {
        MoveJournal {
            id,
            entity_id: Uuid::new_v4(),
            source_store_root: PathBuf::from("/stores/source"),
            target_store_root: PathBuf::from("/stores/target"),
            source_entity_path: PathBuf::from("/stores/source/entity"),
            destination_entity_path: PathBuf::from("/stores/target/entity"),
            phase: MoveExecutionPhase::Planned,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec!["created move journal".to_string()],
            rollback_steps: vec!["rename destination back to source".to_string()],
            lock_paths: Vec::new(),
            migrated_board_entries: Vec::new(),
            rewritten_path_files: Vec::new(),
            manual_followups: Vec::new(),
            phase_timings_ms: std::collections::BTreeMap::new(),
            failure: None,
            next_recovery_step: None,
        }
    }

    #[test]
    fn persist_journal_rejects_non_compliant_journal_and_writes_nothing() {
        let store = tempfile::tempdir().expect("temp store root");
        let id = Uuid::new_v4();
        let mut journal = journal_with_id(id);
        // Strip deterministic mutation payload ownership + operation identity.
        journal.entity_id = Uuid::nil();
        journal.source_store_root = PathBuf::new();

        let error = persist_journal(store.path(), &journal)
            .expect_err("non-compliant journal must be rejected at persistence");
        match error {
            MoveError::InteroperabilityContract {
                artifact_class,
                detail,
            } => {
                assert_eq!(artifact_class, "move-journal");
                assert!(
                    detail.contains("missing authoritative operation identity"),
                    "unexpected detail: {detail}"
                );
                assert!(
                    detail.contains("missing source store root"),
                    "unexpected detail: {detail}"
                );
            }
            other => panic!("unexpected error variant: {other:?}"),
        }

        assert!(
            !journal_path(store.path(), id).exists(),
            "rejected journal must not be written to disk"
        );
    }

    #[test]
    fn persist_journal_writes_compliant_journal() {
        let store = tempfile::tempdir().expect("temp store root");
        let id = Uuid::new_v4();
        let journal = journal_with_id(id);

        persist_journal(store.path(), &journal).expect("compliant journal must persist");

        assert!(
            journal_path(store.path(), id).exists(),
            "compliant journal must be written to disk"
        );
    }
}
