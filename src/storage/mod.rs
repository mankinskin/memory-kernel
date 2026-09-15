pub mod board;
pub mod entity_fs;
pub mod entity_store;
pub mod index;
pub mod indexed;
pub mod local_root;
pub mod move_kernel;
pub mod open_or_init;
pub mod schema;
pub mod search;
pub mod watcher;

pub use board::{
    BoardCleanPreview, BoardCleanResult, BoardConfig, BoardEntry, BoardEntryStatus, BoardError,
    BoardReconcileResult, BoardSnapshot, ReconcileAction,
};
pub use entity_store::EntityStore;
pub use local_root::{ensure_gitignore_entries, ensure_sqlite_index_root};
pub use move_kernel::{
    execute_move_set, normalize_entity_selection, plan_move_set, resume_move_set,
    rollback_move_set, GitWorktreeTopology, MoveBlocker, MoveBoardState, MoveDomain, MoveError,
    MoveExecutionPhase, MoveJournal, MoveLeaseBlock, MoveManualFollowup, MoveOutcome,
    MovePathRewrite, MovePlan, MoveReferenceDirection, MoveReferenceVisibility, MoveReferences,
    MoveResult, MoveSetExecutionPhase, MoveSetJournal, MoveSetOutcome, MoveSetPlan,
};
pub use open_or_init::{open_or_init, NotFoundError, Opened};
pub use watcher::{run_watch_loop, WatchHandle};
