use memory_kernel::lifecycle::{
    BasicLifecycle, Capability, ControlKind, ControlRequest, LifecycleDefinition, LifecycleError,
    LifecycleSchema, LifecycleState, LifecycleSubject, PauseResumeCapability, ReopenCapability,
    TargetControl,
};
use memory_kernel::model::schema::EntityTypeSchema;
use uuid::Uuid;

use LifecycleState::*;

/// A third domain proves capabilities do not depend on Mission or Ticket.
struct Document;

impl LifecycleSubject for Document {
    type Identity = String;
    type Snapshot = LifecycleState;
    type Error = LifecycleError;

    fn provider_name(&self) -> &str {
        "document"
    }

    fn lifecycle_error(error: LifecycleError) -> Self::Error {
        error
    }
}

impl PauseResumeCapability for Document {}
impl ReopenCapability for Document {}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct DocumentSchema {
    statuses: Vec<String>,
}

impl LifecycleSchema for DocumentSchema {}

#[test]
fn parsing_defaults_support_an_unrelated_schema_type() {
    let expected = DocumentSchema {
        statuses: vec!["draft".into(), "published".into()],
    };
    assert_eq!(
        DocumentSchema::from_json(r#"{"statuses":["draft","published"]}"#).unwrap(),
        expected
    );
    assert_eq!(
        DocumentSchema::from_toml(r#"statuses = ["draft", "published"]"#).unwrap(),
        expected
    );
}

#[test]
fn capability_defaults_accept_unrelated_domain_identity() {
    let request = ControlRequest {
        op_id: Uuid::new_v4(),
        expected_control_sequence: 0,
        evidence: None,
    };
    let identity = String::from("document-7");
    for (result, capability) in [
        (Document.pause(&identity, &request), Capability::PauseResume),
        (
            Document.resume(&identity, &request),
            Capability::PauseResume,
        ),
        (Document.reopen(&identity, &request), Capability::Reopen),
    ] {
        assert_eq!(
            result,
            Err(LifecycleError::UnsupportedCapability {
                capability,
                provider: "document".into(),
            })
        );
    }
}

#[test]
fn schemas_parse_in_both_formats_and_use_shared_transition_defaults() {
    let json = r#"{
        "type_id":"document", "fields":{}, "states":["draft","review","published"],
        "transitions":[{"from":"draft","to":"review"},{"from":"review","to":"published"}]
    }"#;
    let schema = EntityTypeSchema::from_json(json).unwrap();
    assert_eq!(LifecycleDefinition::entry_state(&schema), Some("draft"));
    let toml = toml::to_string(&schema).unwrap();
    assert_eq!(EntityTypeSchema::from_toml(&toml).unwrap(), schema);
    for (from, to, expected) in [
        ("draft", "review", true),
        ("review", "draft", true),
        ("draft", "published", false),
        ("missing", "draft", false),
    ] {
        assert_eq!(schema.allows_transition(from, to), expected);
        assert_eq!(
            schema.allows_transition(from, to),
            LifecycleDefinition::allows_transition(&schema, from, to)
        );
    }
    assert_eq!(
        schema.allowed_next_states("review"),
        vec!["draft", "published"]
    );
    assert!(matches!(
        EntityTypeSchema::from_json("{"),
        Err(LifecycleError::InvalidSchema(_))
    ));
    assert!(matches!(
        EntityTypeSchema::from_toml("states ="),
        Err(LifecycleError::InvalidSchema(_))
    ));
}

#[test]
fn defaults_pause_and_resume_each_nonterminal_state() {
    for state in [Planned, Active, Blocked] {
        let mut target = TargetControl::default();
        target
            .apply_control(ControlKind::Pause, state, None, 1)
            .unwrap();
        assert_eq!(target.paused_from, Some(state));
        let before = target.clone();
        assert!(
            target
                .apply_control(ControlKind::Pause, state, None, 2)
                .is_err()
        );
        assert_eq!(target, before);
        assert!(
            target
                .apply_control(ControlKind::Resume, Paused, None, 2)
                .is_err()
        );
        assert_eq!(target, before);
        target
            .apply_control(ControlKind::Resume, state, None, 2)
            .unwrap();
        assert_eq!(target.paused_from, None);
        assert_eq!(target.last_seq, 2);
    }
}

#[test]
fn defaults_require_completion_evidence_and_reject_terminal_mutations() {
    for kind in [ControlKind::Cancel, ControlKind::Complete] {
        let mut target = TargetControl::default();
        if kind == ControlKind::Complete {
            for evidence in [None, Some(""), Some(" \n")] {
                assert!(target.apply_control(kind, Active, evidence, 1).is_err());
                assert_eq!(target, TargetControl::default());
            }
        }
        target
            .apply_control(kind, Active, Some("result"), 1)
            .unwrap();
        assert_eq!(
            target.terminal,
            Some(if kind == ControlKind::Complete {
                Completed
            } else {
                Cancelled
            })
        );
        let before = target.clone();
        for action in [
            ControlKind::Pause,
            ControlKind::Resume,
            ControlKind::Cancel,
            ControlKind::Complete,
        ] {
            assert!(
                target
                    .apply_control(action, Active, Some("result"), 2)
                    .is_err()
            );
            assert_eq!(target, before);
        }
    }
}

#[test]
fn paused_completion_is_refused_and_cancellation_remains_legal() {
    let mut target = TargetControl::default();
    target
        .apply_control(ControlKind::Pause, Active, None, 1)
        .unwrap();
    let before = target.clone();
    assert!(
        target
            .apply_control(ControlKind::Complete, Active, Some("result"), 2)
            .is_err()
    );
    assert_eq!(target, before);
    target
        .apply_control(ControlKind::Cancel, Active, None, 2)
        .unwrap();
    assert_eq!(target.terminal, Some(Cancelled));
    assert_eq!(target.paused_from, None);
}
