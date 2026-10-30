use uuid::Uuid;

use super::{Capability, LifecycleError};

/// Native vocabulary and mutation authority remain in the implementing domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRequest {
    pub op_id: Uuid,
    pub to_native_state: String,
    pub author: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlRequest {
    pub op_id: Uuid,
    pub expected_control_sequence: u64,
    pub evidence: Option<String>,
}

/// Binds neutral capabilities to a domain's identity, read model and errors.
pub trait LifecycleSubject {
    type Identity;
    type Snapshot;
    type Error;

    fn provider_name(&self) -> &str;
    fn lifecycle_error(error: LifecycleError) -> Self::Error;
}

pub trait LifecycleRead: LifecycleSubject {
    fn read_lifecycle(&self, identity: &Self::Identity) -> Result<Self::Snapshot, Self::Error>;
}

pub trait LifecycleTransition: LifecycleSubject {
    fn transition(
        &self,
        identity: &Self::Identity,
        request: &TransitionRequest,
    ) -> Result<Self::Snapshot, Self::Error>;
}

macro_rules! capability_trait {
    ($(#[$doc:meta])* $name:ident { $($method:ident => $cap:ident),+ $(,)? }) => {
        $(#[$doc])*
        pub trait $name: LifecycleSubject {
            $(
                fn $method(
                    &self,
                    _identity: &Self::Identity,
                    _request: &ControlRequest,
                ) -> Result<Self::Snapshot, Self::Error> {
                    Err(Self::lifecycle_error(LifecycleError::UnsupportedCapability {
                        capability: Capability::$cap,
                        provider: self.provider_name().to_owned(),
                    }))
                }
            )+
        }
    };
}

capability_trait!(
    /// Semantic planning edits are separate from approval and execution.
    PlanningCapability { revise_plan => Planning });
capability_trait!(ApprovalCapability { approve => Approval });
capability_trait!(PauseResumeCapability { pause => PauseResume, resume => PauseResume });
capability_trait!(CancellationCapability { cancel => Cancellation });
capability_trait!(CompletionCapability { complete => Completion, review_result => ResultReview });
capability_trait!(HandoffCapability { hand_off => Handoff });
capability_trait!(ReopenCapability { reopen => Reopen });
