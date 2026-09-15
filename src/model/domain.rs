//! Generic domain/entity identity contract shared by every kernel-backed
//! domain (ticket, spec, test, rule, session, feedback, audit, ...).
//!
//! The kernel distinguishes five identifiers instead of collapsing "domain"
//! and "entity type" into one concept: [`DomainId`] (a bounded store),
//! [`EntityTypeId`] (a class of entity within a domain), [`EntityId`]
//! (re-exported from [`super::entity`], one instance), [`DomainSchemaVersion`]
//! (a domain manifest's own version), and [`EntityTypeSchemaVersion`] (an
//! entity type's independently versioned schema). Each entity has exactly one
//! owning domain; [`EntityOwnershipRegistry`] enforces that rule.
//!
//! This module intentionally stops at identity and ownership. It does not
//! define a domain manifest file format, a migration protocol, or a hook
//! system — those are later, separate contracts.

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::entity::EntityId;

/// Identifies a bounded store (e.g. `ticket`, `spec`, `test`, `feedback`,
/// `rule`, `session`, `audit`). Must be non-empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DomainId(String);

/// Identifies a class of entity within a domain (e.g. ticket's `task`,
/// `bug`, `epic` types). Must be non-empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EntityTypeId(String);

/// Errors produced when constructing a [`DomainId`] or [`EntityTypeId`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentifierError {
    #[error("domain id must not be empty")]
    EmptyDomainId,
    #[error("entity type id must not be empty")]
    EmptyEntityTypeId,
}

macro_rules! string_identifier {
    ($ty:ident, $empty_err:expr) => {
        impl $ty {
            /// Construct a validated identifier. Rejects an empty string.
            pub fn new(id: impl Into<String>) -> Result<Self, IdentifierError> {
                let id = id.into();
                if id.is_empty() {
                    return Err($empty_err);
                }
                Ok(Self(id))
            }

            /// Borrow the underlying identifier string.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $ty {
            type Error = IdentifierError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$ty> for String {
            fn from(value: $ty) -> Self {
                value.0
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

string_identifier!(DomainId, IdentifierError::EmptyDomainId);
string_identifier!(EntityTypeId, IdentifierError::EmptyEntityTypeId);

/// A domain manifest's own schema version, independent of any single entity
/// type's [`EntityTypeSchemaVersion`]. A domain may add or activate an entity
/// type without bumping the version of that entity type's own schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DomainSchemaVersion(pub u32);

/// An entity type's own, independently versioned schema version. Bumping this
/// does not require a [`DomainSchemaVersion`] bump unless the domain
/// manifest's own membership or activation state also changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityTypeSchemaVersion(pub u32);

/// Enforces the kernel ownership rule: every [`EntityId`] has exactly one
/// owning [`DomainId`]. Other domains reference the canonical entity through
/// typed edges or external URNs; they never become a second persistence,
/// schema, or migration authority for it.
///
/// This registry is a generic bookkeeping mechanism, not a store: it holds no
/// entity content and does not perform I/O. Callers populate it from whatever
/// authoritative source they trust (e.g. store discovery) and use [`claim`]
/// to reject a conflicting second owner.
///
/// [`claim`]: EntityOwnershipRegistry::claim
#[derive(Debug, Clone, Default)]
pub struct EntityOwnershipRegistry {
    owners: BTreeMap<EntityId, DomainId>,
}

/// Errors produced by [`EntityOwnershipRegistry::claim`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum OwnershipError {
    /// `entity_id` is already owned by a domain other than the one attempting
    /// to claim it.
    #[error(
        "entity {entity_id} is already owned by domain '{existing}', cannot also claim it for domain '{attempted}'"
    )]
    Conflict {
        entity_id: EntityId,
        existing: DomainId,
        attempted: DomainId,
    },
}

impl EntityOwnershipRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Claim `entity_id` for `domain_id`. Re-claiming by the same domain is a
    /// no-op. Claiming an entity already owned by a *different* domain
    /// returns [`OwnershipError::Conflict`] and leaves the existing owner
    /// unchanged.
    pub fn claim(
        &mut self,
        entity_id: EntityId,
        domain_id: DomainId,
    ) -> Result<(), OwnershipError> {
        match self.owners.get(&entity_id) {
            Some(existing) if existing != &domain_id => Err(OwnershipError::Conflict {
                entity_id,
                existing: existing.clone(),
                attempted: domain_id,
            }),
            _ => {
                self.owners.insert(entity_id, domain_id);
                Ok(())
            }
        }
    }

    /// Look up the owning domain for `entity_id`, if claimed.
    pub fn owner(&self, entity_id: &EntityId) -> Option<&DomainId> {
        self.owners.get(entity_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_id_rejects_empty() {
        assert_eq!(DomainId::new(""), Err(IdentifierError::EmptyDomainId));
    }

    #[test]
    fn entity_type_id_rejects_empty() {
        assert_eq!(
            EntityTypeId::new(""),
            Err(IdentifierError::EmptyEntityTypeId)
        );
    }

    #[test]
    fn domain_id_round_trips_through_json() {
        let id = DomainId::new("ticket").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"ticket\"");
        let back: DomainId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn domain_id_json_rejects_empty_string() {
        let err = serde_json::from_str::<DomainId>("\"\"");
        assert!(err.is_err());
    }

    #[test]
    fn ownership_registry_allows_same_domain_reclaim() {
        let mut registry = EntityOwnershipRegistry::new();
        let entity_id = EntityId::new_v4();
        let domain = DomainId::new("ticket").unwrap();

        registry.claim(entity_id, domain.clone()).unwrap();
        // Re-claiming by the same domain is idempotent.
        registry.claim(entity_id, domain.clone()).unwrap();

        assert_eq!(registry.owner(&entity_id), Some(&domain));
    }

    #[test]
    fn ownership_registry_rejects_conflicting_second_owner() {
        let mut registry = EntityOwnershipRegistry::new();
        let entity_id = EntityId::new_v4();
        let ticket = DomainId::new("ticket").unwrap();
        let spec = DomainId::new("spec").unwrap();

        registry.claim(entity_id, ticket.clone()).unwrap();
        let err = registry.claim(entity_id, spec.clone()).unwrap_err();

        assert_eq!(
            err,
            OwnershipError::Conflict {
                entity_id,
                existing: ticket.clone(),
                attempted: spec,
            }
        );
        // The original owner is preserved after the rejected claim.
        assert_eq!(registry.owner(&entity_id), Some(&ticket));
    }

    #[test]
    fn domain_and_entity_type_ids_are_distinct_axes() {
        // Same string value on both axes must not collapse identity: a
        // domain id and an entity type id are different types even when
        // their string content matches.
        let domain = DomainId::new("ticket").unwrap();
        let entity_type = EntityTypeId::new("ticket").unwrap();
        assert_eq!(domain.as_str(), entity_type.as_str());
    }
}
