//! Domain manifest: declares a domain's own [`DomainSchemaVersion`] and the
//! set of entity types it registers, each carrying an independent
//! [`EntityTypeSchemaVersion`] and an explicit active/inactive status.
//!
//! This is the "domain manifest" half of the two-file schema-description
//! model (domain manifest + entity-type schema files) described in
//! `transcripts/15-09-2026_entity-kernel-all-domains/02-schema-versioning.md`.
//! It intentionally stops at declaring membership and versioning; it does not
//! define migration handlers, hooks, or workspace capabilities.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::domain::{DomainId, DomainSchemaVersion, EntityTypeId, EntityTypeSchemaVersion};

/// Whether an entity type registered in a domain manifest is currently
/// active. Inactive types may still retain records and remain reportable;
/// they are simply exempt from the domain's active-migration path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityTypeStatus {
    Active,
    Inactive,
}

impl EntityTypeStatus {
    /// True when this status is [`EntityTypeStatus::Active`].
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// One entity type's membership record within a domain manifest: its id, its
/// own independently versioned schema version, and its activation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTypeMembership {
    pub entity_type_id: EntityTypeId,
    pub schema_version: EntityTypeSchemaVersion,
    pub status: EntityTypeStatus,
}

/// A domain's own schema manifest: its [`DomainId`], its own
/// [`DomainSchemaVersion`], and the entity types it registers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainManifest {
    pub domain_id: DomainId,
    pub schema_version: DomainSchemaVersion,
    entity_types: Vec<EntityTypeMembership>,
}

/// Errors produced when constructing a [`DomainManifest`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainManifestError {
    #[error("domain manifest '{domain_id}' must declare at least one entity type")]
    NoEntityTypes { domain_id: DomainId },
    #[error(
        "domain manifest '{domain_id}' declares entity type '{entity_type_id}' more than once"
    )]
    DuplicateEntityType {
        domain_id: DomainId,
        entity_type_id: EntityTypeId,
    },
}

impl DomainManifest {
    /// Construct a validated domain manifest. Rejects an empty entity-type
    /// set and rejects a duplicate `entity_type_id` across memberships.
    pub fn new(
        domain_id: DomainId,
        schema_version: DomainSchemaVersion,
        entity_types: Vec<EntityTypeMembership>,
    ) -> Result<Self, DomainManifestError> {
        if entity_types.is_empty() {
            return Err(DomainManifestError::NoEntityTypes { domain_id });
        }

        let mut seen = BTreeMap::new();
        for membership in &entity_types {
            if seen.insert(membership.entity_type_id.clone(), ()).is_some() {
                return Err(DomainManifestError::DuplicateEntityType {
                    domain_id,
                    entity_type_id: membership.entity_type_id.clone(),
                });
            }
        }

        Ok(Self {
            domain_id,
            schema_version,
            entity_types,
        })
    }

    /// All registered entity-type memberships, active and inactive.
    pub fn entity_types(&self) -> &[EntityTypeMembership] {
        &self.entity_types
    }

    /// Registered entity types whose status is [`EntityTypeStatus::Active`].
    pub fn active_entity_types(&self) -> impl Iterator<Item = &EntityTypeMembership> {
        self.entity_types.iter().filter(|m| m.status.is_active())
    }

    /// Registered entity types whose status is [`EntityTypeStatus::Inactive`].
    pub fn inactive_entity_types(&self) -> impl Iterator<Item = &EntityTypeMembership> {
        self.entity_types.iter().filter(|m| !m.status.is_active())
    }

    /// Look up a membership by [`EntityTypeId`].
    pub fn membership(&self, entity_type_id: &EntityTypeId) -> Option<&EntityTypeMembership> {
        self.entity_types
            .iter()
            .find(|m| &m.entity_type_id == entity_type_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn membership(id: &str, version: u32, status: EntityTypeStatus) -> EntityTypeMembership {
        EntityTypeMembership {
            entity_type_id: EntityTypeId::new(id).unwrap(),
            schema_version: EntityTypeSchemaVersion(version),
            status,
        }
    }

    #[test]
    fn accepts_two_entity_types_with_independent_versions_and_status() {
        let manifest = DomainManifest::new(
            DomainId::new("rule").unwrap(),
            DomainSchemaVersion(3),
            vec![
                membership("rule-entry", 2, EntityTypeStatus::Active),
                membership("generated-target", 5, EntityTypeStatus::Inactive),
            ],
        )
        .unwrap();

        assert_eq!(manifest.schema_version, DomainSchemaVersion(3));
        assert_eq!(manifest.entity_types().len(), 2);

        let active: Vec<_> = manifest.active_entity_types().collect();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].schema_version, EntityTypeSchemaVersion(2));

        let inactive: Vec<_> = manifest.inactive_entity_types().collect();
        assert_eq!(inactive.len(), 1);
        assert_eq!(inactive[0].schema_version, EntityTypeSchemaVersion(5));
        assert!(!inactive[0].status.is_active());
    }

    #[test]
    fn rejects_duplicate_entity_type() {
        let err = DomainManifest::new(
            DomainId::new("ticket").unwrap(),
            DomainSchemaVersion(1),
            vec![
                membership("task", 1, EntityTypeStatus::Active),
                membership("task", 2, EntityTypeStatus::Active),
            ],
        )
        .unwrap_err();

        assert_eq!(
            err,
            DomainManifestError::DuplicateEntityType {
                domain_id: DomainId::new("ticket").unwrap(),
                entity_type_id: EntityTypeId::new("task").unwrap(),
            }
        );
    }

    #[test]
    fn rejects_empty_domain_manifest() {
        let err = DomainManifest::new(
            DomainId::new("ticket").unwrap(),
            DomainSchemaVersion(1),
            vec![],
        )
        .unwrap_err();

        assert_eq!(
            err,
            DomainManifestError::NoEntityTypes {
                domain_id: DomainId::new("ticket").unwrap(),
            }
        );
    }

    #[test]
    fn membership_lookup_finds_registered_type() {
        let manifest = DomainManifest::new(
            DomainId::new("rule").unwrap(),
            DomainSchemaVersion(1),
            vec![membership("rule-entry", 1, EntityTypeStatus::Active)],
        )
        .unwrap();

        let found = manifest
            .membership(&EntityTypeId::new("rule-entry").unwrap())
            .unwrap();
        assert_eq!(found.schema_version, EntityTypeSchemaVersion(1));

        assert!(manifest
            .membership(&EntityTypeId::new("missing").unwrap())
            .is_none());
    }
}
