use super::*;

pub(in crate::storage::move_kernel) fn classify_git_worktree_topology(
    source_git_root: &Path,
    target_git_root: &Path,
) -> GitWorktreeTopology {
    if source_git_root == target_git_root {
        return GitWorktreeTopology::Same;
    }
    if target_git_root.starts_with(source_git_root) {
        return GitWorktreeTopology::ParentToSubmodule;
    }
    if source_git_root.starts_with(target_git_root) {
        return GitWorktreeTopology::SubmoduleToParent;
    }
    GitWorktreeTopology::Unrelated
}

pub(in crate::storage::move_kernel) fn git_toplevel(path: &Path) -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args([
            "-C",
            &path.to_string_lossy(),
            "rev-parse",
            "--show-toplevel",
        ])
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Err("git rev-parse returned an empty worktree root".to_string());
    }

    Ok(PathBuf::from(stdout))
}

pub(in crate::storage::move_kernel) fn git_tracked_path_reference_files(
    repo_root: &Path,
    entity_path: &Path,
) -> Result<Vec<PathBuf>, String> {
    let mut candidates = BTreeSet::new();
    candidates.insert(entity_path.to_string_lossy().replace('\\', "/"));
    if let Ok(relative) = safe_strip_prefix(entity_path, repo_root) {
        candidates.insert(relative.to_string_lossy().replace('\\', "/"));
    }

    let mut files = BTreeSet::new();
    for candidate in candidates {
        let output = Command::new("git")
            .args([
                "-C",
                &repo_root.to_string_lossy(),
                "grep",
                "-nF",
                "--full-name",
                &candidate,
            ])
            .output()
            .map_err(|error| error.to_string())?;

        if !output.status.success() && output.status.code() != Some(1) {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Some((file, _)) = line.split_once(':') {
                files.insert(PathBuf::from(file));
            }
        }
    }

    Ok(files.into_iter().collect())
}

pub(in crate::storage::move_kernel) fn is_persistent_move_reference_file(
    file: &Path,
    source_store_root: &Path,
    target_store_root: &Path,
    entity_subdir: &str,
) -> bool {
    let entity_subdir = Path::new(entity_subdir);
    for store_root in [source_store_root, target_store_root] {
        let is_hidden_store = store_root.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s.starts_with('.') && !s.starts_with(".tmp")
        });
        if is_hidden_store {
            if let Ok(relative) = safe_strip_prefix(file, store_root) {
                return relative.starts_with(entity_subdir);
            }
        }
    }

    true
}

pub(in crate::storage::move_kernel) fn tracked_repo_for_file<'a>(
    file: &Path,
    source_repo_root: &'a Path,
    target_repo_root: &'a Path,
) -> Option<(&'a Path, PathBuf)> {
    let mut candidates = Vec::new();
    if let Ok(relative) = safe_strip_prefix(file, source_repo_root) {
        candidates.push((source_repo_root, relative));
    }
    if let Ok(relative) = safe_strip_prefix(file, target_repo_root) {
        candidates.push((target_repo_root, relative));
    }

    candidates.sort_by_key(|(_, relative)| std::cmp::Reverse(relative.components().count()));
    candidates.into_iter().next()
}

pub(in crate::storage::move_kernel) fn git_restore_tracked_path(
    repo_root: &Path,
    repo_relative_path: &Path,
) -> MoveResult<()> {
    let relative = repo_relative_path.to_string_lossy().replace('\\', "/");
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root.to_string_lossy(),
            "restore",
            "--worktree",
            "--source=HEAD",
            "--",
            &relative,
        ])
        .output()
        .map_err(|error| MoveError::Domain(error.to_string()))?;

    if !output.status.success() {
        return Err(MoveError::Domain(format!(
            "git restore failed for {} in {}: {}",
            relative,
            normalize_slashes(repo_root),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(())
}

pub(in crate::storage::move_kernel) fn safe_strip_prefix(
    path: &Path,
    prefix: &Path,
) -> Result<PathBuf, std::path::StripPrefixError> {
    let p_norm = PathBuf::from(crate::workspace::normalize_slashes(path));
    let s_norm = PathBuf::from(crate::workspace::normalize_slashes(prefix));
    p_norm.strip_prefix(&s_norm).map(|p| p.to_path_buf())
}
