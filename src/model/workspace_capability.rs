//! Workspace-level domain capability contract.
//!
//! [`transcripts/15-09-2026_entity-kernel-all-domains/05-workspace-composition.md`]
//! identifies a gap between mechanical store discovery (already provided by
//! [`crate::discovery`]/[`crate::workspace`]/[`crate::workspace_policy`]) and a
//! capability model: which domains are installed vs. active in a workspace,
//! what another domain depends on, and what a caller receives when a needed
//! domain is absent or inactive.
//!
//! This module is intentionally a pure, serializable data/validation
//! contract: it does not discover stores, does not implement a plugin
//! system, and does not change any domain's business schema. Callers (CLI,
//! HTTP surfaces, domain adapters) populate [`WorkspaceCapabilities`] from
//! whatever discovery/activation source they trust and use [`resolve`] to
//! get a deterministic, structured answer.
//!
//! [`resolve`]: WorkspaceCapabilities::resolve

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::domain::DomainId;

/// A domain's capability state within one workspace.
///
/// "Installed" is not a separate exclusive variant: an installed domain is
/// either [`Active`](Self::Active) or [`Inactive`](Self::Inactive), and
/// [`is_installed`](Self::is_installed) distinguishes that from
/// [`Unavailable`](Self::Unavailable) (not installed at all).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainCapabilityState {
    /// The domain's store is discoverable and the workspace has explicitly
    /// activated it.
    Active,
    /// The domain's store is discoverable, but the workspace has not
    /// activated it for use.
    Inactive,
    /// The domain's store is not discoverable in this workspace at all.
    Unavailable,
}

impl DomainCapabilityState {
    /// True for [`Active`](Self::Active) and [`Inactive`](Self::Inactive):
    /// the domain's store is present, independent of activation.
    pub fn is_installed(&self) -> bool {
        matches!(self, Self::Active | Self::Inactive)
    }

    /// True only for [`Active`](Self::Active).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// One domain's declared dependency on another domain's capability, carrying
/// the reason the dependency exists. The reason is surfaced verbatim in the
/// [`UnavailableCapability`] result when the dependency is unsatisfied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainDependency {
    pub domain_id: DomainId,
    pub reason: String,
}

impl DomainDependency {
    /// Construct a dependency on `domain_id` with the given human-readable
    /// `reason`.
    pub fn new(domain_id: DomainId, reason: impl Into<String>) -> Self {
        Self {
            domain_id,
            reason: reason.into(),
        }
    }
}

/// Structured result returned when an operation depends on a domain that is
/// not [`DomainCapabilityState::Active`]. Names the operation, the
/// missing/inactive domain, its actual state, and the dependency reason, so a
/// caller renders an actionable message instead of a silent no-op or an
/// unstructured error string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnavailableCapability {
    pub operation: String,
    pub domain_id: DomainId,
    pub state: DomainCapabilityState,
    pub reason: String,
}

/// Workspace-level view of every domain's capability state, keyed by
/// [`DomainId`]. A domain with no recorded state resolves to
/// [`DomainCapabilityState::Unavailable`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCapabilities {
    states: BTreeMap<DomainId, DomainCapabilityState>,
}

impl WorkspaceCapabilities {
    /// Create an empty capability set (every domain resolves as
    /// [`DomainCapabilityState::Unavailable`] until recorded).
    pub fn new() -> Self {
        Self::default()
    }

    /// Record `domain_id`'s capability state, replacing any prior value.
    pub fn set_state(&mut self, domain_id: DomainId, state: DomainCapabilityState) {
        self.states.insert(domain_id, state);
    }

    /// The recorded state for `domain_id`, or
    /// [`DomainCapabilityState::Unavailable`] when nothing was recorded.
    pub fn state(&self, domain_id: &DomainId) -> DomainCapabilityState {
        self.states
            .get(domain_id)
            .copied()
            .unwrap_or(DomainCapabilityState::Unavailable)
    }

    /// Resolve `dependencies` against this workspace's recorded states.
    ///
    /// Deterministic: dependencies are checked in the order given, and the
    /// *first* unsatisfied dependency in that order is reported — never an
    /// arbitrary one, regardless of how many dependencies are unsatisfied.
    /// Returns `Ok(())` only when every dependency is
    /// [`DomainCapabilityState::Active`].
    pub fn resolve(
        &self,
        operation: impl Into<String>,
        dependencies: &[DomainDependency],
    ) -> Result<(), UnavailableCapability> {
        let operation = operation.into();
        for dependency in dependencies {
            let state = self.state(&dependency.domain_id);
            if !state.is_active() {
                return Err(UnavailableCapability {
                    operation,
                    domain_id: dependency.domain_id.clone(),
                    state,
                    reason: dependency.reason.clone(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain(id: &str) -> DomainId {
        DomainId::new(id).unwrap()
    }

    #[test]
    fn resolve_succeeds_when_all_dependencies_are_active() {
        let mut capabilities = WorkspaceCapabilities::new();
        capabilities.set_state(domain("ticket"), DomainCapabilityState::Active);
        capabilities.set_state(domain("spec"), DomainCapabilityState::Active);

        let dependencies = vec![DomainDependency::new(domain("spec"), "cross-reference")];

        assert_eq!(capabilities.resolve("link-spec", &dependencies), Ok(()));
    }

    #[test]
    fn resolve_reports_missing_domain_as_unavailable() {
        let capabilities = WorkspaceCapabilities::new();
        let dependencies = vec![DomainDependency::new(domain("spec"), "cross-reference")];

        let err = capabilities
            .resolve("link-spec", &dependencies)
            .unwrap_err();

        assert_eq!(
            err,
            UnavailableCapability {
                operation: "link-spec".to_string(),
                domain_id: domain("spec"),
                state: DomainCapabilityState::Unavailable,
                reason: "cross-reference".to_string(),
            }
        );
    }

    #[test]
    fn resolve_reports_inactive_domain() {
        let mut capabilities = WorkspaceCapabilities::new();
        capabilities.set_state(domain("spec"), DomainCapabilityState::Inactive);
        let dependencies = vec![DomainDependency::new(domain("spec"), "cross-reference")];

        let err = capabilities
            .resolve("link-spec", &dependencies)
            .unwrap_err();

        assert_eq!(err.state, DomainCapabilityState::Inactive);
        assert_eq!(err.domain_id, domain("spec"));
        assert!(err.state.is_installed());
        assert!(!err.state.is_active());
    }

    #[test]
    fn resolve_is_deterministic_and_reports_first_unsatisfied_dependency_in_order() {
        let mut capabilities = WorkspaceCapabilities::new();
        capabilities.set_state(domain("ticket"), DomainCapabilityState::Active);
        // Both "spec" and "rule" are unsatisfied; "spec" is declared first.
        let dependencies = vec![
            DomainDependency::new(domain("spec"), "spec tracking"),
            DomainDependency::new(domain("rule"), "rule linting"),
        ];

        let first = capabilities.resolve("audit", &dependencies).unwrap_err();
        // Re-resolving with the same input is stable, not arbitrary.
        let second = capabilities.resolve("audit", &dependencies).unwrap_err();

        assert_eq!(first.domain_id, domain("spec"));
        assert_eq!(first, second);

        // Reversing declaration order changes which dependency is reported
        // first, confirming the check follows declared order rather than
        // some fixed internal iteration order.
        let reversed = vec![
            DomainDependency::new(domain("rule"), "rule linting"),
            DomainDependency::new(domain("spec"), "spec tracking"),
        ];
        let reversed_err = capabilities.resolve("audit", &reversed).unwrap_err();
        assert_eq!(reversed_err.domain_id, domain("rule"));
    }

    #[test]
    fn unavailable_capability_names_operation_domain_and_reason() {
        let capabilities = WorkspaceCapabilities::new();
        let dependencies = vec![DomainDependency::new(
            domain("spec"),
            "spec domain not installed in this workspace",
        )];

        let err = capabilities
            .resolve("ticket-spec-cross-reference", &dependencies)
            .unwrap_err();

        assert_eq!(err.operation, "ticket-spec-cross-reference");
        assert_eq!(err.domain_id, domain("spec"));
        assert_eq!(err.reason, "spec domain not installed in this workspace");
    }

    #[test]
    fn missing_domain_defaults_to_unavailable_state() {
        let capabilities = WorkspaceCapabilities::new();
        assert_eq!(
            capabilities.state(&domain("spec")),
            DomainCapabilityState::Unavailable
        );
    }

    #[test]
    fn installed_state_distinguishes_active_and_inactive_from_unavailable() {
        assert!(DomainCapabilityState::Active.is_installed());
        assert!(DomainCapabilityState::Inactive.is_installed());
        assert!(!DomainCapabilityState::Unavailable.is_installed());
    }
}
