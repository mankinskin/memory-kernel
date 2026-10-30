use super::*;

#[test]
fn find_local_root_from_discovers_parent_workspace() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let nested = repo.join("a").join("b");
    std::fs::create_dir_all(canonical_store_root(&repo, ".ticket")).unwrap();
    std::fs::create_dir_all(&nested).unwrap();

    let found = find_local_root_from(&nested, ".ticket").unwrap();

    assert_eq!(found, canonical_store_root(&repo, ".ticket"));
}

#[test]
fn resolve_local_root_from_defaults_to_start_directory() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let nested = repo.join("src");
    std::fs::create_dir_all(&nested).unwrap();

    let resolved = resolve_local_root_from(&nested, ".spec-test-only");

    assert_eq!(
        resolved,
        nested.join(CANONICAL_STORES_DIR).join("spec-test-only")
    );
}

#[test]
fn resolve_store_root_from_uses_existing_canonical_store() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let nested = repo.join("src");
    std::fs::create_dir_all(canonical_store_root(&repo, ".ticket")).unwrap();
    std::fs::create_dir_all(&nested).unwrap();

    let resolved = resolve_store_root_from(&nested, ".ticket");

    assert_eq!(resolved, canonical_store_root(&repo, ".ticket"));
}

#[test]
fn resolve_store_root_from_preserves_direct_store_root() {
    let dir = tempdir().unwrap();
    let store = canonical_store_root(dir.path(), ".ticket");
    std::fs::create_dir_all(&store).unwrap();

    let resolved = resolve_store_root_from(&store, ".ticket");

    assert_eq!(resolved, store);
}

#[test]
fn resolve_store_root_from_preserves_non_workspace_directory() {
    let dir = tempdir().unwrap();
    let scratch = dir.path().join("scratch");
    std::fs::create_dir_all(&scratch).unwrap();

    let resolved = resolve_store_root_from(&scratch, ".ticket");

    assert_eq!(resolved, scratch.join(CANONICAL_STORES_DIR).join("ticket"));
}

#[test]
fn resolve_explicit_store_root_from_does_not_use_parent_store() {
    let dir = tempdir().unwrap();
    let parent = dir.path().join("parent-workspace");
    let child = parent.join("child-workspace");
    std::fs::create_dir_all(canonical_store_root(&parent, ".spec")).unwrap();
    std::fs::create_dir_all(&child).unwrap();

    let resolution = resolve_explicit_store_root_from(&child, ".spec");

    assert_eq!(resolution.store_root, canonical_store_root(&child, ".spec"));
    assert!(resolution.diagnostics.is_empty());
}

#[test]
fn resolve_explicit_store_root_from_preserves_store_paths() {
    let dir = tempdir().unwrap();
    let store = canonical_store_root(dir.path(), ".spec");
    let nested = store.join("specs");
    std::fs::create_dir_all(&nested).unwrap();

    let direct = resolve_explicit_store_root_from(&store, ".spec");
    let nested_resolution = resolve_explicit_store_root_from(&nested, ".spec");

    assert_eq!(direct.store_root, store);
    assert_eq!(nested_resolution.store_root, store);
}

#[test]
fn resolve_explicit_store_root_from_reports_both_spec_layouts() {
    let dir = tempdir().unwrap();
    let canonical = canonical_store_root(dir.path(), ".spec");
    let legacy = dir.path().join(".spec");
    std::fs::create_dir_all(&canonical).unwrap();
    std::fs::create_dir_all(&legacy).unwrap();

    let resolution = resolve_explicit_store_root_from(dir.path(), ".spec");

    assert_eq!(resolution.store_root, canonical);
    assert!(matches!(
        resolution.diagnostics.as_slice(),
        [StoreRootDiagnostic::BothLayoutsPresent { .. }]
    ));
}

#[test]
fn resolve_explicit_legacy_store_root_prefers_canonical_when_both_exist() {
    let dir = tempdir().unwrap();
    let canonical = canonical_store_root(dir.path(), ".test");
    let legacy = dir.path().join(".test");
    std::fs::create_dir_all(&canonical).unwrap();
    std::fs::create_dir_all(&legacy).unwrap();

    let resolution = resolve_explicit_store_root_from(&legacy, ".test");

    assert_eq!(resolution.store_root, canonical);
    assert!(matches!(
        resolution.diagnostics.as_slice(),
        [StoreRootDiagnostic::BothLayoutsPresent { .. }]
    ));
}

#[test]
fn resolve_explicit_store_root_from_reports_legacy_spec_layout() {
    let dir = tempdir().unwrap();
    let legacy = dir.path().join(".spec");
    std::fs::create_dir_all(&legacy).unwrap();

    let resolution = resolve_explicit_store_root_from(dir.path(), ".spec");

    assert_eq!(resolution.store_root, legacy);
    assert!(matches!(
        resolution.diagnostics.as_slice(),
        [StoreRootDiagnostic::LegacyStore { .. }]
    ));
}

#[test]
fn store_layout_resolution_covers_every_registered_domain() {
    for (legacy_name, _) in STORE_MARKERS {
        let domain = legacy_name.trim_start_matches('.');
        let dir = tempdir().unwrap();
        let repo = dir.path().join("repo");
        let nested = repo.join("src");
        let canonical = repo.join(CANONICAL_STORES_DIR).join(domain);
        std::fs::create_dir_all(&nested).unwrap();

        std::fs::create_dir_all(&canonical).unwrap();
        let resolution = resolve_store_root_from_with_diagnostics(&nested, legacy_name);
        assert_eq!(resolution.store_root, canonical);
        assert!(resolution.diagnostics.is_empty());
        std::fs::remove_dir_all(repo.join(CANONICAL_STORES_DIR)).unwrap();

        // No store yet: resolution and initialization both target canonical.
        let resolution = resolve_store_root_at_workspace(&nested, legacy_name);
        assert_eq!(
            resolution.store_root,
            nested.join(CANONICAL_STORES_DIR).join(domain)
        );
        assert!(resolution.diagnostics.is_empty());
        assert_eq!(
            resolve_requested_store_root_for_initialization_from(
                None,
                None,
                None,
                Some(&nested),
                legacy_name,
            ),
            nested.join(CANONICAL_STORES_DIR).join(domain),
        );
    }
}

#[test]
fn resolve_requested_store_root_from_normalizes_explicit_workspace_root() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("child");
    std::fs::create_dir_all(canonical_store_root(&repo, ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".spec")).unwrap();

    let resolved =
        resolve_requested_store_root_from(None, Some(&child), None, Some(&repo), ".spec");

    assert_eq!(resolved, canonical_store_root(&child, ".spec"));
}

#[test]
fn explicit_workspace_does_not_fall_back_to_parent_store() {
    let dir = tempdir().unwrap();
    let parent = dir.path().join("meta-workspace");
    let consumer = parent.join("workflow-minimal-demo");
    std::fs::create_dir_all(canonical_store_root(&parent, ".ticket")).unwrap();
    std::fs::create_dir_all(&consumer).unwrap();

    let resolved =
        resolve_requested_store_root_from(None, Some(&consumer), None, Some(&parent), ".ticket");

    assert_eq!(resolved, consumer.join(CANONICAL_STORES_DIR).join("ticket"));
}

#[test]
fn resolve_requested_store_root_from_prefers_explicit_store_root() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("child");
    std::fs::create_dir_all(canonical_store_root(&repo, ".ticket")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".ticket")).unwrap();

    let resolved = resolve_requested_store_root_from(
        Some(&canonical_store_root(&repo, ".ticket")),
        Some(&child),
        Some(&canonical_store_root(&child, ".ticket")),
        Some(&child),
        ".ticket",
    );

    assert_eq!(resolved, canonical_store_root(&repo, ".ticket"));
}

#[test]
fn resolve_requested_store_root_from_workspace_pins_index_unless_overridden() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let sibling = dir.path().join("sibling");
    let explicit_index = dir.path().join("explicit-index");
    std::fs::create_dir_all(canonical_store_root(&repo, ".ticket")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&sibling, ".ticket")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&explicit_index, ".ticket")).unwrap();

    let workspace_selected = resolve_requested_store_root_from(
        None,
        Some(&repo),
        Some(&canonical_store_root(&sibling, ".ticket")),
        Some(&sibling),
        ".ticket",
    );
    let explicit_selected = resolve_requested_store_root_from(
        Some(&explicit_index),
        Some(&repo),
        Some(&canonical_store_root(&sibling, ".ticket")),
        Some(&sibling),
        ".ticket",
    );

    assert_eq!(workspace_selected, canonical_store_root(&repo, ".ticket"));
    assert_eq!(
        explicit_selected,
        canonical_store_root(&explicit_index, ".ticket")
    );
}

#[test]
fn resolve_requested_store_root_from_falls_back_to_local_discovery() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let nested = repo.join("tools").join("cli");
    std::fs::create_dir_all(canonical_store_root(&repo, ".ticket")).unwrap();
    std::fs::create_dir_all(&nested).unwrap();

    let resolved = resolve_requested_store_root_from(None, None, None, Some(&nested), ".ticket");

    assert_eq!(resolved, canonical_store_root(&repo, ".ticket"));
}

#[test]
fn consumer_resolver_rejects_ambiguous_superproject() {
    let dir = tempdir().unwrap();
    let superproject = dir.path().join("meta-workspace");
    let demo = superproject.join("minimal-demo");
    let example = superproject.join("context-engine");
    std::fs::create_dir_all(canonical_store_root(&demo, ".ticket")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&example, ".ticket")).unwrap();

    let error = resolve_consumer_store_root_from(None, None, None, Some(&superproject), ".ticket")
        .unwrap_err();

    assert_eq!(
        error,
        ConsumerWorkspaceError::AmbiguousSuperproject {
            workspace: superproject,
            stores: vec![
                canonical_store_root(&example, ".ticket"),
                canonical_store_root(&demo, ".ticket"),
            ],
        },
    );
}

#[test]
fn consumer_resolver_prefers_current_workspace_store_over_descendants() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("meta-workspace");
    let demo = workspace.join("minimal-demo");
    let example = workspace.join("context-engine");
    std::fs::create_dir_all(canonical_store_root(&workspace, ".session")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&demo, ".session")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&example, ".session")).unwrap();

    let resolved =
        resolve_consumer_store_root_from(None, None, None, Some(&workspace), ".session").unwrap();

    assert_eq!(resolved, canonical_store_root(&workspace, ".session"));
}

#[test]
fn consumer_resolver_allows_explicit_consumer_workspace() {
    let dir = tempdir().unwrap();
    let superproject = dir.path().join("meta-workspace");
    let demo = superproject.join("minimal-demo");
    let example = superproject.join("context-engine");
    std::fs::create_dir_all(canonical_store_root(&demo, ".ticket")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&example, ".ticket")).unwrap();

    let resolved =
        resolve_consumer_store_root_from(None, Some(&demo), None, Some(&superproject), ".ticket")
            .unwrap();

    assert_eq!(resolved, canonical_store_root(&demo, ".ticket"));
}

#[test]
fn resolve_workspace_root_from_store_root_uses_parent_of_hidden_store() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let store = canonical_store_root(&repo, ".spec");
    std::fs::create_dir_all(&store).unwrap();

    let resolved = resolve_workspace_root_from_store_root(&store, ".spec");

    assert_eq!(resolved, repo);
}

#[test]
fn resolve_workspace_root_from_store_root_preserves_direct_non_store_path() {
    let dir = tempdir().unwrap();
    let scratch = dir.path().join("scratch-store");
    std::fs::create_dir_all(&scratch).unwrap();

    let resolved = resolve_workspace_root_from_store_root(&scratch, ".spec");

    assert_eq!(resolved, scratch);
}

#[test]
fn resolve_session_store_root_from_prefers_ancestor_store() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let memory_api = repo.join("memory-viewers").join("memory-api");
    let nested = memory_api.join("crates").join("session-api");
    std::fs::create_dir_all(canonical_store_root(&memory_api, ".memory-api")).unwrap();
    std::fs::create_dir_all(&nested).unwrap();

    let resolved = resolve_session_store_root_from(Some(&nested), ".memory-api");

    assert_eq!(
        resolved,
        normalize_working_dir_path(&canonical_store_root(&memory_api, ".memory-api"))
    );
}

#[test]
fn resolve_session_store_root_from_prefers_nested_store_under_execution_root() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let memory_api = repo.join("memory-viewers").join("memory-api");
    std::fs::create_dir_all(canonical_store_root(&memory_api, ".memory-api")).unwrap();
    std::fs::create_dir_all(repo.join("src")).unwrap();

    let resolved = resolve_session_store_root_from(Some(&repo), ".memory-api");

    assert_eq!(
        resolved,
        normalize_working_dir_path(&canonical_store_root(&memory_api, ".memory-api"))
    );
}

#[test]
fn resolve_session_store_root_from_falls_back_to_execution_root() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();

    let resolved = resolve_session_store_root_from(Some(&repo), ".memory-api");

    assert_eq!(
        resolved,
        normalize_working_dir_path(&repo.join(".memory-api"))
    );
}

#[test]
fn resolve_session_store_root_from_defaults_without_cwd() {
    let resolved = resolve_session_store_root_from(None, ".memory-api");

    assert_eq!(resolved, PathBuf::from(".memory-api"));
}
