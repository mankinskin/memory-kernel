use super::*;

#[test]
fn find_descendant_store_roots_from_discovers_nested_hidden_stores() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("memory-api");
    let nested = child.join("tools").join("cli");
    std::fs::create_dir_all(canonical_store_root(&repo, ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".spec")).unwrap();
    std::fs::create_dir_all(&nested).unwrap();

    let roots = find_descendant_store_roots_from(&repo, ".spec");

    assert_eq!(
        roots,
        vec![
            canonical_store_root(&repo, ".spec"),
            canonical_store_root(&child, ".spec"),
        ]
    );
}

#[test]
fn find_descendant_store_roots_from_skips_known_non_workspace_dirs() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("memory-api");
    std::fs::create_dir_all(canonical_store_root(&repo, ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(
        &repo.join("target").join("build"),
        ".spec",
    ))
    .unwrap();
    std::fs::create_dir_all(canonical_store_root(
        &repo.join("node_modules").join("pkg"),
        ".spec",
    ))
    .unwrap();
    std::fs::create_dir_all(canonical_store_root(
        &repo.join("release").join("notes"),
        ".spec",
    ))
    .unwrap();
    std::fs::create_dir_all(canonical_store_root(
        &repo.join("tmp").join("scratch"),
        ".spec",
    ))
    .unwrap();
    std::fs::create_dir_all(canonical_store_root(
        &repo.join(".git").join("worktree"),
        ".spec",
    ))
    .unwrap();
    std::fs::create_dir_all(canonical_store_root(
        &repo.join(".worktrees").join("sibling"),
        ".spec",
    ))
    .unwrap();

    let roots = find_descendant_store_roots_from(&repo, ".spec");

    assert_eq!(
        roots,
        vec![
            canonical_store_root(&repo, ".spec"),
            canonical_store_root(&child, ".spec"),
        ]
    );
}

#[test]
fn discover_workspace_scan_roots_maps_store_roots_to_entity_roots() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("memory-api");
    std::fs::create_dir_all(canonical_store_root(&repo, ".rule")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".rule")).unwrap();

    let roots = discover_workspace_scan_roots(&repo, ".rule", "rules");

    assert_eq!(
        roots,
        vec![
            ScanRoot {
                path: canonical_store_root(&repo, ".rule").join("rules"),
                label: ".".to_string(),
            },
            ScanRoot {
                path: canonical_store_root(&child, ".rule").join("rules"),
                label: "memory-api".to_string(),
            },
        ]
    );
}

#[test]
fn discover_workspace_scan_roots_includes_ancestor_store_roots() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("memory-viewers").join("memory-api");
    std::fs::create_dir_all(canonical_store_root(&repo, ".rule")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".rule")).unwrap();

    let roots = discover_workspace_scan_roots(&child, ".rule", "rules");

    assert_eq!(
        roots,
        vec![
            ScanRoot {
                path: canonical_store_root(&repo, ".rule").join("rules"),
                label: "ancestor:repo".to_string(),
            },
            ScanRoot {
                path: canonical_store_root(&child, ".rule").join("rules"),
                label: ".".to_string(),
            },
        ]
    );
}

#[test]
fn policy_gates_descendant_discovery() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("memory-api");
    std::fs::create_dir_all(canonical_store_root(&repo, ".rule")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".rule")).unwrap();

    let policy = WorkspacePolicy {
        include_descendants: false,
        ..WorkspacePolicy::default()
    };
    let roots = discover_workspace_scan_roots_with_policy(&repo, ".rule", "rules", &policy);

    // Only the active workspace root store remains.
    assert_eq!(
        roots,
        vec![ScanRoot {
            path: canonical_store_root(&repo, ".rule").join("rules"),
            label: ".".to_string(),
        }]
    );
}

#[test]
fn policy_gates_ancestor_inclusion() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("memory-viewers").join("memory-api");
    std::fs::create_dir_all(canonical_store_root(&repo, ".rule")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".rule")).unwrap();

    // Ancestors excluded when include_ancestors is false.
    let policy = WorkspacePolicy {
        include_ancestors: false,
        deny_external_paths: false,
        ..WorkspacePolicy::default()
    };
    let roots = discover_workspace_scan_roots_with_policy(&child, ".rule", "rules", &policy);
    assert_eq!(
        roots,
        vec![ScanRoot {
            path: canonical_store_root(&child, ".rule").join("rules"),
            label: ".".to_string(),
        }]
    );
}

#[test]
fn deny_external_paths_suppresses_ancestors() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("memory-viewers").join("memory-api");
    std::fs::create_dir_all(canonical_store_root(&repo, ".rule")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".rule")).unwrap();

    // include_ancestors requested but external paths denied.
    let policy = WorkspacePolicy {
        include_ancestors: true,
        deny_external_paths: true,
        ..WorkspacePolicy::default()
    };
    let roots = discover_workspace_scan_roots_with_policy(&child, ".rule", "rules", &policy);
    assert_eq!(
        roots,
        vec![ScanRoot {
            path: canonical_store_root(&child, ".rule").join("rules"),
            label: ".".to_string(),
        }]
    );
}

#[test]
fn ignore_glob_excludes_descendant_and_override_reincludes() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let fixtures = repo.join("test-fixtures");
    std::fs::create_dir_all(canonical_store_root(&repo, ".rule")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&fixtures, ".rule")).unwrap();

    let ignored = WorkspacePolicy {
        ignore_workspaces: vec!["test-fixtures*".to_string()],
        ..WorkspacePolicy::default()
    };
    let roots = discover_workspace_scan_roots_with_policy(&repo, ".rule", "rules", &ignored);
    assert_eq!(
        roots,
        vec![ScanRoot {
            path: canonical_store_root(&repo, ".rule").join("rules"),
            label: ".".to_string(),
        }]
    );

    let overridden = WorkspacePolicy {
        ignore_workspaces: vec!["test-fixtures*".to_string()],
        include_overrides: vec!["test-fixtures".to_string()],
        ..WorkspacePolicy::default()
    };
    let roots = discover_workspace_scan_roots_with_policy(&repo, ".rule", "rules", &overridden);
    assert!(roots.iter().any(|r| r.label == "test-fixtures"));
}

#[test]
fn ignore_marker_excludes_descendant() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let child = repo.join("child");
    std::fs::create_dir_all(canonical_store_root(&repo, ".rule")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&child, ".rule")).unwrap();
    std::fs::write(child.join(".ticket-ignore"), "").unwrap();

    let policy = WorkspacePolicy::default();
    let roots = discover_workspace_scan_roots_with_policy(&repo, ".rule", "rules", &policy);
    assert_eq!(
        roots,
        vec![ScanRoot {
            path: canonical_store_root(&repo, ".rule").join("rules"),
            label: ".".to_string(),
        }]
    );
}

#[test]
fn workspace_recovery_hint_uses_policy_aware_discovery() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    let fixtures = repo.join("test-fixtures");
    std::fs::create_dir_all(repo.join(CANONICAL_STORES_DIR).join("ticket")).unwrap();
    std::fs::create_dir_all(fixtures.join(CANONICAL_STORES_DIR).join("ticket")).unwrap();
    std::fs::write(
        repo.join(CANONICAL_STORES_DIR)
            .join("ticket")
            .join("workspace-policy.toml"),
        "include_descendants = true\nignore_workspaces = [\"test-fixtures\"]\n",
    )
    .unwrap();

    let hint = workspace_recovery_hint_for_store(
        &repo.join(CANONICAL_STORES_DIR).join("ticket"),
        ".ticket",
        "tickets",
        "ticket",
    );

    assert!(hint.contains("Discovered ticket stores"));
    assert!(!hint.contains("test-fixtures/.ticket"));
}
