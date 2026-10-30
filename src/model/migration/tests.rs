use super::*;
use crate::model::domain_manifest::{EntityTypeMembership, EntityTypeStatus};
use crate::model::index_entry::ContentKind;

fn membership(id: &str, version: u32, status: EntityTypeStatus) -> EntityTypeMembership {
    EntityTypeMembership {
        entity_type_id: EntityTypeId::new(id).unwrap(),
        schema_version: EntityTypeSchemaVersion(version),
        status,
    }
}

fn sample_manifest() -> DomainManifest {
    DomainManifest::new(
        DomainId::new("ticket").unwrap(),
        DomainSchemaVersion(1),
        vec![
            membership("task", 1, EntityTypeStatus::Active),
            membership("epic", 2, EntityTypeStatus::Active),
            membership("legacy-note", 3, EntityTypeStatus::Inactive),
        ],
    )
    .unwrap()
}

// --- MigrationPhase transitions ---

#[test]
fn valid_forward_and_rollback_transitions_succeed() {
    assert_eq!(
        MigrationPhase::Planned.transition_to(MigrationPhase::Staged),
        Ok(MigrationPhase::Staged)
    );
    assert_eq!(
        MigrationPhase::Staged.transition_to(MigrationPhase::Published),
        Ok(MigrationPhase::Published)
    );
    assert_eq!(
        MigrationPhase::Staged.transition_to(MigrationPhase::RolledBack),
        Ok(MigrationPhase::RolledBack)
    );
    assert_eq!(
        MigrationPhase::Published.transition_to(MigrationPhase::RolledBack),
        Ok(MigrationPhase::RolledBack)
    );
}

#[test]
fn invalid_transitions_are_rejected() {
    assert_eq!(
        MigrationPhase::Planned.transition_to(MigrationPhase::Published),
        Err(MigrationPhaseError::InvalidTransition {
            from: MigrationPhase::Planned,
            to: MigrationPhase::Published,
        })
    );
    assert_eq!(
        MigrationPhase::Published.transition_to(MigrationPhase::Staged),
        Err(MigrationPhaseError::InvalidTransition {
            from: MigrationPhase::Published,
            to: MigrationPhase::Staged,
        })
    );
    // RolledBack is terminal: nothing transitions out of it.
    assert!(!MigrationPhase::RolledBack.can_transition_to(MigrationPhase::Planned));
    assert!(!MigrationPhase::RolledBack.can_transition_to(MigrationPhase::Staged));
}

// --- Deterministic plan ordering ---

#[test]
fn dry_run_plan_orders_steps_deterministically_regardless_of_manifest_order() {
    let manifest = sample_manifest();
    let mut targets = BTreeMap::new();
    // Inserted in reverse-alphabetical order to prove sort is applied,
    // not merely inherited from insertion or manifest declaration order.
    targets.insert(
        EntityTypeId::new("task").unwrap(),
        EntityTypeSchemaVersion(2),
    );
    targets.insert(
        EntityTypeId::new("epic").unwrap(),
        EntityTypeSchemaVersion(3),
    );

    let report = MigrationDryRunReport::plan(
        &manifest,
        DomainSchemaVersion(2),
        &targets,
        &BTreeMap::new(),
        &[],
    );

    let ids: Vec<&str> = report
        .steps
        .iter()
        .map(|s| s.entity_type_id.as_str())
        .collect();
    assert_eq!(ids, vec!["epic", "task"]);
    assert_eq!(
        report.steps[0].from_schema_version,
        EntityTypeSchemaVersion(2)
    );
    assert_eq!(
        report.steps[0].to_schema_version,
        EntityTypeSchemaVersion(3)
    );
}

#[test]
fn no_op_target_version_produces_no_step() {
    let manifest = sample_manifest();
    let mut targets = BTreeMap::new();
    targets.insert(
        EntityTypeId::new("task").unwrap(),
        EntityTypeSchemaVersion(1),
    );

    let report = MigrationDryRunReport::plan(
        &manifest,
        DomainSchemaVersion(1),
        &targets,
        &BTreeMap::new(),
        &[],
    );

    assert!(report.steps.is_empty());
}

// --- Inactive-type reporting ---

#[test]
fn inactive_type_with_retained_records_is_reported_present_untouched_inactive() {
    let manifest = sample_manifest();
    let mut retained = BTreeMap::new();
    retained.insert(EntityTypeId::new("legacy-note").unwrap(), true);

    let report = MigrationDryRunReport::plan(
        &manifest,
        DomainSchemaVersion(1),
        &BTreeMap::new(),
        &retained,
        &[],
    );

    assert_eq!(report.inactive_entity_types.len(), 1);
    let entry = &report.inactive_entity_types[0];
    assert_eq!(entry.entity_type_id.as_str(), "legacy-note");
    assert_eq!(entry.schema_version, EntityTypeSchemaVersion(3));
    assert!(entry.has_retained_records);
    // Inactive types are never migrated: they never appear in `steps`.
    assert!(
        !report
            .steps
            .iter()
            .any(|s| s.entity_type_id.as_str() == "legacy-note")
    );
}

#[test]
fn inactive_type_without_recorded_presence_defaults_to_not_present() {
    let manifest = sample_manifest();

    let report = MigrationDryRunReport::plan(
        &manifest,
        DomainSchemaVersion(1),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &[],
    );

    assert_eq!(report.inactive_entity_types.len(), 1);
    assert!(!report.inactive_entity_types[0].has_retained_records);
}

// --- External URN version-constraint validation ---

#[test]
fn external_urn_constraint_satisfied_by_sufficient_version() {
    let urn = Urn::new("default", ContentKind::Spec, Uuid::new_v4()).unwrap();
    let constraint = ExternalUrnVersionConstraint::new(
        urn,
        EntityTypeId::new("task").unwrap(),
        EntityTypeSchemaVersion(2),
    );

    assert!(constraint.is_satisfied_by(EntityTypeSchemaVersion(2)));
    assert!(constraint.is_satisfied_by(EntityTypeSchemaVersion(3)));
    assert!(!constraint.is_satisfied_by(EntityTypeSchemaVersion(1)));
}

#[test]
fn dry_run_plan_surfaces_external_constraint_violation() {
    let manifest = sample_manifest();
    let mut targets = BTreeMap::new();
    // Migration plans to move `task` backward relative to what an
    // external URN requires, which must surface as a violation.
    targets.insert(
        EntityTypeId::new("task").unwrap(),
        EntityTypeSchemaVersion(1),
    );

    let urn = Urn::new("default", ContentKind::Spec, Uuid::new_v4()).unwrap();
    let constraint = ExternalUrnVersionConstraint::new(
        urn.clone(),
        EntityTypeId::new("task").unwrap(),
        EntityTypeSchemaVersion(2),
    );

    let report = MigrationDryRunReport::plan(
        &manifest,
        DomainSchemaVersion(1),
        &targets,
        &BTreeMap::new(),
        &[constraint.clone()],
    );

    assert_eq!(report.external_urn_violations.len(), 1);
    assert_eq!(report.external_urn_violations[0].constraint, constraint);
    assert_eq!(
        report.external_urn_violations[0].actual_schema_version,
        EntityTypeSchemaVersion(1)
    );
}

#[test]
fn dry_run_plan_reports_no_violation_when_constraint_untouched_by_migration() {
    let manifest = sample_manifest();
    let urn = Urn::new("default", ContentKind::Spec, Uuid::new_v4()).unwrap();
    // `epic` is not part of `target_entity_type_versions`, so the
    // constraint falls back to "no change planned" and is satisfied.
    let constraint = ExternalUrnVersionConstraint::new(
        urn,
        EntityTypeId::new("epic").unwrap(),
        EntityTypeSchemaVersion(2),
    );

    let report = MigrationDryRunReport::plan(
        &manifest,
        DomainSchemaVersion(1),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &[constraint],
    );

    assert!(report.external_urn_violations.is_empty());
}

// --- Journal resume/rollback metadata ---

fn sample_report() -> MigrationDryRunReport {
    MigrationDryRunReport::plan(
        &sample_manifest(),
        DomainSchemaVersion(2),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &[],
    )
}

#[test]
fn journal_starts_planned_and_advances_through_publish() {
    let mut journal = MigrationJournal::new(Uuid::new_v4(), sample_report(), Utc::now());
    assert_eq!(journal.current_phase(), MigrationPhase::Planned);
    assert_eq!(journal.resume_phase(), MigrationPhase::Planned);

    journal.advance(MigrationPhase::Staged, Utc::now()).unwrap();
    assert_eq!(journal.current_phase(), MigrationPhase::Staged);

    journal
        .advance(MigrationPhase::Published, Utc::now())
        .unwrap();
    assert_eq!(journal.current_phase(), MigrationPhase::Published);
    assert_eq!(journal.history().len(), 3);
}

#[test]
fn journal_rejects_invalid_advance_and_preserves_prior_phase() {
    let mut journal = MigrationJournal::new(Uuid::new_v4(), sample_report(), Utc::now());

    let err = journal
        .advance(MigrationPhase::Published, Utc::now())
        .unwrap_err();
    assert_eq!(
        err,
        MigrationJournalError::InvalidTransition(MigrationPhaseError::InvalidTransition {
            from: MigrationPhase::Planned,
            to: MigrationPhase::Published,
        })
    );
    // The rejected transition must not have been recorded.
    assert_eq!(journal.current_phase(), MigrationPhase::Planned);
    assert_eq!(journal.history().len(), 1);
}

#[test]
fn journal_resume_phase_reflects_last_durable_phase_after_interruption() {
    let mut journal = MigrationJournal::new(Uuid::new_v4(), sample_report(), Utc::now());
    journal.advance(MigrationPhase::Staged, Utc::now()).unwrap();

    // Simulate resuming a fresh in-memory journal instance from the
    // durably-recorded phase (as if reloaded from disk).
    assert_eq!(journal.resume_phase(), MigrationPhase::Staged);

    journal
        .advance(MigrationPhase::RolledBack, Utc::now())
        .unwrap();
    assert_eq!(journal.resume_phase(), MigrationPhase::RolledBack);
}
