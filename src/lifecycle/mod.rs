//! Domain-neutral lifecycle schemas, projections and capability contracts.
//!
//! Persistence, entity identities, approval checks and native vocabularies
//! belong to consumers, not to this module.

mod capabilities;
mod control;
mod schema;
mod state;

pub use capabilities::*;
pub use control::{BasicLifecycle, ControlKind, TargetControl};
pub use schema::{LifecycleDefinition, LifecycleSchema};
pub use state::{Capability, LifecycleError, LifecycleState, NativeLifecycle};
