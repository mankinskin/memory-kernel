use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidWorkspaceSelector {
    value: String,
}

impl InvalidWorkspaceSelector {
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for InvalidWorkspaceSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid workspace selector '{}': entity creation requires an explicit workspace path; do not use omitted, empty, 'default', or '..'; use '.' explicitly to target the MCP server process's current working directory",
            self.value
        )
    }
}

impl std::error::Error for InvalidWorkspaceSelector {}

#[derive(Debug)]
pub enum WorkspaceSelectorNormalizationError {
    Invalid(InvalidWorkspaceSelector),
    CurrentDirectory(std::io::Error),
}

impl std::fmt::Display for WorkspaceSelectorNormalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(error) => error.fmt(f),
            Self::CurrentDirectory(error) => {
                write!(f, "failed to resolve current workspace: {error}")
            }
        }
    }
}

impl std::error::Error for WorkspaceSelectorNormalizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Invalid(error) => Some(error),
            Self::CurrentDirectory(error) => Some(error),
        }
    }
}

pub fn validate_explicit_workspace_selector(
    workspace: Option<&str>,
) -> Result<&str, InvalidWorkspaceSelector> {
    let Some(workspace) = workspace else {
        return Err(InvalidWorkspaceSelector {
            value: "<omitted>".to_string(),
        });
    };
    let trimmed = workspace.trim();
    if matches!(trimmed, "" | "default" | "..") {
        return Err(InvalidWorkspaceSelector {
            value: trimmed.to_string(),
        });
    }
    Ok(trimmed)
}

pub fn normalize_explicit_workspace_selector(
    workspace: Option<&str>,
) -> Result<PathBuf, WorkspaceSelectorNormalizationError> {
    let selector = validate_explicit_workspace_selector(workspace)
        .map_err(WorkspaceSelectorNormalizationError::Invalid)?;
    if selector == "." {
        std::env::current_dir()
            .map(|path| normalize_working_dir_path(&path))
            .map_err(WorkspaceSelectorNormalizationError::CurrentDirectory)
    } else {
        Ok(PathBuf::from(selector))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyStoreWriteOverride {
    pub(super) path: PathBuf,
}

impl std::fmt::Display for LegacyStoreWriteOverride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "explicit legacy store path '{}' is not allowed for writes; select its workspace root or a canonical `.workflow-tools` store",
            self.path.display()
        )
    }
}

impl std::error::Error for LegacyStoreWriteOverride {}

pub fn validate_explicit_store_root_for_write(
    path: &Path,
    dir_name: &str,
) -> Result<(), LegacyStoreWriteOverride> {
    let normalized = normalize_working_dir_path(path);
    if normalized
        .ancestors()
        .any(|ancestor| ancestor.file_name().and_then(|name| name.to_str()) == Some(dir_name))
    {
        return Err(LegacyStoreWriteOverride { path: normalized });
    }
    Ok(())
}
