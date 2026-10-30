//! Local index-root discovery helpers shared across the CLI tools.
//!
//! Tool callers handle explicit overrides (`--index-root`, env vars) first.
//! This module resolves repo-local roots by walking upward from the current
//! directory and falling back to a sibling hidden folder in the current repo.

use std::path::{Path, PathBuf};

use crate::{
    model::filesystem::ScanRoot,
    workspace_policy::{WorkspacePolicy, load_workspace_policy},
};

pub const TICKET_INDEX_DIR: &str = ".ticket";

pub const CANONICAL_STORES_DIR: &str = ".workflow-tools";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreRootDiagnostic {
    LegacyStore {
        domain: String,
        legacy_path: PathBuf,
        canonical_path: PathBuf,
    },
    BothLayoutsPresent {
        domain: String,
        legacy_path: PathBuf,
        canonical_path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRootResolution {
    pub store_root: PathBuf,
    pub diagnostics: Vec<StoreRootDiagnostic>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumerWorkspaceError {
    AmbiguousSuperproject {
        workspace: PathBuf,
        stores: Vec<PathBuf>,
    },
}

impl std::fmt::Display for ConsumerWorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AmbiguousSuperproject { workspace, stores } => write!(
                f,
                "workspace '{}' contains multiple consumer stores ({}); select a consumer workspace explicitly",
                workspace.display(),
                stores
                    .iter()
                    .map(|store| store.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        }
    }
}

impl std::error::Error for ConsumerWorkspaceError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSource {
    Discovered(PathBuf),
    Default(PathBuf),
}

impl WorkspaceSource {
    pub fn description(&self) -> String {
        match self {
            Self::Discovered(path) => {
                format!("discovered local .ticket ({})", path.display())
            }
            Self::Default(path) => {
                format!("default local .ticket ({})", path.display())
            }
        }
    }
}

pub fn resolve_workspace() -> (PathBuf, WorkspaceSource) {
    working_dir()
        .map(|cwd| resolve_workspace_from(&cwd))
        .unwrap_or_else(|| {
            let path = canonical_store_root(Path::new("."), TICKET_INDEX_DIR);
            (path.clone(), WorkspaceSource::Default(path))
        })
}

pub fn resolve_workspace_from(start: &Path) -> (PathBuf, WorkspaceSource) {
    if let Some(path) = find_local_root_from(start, TICKET_INDEX_DIR) {
        return (path.clone(), WorkspaceSource::Discovered(path));
    }

    let path = canonical_store_root(start_dir(start), TICKET_INDEX_DIR);
    (path.clone(), WorkspaceSource::Default(path))
}

mod discovery;
mod layout;
mod paths;
mod resolution;
mod selectors;
pub use discovery::*;
pub use layout::*;
use layout::{
    find_store_at_workspace, is_store_root, resolve_explicit_store_root_at_workspace,
    resolve_store_root_at_workspace, resolve_store_root_at_workspace_read,
};
#[cfg(test)]
use paths::resolve_working_dir;
pub use paths::*;
use paths::{normalize_working_dir_path, start_dir};
pub use resolution::*;
pub use selectors::*;

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;
