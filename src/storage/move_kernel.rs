//! Domain-neutral cross-workspace move kernel.
//!
//! This module owns the *generic* machinery for moving a filesystem-backed
//! entity (a folder identified by a [`Uuid`]) between two stores that may live
//! in the same git worktree or in a parent/submodule relationship:
//!
//! - read-only preflight planning ([`plan_move`]),
//! - journaled / resumable / rollbackable execution
//!   ([`execute_move`], [`resume_move`], [`rollback_move`]),
//! - set-level preflight and execution for an arbitrary, deterministically
//!   normalized selection of entities ([`plan_move_set`],
//!   [`execute_move_set`]), which share store-root/git-topology resolution
//!   and lock acquisition across the whole set instead of once per entity,
//! - git worktree topology classification, tracked-path-reference scanning and
//!   rewriting, dirty-file detection, lock-set management, and journal
//!   persistence.
//!
//! All domain-specific behavior (entity path resolution, edge/reference
//! enumeration, destination visibility, board/lease detection and historical
//! migration, store rescans) is injected through the [`MoveDomain`] trait, so
//! every domain store (ticket, spec, rule, audit, session, feedback) can opt
//! into the same safe move featureset without copying logic.
//!
//! The kernel contains **no** ticket-specific types: identities are bare
//! [`Uuid`]s and board rows are the shared [`BoardEntry`].

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::field::Empty;
use uuid::Uuid;

use crate::storage::board::BoardEntry;

const MOVE_LOCKS_DIR: &str = "move-locks";
const MOVE_JOURNALS_DIR: &str = "move-journals";
const MOVE_TRACE_TARGET: &str = "memory_kernel::storage::move_kernel";
const MOVE_SET_JOURNAL_CHECKPOINT_INTERVAL: usize = 8;

#[path = "move_kernel/internal.rs"]
mod internal;
use internal::*;
pub use internal::{
    collect_lock_paths, load_journal, load_move_set_journal, persist_journal,
    persist_move_set_journal,
};
/// Error type for the move kernel.
///
/// Domain trait implementations map their own storage errors onto
/// [`MoveError::Domain`]; the kernel raises [`MoveError::Io`] for filesystem
/// failures it performs directly (rename, lock files, journal persistence).

#[path = "move_kernel_types.rs"]
mod move_kernel_types;
use move_kernel_types::path_buf_is_empty;
pub use move_kernel_types::*;

mod execution;
mod planning;
mod set_execution;
pub use execution::{execute_move, resume_move, rollback_move};
use execution::{execute_or_resume, journal_from_plan, rollback_move_within_lock};
pub use planning::{plan_move, plan_move_set};
pub use set_execution::{execute_move_set, resume_move_set, rollback_move_set};

#[cfg(test)]
mod move_set_tests;
