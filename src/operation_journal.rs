use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::storage::{MoveError, MoveExecutionPhase, MoveJournal};

pub const OPERATION_JOURNAL_SCHEMA_VERSION: &str = "operation-journal/v1";

/// Domain-neutral envelope for durable operation recovery metadata.
///
/// `domain_data` preserves the complete domain journal during the migration to
/// the shared envelope, while the other fields provide cross-domain discovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationJournal {
    pub journal_id: Uuid,
    pub operation_id: Uuid,
    pub run_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub schema_version: String,
    pub operation_kind: String,
    pub component: String,
    pub preflight: OperationPreflight,
    pub steps: Vec<OperationJournalStep>,
    pub phases: Vec<OperationJournalPhase>,
    pub reversibility: OperationReversibility,
    pub recovery: OperationRecovery,
    #[serde(default)]
    pub links: OperationJournalLinks,
    pub domain_data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationPreflight {
    pub inputs: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationJournalStep {
    pub step_id: String,
    pub planned_mutation: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_entities: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse_operation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationJournalPhase {
    pub phase_key: String,
    pub lifecycle_state: String,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationReversibility {
    Replayable,
    Rollbackable,
    ManualRecovery,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationRecovery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_step: Option<String>,
    pub resume_guidance: String,
    pub rollback_guidance: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationJournalLinks {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub session_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graph_operation_ids: Vec<String>,
}

impl OperationJournal {
    /// Projects a legacy move journal into the shared envelope without changing
    /// the legacy on-disk journal schema.
    pub fn from_move_journal(journal: &MoveJournal) -> Result<Self, MoveError> {
        let domain_data = serde_json::to_value(journal)
            .map_err(|error| MoveError::Domain(format!("serialize move journal: {error}")))?;
        let affected_files = journal
            .rewritten_path_files
            .iter()
            .map(|rewrite| rewrite.path.to_string_lossy().into_owned())
            .collect();
        let manual_recovery = !journal.manual_followups.is_empty();
        let phase_key = move_phase_key(&journal.phase).to_string();

        Ok(Self {
            journal_id: journal.id,
            operation_id: Uuid::new_v5(&journal.id, b"move-operation"),
            run_id: Uuid::new_v5(&journal.id, b"move-run"),
            session_id: None,
            schema_version: OPERATION_JOURNAL_SCHEMA_VERSION.to_string(),
            operation_kind: "move".to_string(),
            component: "memory-kernel".to_string(),
            preflight: OperationPreflight {
                inputs: serde_json::json!({
                    "source_store_root": journal.source_store_root,
                    "target_store_root": journal.target_store_root,
                    "source_entity_path": journal.source_entity_path,
                    "destination_entity_path": journal.destination_entity_path,
                    "lock_paths": journal.lock_paths,
                }),
                blockers: Vec::new(),
                warnings: journal
                    .manual_followups
                    .iter()
                    .map(|followup| followup.reason.clone())
                    .collect(),
                ready: journal.failure.is_none(),
            },
            steps: vec![OperationJournalStep {
                step_id: "move-entity".to_string(),
                planned_mutation: "move entity between stores".to_string(),
                affected_entities: vec![journal.entity_id],
                affected_files,
                inverse_operation: (!journal.rollback_steps.is_empty())
                    .then(|| journal.rollback_steps.join("; ")),
            }],
            phases: vec![OperationJournalPhase {
                phase_key: phase_key.clone(),
                lifecycle_state: phase_key,
                recorded_at: journal.updated_at,
            }],
            reversibility: if manual_recovery {
                OperationReversibility::ManualRecovery
            } else {
                OperationReversibility::Rollbackable
            },
            recovery: OperationRecovery {
                failure: journal.failure.clone(),
                next_step: journal.next_recovery_step.clone(),
                resume_guidance: "resume the move with the stable journal_id".to_string(),
                rollback_guidance: journal.rollback_steps.join("; "),
            },
            links: OperationJournalLinks::default(),
            domain_data,
        })
    }

    /// Restores the original legacy journal from a move envelope.
    pub fn into_move_journal(self) -> Result<MoveJournal, MoveError> {
        if self.schema_version != OPERATION_JOURNAL_SCHEMA_VERSION {
            return Err(MoveError::Domain(format!(
                "unsupported operation journal schema version {}",
                self.schema_version
            )));
        }
        if self.operation_kind != "move" {
            return Err(MoveError::Domain(format!(
                "operation journal kind {} is not a move",
                self.operation_kind
            )));
        }
        let journal = serde_json::from_value(self.domain_data)
            .map_err(|error| MoveError::Domain(format!("deserialize move journal: {error}")))?;
        Ok(journal)
    }
}

fn move_phase_key(phase: &MoveExecutionPhase) -> &'static str {
    match phase {
        MoveExecutionPhase::Planned => "planned",
        MoveExecutionPhase::Locked => "locked",
        MoveExecutionPhase::Moved => "moved",
        MoveExecutionPhase::SourceScanned => "source_scanned",
        MoveExecutionPhase::TargetScanned => "target_scanned",
        MoveExecutionPhase::Validated => "validated",
        MoveExecutionPhase::RolledBack => "rolled_back",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn move_journal_projection_round_trips_recovery_data() {
        let journal = MoveJournal {
            id: Uuid::new_v4(),
            entity_id: Uuid::new_v4(),
            source_store_root: PathBuf::from("/stores/source"),
            target_store_root: PathBuf::from("/stores/target"),
            source_entity_path: PathBuf::from("/stores/source/entity"),
            destination_entity_path: PathBuf::from("/stores/target/entity"),
            phase: MoveExecutionPhase::Moved,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec!["moved entity".to_string()],
            rollback_steps: vec!["restore source entity".to_string()],
            lock_paths: vec![PathBuf::from("/stores/source/move.lock")],
            migrated_board_entries: Vec::new(),
            rewritten_path_files: Vec::new(),
            manual_followups: Vec::new(),
            phase_timings_ms: Default::default(),
            failure: Some("target scan interrupted".to_string()),
            next_recovery_step: Some("resume target scan".to_string()),
        };

        let envelope = OperationJournal::from_move_journal(&journal).unwrap();

        assert_eq!(envelope.journal_id, journal.id);
        assert_eq!(envelope.operation_kind, "move");
        assert_eq!(envelope.reversibility, OperationReversibility::Rollbackable);
        let restored = envelope.into_move_journal().unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(journal).unwrap()
        );
    }
}
