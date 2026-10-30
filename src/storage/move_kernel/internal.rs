use super::*;
use crate::InteroperableArtifact;
use std::collections::BTreeMap;

pub(super) fn resolve_target_git_root_or_block(
    target_workspace_root: &Path,
    source_git_root: &Path,
    blockers: &mut Vec<MoveBlocker>,
) -> PathBuf {
    match git_toplevel(target_workspace_root) {
        Ok(root) => root,
        Err(reason) => {
            blockers.push(MoveBlocker::PathReferenceScanUnavailable { reason });
            source_git_root.to_path_buf()
        }
    }
}

#[path = "internal/git.rs"]
mod git;
#[path = "internal/locking.rs"]
mod locking;
#[path = "internal/persistence.rs"]
mod persistence;
#[path = "internal/phases.rs"]
mod phases;
#[path = "internal/references.rs"]
mod references;
pub(super) use git::*;
pub use locking::collect_lock_paths;
pub(super) use locking::{
    acquire_lock_set, phase_name, record_phase_timing, recovery_hint_for_phase, release_lock_set,
};
#[cfg(test)]
pub(super) use persistence::move_set_journal_path;
pub(super) use persistence::normalize_journal_entity_paths;
pub use persistence::{
    load_journal, load_move_set_journal, persist_journal, persist_move_set_journal,
};
pub(super) use phases::*;
pub(super) use references::*;
