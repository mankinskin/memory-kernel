use super::*;

/// A tracked text file rewritten during move execution.
///
/// Rollback prefers git-backed restore metadata and falls back to the legacy
/// inline snapshot form when resuming older journals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovePathRewrite {
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub path: PathBuf,
    #[serde(
        default,
        skip_serializing_if = "path_buf_is_empty",
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub repo_root: PathBuf,
    #[serde(
        default,
        skip_serializing_if = "path_buf_is_empty",
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub repo_relative_path: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replacements: Vec<MoveTextReplacement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveTextReplacement {
    pub before: String,
    pub after: String,
}

/// A tracked reference that requires manual follow-up (binary content, no match).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveManualFollowup {
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub path: PathBuf,
    pub reason: String,
}

/// Phases of a journaled move, persisted for resume/rollback.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoveExecutionPhase {
    Planned,
    Locked,
    Moved,
    SourceScanned,
    TargetScanned,
    Validated,
    RolledBack,
}

/// Durable journal of a move, written after every phase transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveJournal {
    pub id: Uuid,
    /// Identity of the moved entity. Accepts the legacy `ticket_id` key for
    /// journals written before the kernel generalization.
    #[serde(alias = "ticket_id")]
    pub entity_id: Uuid,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub source_store_root: PathBuf,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub target_store_root: PathBuf,
    /// Source on-disk entity path. Accepts the legacy `source_ticket_path` key.
    #[serde(alias = "source_ticket_path")]
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub source_entity_path: PathBuf,
    /// Destination on-disk entity path. Accepts the legacy
    /// `destination_ticket_path` key.
    #[serde(alias = "destination_ticket_path")]
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub destination_entity_path: PathBuf,
    pub phase: MoveExecutionPhase,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub steps: Vec<String>,
    pub rollback_steps: Vec<String>,
    #[serde(
        default,
        serialize_with = "serialize_normalized_path_vec",
        deserialize_with = "deserialize_pathbuf_vec"
    )]
    pub lock_paths: Vec<PathBuf>,
    #[serde(default)]
    pub migrated_board_entries: Vec<BoardEntry>,
    #[serde(default)]
    pub rewritten_path_files: Vec<MovePathRewrite>,
    #[serde(default)]
    pub manual_followups: Vec<MoveManualFollowup>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub phase_timings_ms: std::collections::BTreeMap<String, u64>,
    pub failure: Option<String>,
    #[serde(default)]
    pub next_recovery_step: Option<String>,
}

/// Result of a journaled move execution / resume / rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveOutcome {
    pub journal: MoveJournal,
    pub resumed: bool,
    pub rolled_back: bool,
}

impl InteroperableArtifact for MoveJournal {
    fn artifact_class(&self) -> &'static str {
        "move-journal"
    }

    fn interoperability_gaps(&self) -> Vec<&'static str> {
        let mut gaps = Vec::new();
        if self.id.is_nil() {
            gaps.push("missing authoritative journal identity");
        }
        if self.entity_id.is_nil() {
            gaps.push("missing authoritative operation identity");
        }
        if self.source_store_root.as_os_str().is_empty() {
            gaps.push("missing source store root");
        }
        if self.target_store_root.as_os_str().is_empty() {
            gaps.push("missing target store root");
        }
        if self.source_entity_path.as_os_str().is_empty() {
            gaps.push("missing source entity path");
        }
        if self.destination_entity_path.as_os_str().is_empty() {
            gaps.push("missing destination entity path");
        }
        if self.steps.is_empty() {
            gaps.push("missing replay/rollback lineage");
        }
        gaps
    }
}

impl MoveJournal {
    /// Return the interoperability-contract gaps for this journal-backed
    /// operation.
    ///
    /// A journal is the authoritative record of an operation and must carry
    /// authoritative identity, replay/rollback lineage, and deterministic
    /// mutation payload ownership before it is persisted. Outward links to
    /// tests or logs stay optional and are intentionally not required here so
    /// journal authority never moves elsewhere.
    pub fn interoperability_gaps(&self) -> Vec<&'static str> {
        <Self as InteroperableArtifact>::interoperability_gaps(self)
    }

    /// Validate the journal-backed operation interoperability contract.
    ///
    /// Returns [`MoveError::InteroperabilityContract`] when a required
    /// identity, lineage, or mutation-payload-ownership field is missing. This
    /// is enforced at the journal persistence boundary so a non-compliant
    /// journal is never written to disk.
    pub fn validate_interoperability_contract(&self) -> MoveResult<()> {
        let gaps = self.interoperability_gaps();
        if gaps.is_empty() {
            return Ok(());
        }
        Err(MoveError::InteroperabilityContract {
            artifact_class: <Self as InteroperableArtifact>::artifact_class(self),
            detail: gaps.join(", "),
        })
    }
}

#[cfg(test)]
mod interoperability_contract_tests {
    use super::*;

    fn compliant_journal() -> MoveJournal {
        MoveJournal {
            id: Uuid::new_v4(),
            entity_id: Uuid::new_v4(),
            source_store_root: PathBuf::from("/stores/source"),
            target_store_root: PathBuf::from("/stores/target"),
            source_entity_path: PathBuf::from("/stores/source/entity"),
            destination_entity_path: PathBuf::from("/stores/target/entity"),
            phase: MoveExecutionPhase::Planned,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec!["created move journal".to_string()],
            rollback_steps: vec!["rename destination back to source".to_string()],
            lock_paths: Vec::new(),
            migrated_board_entries: Vec::new(),
            rewritten_path_files: Vec::new(),
            manual_followups: Vec::new(),
            phase_timings_ms: std::collections::BTreeMap::new(),
            failure: None,
            next_recovery_step: None,
        }
    }

    #[test]
    fn compliant_journal_satisfies_interoperability_contract() {
        let journal = compliant_journal();
        assert!(journal.interoperability_gaps().is_empty());
        assert!(journal.validate_interoperability_contract().is_ok());
    }

    #[test]
    fn nil_operation_identity_violates_interoperability_contract() {
        let mut journal = compliant_journal();
        journal.entity_id = Uuid::nil();

        let error = journal
            .validate_interoperability_contract()
            .expect_err("nil operation identity must fail the contract");
        match error {
            MoveError::InteroperabilityContract {
                artifact_class,
                detail,
            } => {
                assert!(
                    artifact_class == "move-journal",
                    "unexpected detail: {detail}"
                );
                assert!(
                    detail.contains("missing authoritative operation identity"),
                    "unexpected detail: {detail}"
                );
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn missing_mutation_payload_ownership_violates_contract() {
        let mut journal = compliant_journal();
        journal.source_store_root = PathBuf::new();
        journal.destination_entity_path = PathBuf::new();

        let gaps = journal.interoperability_gaps();
        assert!(gaps.contains(&"missing source store root"));
        assert!(gaps.contains(&"missing destination entity path"));
        assert!(journal.validate_interoperability_contract().is_err());
    }
}
