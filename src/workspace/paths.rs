use super::*;

#[derive(Debug)]
pub enum WorkspacePathError {
    CanonicalizeFailed {
        input: String,
        source: std::io::Error,
    },
    InvalidWindowsPrefix {
        input: String,
        detail: String,
    },
    UnrepresentablePath {
        input: String,
        detail: String,
    },
}

impl std::fmt::Display for WorkspacePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CanonicalizeFailed { input, source } => write!(
                f,
                "failed to canonicalize workspace root '{input}': {source}"
            ),
            Self::InvalidWindowsPrefix { input, detail } => {
                write!(f, "invalid Windows path prefix for '{input}': {detail}")
            }
            Self::UnrepresentablePath { input, detail } => {
                write!(f, "unrepresentable path '{input}': {detail}")
            }
        }
    }
}

impl std::error::Error for WorkspacePathError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CanonicalizeFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub fn working_dir() -> Option<PathBuf> {
    resolve_working_dir(
        std::env::current_dir().ok().as_deref(),
        std::env::var_os("PWD").as_deref().map(Path::new),
    )
}

pub fn normalize_path_for_display(path: &Path) -> String {
    normalize_path_for_display_impl(path).unwrap_or_else(|_| {
        let fallback = normalize_path_for_workspace_string(&path.to_string_lossy());
        normalize_drive_letter_for_display(&fallback)
    })
}

pub fn normalize_path_for_display_strict(path: &Path) -> Result<String, WorkspacePathError> {
    normalize_path_for_display_impl(path)
}

/// Canonicalize a path for use as a workspace/store root, stripping the Windows
/// `\\?\` verbatim prefix that [`std::fs::canonicalize`] emits.
///
/// The verbatim prefix must never reach stored move journals, rewritten path
/// references, or post-move validation comparisons: it breaks string matching
/// against clean paths and corrupts tracked files. Falls back to the input path
/// when canonicalization fails (for example when the directory does not exist
/// yet).
pub fn canonicalize_workspace_root(path: &Path) -> PathBuf {
    canonicalize_workspace_root_lossy(path)
}

pub fn canonicalize_workspace_root_strict(path: &Path) -> Result<PathBuf, WorkspacePathError> {
    let canonical =
        std::fs::canonicalize(path).map_err(|source| WorkspacePathError::CanonicalizeFailed {
            input: path.to_string_lossy().to_string(),
            source,
        })?;
    Ok(strip_verbatim_prefix(&canonical))
}

pub fn canonicalize_workspace_root_lossy(path: &Path) -> PathBuf {
    canonicalize_workspace_root_strict(path).unwrap_or_else(|_| strip_verbatim_prefix(path))
}

/// Remove the Windows `\\?\` (and slash-normalized `//?/`) verbatim prefix from
/// a path and normalize separators, without touching the filesystem.
pub fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    let normalized = normalize_path_for_workspace_string(&raw);
    PathBuf::from(normalized)
}

/// Enforce consistent path separators by converting all backslashes to forward slashes,
/// and strip any Windows verbatim (extended-path) prefixes.
pub fn normalize_slashes(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    let raw = raw.strip_prefix("//?/").unwrap_or(&raw);
    raw.strip_prefix(r"\\?\").unwrap_or(raw).to_string()
}

pub(super) fn normalize_path_for_display_impl(path: &Path) -> Result<String, WorkspacePathError> {
    let raw = path.to_str().map(str::to_string).ok_or_else(|| {
        WorkspacePathError::UnrepresentablePath {
            input: path.to_string_lossy().to_string(),
            detail: "path is not valid UTF-8".to_string(),
        }
    })?;

    let workspace = normalize_path_for_workspace_string_strict(&raw)?;
    Ok(normalize_drive_letter_for_display(&workspace))
}

pub(super) fn normalize_drive_letter_for_display(value: &str) -> String {
    if let Some((drive, remainder)) = split_windows_drive_prefix(value) {
        let mut out = String::with_capacity(value.len() + 1);
        out.push('/');
        out.push(drive.to_ascii_lowercase());
        if !remainder.is_empty() {
            out.push('/');
            out.push_str(remainder.trim_start_matches('/'));
        }
        return out;
    }

    value.to_string()
}

pub(super) fn normalize_path_for_workspace_string(raw: &str) -> String {
    normalize_path_for_workspace_string_impl(raw, false)
        .unwrap_or_else(|_| collapse_slashes_preserving_root(&raw.replace('\\', "/")))
}

pub(super) fn normalize_path_for_workspace_string_strict(
    raw: &str,
) -> Result<String, WorkspacePathError> {
    normalize_path_for_workspace_string_impl(raw, true)
}

pub(super) fn normalize_path_for_workspace_string_impl(
    raw: &str,
    strict: bool,
) -> Result<String, WorkspacePathError> {
    let input = raw.to_string();

    if let Some(rest) = raw
        .strip_prefix(r"\\?\UNC\")
        .or_else(|| raw.strip_prefix("//?/UNC/"))
    {
        let rest_normalized = rest.replace('\\', "/");
        if strict {
            validate_unc_remainder(&rest_normalized, &input)?;
        }
        return Ok(collapse_slashes_preserving_root(&format!(
            "//{}",
            rest_normalized
        )));
    }

    let without_verbatim = raw
        .strip_prefix(r"\\?\")
        .or_else(|| raw.strip_prefix("//?/"))
        .unwrap_or(raw);
    let normalized = collapse_slashes_preserving_root(&without_verbatim.replace('\\', "/"));

    if strict && normalized.starts_with("//") {
        let remainder = normalized.trim_start_matches('/');
        validate_unc_remainder(remainder, &input)?;
    }

    Ok(normalized)
}

pub(super) fn validate_unc_remainder(
    remainder: &str,
    input: &str,
) -> Result<(), WorkspacePathError> {
    let mut parts = remainder.split('/').filter(|part| !part.is_empty());
    let server = parts.next();
    let share = parts.next();
    if server.is_none() || share.is_none() {
        return Err(WorkspacePathError::InvalidWindowsPrefix {
            input: input.to_string(),
            detail: "UNC path must include both server and share segments".to_string(),
        });
    }
    Ok(())
}

pub(super) fn split_windows_drive_prefix(value: &str) -> Option<(char, &str)> {
    let bytes = value.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let drive = bytes[0] as char;
    if !drive.is_ascii_alphabetic() || bytes[1] != b':' {
        return None;
    }
    let remainder = &value[2..];
    Some((drive, remainder))
}

pub(super) fn collapse_slashes_preserving_root(value: &str) -> String {
    let (prefix, remainder) = if let Some(rest) = value.strip_prefix("//") {
        ("//", rest)
    } else if let Some(rest) = value.strip_prefix('/') {
        ("/", rest)
    } else {
        ("", value)
    };

    let mut out = String::with_capacity(value.len());
    out.push_str(prefix);

    let mut prev_was_slash = false;
    for ch in remainder.chars() {
        if ch == '/' {
            if !prev_was_slash {
                out.push(ch);
            }
            prev_was_slash = true;
        } else {
            out.push(ch);
            prev_was_slash = false;
        }
    }

    out
}

pub(super) fn start_dir(start: &Path) -> &Path {
    if start.is_dir() {
        start
    } else {
        start.parent().unwrap_or(start)
    }
}

pub(super) fn resolve_working_dir(cwd: Option<&Path>, pwd: Option<&Path>) -> Option<PathBuf> {
    cwd.or(pwd).map(normalize_working_dir_path)
}

pub(super) fn normalize_working_dir_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let raw = path.to_string_lossy();
        if let Some(normalized) = normalize_git_bash_pwd(&raw) {
            return PathBuf::from(normalized);
        }
        return path.to_path_buf();
    }

    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

#[cfg(windows)]
pub(super) fn normalize_git_bash_pwd(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'/' || bytes[2] != b'/' {
        return None;
    }

    let drive = bytes[1] as char;
    if !drive.is_ascii_alphabetic() {
        return None;
    }

    let mut normalized = String::with_capacity(raw.len());
    normalized.push(drive.to_ascii_uppercase());
    normalized.push(':');
    normalized.push('/');
    normalized.push_str(&raw[3..]);
    Some(normalized)
}
