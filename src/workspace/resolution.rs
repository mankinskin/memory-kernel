use super::*;

pub fn find_local_root(dir_name: &str) -> Option<PathBuf> {
    let cwd = working_dir()?;
    find_local_root_from(&cwd, dir_name)
}

pub fn find_local_root_from(start: &Path, dir_name: &str) -> Option<PathBuf> {
    let mut dir = start_dir(start);
    loop {
        if let Some(store_root) = find_store_at_workspace(dir, dir_name) {
            return Some(store_root);
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return None,
        }
    }
}

pub fn resolve_local_root(dir_name: &str) -> PathBuf {
    working_dir()
        .map(|cwd| resolve_local_root_from(&cwd, dir_name))
        .unwrap_or_else(|| PathBuf::from(dir_name))
}

pub fn resolve_local_root_from(start: &Path, dir_name: &str) -> PathBuf {
    find_local_root_from(start, dir_name)
        .unwrap_or_else(|| canonical_store_root(start_dir(start), dir_name))
}

pub fn resolve_store_root_from(start: &Path, dir_name: &str) -> PathBuf {
    resolve_store_root_from_with_diagnostics(start, dir_name).store_root
}

pub fn resolve_store_root_from_with_diagnostics(
    start: &Path,
    dir_name: &str,
) -> StoreRootResolution {
    let normalized = normalize_working_dir_path(start);
    if is_store_root(&normalized, dir_name) {
        return StoreRootResolution {
            store_root: normalized,
            diagnostics: Vec::new(),
        };
    }

    let dir = start_dir(&normalized);
    if is_store_root(dir, dir_name) {
        return resolve_store_root_at_workspace(
            &resolve_workspace_root_from_store_root(dir, dir_name),
            dir_name,
        );
    }

    let mut workspace = dir;
    loop {
        if find_store_at_workspace(workspace, dir_name).is_some() {
            return resolve_store_root_at_workspace(workspace, dir_name);
        }
        match workspace.parent() {
            Some(parent) => workspace = parent,
            None => {
                return StoreRootResolution {
                    store_root: canonical_store_root(start_dir(&normalized), dir_name),
                    diagnostics: Vec::new(),
                };
            }
        }
    }
}

/// Resolve an explicit workspace without searching sibling or ancestor
/// workspaces for a store. Paths already inside a store resolve to that store.
pub fn resolve_explicit_store_root_from(start: &Path, dir_name: &str) -> StoreRootResolution {
    let normalized = normalize_working_dir_path(start);
    let mut candidate = normalized.as_path();
    loop {
        if is_store_root(candidate, dir_name) {
            let workspace = resolve_workspace_root_from_store_root(candidate, dir_name);
            return resolve_explicit_store_root_at_workspace(&workspace, dir_name);
        }
        match candidate.parent() {
            Some(parent) => candidate = parent,
            None => break,
        }
    }

    resolve_explicit_store_root_at_workspace(&normalized, dir_name)
}

pub fn resolve_requested_store_root(
    explicit_store_root: Option<&Path>,
    explicit_workspace_root: Option<&Path>,
    env_store_root: Option<&Path>,
    dir_name: &str,
) -> PathBuf {
    resolve_requested_store_root_with_diagnostics(
        explicit_store_root,
        explicit_workspace_root,
        env_store_root,
        dir_name,
    )
    .store_root
}

pub fn resolve_requested_store_root_with_diagnostics(
    explicit_store_root: Option<&Path>,
    explicit_workspace_root: Option<&Path>,
    env_store_root: Option<&Path>,
    dir_name: &str,
) -> StoreRootResolution {
    let cwd = working_dir();
    resolve_requested_store_root_from_with_diagnostics(
        explicit_store_root,
        explicit_workspace_root,
        env_store_root,
        cwd.as_deref(),
        dir_name,
    )
}

pub fn resolve_requested_store_root_from(
    explicit_store_root: Option<&Path>,
    explicit_workspace_root: Option<&Path>,
    env_store_root: Option<&Path>,
    cwd: Option<&Path>,
    dir_name: &str,
) -> PathBuf {
    resolve_requested_store_root_from_with_diagnostics(
        explicit_store_root,
        explicit_workspace_root,
        env_store_root,
        cwd,
        dir_name,
    )
    .store_root
}

pub fn resolve_requested_store_root_from_with_diagnostics(
    explicit_store_root: Option<&Path>,
    explicit_workspace_root: Option<&Path>,
    env_store_root: Option<&Path>,
    cwd: Option<&Path>,
    dir_name: &str,
) -> StoreRootResolution {
    if let Some(path) = explicit_store_root {
        return resolve_store_root_from_with_diagnostics(path, dir_name);
    }

    if let Some(path) = explicit_workspace_root {
        let workspace = normalize_working_dir_path(start_dir(path));
        return resolve_store_root_at_workspace_read(&workspace, dir_name);
    }

    if let Some(path) = env_store_root {
        return resolve_store_root_from_with_diagnostics(path, dir_name);
    }

    if let Some(cwd) = cwd {
        if let Some(store_root) = find_local_root_from(cwd, dir_name) {
            let workspace = resolve_workspace_root_from_store_root(&store_root, dir_name);
            return resolve_store_root_at_workspace(&workspace, dir_name);
        }
        return StoreRootResolution {
            store_root: canonical_store_root(start_dir(cwd), dir_name),
            diagnostics: Vec::new(),
        };
    }

    StoreRootResolution {
        store_root: PathBuf::from(dir_name),
        diagnostics: Vec::new(),
    }
}

/// Resolve a canonical target for store initialization or writes.
///
/// Unlike the compatibility read resolvers, this never returns a legacy
/// hidden-store path.
pub fn resolve_store_root_for_initialization_from(
    workspace_root: &Path,
    dir_name: &str,
) -> PathBuf {
    let workspace = resolve_workspace_root_from_store_root(workspace_root, dir_name);
    canonical_store_root(&workspace, dir_name)
}

/// Resolve the canonical target for a requested store initialization or write.
pub fn resolve_requested_store_root_for_initialization_from(
    explicit_store_root: Option<&Path>,
    explicit_workspace_root: Option<&Path>,
    env_store_root: Option<&Path>,
    cwd: Option<&Path>,
    dir_name: &str,
) -> PathBuf {
    let root = explicit_store_root
        .or(explicit_workspace_root)
        .or(env_store_root)
        .or(cwd)
        .map(start_dir)
        .unwrap_or_else(|| Path::new(""));
    resolve_store_root_for_initialization_from(root, dir_name)
}

/// Resolve a consumer store using the current working directory as `cwd`.
///
/// See [`resolve_consumer_store_root_from`] for resolution semantics.
pub fn resolve_consumer_store_root(
    explicit_store_root: Option<&Path>,
    explicit_workspace_root: Option<&Path>,
    env_store_root: Option<&Path>,
    dir_name: &str,
) -> Result<PathBuf, ConsumerWorkspaceError> {
    let cwd = working_dir();
    resolve_consumer_store_root_from(
        explicit_store_root,
        explicit_workspace_root,
        env_store_root,
        cwd.as_deref(),
        dir_name,
    )
}

/// Resolve a consumer store without silently selecting a sibling workspace
/// from a superproject directory.
///
/// Explicit store, workspace, and environment selections preserve the existing
/// precedence order. An ambient invocation first uses the local workspace
/// store discovered from the current directory. Without a local store, the
/// current directory may contain at most one descendant store.
pub fn resolve_consumer_store_root_from(
    explicit_store_root: Option<&Path>,
    explicit_workspace_root: Option<&Path>,
    env_store_root: Option<&Path>,
    cwd: Option<&Path>,
    dir_name: &str,
) -> Result<PathBuf, ConsumerWorkspaceError> {
    if explicit_store_root.is_some()
        || explicit_workspace_root.is_some()
        || env_store_root.is_some()
    {
        return Ok(resolve_requested_store_root_from(
            explicit_store_root,
            explicit_workspace_root,
            env_store_root,
            cwd,
            dir_name,
        ));
    }

    let Some(cwd) = cwd else {
        return Ok(PathBuf::from(dir_name));
    };
    if find_local_root_from(cwd, dir_name).is_some() {
        return Ok(resolve_requested_store_root_from(
            None,
            None,
            None,
            Some(cwd),
            dir_name,
        ));
    }
    let workspace = normalize_working_dir_path(start_dir(cwd));
    let stores = find_descendant_store_roots_from(&workspace, dir_name);
    if stores.len() > 1 {
        return Err(ConsumerWorkspaceError::AmbiguousSuperproject { workspace, stores });
    }

    Ok(resolve_requested_store_root_from(
        None,
        None,
        None,
        Some(cwd),
        dir_name,
    ))
}

/// Resolve a session-style store root relative to the tool execution directory.
///
/// Unlike [`resolve_local_root_from`], which only walks upward, this helper also
/// prefers an existing hidden store nested *below* the execution root. This is
/// required for stores that live inside a submodule (for example the
/// `memory-kernel` workspace's `.memory-kernel` directory): running the tool from the
/// repository root must reuse that nested store instead of creating a duplicate
/// hidden directory at the root.
///
/// Resolution order:
/// 1. An existing store discovered by walking upward from `cwd`.
/// 2. An existing store nested under `cwd` (shallowest first).
/// 3. A hidden store at the execution root (`cwd/<dir_name>`).
pub fn resolve_session_store_root_from(cwd: Option<&Path>, dir_name: &str) -> PathBuf {
    resolve_session_store_root_from_with_diagnostics(cwd, dir_name).store_root
}

pub fn resolve_session_store_root_from_with_diagnostics(
    cwd: Option<&Path>,
    dir_name: &str,
) -> StoreRootResolution {
    let Some(cwd) = cwd else {
        return StoreRootResolution {
            store_root: PathBuf::from(dir_name),
            diagnostics: Vec::new(),
        };
    };

    if let Some(existing) = find_local_root_from(cwd, dir_name) {
        let workspace = resolve_workspace_root_from_store_root(&existing, dir_name);
        return resolve_store_root_at_workspace(&workspace, dir_name);
    }

    if let Some(nested) = find_descendant_store_roots_from(cwd, dir_name)
        .into_iter()
        .next()
    {
        let workspace = resolve_workspace_root_from_store_root(&nested, dir_name);
        return resolve_store_root_at_workspace(&workspace, dir_name);
    }

    StoreRootResolution {
        store_root: start_dir(cwd).join(dir_name),
        diagnostics: Vec::new(),
    }
}

pub fn resolve_workspace_root_from_store_root(store_root: &Path, dir_name: &str) -> PathBuf {
    let normalized = normalize_working_dir_path(store_root);
    if is_store_root(&normalized, dir_name) {
        if normalized
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            == Some(CANONICAL_STORES_DIR)
        {
            return normalized
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or(normalized);
        }
        return normalized
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(normalized);
    }

    normalized
}
