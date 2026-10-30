//! Generic domain migration protocol shared by every kernel-backed domain.
//!
//! [`transcripts/15-09-2026_entity-kernel-all-domains/03-migration-orchestration.md`]
//! converges three divergent domain migration implementations (ticket's
//! dry-run/apply split, test's durable two-phase manifest, feedback's absent
//! migration) into one protocol: dry-run -> apply (staged -> published) ->
//! journal -> resume -> rollback. This module defines the protocol's
//! reusable data contracts and pure state-transition logic; it deliberately
//! stops short of a filesystem or async runner, and it does not migrate any
//! domain's live data itself.
//!
//! Only [`DomainManifest`]'s active entity types are eligible for a
//! migration step. Inactive types with retained records are never silently
//! skipped: [`MigrationDryRunReport::plan`] reports them explicitly via
//! [`InactiveEntityTypeReportEntry`] instead.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::domain::{DomainId, DomainSchemaVersion, EntityTypeId, EntityTypeSchemaVersion};
use super::domain_manifest::DomainManifest;
use crate::model::urn::Urn;

/// A durable phase in the migration lifecycle.
///
/// `Planned` is the dry-run stage: a report exists but no live state has
/// been touched. `Staged` means the migration's changes have been written to
/// a non-live location but not yet made visible. `Published` means the
/// staged changes are now live. `RolledBack` is terminal: everything the
/// migration itself created or changed has been undone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationPhase {
    Planned,
    Staged,
    Published,
    RolledBack,
}

/// Errors produced by an invalid [`MigrationPhase`] transition.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MigrationPhaseError {
    #[error("cannot transition migration phase from {from:?} to {to:?}")]
    InvalidTransition {
        from: MigrationPhase,
        to: MigrationPhase,
    },
}

impl MigrationPhase {
    /// The allowed forward/rollback transitions: `Planned -> Staged ->
    /// Published`, with rollback permitted from either `Staged` or
    /// `Published`. Every other pair (including any transition out of the
    /// terminal `RolledBack` phase) is rejected.
    pub fn can_transition_to(&self, next: MigrationPhase) -> bool {
        use MigrationPhase::*;
        matches!(
            (self, next),
            (Planned, Staged)
                | (Staged, Published)
                | (Staged, RolledBack)
                | (Published, RolledBack)
        )
    }

    /// Validate and perform a transition, returning the new phase on success.
    pub fn transition_to(
        &self,
        next: MigrationPhase,
    ) -> Result<MigrationPhase, MigrationPhaseError> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(MigrationPhaseError::InvalidTransition {
                from: *self,
                to: next,
            })
        }
    }
}

/// One active entity type's planned schema-version change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTypeMigrationStep {
    pub entity_type_id: EntityTypeId,
    pub from_schema_version: EntityTypeSchemaVersion,
    pub to_schema_version: EntityTypeSchemaVersion,
}

/// Dry-run report entry for one **inactive** entity-type membership found in
/// the domain manifest. Inactive types are never migrated; this entry
/// records that the type is present in the manifest, left untouched by the
/// migration, and inactive per its manifest status, plus whether the caller
/// found retained records for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InactiveEntityTypeReportEntry {
    pub entity_type_id: EntityTypeId,
    pub schema_version: EntityTypeSchemaVersion,
    pub has_retained_records: bool,
}

/// An external URN (a cross-store/cross-workspace reference, see
/// [`crate::model::urn::Urn`]) augmented with a minimum entity-type schema
/// version the referencing side requires of the target entity type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalUrnVersionConstraint {
    pub urn: Urn,
    pub entity_type_id: EntityTypeId,
    pub minimum_schema_version: EntityTypeSchemaVersion,
}

impl ExternalUrnVersionConstraint {
    pub fn new(
        urn: Urn,
        entity_type_id: EntityTypeId,
        minimum_schema_version: EntityTypeSchemaVersion,
    ) -> Self {
        Self {
            urn,
            entity_type_id,
            minimum_schema_version,
        }
    }

    /// True when `actual_version` still satisfies this constraint.
    pub fn is_satisfied_by(&self, actual_version: EntityTypeSchemaVersion) -> bool {
        actual_version >= self.minimum_schema_version
    }
}

/// A constraint that a planned migration's target version would violate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalUrnConstraintViolation {
    pub constraint: ExternalUrnVersionConstraint,
    pub actual_schema_version: EntityTypeSchemaVersion,
}

/// A deterministic dry-run plan for migrating one domain's active entity
/// types, produced without touching any live state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationDryRunReport {
    pub domain_id: DomainId,
    pub from_domain_schema_version: DomainSchemaVersion,
    pub to_domain_schema_version: DomainSchemaVersion,
    pub steps: Vec<EntityTypeMigrationStep>,
    pub inactive_entity_types: Vec<InactiveEntityTypeReportEntry>,
    pub external_urn_violations: Vec<ExternalUrnConstraintViolation>,
}

impl MigrationDryRunReport {
    /// Compute a deterministic dry-run report for `manifest`.
    ///
    /// `target_entity_type_versions` supplies each active entity type's
    /// intended post-migration [`EntityTypeSchemaVersion`]; a type absent
    /// from the map, or mapped to its current version, produces no step.
    /// `retained_inactive_type_records` supplies, for each inactive type the
    /// caller checked, whether records were found. `external_constraints`
    /// are checked against `target_entity_type_versions` (falling back to
    /// the constraint's own minimum, i.e. "no change planned", when the
    /// constrained type is not part of this migration).
    ///
    /// Steps, inactive-type entries, and violations are all sorted for
    /// deterministic, reproducible plan ordering independent of manifest or
    /// map iteration order.
    pub fn plan(
        manifest: &DomainManifest,
        to_domain_schema_version: DomainSchemaVersion,
        target_entity_type_versions: &BTreeMap<EntityTypeId, EntityTypeSchemaVersion>,
        retained_inactive_type_records: &BTreeMap<EntityTypeId, bool>,
        external_constraints: &[ExternalUrnVersionConstraint],
    ) -> Self {
        let mut steps: Vec<EntityTypeMigrationStep> = manifest
            .active_entity_types()
            .filter_map(|membership| {
                let to_version = *target_entity_type_versions.get(&membership.entity_type_id)?;
                if to_version == membership.schema_version {
                    return None;
                }
                Some(EntityTypeMigrationStep {
                    entity_type_id: membership.entity_type_id.clone(),
                    from_schema_version: membership.schema_version,
                    to_schema_version: to_version,
                })
            })
            .collect();
        steps.sort_by(|a, b| a.entity_type_id.cmp(&b.entity_type_id));

        let mut inactive_entity_types: Vec<InactiveEntityTypeReportEntry> = manifest
            .inactive_entity_types()
            .map(|membership| InactiveEntityTypeReportEntry {
                entity_type_id: membership.entity_type_id.clone(),
                schema_version: membership.schema_version,
                has_retained_records: retained_inactive_type_records
                    .get(&membership.entity_type_id)
                    .copied()
                    .unwrap_or(false),
            })
            .collect();
        inactive_entity_types.sort_by(|a, b| a.entity_type_id.cmp(&b.entity_type_id));

        let mut external_urn_violations: Vec<ExternalUrnConstraintViolation> = external_constraints
            .iter()
            .filter_map(|constraint| {
                let actual_schema_version = target_entity_type_versions
                    .get(&constraint.entity_type_id)
                    .copied()
                    .unwrap_or(constraint.minimum_schema_version);
                if constraint.is_satisfied_by(actual_schema_version) {
                    None
                } else {
                    Some(ExternalUrnConstraintViolation {
                        constraint: constraint.clone(),
                        actual_schema_version,
                    })
                }
            })
            .collect();
        external_urn_violations.sort_by(|a, b| {
            a.constraint
                .urn
                .to_string()
                .cmp(&b.constraint.urn.to_string())
        });

        Self {
            domain_id: manifest.domain_id.clone(),
            from_domain_schema_version: manifest.schema_version,
            to_domain_schema_version,
            steps,
            inactive_entity_types,
            external_urn_violations,
        }
    }
}

/// One durably-recorded phase transition, with the timestamp it was recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPhaseRecord {
    pub phase: MigrationPhase,
    pub recorded_at: DateTime<Utc>,
}

/// Durable journal metadata sufficient to resume an interrupted migration
/// apply from its last completed phase, or to roll it back.
///
/// The journal always carries the [`MigrationDryRunReport`] that produced
/// it: apply only ever proceeds from a prior dry-run, and rollback/resume
/// logic can recompute what the migration was supposed to do from the same
/// report rather than trusting partially-applied live state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationJournal {
    pub migration_id: Uuid,
    pub domain_id: DomainId,
    pub dry_run_report: MigrationDryRunReport,
    history: Vec<MigrationPhaseRecord>,
}

/// Errors produced while advancing a [`MigrationJournal`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MigrationJournalError {
    #[error(transparent)]
    InvalidTransition(#[from] MigrationPhaseError),
}

impl MigrationJournal {
    /// Start a new journal in the [`MigrationPhase::Planned`] phase.
    pub fn new(
        migration_id: Uuid,
        dry_run_report: MigrationDryRunReport,
        recorded_at: DateTime<Utc>,
    ) -> Self {
        Self {
            migration_id,
            domain_id: dry_run_report.domain_id.clone(),
            dry_run_report,
            history: vec![MigrationPhaseRecord {
                phase: MigrationPhase::Planned,
                recorded_at,
            }],
        }
    }

    /// The most recently durably-recorded phase.
    pub fn current_phase(&self) -> MigrationPhase {
        self.history
            .last()
            .expect("history always has an initial Planned record")
            .phase
    }

    /// Validate and durably record a transition to `next`.
    pub fn advance(
        &mut self,
        next: MigrationPhase,
        recorded_at: DateTime<Utc>,
    ) -> Result<(), MigrationJournalError> {
        self.current_phase().transition_to(next)?;
        self.history.push(MigrationPhaseRecord {
            phase: next,
            recorded_at,
        });
        Ok(())
    }

    /// The phase an interrupted apply should resume from: the last phase
    /// this journal durably recorded.
    pub fn resume_phase(&self) -> MigrationPhase {
        self.current_phase()
    }

    /// The full, ordered phase-transition history.
    pub fn history(&self) -> &[MigrationPhaseRecord] {
        &self.history
    }
}

#[cfg(test)]
mod tests;
