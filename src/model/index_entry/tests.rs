use super::*;

fn make_entry() -> IndexEntry {
    IndexEntry {
        id: Uuid::nil(),
        kind: ContentKind::Ticket,
        source_path: "some/ticket.toml".to_string(),
        title: "Example ticket".to_string(),
        summary: "A test entry.".to_string(),
        keywords: vec!["alpha".to_string(), "beta".to_string()],
        scope: None,
        non_goals: None,
        relations: IndexRelations::default(),
        digest: String::new(),
        tags: vec!["tag-a".to_string()],
        generated_at: DateTime::from_timestamp(0, 0).unwrap(),
        source_modified_at: None,
    }
}

#[test]
fn digest_is_stable_across_calls() {
    let entry = make_entry();
    let d1 = entry.compute_digest();
    let d2 = entry.compute_digest();
    assert_eq!(d1, d2, "digest must be deterministic");
}

#[test]
fn digest_changes_when_title_changes() {
    let mut a = make_entry();
    let mut b = make_entry();
    b.title = "Different title".to_string();
    assert_ne!(
        a.compute_digest(),
        b.compute_digest(),
        "digest must change when title changes"
    );
    // generated_at does NOT affect digest
    a.generated_at = DateTime::from_timestamp(9999, 0).unwrap();
    assert_eq!(
        a.compute_digest(),
        make_entry().compute_digest(),
        "generated_at must not affect digest"
    );
}

#[test]
fn digest_not_affected_by_generated_at() {
    let mut a = make_entry();
    let mut b = make_entry();
    b.generated_at = DateTime::from_timestamp(99999, 0).unwrap();
    assert_eq!(
        a.compute_digest(),
        b.compute_digest(),
        "generated_at must be excluded from digest"
    );
    // source_modified_at also excluded
    a.source_modified_at = Some(DateTime::from_timestamp(1, 0).unwrap());
    assert_eq!(
        a.compute_digest(),
        make_entry().compute_digest(),
        "source_modified_at must be excluded from digest"
    );
}

#[test]
fn seal_sets_digest_field() {
    let mut entry = make_entry();
    assert!(entry.digest.is_empty());
    entry.seal();
    assert!(!entry.digest.is_empty());
    assert!(entry.is_digest_valid());
}

#[test]
fn is_digest_valid_detects_mutation() {
    let mut entry = make_entry();
    entry.seal();
    assert!(entry.is_digest_valid());
    entry.title = "mutated".to_string();
    assert!(!entry.is_digest_valid());
}

#[test]
fn roundtrip_json() {
    let mut entry = make_entry();
    entry.seal();
    let json = serde_json::to_string(&entry).unwrap();
    let decoded: IndexEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(entry, decoded);
}

#[test]
fn keywords_sorted_before_digest() {
    let mut a = make_entry();
    let mut b = make_entry();
    // same keywords, different insertion order
    a.keywords = vec!["beta".to_string(), "alpha".to_string()];
    b.keywords = vec!["alpha".to_string(), "beta".to_string()];
    assert_eq!(
        a.compute_digest(),
        b.compute_digest(),
        "keyword order must not affect digest"
    );
}

#[test]
fn content_kind_serde_roundtrip() {
    let kinds = [
        ContentKind::Ticket,
        ContentKind::Spec,
        ContentKind::Rule,
        ContentKind::Test,
        ContentKind::AuditFinding,
        ContentKind::WorkspaceSummary,
        ContentKind::RuleCatalog,
        ContentKind::Index,
        ContentKind::AgentHook,
    ];
    for kind in kinds {
        let s = serde_json::to_string(&kind).unwrap();
        let decoded: ContentKind = serde_json::from_str(&s).unwrap();
        assert_eq!(kind, decoded);
    }
}

#[test]
fn relation_kind_serde_roundtrip() {
    let kinds = [
        RelationKind::Parent,
        RelationKind::Child,
        RelationKind::DependsOn,
        RelationKind::Related,
        RelationKind::Blocks,
        RelationKind::Supersedes,
    ];
    for kind in kinds {
        let s = serde_json::to_string(&kind).unwrap();
        let decoded: RelationKind = serde_json::from_str(&s).unwrap();
        assert_eq!(kind, decoded);
    }
}
