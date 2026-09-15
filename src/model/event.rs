//! Generic mutation event envelope shared by every kernel-backed domain.
//!
//! [`transcripts/15-09-2026_entity-kernel-all-domains/04-events-hooks.md`]
//! generalizes the shape already present independently in ticket's
//! `StoreHook` and session's `CopilotHookEvent` into one domain-neutral
//! envelope. Domain adapters populate an [`EventEnvelope`] from their own
//! mutation path; the kernel does not detect mutations itself and this
//! module performs no filesystem I/O.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::domain::{DomainId, DomainSchemaVersion, EntityTypeId, EntityTypeSchemaVersion};
use super::entity::EntityId;

/// The kind of mutation an [`EventEnvelope`] describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationOperation {
    Create,
    Update,
    Transition,
    Delete,
}

/// A domain-neutral mutation event envelope, dispatched to a
/// [`super::hook::HookChain`] before a mutation is considered committed.
///
/// `pre_state`/`post_state` are optional, domain-supplied snapshots of the
/// mutated entity relevant to this mutation (e.g. changed fields only); the
/// kernel does not interpret their shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub domain_id: DomainId,
    pub entity_type_id: EntityTypeId,
    pub entity_id: EntityId,
    pub operation: MutationOperation,
    pub occurred_at: DateTime<Utc>,
    /// Correlates envelopes produced by one logical mutation attempt (e.g. a
    /// batch of related entity writes) across hooks and domains.
    pub correlation_id: Uuid,
    pub domain_schema_version: DomainSchemaVersion,
    pub entity_type_schema_version: EntityTypeSchemaVersion,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_state: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_state: Option<Value>,
}

impl EventEnvelope {
    /// Construct an envelope with no pre/post state metadata attached.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_id: Uuid,
        domain_id: DomainId,
        entity_type_id: EntityTypeId,
        entity_id: EntityId,
        operation: MutationOperation,
        occurred_at: DateTime<Utc>,
        correlation_id: Uuid,
        domain_schema_version: DomainSchemaVersion,
        entity_type_schema_version: EntityTypeSchemaVersion,
    ) -> Self {
        Self {
            event_id,
            domain_id,
            entity_type_id,
            entity_id,
            operation,
            occurred_at,
            correlation_id,
            domain_schema_version,
            entity_type_schema_version,
            pre_state: None,
            post_state: None,
        }
    }

    /// Attach pre-mutation state metadata, returning `self` for chaining.
    pub fn with_pre_state(mut self, pre_state: Value) -> Self {
        self.pre_state = Some(pre_state);
        self
    }

    /// Attach post-mutation state metadata, returning `self` for chaining.
    pub fn with_post_state(mut self, post_state: Value) -> Self {
        self.post_state = Some(post_state);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_envelope() -> EventEnvelope {
        EventEnvelope::new(
            Uuid::nil(),
            DomainId::new("ticket").unwrap(),
            EntityTypeId::new("task").unwrap(),
            Uuid::nil(),
            MutationOperation::Update,
            DateTime::<Utc>::MIN_UTC,
            Uuid::nil(),
            DomainSchemaVersion(1),
            EntityTypeSchemaVersion(1),
        )
    }

    #[test]
    fn new_envelope_has_no_state_metadata() {
        let envelope = sample_envelope();
        assert_eq!(envelope.pre_state, None);
        assert_eq!(envelope.post_state, None);
    }

    #[test]
    fn with_pre_and_post_state_attaches_metadata() {
        let envelope = sample_envelope()
            .with_pre_state(serde_json::json!({"state": "open"}))
            .with_post_state(serde_json::json!({"state": "done"}));
        assert_eq!(
            envelope.pre_state,
            Some(serde_json::json!({"state": "open"}))
        );
        assert_eq!(
            envelope.post_state,
            Some(serde_json::json!({"state": "done"}))
        );
    }

    #[test]
    fn envelope_round_trips_through_json() {
        let envelope = sample_envelope().with_pre_state(serde_json::json!({"a": 1}));
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(envelope, decoded);
    }
}
