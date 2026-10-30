use super::*;

pub fn find_descendant_store_roots_from(start: &Path, dir_name: &str) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    collect_descendant_store_roots(
        &normalize_working_dir_path(start_dir(start)),
        dir_name,
        &mut roots,
    );
    roots.sort();
    roots.dedup();
    roots
}

pub fn discover_workspace_scan_roots(
    workspace_root: &Path,
    store_dir: &str,
    entity_dir: &str,
) -> Vec<ScanRoot> {
    let normalized_root = normalize_working_dir_path(start_dir(workspace_root));
    let policy = load_workspace_policy(&normalized_root);
    discover_workspace_scan_roots_with_policy(workspace_root, store_dir, entity_dir, &policy)
}

/// Discover canonical hidden store roots for a workspace using active
/// workspace-policy rules.
pub fn discover_workspace_store_roots(
    workspace_root: &Path,
    store_dir: &str,
    entity_dir: &str,
) -> Vec<PathBuf> {
    let mut roots = discover_workspace_scan_roots(workspace_root, store_dir, entity_dir)
        .into_iter()
        .map(|root| resolve_store_root_from(&root.path, store_dir))
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();
    roots
}

/// Render a consistent workspace recovery hint for entity lookups.
///
/// The discovered stores are computed via the same policy-aware scan-root
/// resolution path used by active entity indexing.
pub fn workspace_recovery_hint_for_store(
    active_index_root: &Path,
    store_dir: &str,
    entity_dir: &str,
    store_label: &str,
) -> String {
    let active_store_root = resolve_store_root_from(active_index_root, store_dir);
    let workspace_root = resolve_workspace_root_from_store_root(&active_store_root, store_dir);
    let discovered = discover_workspace_store_roots(&workspace_root, store_dir, entity_dir)
        .into_iter()
        .map(|path| normalize_path_for_display(&path))
        .collect::<Vec<_>>();

    if discovered.is_empty() {
        return format!(
            "active index root: {}. Retry with --workspace-root <workspace-path> or --index-root <path-to-{store_dir}>",
            normalize_path_for_display(&active_store_root)
        );
    }

    format!(
        "active index root: {}\n\n\tRetry with --workspace-root <workspace-path> or --index-root <path-to-{store_dir}>.\n\nDiscovered {store_label} stores:\n- {}",
        normalize_path_for_display(&active_store_root),
        discovered.join(",\n- ")
    )
}

/// Policy-aware variant of [`discover_workspace_scan_roots`].
///
/// The supplied [`WorkspacePolicy`] governs which store roots are collected:
/// descendant discovery is gated on `include_descendants`, ancestor stores are
/// gated on `include_ancestors` and suppressed entirely when
/// `deny_external_paths` is set, and every candidate is filtered through the
/// policy's ignore globs, ignore markers, and include overrides (overrides
/// win). The active workspace root store is always included when present.
pub fn discover_workspace_scan_roots_with_policy(
    workspace_root: &Path,
    store_dir: &str,
    entity_dir: &str,
    policy: &WorkspacePolicy,
) -> Vec<ScanRoot> {
    let workspace_root = normalize_working_dir_path(start_dir(workspace_root));
    let mut store_roots = Vec::new();

    // The active workspace root store is always included when present.
    let root_store = resolve_store_root_at_workspace(&workspace_root, store_dir).store_root;
    if root_store.is_dir() {
        store_roots.push(normalize_working_dir_path(&root_store));
    }

    if policy.include_descendants {
        for store_root in find_descendant_store_roots_from(&workspace_root, store_dir) {
            let owning_workspace = resolve_workspace_root_from_store_root(&store_root, store_dir);
            if owning_workspace == workspace_root {
                continue;
            }
            if policy_allows(policy, &workspace_root, &owning_workspace) {
                store_roots.push(store_root);
            }
        }
    }

    // Ancestor stores live outside the workspace subtree, so they are only
    // eligible when external paths are permitted and ancestors are requested.
    if policy.include_ancestors && !policy.deny_external_paths {
        for ancestor in workspace_root.ancestors().skip(1) {
            let candidate = canonical_store_root(ancestor, store_dir);
            if !candidate.is_dir() {
                continue;
            }
            let owning_workspace = resolve_workspace_root_from_store_root(&candidate, store_dir);
            if policy_allows(policy, &workspace_root, &owning_workspace) {
                store_roots.push(normalize_working_dir_path(&candidate));
            }
        }
    }

    store_roots.sort();
    store_roots.dedup();

    store_roots
        .into_iter()
        .map(|store_root| {
            let owning_workspace = resolve_workspace_root_from_store_root(&store_root, store_dir);
            let label = if owning_workspace == workspace_root {
                ".".to_string()
            } else {
                owning_workspace
                    .strip_prefix(&workspace_root)
                    .ok()
                    .filter(|path| !path.as_os_str().is_empty())
                    .map(|path| path.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|| {
                        owning_workspace
                            .file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| format!("ancestor:{name}"))
                            .unwrap_or_else(|| "ancestor".to_string())
                    })
            };

            ScanRoot {
                path: normalize_working_dir_path(&store_root.join(entity_dir)),
                label,
            }
        })
        .collect()
}

/// Evaluate the policy against a candidate owning workspace.
///
/// Include overrides win over ignore globs and ignore markers. When the
/// candidate cannot be expressed relative to the workspace root (e.g. an
/// ancestor store), its normalized absolute path string is used for glob
/// matching instead.
pub(super) fn policy_allows(
    policy: &WorkspacePolicy,
    workspace_root: &Path,
    owning_workspace: &Path,
) -> bool {
    let match_path = owning_workspace
        .strip_prefix(workspace_root)
        .ok()
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| owning_workspace.to_path_buf());

    !policy.is_ignored(&match_path, owning_workspace)
}

pub(super) fn collect_descendant_store_roots(dir: &Path, dir_name: &str, roots: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if name == dir_name && supports_legacy_layout(dir_name) {
            roots.push(normalize_working_dir_path(&path));
            continue;
        }

        if name == CANONICAL_STORES_DIR {
            let canonical = canonical_store_root(dir, dir_name);
            if canonical.is_dir() {
                roots.push(normalize_working_dir_path(&canonical));
            }
            continue;
        }

        if should_skip_descendant_dir(name) {
            continue;
        }

        collect_descendant_store_roots(&path, dir_name, roots);
    }
}

pub(super) fn should_skip_descendant_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".hg" | ".svn" | ".worktrees" | "target" | "node_modules" | "release" | "tmp"
    )
}
