//! Deterministic transactional hook contract for [`EventEnvelope`] dispatch.
//!
//! Per the interview decision recorded in
//! [`transcripts/15-09-2026_entity-kernel-all-domains/04-events-hooks.md`],
//! every registered hook may veto a mutation, and a veto — whether explicit
//! or a hook failure — always produces the same rollback-required outcome.
//! This module defines the pure, synchronous dispatch contract; it performs
//! no filesystem mutation and no rollback execution itself, it only computes
//! whether one is required. The kernel does not invent a general async
//! runtime for hook dispatch: hooks run synchronously, in registration
//! order, on the calling thread.

use super::event::EventEnvelope;

/// What one hook decided about a mutation it observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookOutcome {
    /// The hook accepted the mutation.
    Allow,
    /// The hook explicitly vetoed the mutation, with a human-readable reason.
    Veto { reason: String },
}

/// A hook failed to run to completion (e.g. an internal error) rather than
/// returning an explicit [`HookOutcome`]. Per the transactional contract,
/// this is treated identically to an explicit veto: both require rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookFailure {
    pub message: String,
}

/// A single registered mutation hook. Implementations are domain adapters
/// (e.g. ticket's `StoreHook`, session's transcript capture) that decide
/// whether to allow or veto a mutation described by an [`EventEnvelope`].
pub trait MutationHook: Send + Sync {
    /// A stable, human-readable identifier for this hook, used to attribute
    /// a veto or failure to its source hook.
    fn name(&self) -> &str;

    /// Observe `envelope` and decide whether to allow or veto the mutation
    /// it describes. Returning `Err` is a hook failure, not an explicit
    /// veto, but both are treated identically by [`HookChain::dispatch`].
    fn on_mutation(&self, envelope: &EventEnvelope) -> Result<HookOutcome, HookFailure>;
}

/// Why a mutation is being rolled back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VetoCause {
    /// A hook returned [`HookOutcome::Veto`].
    ExplicitVeto { reason: String },
    /// A hook returned `Err`.
    HookFailure { message: String },
}

/// The result of dispatching an [`EventEnvelope`] through a [`HookChain`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationOutcome {
    /// Every registered hook allowed the mutation; it may be committed.
    Committed,
    /// A hook vetoed or failed; the mutation must be rolled back to its
    /// pre-mutation state. No hook after the vetoing/failing hook is run.
    RollbackRequired {
        /// The name of the hook that vetoed or failed.
        hook_name: String,
        cause: VetoCause,
    },
}

impl MutationOutcome {
    /// True when this outcome is [`MutationOutcome::Committed`].
    pub fn is_committed(&self) -> bool {
        matches!(self, Self::Committed)
    }
}

/// An ordered, deterministic chain of [`MutationHook`]s. Hooks run in
/// registration order (first-registered, first-run); dispatch stops at the
/// first hook that vetoes or fails, since a rollback is already required and
/// running further hooks would add non-deterministic side effects to a
/// mutation that will not be committed. A hook is invoked at most once per
/// dispatched envelope; a retried mutation after rollback is a new dispatch,
/// not a replay of the same invocation.
#[derive(Default)]
pub struct HookChain {
    hooks: Vec<Box<dyn MutationHook>>,
}

impl HookChain {
    /// Create an empty chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a hook. Hooks are dispatched in the order they are
    /// registered.
    pub fn register(&mut self, hook: impl MutationHook + 'static) {
        self.hooks.push(Box::new(hook));
    }

    /// The number of registered hooks.
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// True when no hooks are registered.
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    /// Dispatch `envelope` through every registered hook in registration
    /// order. Performs no filesystem mutation or rollback execution itself;
    /// it only computes whether a rollback is required.
    pub fn dispatch(&self, envelope: &EventEnvelope) -> MutationOutcome {
        for hook in &self.hooks {
            match hook.on_mutation(envelope) {
                Ok(HookOutcome::Allow) => continue,
                Ok(HookOutcome::Veto { reason }) => {
                    return MutationOutcome::RollbackRequired {
                        hook_name: hook.name().to_string(),
                        cause: VetoCause::ExplicitVeto { reason },
                    };
                }
                Err(HookFailure { message }) => {
                    return MutationOutcome::RollbackRequired {
                        hook_name: hook.name().to_string(),
                        cause: VetoCause::HookFailure { message },
                    };
                }
            }
        }
        MutationOutcome::Committed
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use super::*;
    use crate::model::domain::{
        DomainId, DomainSchemaVersion, EntityTypeId, EntityTypeSchemaVersion,
    };

    struct RecordingHook {
        name: &'static str,
        outcome: Result<HookOutcome, HookFailure>,
        calls: &'static AtomicUsize,
    }

    impl MutationHook for RecordingHook {
        fn name(&self) -> &str {
            self.name
        }

        fn on_mutation(&self, _envelope: &EventEnvelope) -> Result<HookOutcome, HookFailure> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.outcome.clone()
        }
    }

    fn sample_envelope() -> EventEnvelope {
        EventEnvelope::new(
            Uuid::nil(),
            DomainId::new("ticket").unwrap(),
            EntityTypeId::new("task").unwrap(),
            Uuid::nil(),
            crate::model::event::MutationOperation::Update,
            DateTime::<Utc>::MIN_UTC,
            Uuid::nil(),
            DomainSchemaVersion(1),
            EntityTypeSchemaVersion(1),
        )
    }

    #[test]
    fn empty_chain_commits() {
        let chain = HookChain::new();
        assert_eq!(
            chain.dispatch(&sample_envelope()),
            MutationOutcome::Committed
        );
    }

    #[test]
    fn all_allow_commits() {
        static CALLS_A: AtomicUsize = AtomicUsize::new(0);
        static CALLS_B: AtomicUsize = AtomicUsize::new(0);
        let mut chain = HookChain::new();
        chain.register(RecordingHook {
            name: "a",
            outcome: Ok(HookOutcome::Allow),
            calls: &CALLS_A,
        });
        chain.register(RecordingHook {
            name: "b",
            outcome: Ok(HookOutcome::Allow),
            calls: &CALLS_B,
        });

        let outcome = chain.dispatch(&sample_envelope());

        assert!(outcome.is_committed());
        assert_eq!(CALLS_A.load(Ordering::SeqCst), 1);
        assert_eq!(CALLS_B.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn veto_requires_rollback_and_stops_dispatch() {
        static CALLS_A: AtomicUsize = AtomicUsize::new(0);
        static CALLS_B: AtomicUsize = AtomicUsize::new(0);
        let mut chain = HookChain::new();
        chain.register(RecordingHook {
            name: "vetoer",
            outcome: Ok(HookOutcome::Veto {
                reason: "not allowed".to_string(),
            }),
            calls: &CALLS_A,
        });
        chain.register(RecordingHook {
            name: "never-run",
            outcome: Ok(HookOutcome::Allow),
            calls: &CALLS_B,
        });

        let outcome = chain.dispatch(&sample_envelope());

        assert_eq!(
            outcome,
            MutationOutcome::RollbackRequired {
                hook_name: "vetoer".to_string(),
                cause: VetoCause::ExplicitVeto {
                    reason: "not allowed".to_string(),
                },
            }
        );
        assert_eq!(CALLS_A.load(Ordering::SeqCst), 1);
        assert_eq!(
            CALLS_B.load(Ordering::SeqCst),
            0,
            "hooks after a veto must not run"
        );
    }

    #[test]
    fn hook_failure_is_treated_identically_to_a_veto() {
        static CALLS_A: AtomicUsize = AtomicUsize::new(0);
        let mut chain = HookChain::new();
        chain.register(RecordingHook {
            name: "crashy",
            outcome: Err(HookFailure {
                message: "internal error".to_string(),
            }),
            calls: &CALLS_A,
        });

        let outcome = chain.dispatch(&sample_envelope());

        assert_eq!(
            outcome,
            MutationOutcome::RollbackRequired {
                hook_name: "crashy".to_string(),
                cause: VetoCause::HookFailure {
                    message: "internal error".to_string(),
                },
            }
        );
    }

    #[test]
    fn hooks_dispatch_in_registration_order() {
        let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        struct OrderRecordingHook {
            name: &'static str,
            order: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
        }

        impl MutationHook for OrderRecordingHook {
            fn name(&self) -> &str {
                self.name
            }

            fn on_mutation(&self, _envelope: &EventEnvelope) -> Result<HookOutcome, HookFailure> {
                self.order.lock().unwrap().push(self.name);
                Ok(HookOutcome::Allow)
            }
        }

        let mut chain = HookChain::new();
        chain.register(OrderRecordingHook {
            name: "first",
            order: order.clone(),
        });
        chain.register(OrderRecordingHook {
            name: "second",
            order: order.clone(),
        });
        chain.register(OrderRecordingHook {
            name: "third",
            order: order.clone(),
        });

        let outcome = chain.dispatch(&sample_envelope());

        assert!(outcome.is_committed());
        assert_eq!(*order.lock().unwrap(), vec!["first", "second", "third"]);
    }
}
