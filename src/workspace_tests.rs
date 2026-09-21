use crate::discovery::STORE_MARKERS;
use tempfile::tempdir;

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
fn explicit_workspace_selector_rejects_ambient_aliases() {
    for value in [None, Some(""), Some("  "), Some("default"), Some("..")] {
        assert!(validate_explicit_workspace_selector(value).is_err());
    }

    assert_eq!(
        validate_explicit_workspace_selector(Some("memory-api")).unwrap(),
        "memory-api"
    );
}

#[test]
fn explicit_workspace_selector_accepts_current_directory() {
    assert_eq!(
        validate_explicit_workspace_selector(Some(".")).unwrap(),
        "."
    );
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

    assert_eq!(
        resolved,
        scratch.join(CANONICAL_STORES_DIR).join("ticket")
    );
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

    assert_eq!(
        resolved,
        consumer.join(CANONICAL_STORES_DIR).join("ticket")
    );
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
    assert_eq!(explicit_selected, canonical_store_root(&explicit_index, ".ticket"));
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
    std::fs::create_dir_all(canonical_store_root(&repo.join("target").join("build"), ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&repo.join("node_modules").join("pkg"), ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&repo.join("release").join("notes"), ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&repo.join("tmp").join("scratch"), ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&repo.join(".git").join("worktree"), ".spec")).unwrap();
    std::fs::create_dir_all(canonical_store_root(&repo.join(".worktrees").join("sibling"), ".spec")).unwrap();

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
    std::fs::create_dir_all(
        fixtures.join(CANONICAL_STORES_DIR).join("ticket"),
    )
    .unwrap();
    std::fs::write(
        repo.join(CANONICAL_STORES_DIR)
            .join("ticket")
            .join("workspace-policy.toml"),
        "include_descendants = true\nignore_workspaces = [\"test-fixtures\"]\n",
    )
    .unwrap();

    let hint =
        workspace_recovery_hint_for_store(
            &repo.join(CANONICAL_STORES_DIR).join("ticket"),
            ".ticket",
            "tickets",
            "ticket",
        );

    assert!(hint.contains("Discovered ticket stores"));
    assert!(!hint.contains("test-fixtures/.ticket"));
}

#[test]
fn resolve_workspace_from_reports_default_local_ticket() {
    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();

    let (path, source) = resolve_workspace_from(&repo);

    assert_eq!(path, repo.join(CANONICAL_STORES_DIR).join("ticket"));
    assert_eq!(
        source,
        WorkspaceSource::Default(repo.join(CANONICAL_STORES_DIR).join("ticket"))
    );
}

#[test]
fn resolve_working_dir_prefers_cwd() {
    let cwd = Path::new("repo/current");
    let pwd = Path::new("repo/pwd");

    let resolved = resolve_working_dir(Some(cwd), Some(pwd));

    assert_eq!(resolved, Some(normalize_working_dir_path(cwd)));
}

#[test]
fn resolve_working_dir_falls_back_to_pwd() {
    let pwd = Path::new("repo/pwd");

    let resolved = resolve_working_dir(None, Some(pwd));

    assert_eq!(resolved, Some(normalize_working_dir_path(pwd)));
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

#[cfg(windows)]
#[test]
fn normalize_working_dir_path_converts_backslashes() {
    let normalized = normalize_working_dir_path(Path::new(r"C:\repo\memory-api"));

    assert_eq!(normalized, PathBuf::from("C:/repo/memory-api"));
}

#[cfg(windows)]
#[test]
fn normalize_working_dir_path_converts_git_bash_pwd() {
    let normalized = normalize_working_dir_path(Path::new("/c/repo/memory-api"));

    assert_eq!(normalized, PathBuf::from("C:/repo/memory-api"));
}

#[cfg(windows)]
#[test]
fn strip_verbatim_prefix_removes_windows_extended_length_prefix() {
    let stripped = strip_verbatim_prefix(Path::new(r"\\?\C:\repo\memory-api\.ticket"));

    assert_eq!(stripped, PathBuf::from("C:/repo/memory-api/.ticket"));
}

#[cfg(windows)]
#[test]
fn strip_verbatim_prefix_normalizes_verbatim_unc_prefix() {
    let stripped = strip_verbatim_prefix(Path::new(r"\\?\UNC\server\share\memory-api\.ticket"));

    assert_eq!(stripped, PathBuf::from("//server/share/memory-api/.ticket"));
}

#[cfg(windows)]
#[test]
fn strip_verbatim_prefix_preserves_unc_root() {
    let stripped = strip_verbatim_prefix(Path::new(r"\\server\share\memory-api\.ticket"));

    assert_eq!(stripped, PathBuf::from("//server/share/memory-api/.ticket"));
}

#[test]
fn strip_verbatim_prefix_removes_slash_normalized_prefix() {
    let stripped = strip_verbatim_prefix(Path::new("//?/C:/repo/memory-api/.ticket"));

    assert_eq!(stripped, PathBuf::from("C:/repo/memory-api/.ticket"));
}

#[test]
fn strip_verbatim_prefix_is_noop_for_clean_paths() {
    let clean = Path::new("C:/repo/memory-api/.ticket");

    assert_eq!(
        strip_verbatim_prefix(clean),
        PathBuf::from("C:/repo/memory-api/.ticket")
    );
}

#[test]
fn canonicalize_workspace_root_never_emits_verbatim_prefix() {
    let dir = tempdir().unwrap();
    let resolved = canonicalize_workspace_root(dir.path());

    let rendered = resolved.to_string_lossy();
    assert!(
        !rendered.contains("//?/") && !rendered.contains(r"\\?\"),
        "canonicalized workspace root leaked a verbatim prefix: {rendered}"
    );
}
