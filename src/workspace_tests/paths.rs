use super::*;

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
