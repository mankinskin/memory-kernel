use super::*;

pub(super) fn store_domain(dir_name: &str) -> &str {
    dir_name.trim_start_matches('.')
}

pub(super) fn supports_legacy_layout(dir_name: &str) -> bool {
    !matches!(dir_name, ".ticket" | ".spec")
}

/// The canonical `.workflow-tools/<domain>` location for a store at
/// `workspace`, whether or not it exists yet.
///
/// Use this when creating a new store so it lands in the canonical layout
/// instead of the legacy bare `<dir_name>` path that
/// [`resolve_store_root_at_fixed_workspace`] still returns as its default.
pub fn canonical_store_root(workspace: &Path, dir_name: &str) -> PathBuf {
    workspace
        .join(CANONICAL_STORES_DIR)
        .join(store_domain(dir_name))
}

pub(super) fn find_store_at_workspace(workspace: &Path, dir_name: &str) -> Option<PathBuf> {
    let canonical = canonical_store_root(workspace, dir_name);
    if canonical.is_dir() {
        return Some(canonical);
    }
    if !supports_legacy_layout(dir_name) {
        return None;
    }
    let legacy = workspace.join(dir_name);
    legacy.is_dir().then_some(legacy)
}

/// Resolve a store's location at a single, already-fixed workspace root,
/// without walking any ancestor directory.
///
/// This is the bounded counterpart to [`resolve_store_root_from`] for callers
/// (worktree-scoped resolvers, security-sensitive mutation gates) that must
/// not escape a specific workspace/worktree boundary by searching upward.
/// Resolution order: an existing canonical `.workflow-tools/<domain>` store,
/// then the canonical path for a not-yet-created store. Domains that still
/// support legacy layouts may resolve their bare `<dir_name>` store.
pub fn resolve_store_root_at_fixed_workspace(workspace: &Path, dir_name: &str) -> PathBuf {
    let normalized = normalize_working_dir_path(workspace);
    if is_store_root(&normalized, dir_name) {
        return normalized;
    }
    find_store_at_workspace(&normalized, dir_name)
        .unwrap_or_else(|| canonical_store_root(&normalized, dir_name))
}

pub(super) fn resolve_store_root_at_workspace(
    workspace: &Path,
    dir_name: &str,
) -> StoreRootResolution {
    let canonical_path = canonical_store_root(workspace, dir_name);
    let legacy_path = workspace.join(dir_name);
    let canonical_exists = canonical_path.is_dir();
    let legacy_exists = supports_legacy_layout(dir_name) && legacy_path.is_dir();
    let domain = store_domain(dir_name).to_string();

    let diagnostics = match (canonical_exists, legacy_exists) {
        (true, true) => vec![StoreRootDiagnostic::BothLayoutsPresent {
            domain,
            legacy_path: legacy_path.clone(),
            canonical_path: canonical_path.clone(),
        }],
        (false, true) => vec![StoreRootDiagnostic::LegacyStore {
            domain,
            legacy_path: legacy_path.clone(),
            canonical_path: canonical_path.clone(),
        }],
        _ => Vec::new(),
    };
    StoreRootResolution {
        store_root: if canonical_exists {
            canonical_path
        } else if legacy_exists {
            legacy_path
        } else {
            canonical_path
        },
        diagnostics,
    }
}

pub(super) fn resolve_explicit_store_root_at_workspace(
    workspace: &Path,
    dir_name: &str,
) -> StoreRootResolution {
    if dir_name != ".spec" {
        return resolve_store_root_at_workspace(workspace, dir_name);
    }

    let canonical_path = canonical_store_root(workspace, dir_name);
    let legacy_path = workspace.join(dir_name);
    let canonical_exists = canonical_path.is_dir();
    let legacy_exists = legacy_path.is_dir();
    let domain = store_domain(dir_name).to_string();
    let diagnostics = match (canonical_exists, legacy_exists) {
        (true, true) => vec![StoreRootDiagnostic::BothLayoutsPresent {
            domain,
            legacy_path: legacy_path.clone(),
            canonical_path: canonical_path.clone(),
        }],
        (false, true) => vec![StoreRootDiagnostic::LegacyStore {
            domain,
            legacy_path: legacy_path.clone(),
            canonical_path: canonical_path.clone(),
        }],
        _ => Vec::new(),
    };

    StoreRootResolution {
        store_root: if canonical_exists {
            canonical_path
        } else if legacy_exists {
            legacy_path
        } else {
            canonical_path
        },
        diagnostics,
    }
}

pub(super) fn resolve_store_root_at_workspace_read(
    workspace: &Path,
    dir_name: &str,
) -> StoreRootResolution {
    resolve_store_root_at_workspace(workspace, dir_name)
}

pub(super) fn is_store_root(path: &Path, dir_name: &str) -> bool {
    if path.file_name().and_then(|name| name.to_str()) == Some(dir_name) {
        return true;
    }
    path.file_name().and_then(|name| name.to_str()) == Some(store_domain(dir_name))
        && path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            == Some(CANONICAL_STORES_DIR)
}
