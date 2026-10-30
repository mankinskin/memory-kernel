use super::*;
use crate::InteroperableArtifact;

#[derive(Debug)]
pub enum MoveError {
    Io(std::io::Error),
    Domain(String),
    InteroperabilityContract {
        artifact_class: &'static str,
        detail: String,
    },
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveError::Io(error) => write!(f, "{error}"),
            MoveError::Domain(message) => write!(f, "{message}"),
            MoveError::InteroperabilityContract {
                artifact_class,
                detail,
            } => write!(
                f,
                "interoperability contract violation for {artifact_class}: {detail}"
            ),
        }
    }
}

impl std::error::Error for MoveError {}

impl From<std::io::Error> for MoveError {
    fn from(error: std::io::Error) -> Self {
        MoveError::Io(error)
    }
}

/// Convenience result alias for kernel operations.
pub type MoveResult<T> = Result<T, MoveError>;

#[path = "move_kernel_types/domain.rs"]
mod domain;
#[path = "move_kernel_types/journal.rs"]
mod journal;
#[path = "move_kernel_types/path_serialization.rs"]
mod path_serialization;
#[path = "move_kernel_types/plans.rs"]
mod plans;
#[path = "move_kernel_types/set_journal.rs"]
mod set_journal;
pub use domain::*;
pub use journal::*;
pub(super) use path_serialization::path_buf_is_empty;
use path_serialization::{
    deserialize_pathbuf, deserialize_pathbuf_vec, serialize_normalized_path,
    serialize_normalized_path_vec,
};
pub use plans::*;
pub use set_journal::*;
