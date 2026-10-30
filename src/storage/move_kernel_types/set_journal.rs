use super::*;

/// Result of a set-level journaled move execution.
///
/// Each entity in the set is still executed with its own [`MoveJournal`] (see
/// [`execute_move_set`]'s documentation for why a combined single-journal
/// schema is not implemented in this package), so any entity's outcome can be
/// independently resumed or rolled back via the existing
/// [`resume_move`]/[`rollback_move`] using `entity_outcomes[i].journal.id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveSetOutcome {
    pub journal: MoveSetJournal,
    pub entity_ids: Vec<Uuid>,
    pub entity_outcomes: Vec<MoveOutcome>,
}

/// Durable phase of a set-level move operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoveSetExecutionPhase {
    Planned,
    InProgress,
    Validated,
    RolledBack,
}

/// Durable operation record for a normalized set move.
///
/// The immutable [`MoveSetPlan`] payload lets recovery continue an interrupted
/// operation without rebuilding preflight state. `entity_journal_ids` retains
/// the compatibility journals used by the established single-entity APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveSetJournal {
    pub id: Uuid,
    pub entity_ids: Vec<Uuid>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entity_plans: Vec<MovePlan>,
    pub entity_journal_ids: std::collections::BTreeMap<Uuid, Uuid>,
    #[serde(
        serialize_with = "serialize_normalized_path_vec",
        deserialize_with = "deserialize_pathbuf_vec"
    )]
    pub lock_paths: Vec<PathBuf>,
    pub phase: MoveSetExecutionPhase,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed_entity_ids: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rollback_completed_entity_ids: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub entity_errors: std::collections::BTreeMap<Uuid, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_recovery_step: Option<String>,
}

impl MoveSetJournal {
    /// Validate the immutable selection, plan payload, and recovery identity.
    pub fn validate(&self) -> MoveResult<()> {
        if self.id.is_nil()
            || self.entity_ids.is_empty()
            || self.source_store_root.as_os_str().is_empty()
            || self.target_store_root.as_os_str().is_empty()
            || self.entity_ids != normalize_entity_selection(&self.entity_ids)
            || (!matches!(
                self.phase,
                MoveSetExecutionPhase::Validated | MoveSetExecutionPhase::RolledBack
            ) && self.entity_plans.len() != self.entity_ids.len())
            || self.entity_journal_ids.len() != self.entity_ids.len()
        {
            return Err(MoveError::Domain(
                "move set journal is missing valid immutable operation state".to_string(),
            ));
        }
        let planned_ids = normalize_entity_selection(
            &self
                .entity_plans
                .iter()
                .map(|plan| plan.entity_id)
                .collect::<Vec<_>>(),
        );
        let journal_ids = self.entity_journal_ids.keys().copied().collect::<Vec<_>>();
        if (!matches!(
            self.phase,
            MoveSetExecutionPhase::Validated | MoveSetExecutionPhase::RolledBack
        ) && planned_ids != self.entity_ids)
            || journal_ids != self.entity_ids
        {
            return Err(MoveError::Domain(
                "move set journal entity plan does not match selection".to_string(),
            ));
        }
        Ok(())
    }
}
