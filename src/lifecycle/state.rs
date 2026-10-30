use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Shared state vocabulary, independent of any native entity schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleState {
    Planned,
    Active,
    Paused,
    Blocked,
    Completed,
    Cancelled,
}

impl LifecycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }

    pub fn is_resumable(self) -> bool {
        matches!(self, Self::Planned | Self::Active | Self::Blocked)
    }
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A domain adapter translates its own vocabulary without extending the kernel.
pub trait NativeLifecycle {
    fn shared_state(&self, native_state: &str) -> Result<LifecycleState, LifecycleError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    Read,
    Transition,
    Planning,
    Approval,
    PauseResume,
    Cancellation,
    Completion,
    ResultReview,
    Handoff,
    Reopen,
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Read => "read",
            Self::Transition => "transition",
            Self::Planning => "planning",
            Self::Approval => "approval",
            Self::PauseResume => "pause-resume",
            Self::Cancellation => "cancellation",
            Self::Completion => "completion",
            Self::ResultReview => "result-review",
            Self::Handoff => "handoff",
            Self::Reopen => "reopen",
        })
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    #[error("lifecycle_state_unmapped: native state '{0}' has no shared mapping")]
    StateUnmapped(String),
    #[error("unsupported_capability: {capability} is not supported by {provider}")]
    UnsupportedCapability {
        capability: Capability,
        provider: String,
    },
    #[error("illegal_transition: {0}")]
    IllegalTransition(String),
    #[error("invalid_lifecycle_schema: {0}")]
    InvalidSchema(String),
}
