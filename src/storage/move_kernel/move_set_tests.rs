use super::*;
use std::cell::Cell;

/// Minimal [`MoveDomain`] fixture backed by plain folders under a
/// `.workflow-tools/fixture` store marker: an entity "exists" iff
/// `<store_root>/entities/<id>` is a directory. No edges, board, or
/// leases — those hooks use the trait's empty defaults.
struct FixtureDomain {
    source_store_root: PathBuf,
    fail_scan_number: Cell<Option<usize>>,
    scan_calls: Cell<usize>,
}

impl MoveDomain for FixtureDomain {
    fn entity_subdir(&self) -> &str {
        "entities"
    }

    fn store_index_dir(&self) -> &str {
        ".fixture"
    }

    fn source_store_root(&self) -> PathBuf {
        self.source_store_root.clone()
    }

    fn source_entity_path(&self, entity_id: &Uuid) -> MoveResult<Option<PathBuf>> {
        let path = self
            .source_store_root
            .join("entities")
            .join(entity_id.to_string());
        Ok(path.is_dir().then_some(path))
    }

    fn related_entities(&self, _entity_id: &Uuid) -> MoveResult<MoveReferences> {
        Ok(MoveReferences::default())
    }

    fn target_store_present(&self, target_store_root: &Path) -> MoveResult<bool> {
        Ok(target_store_root.is_dir())
    }

    fn entity_indexed_in(&self, store_root: &Path, entity_id: &Uuid) -> MoveResult<bool> {
        Ok(store_root
            .join("entities")
            .join(entity_id.to_string())
            .is_dir())
    }

    fn scan_store(&self, _store_root: &Path) -> MoveResult<()> {
        let scan_number = self.scan_calls.get() + 1;
        self.scan_calls.set(scan_number);
        if self.fail_scan_number.get() == Some(scan_number) {
            return Err(MoveError::Domain("fixture scan failure".to_string()));
        }
        Ok(())
    }
}

fn init_git_repo(root: &Path) {
    let status = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(root)
        .status()
        .expect("git init must run");
    assert!(status.success(), "git init failed");
}

fn make_entity(store_root: &Path, entity_id: Uuid) {
    fs::create_dir_all(store_root.join("entities").join(entity_id.to_string()))
        .expect("create entity folder");
}

/// One git repo containing both `source/` and `target/` workspaces, so
/// git worktree topology resolves to `Same` without a second repo.
fn setup_repo() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let repo = tempfile::tempdir().expect("temp repo");
    init_git_repo(repo.path());
    let source_ws = repo.path().join("source");
    let target_ws = repo.path().join("target");
    fs::create_dir_all(
        source_ws
            .join(".workflow-tools")
            .join("fixture")
            .join("entities"),
    )
    .expect("create source store");
    fs::create_dir_all(target_ws.join(".workflow-tools").join("fixture"))
        .expect("create target store");
    (repo, source_ws, target_ws)
}

#[test]
fn normalize_entity_selection_sorts_and_dedupes() {
    let a = Uuid::from_u128(3);
    let b = Uuid::from_u128(1);
    let c = Uuid::from_u128(2);
    let normalized = normalize_entity_selection(&[a, b, c, b, a]);
    assert_eq!(normalized, vec![b, c, a]);
}

#[test]
fn plan_move_set_matches_plan_move_for_single_entity() {
    let (_repo, source_ws, target_ws) = setup_repo();
    let domain = FixtureDomain {
        source_store_root: source_ws.join(".workflow-tools").join("fixture"),
        fail_scan_number: Cell::new(None),
        scan_calls: Cell::new(0),
    };
    let entity_id = Uuid::new_v4();
    make_entity(&domain.source_store_root, entity_id);

    let single_plan = plan_move(&domain, &entity_id, &target_ws).expect("plan_move");
    let set_plan = plan_move_set(&domain, &[entity_id], &target_ws).expect("plan_move_set");

    assert_eq!(set_plan.entity_ids, vec![entity_id]);
    assert_eq!(set_plan.entity_plans.len(), 1);
    let entity_plan = &set_plan.entity_plans[0];
    assert_eq!(entity_plan.entity_id, single_plan.entity_id);
    assert_eq!(
        entity_plan.source_entity_path,
        single_plan.source_entity_path
    );
    assert_eq!(
        entity_plan.destination_entity_path,
        single_plan.destination_entity_path
    );
    assert_eq!(
        entity_plan.git_worktree_topology,
        single_plan.git_worktree_topology
    );
    assert!(entity_plan.supported());
    assert!(single_plan.supported());
    assert!(set_plan.supported());
}

#[test]
fn plan_move_set_batches_tracked_path_references_per_entity() {
    let (repo, source_ws, target_ws) = setup_repo();
    let domain = FixtureDomain {
        source_store_root: source_ws.join(".workflow-tools").join("fixture"),
        fail_scan_number: Cell::new(None),
        scan_calls: Cell::new(0),
    };
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    make_entity(&domain.source_store_root, entity_a);
    make_entity(&domain.source_store_root, entity_b);
    let entity_a_path = domain
        .source_store_root
        .join("entities")
        .join(entity_a.to_string());
    let entity_b_path = domain
        .source_store_root
        .join("entities")
        .join(entity_b.to_string());
    fs::write(
        repo.path().join("references.txt"),
        format!(
            "{}\n{}\n",
            entity_a_path.to_string_lossy().replace('\\', "/"),
            entity_b_path.to_string_lossy().replace('\\', "/")
        ),
    )
    .expect("write tracked path references");
    let status = Command::new("git")
        .args(["add", "references.txt"])
        .current_dir(repo.path())
        .status()
        .expect("git add must run");
    assert!(status.success(), "git add failed");

    let set_plan =
        plan_move_set(&domain, &[entity_a, entity_b], &target_ws).expect("plan_move_set");
    let reference_file = repo.path().join("references.txt");
    for entity_id in [entity_a, entity_b] {
        let plan = set_plan
            .entity_plans
            .iter()
            .find(|plan| plan.entity_id == entity_id)
            .expect("entity plan");
        assert_eq!(plan.path_reference_files, vec![reference_file.clone()]);
    }
}

#[test]
fn plan_move_set_chunks_large_path_reference_scans() {
    let (repo, source_ws, target_ws) = setup_repo();
    let domain = FixtureDomain {
        source_store_root: source_ws.join(".workflow-tools").join("fixture"),
        fail_scan_number: Cell::new(None),
        scan_calls: Cell::new(0),
    };
    let entity_ids = (0..384)
        .map(|_| {
            let entity_id = Uuid::new_v4();
            make_entity(&domain.source_store_root, entity_id);
            entity_id
        })
        .collect::<Vec<_>>();
    let referenced_id = entity_ids[entity_ids.len() - 1];
    let referenced_path = domain
        .source_store_root
        .join("entities")
        .join(referenced_id.to_string());
    fs::write(
        repo.path().join("large-set-reference.txt"),
        referenced_path.to_string_lossy().replace('\\', "/"),
    )
    .expect("write tracked path reference");
    let status = Command::new("git")
        .args(["add", "large-set-reference.txt"])
        .current_dir(repo.path())
        .status()
        .expect("git add must run");
    assert!(status.success(), "git add failed");

    let set_plan =
        plan_move_set(&domain, &entity_ids, &target_ws).expect("large move set must plan");

    assert!(set_plan.supported());
    let referenced_plan = set_plan
        .entity_plans
        .iter()
        .find(|plan| plan.entity_id == referenced_id)
        .expect("referenced entity plan");
    assert_eq!(
        referenced_plan.path_reference_files,
        vec![repo.path().join("large-set-reference.txt")]
    );
}

#[test]
fn plan_move_set_propagates_missing_source_blocker_per_entity() {
    let (_repo, source_ws, target_ws) = setup_repo();
    let domain = FixtureDomain {
        source_store_root: source_ws.join(".workflow-tools").join("fixture"),
        fail_scan_number: Cell::new(None),
        scan_calls: Cell::new(0),
    };
    let present_id = Uuid::new_v4();
    let missing_id = Uuid::new_v4();
    make_entity(&domain.source_store_root, present_id);

    let set_plan =
        plan_move_set(&domain, &[present_id, missing_id], &target_ws).expect("plan_move_set");

    assert!(!set_plan.supported());
    let present_plan = set_plan
        .entity_plans
        .iter()
        .find(|plan| plan.entity_id == present_id)
        .expect("present entity plan");
    assert!(present_plan.supported());

    let missing_plan = set_plan
        .entity_plans
        .iter()
        .find(|plan| plan.entity_id == missing_id)
        .expect("missing entity plan");
    assert!(
        missing_plan.blockers.iter().any(|blocker| matches!(
            blocker,
            MoveBlocker::MissingSourceEntity { entity_id }
                if *entity_id == missing_id
        )),
        "expected MissingSourceEntity blocker, got {:?}",
        missing_plan.blockers
    );
}

#[test]
fn execute_move_set_moves_all_entities_and_supports_per_entity_rollback() {
    let (_repo, source_ws, target_ws) = setup_repo();
    let domain = FixtureDomain {
        source_store_root: source_ws.join(".workflow-tools").join("fixture"),
        fail_scan_number: Cell::new(None),
        scan_calls: Cell::new(0),
    };
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    make_entity(&domain.source_store_root, entity_a);
    make_entity(&domain.source_store_root, entity_b);

    let set_plan =
        plan_move_set(&domain, &[entity_a, entity_b], &target_ws).expect("plan_move_set");
    assert!(set_plan.supported());

    let outcome = execute_move_set(&domain, &set_plan).expect("execute_move_set");
    assert_eq!(outcome.entity_ids, set_plan.entity_ids);
    assert_eq!(outcome.entity_outcomes.len(), 2);
    assert_eq!(domain.scan_calls.get(), 2);
    let loaded_set_journal = load_move_set_journal(&domain.source_store_root, outcome.journal.id)
        .expect("set journal must round-trip");
    assert_eq!(loaded_set_journal.entity_ids, set_plan.entity_ids);
    assert_eq!(loaded_set_journal.phase, MoveSetExecutionPhase::Validated);
    assert!(loaded_set_journal.entity_plans.is_empty());
    let persisted_payload = fs::read_to_string(move_set_journal_path(
        &domain.source_store_root,
        outcome.journal.id,
    ))
    .expect("persisted set journal");
    assert!(!persisted_payload.contains("entity_plans"));
    for entity_outcome in &outcome.entity_outcomes {
        assert_eq!(entity_outcome.journal.phase, MoveExecutionPhase::Validated);
        assert!(!entity_outcome.journal.source_entity_path.exists());
        assert!(entity_outcome.journal.destination_entity_path.exists());
    }

    // Recovery path: an individual entity's journal produced by the set
    // operation remains independently rollback-able via the existing
    // single-entity `rollback_move`.
    let target_store_root = target_ws.join(".workflow-tools").join("fixture");
    let entity_a_outcome = outcome
        .entity_outcomes
        .iter()
        .find(|entity_outcome| entity_outcome.journal.entity_id == entity_a)
        .expect("entity_a outcome");
    let rolled_back = rollback_move(&domain, entity_a_outcome.journal.id)
        .expect("rollback_move on set-produced journal");
    assert_eq!(rolled_back.journal.phase, MoveExecutionPhase::RolledBack);
    assert!(
        domain
            .source_entity_path(&entity_a)
            .expect("source_entity_path")
            .is_some(),
        "entity_a must be restored to the source store after rollback"
    );
    assert!(
        !target_store_root
            .join("entities")
            .join(entity_a.to_string())
            .exists()
    );

    // entity_b remains moved (untouched by entity_a's rollback).
    assert!(
        target_store_root
            .join("entities")
            .join(entity_b.to_string())
            .exists()
    );
}

#[test]
fn resume_move_set_skips_completed_entities() {
    let (_repo, source_ws, target_ws) = setup_repo();
    let domain = FixtureDomain {
        source_store_root: source_ws.join(".workflow-tools").join("fixture"),
        fail_scan_number: Cell::new(Some(2)),
        scan_calls: Cell::new(0),
    };
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    make_entity(&domain.source_store_root, entity_a);
    make_entity(&domain.source_store_root, entity_b);
    let set_plan =
        plan_move_set(&domain, &[entity_a, entity_b], &target_ws).expect("plan_move_set");

    execute_move_set(&domain, &set_plan)
        .expect_err("batched target reconciliation must interrupt the set move");
    let journal_path = fs::read_dir(domain.source_store_root.join(MOVE_JOURNALS_DIR))
        .expect("set journal directory")
        .map(|entry| entry.expect("journal entry").path())
        .find(|path| path.to_string_lossy().ends_with(".set.json"))
        .expect("set journal path");
    let journal_id = Uuid::parse_str(
        journal_path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .trim_end_matches(".set.json"),
    )
    .expect("set journal id");
    let interrupted = load_move_set_journal(&domain.source_store_root, journal_id)
        .expect("interrupted set journal");
    assert_eq!(interrupted.completed_entity_ids, set_plan.entity_ids);

    domain.fail_scan_number.set(None);
    let resumed = resume_move_set(&domain, journal_id).expect("resume only unfinished entity");
    assert_eq!(resumed.entity_outcomes.len(), 2);
    assert!(
        resumed
            .entity_outcomes
            .iter()
            .any(|outcome| { outcome.journal.entity_id == entity_a && outcome.resumed })
    );
    let completed = load_move_set_journal(&domain.source_store_root, journal_id)
        .expect("completed set journal");
    assert_eq!(completed.completed_entity_ids, set_plan.entity_ids);
}

#[test]
fn rollback_move_set_is_idempotent() {
    let (_repo, source_ws, target_ws) = setup_repo();
    let domain = FixtureDomain {
        source_store_root: source_ws.join(".workflow-tools").join("fixture"),
        fail_scan_number: Cell::new(None),
        scan_calls: Cell::new(0),
    };
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    make_entity(&domain.source_store_root, entity_a);
    make_entity(&domain.source_store_root, entity_b);
    let plan = plan_move_set(&domain, &[entity_a, entity_b], &target_ws).expect("plan_move_set");
    let moved = execute_move_set(&domain, &plan).expect("execute_move_set");

    let rolled_back = rollback_move_set(&domain, moved.journal.id).expect("first rollback");
    assert_eq!(rolled_back.entity_outcomes.len(), 2);
    assert!(domain.source_entity_path(&entity_a).unwrap().is_some());
    assert!(domain.source_entity_path(&entity_b).unwrap().is_some());

    let repeated = rollback_move_set(&domain, moved.journal.id).expect("idempotent rollback");
    assert!(repeated.entity_outcomes.is_empty());
    assert_eq!(repeated.journal.phase, MoveSetExecutionPhase::RolledBack);
}
