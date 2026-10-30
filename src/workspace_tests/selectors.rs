use super::*;

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
fn explicit_workspace_selector_normalizes_current_directory() {
    let current_dir = std::env::current_dir().unwrap();

    for selector in [".", "  .  "] {
        assert_eq!(
            normalize_explicit_workspace_selector(Some(selector)).unwrap(),
            normalize_working_dir_path(&current_dir)
        );
    }
}

#[test]
fn explicit_workspace_normalization_rejects_ambient_aliases() {
    for selector in [None, Some(""), Some("  "), Some("default"), Some("..")] {
        let error = normalize_explicit_workspace_selector(selector).unwrap_err();

        assert!(matches!(
            error,
            WorkspaceSelectorNormalizationError::Invalid(_)
        ));
        assert!(
            error
                .to_string()
                .contains("requires an explicit workspace path")
        );
        assert!(std::error::Error::source(&error).is_some());
    }
}

#[test]
fn explicit_workspace_normalization_preserves_concrete_paths_without_initializing() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("not-created");
    let selector = workspace.to_str().unwrap();

    assert_eq!(
        normalize_explicit_workspace_selector(Some(selector)).unwrap(),
        workspace
    );
    assert!(!workspace.exists());
}

#[test]
fn explicit_legacy_store_write_path_is_rejected_regardless_of_existence() {
    for exists in [false, true] {
        let dir = tempdir().unwrap();
        let legacy = dir.path().join(".test");
        if exists {
            std::fs::create_dir_all(&legacy).unwrap();
        }

        let error = validate_explicit_store_root_for_write(&legacy, ".test").unwrap_err();

        assert!(error.to_string().contains("not allowed for writes"));
        assert_eq!(error.path, legacy);
    }

    let dir = tempdir().unwrap();
    for (path, dir_name) in [
        (dir.path().join(".ticket"), ".ticket"),
        (dir.path().join(".test").join("specs"), ".test"),
    ] {
        assert!(validate_explicit_store_root_for_write(&path, dir_name).is_err());
    }
}

#[test]
fn explicit_canonical_store_write_path_is_allowed() {
    let dir = tempdir().unwrap();
    let canonical = canonical_store_root(dir.path(), ".test");

    assert!(validate_explicit_store_root_for_write(&canonical, ".test").is_ok());
}
