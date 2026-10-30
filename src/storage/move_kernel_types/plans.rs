use super::*;

/// Direction of a cross-entity reference relative to the entity being moved.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoveReferenceDirection {
    Inbound,
    Outbound,
}

/// Relationship between the source and target git worktrees.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GitWorktreeTopology {
    /// Both stores resolve to the same git worktree root.
    Same,
    /// Target worktree is nested inside the source worktree (submodule).
    ParentToSubmodule,
    /// Source worktree is nested inside the target worktree (submodule).
    SubmoduleToParent,
    /// The two worktrees are unrelated repositories.
    Unrelated,
}

/// Visibility of a related entity from the destination store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveReferenceVisibility {
    pub related_entity_id: Uuid,
    pub direction: MoveReferenceDirection,
    pub visible_from_destination: bool,
}

/// A lease that blocks the move because the entity is actively leased.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveLeaseBlock {
    pub entity_id: Uuid,
    pub working_by: String,
}

/// A domain-neutral reason a move cannot proceed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveBlocker {
    DifferentGitWorktree {
        #[serde(
            serialize_with = "serialize_normalized_path",
            deserialize_with = "deserialize_pathbuf"
        )]
        source_worktree_root: PathBuf,
        #[serde(
            serialize_with = "serialize_normalized_path",
            deserialize_with = "deserialize_pathbuf"
        )]
        target_worktree_root: PathBuf,
    },
    MissingSourceEntity {
        entity_id: Uuid,
    },
    MissingTargetStore {
        #[serde(
            serialize_with = "serialize_normalized_path",
            deserialize_with = "deserialize_pathbuf"
        )]
        target_store_root: PathBuf,
    },
    TargetStoreOutsideWorkspace {
        #[serde(
            serialize_with = "serialize_normalized_path",
            deserialize_with = "deserialize_pathbuf"
        )]
        target_workspace_root: PathBuf,
        #[serde(
            serialize_with = "serialize_normalized_path",
            deserialize_with = "deserialize_pathbuf"
        )]
        target_store_root: PathBuf,
    },
    ActiveOrStaleBoardEntry {
        entry_id: Uuid,
        status: String,
        agent_id: String,
    },
    ActiveLease {
        entity_id: Uuid,
        working_by: String,
    },
    InvisibleReference {
        related_entity_id: Uuid,
        direction: MoveReferenceDirection,
    },
    DirtyTrackedFiles {
        #[serde(
            serialize_with = "serialize_normalized_path_vec",
            deserialize_with = "deserialize_pathbuf_vec"
        )]
        files: Vec<PathBuf>,
    },
    PathReferenceScanUnavailable {
        reason: String,
    },
}

/// Inbound / outbound related entity ids enumerated by the domain.
#[derive(Debug, Clone, Default)]
pub struct MoveReferences {
    pub inbound: Vec<Uuid>,
    pub outbound: Vec<Uuid>,
}

/// Board rows associated with the entity, split into active/stale vs historical.
///
/// Domains without a board return an empty value via [`MoveBoardState::default`].
#[derive(Debug, Clone, Default)]
pub struct MoveBoardState {
    /// Entries that are currently active or stale (these block the move).
    pub active_entries: Vec<BoardEntry>,
    /// Completed / historical entries (migrated alongside the entity).
    pub historical_entries: Vec<BoardEntry>,
}

/// Tracked files discovered for a complete move set, plus any scan blockers
/// that apply to every entity with an existing source path.
#[derive(Debug, Clone, Default)]
pub struct MovePathReferenceSet {
    pub files_by_entity: std::collections::BTreeMap<Uuid, Vec<PathBuf>>,
    pub blockers: Vec<MoveBlocker>,
}
/// Deterministically normalize a caller-supplied entity-id selection: sort
/// ascending and remove duplicates.
///
/// [`plan_move_set`] applies this before planning so a set plan's identity
/// and lock-path union are reproducible regardless of caller-supplied order
/// or accidental duplicate ids.
pub fn normalize_entity_selection(entity_ids: &[Uuid]) -> Vec<Uuid> {
    let deduped: BTreeSet<Uuid> = entity_ids.iter().copied().collect();
    deduped.into_iter().collect()
}

/// Read-only preflight plan for a move.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovePlan {
    pub entity_id: Uuid,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub source_workspace_root: PathBuf,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub target_workspace_root: PathBuf,
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
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub source_git_worktree_root: PathBuf,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub target_git_worktree_root: PathBuf,
    pub git_worktree_topology: GitWorktreeTopology,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub source_entity_path: PathBuf,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub destination_entity_path: PathBuf,
    pub inbound_related_entity_ids: Vec<Uuid>,
    pub outbound_related_entity_ids: Vec<Uuid>,
    pub reference_visibility: Vec<MoveReferenceVisibility>,
    pub active_board_entries: Vec<BoardEntry>,
    pub historical_board_entries: Vec<BoardEntry>,
    pub active_leases: Vec<MoveLeaseBlock>,
    #[serde(
        serialize_with = "serialize_normalized_path_vec",
        deserialize_with = "deserialize_pathbuf_vec"
    )]
    pub path_reference_files: Vec<PathBuf>,
    pub blockers: Vec<MoveBlocker>,
    pub captured_at: chrono::DateTime<Utc>,
}

impl MovePlan {
    /// The move is supported only when there are no blockers.
    pub fn supported(&self) -> bool {
        self.blockers.is_empty()
    }
}

/// Read-only preflight plan for moving a normalized set of entities to one
/// target workspace.
///
/// `entity_ids` is sorted/deduplicated by [`normalize_entity_selection`].
/// Shared preflight work — store-root resolution, git worktree topology,
/// target-store presence, and the bulk [`MoveDomain`] set hooks — is computed
/// once for the whole set and threaded into each entry of `entity_plans`, so
/// a set of N entities does not repeat that discovery N times even when the
/// domain has not opted into the bulk hooks (in that case the bulk hooks'
/// default fallback still loops once per entity, but from one call site).
/// Per-entity results (source path resolution, path-reference file scan,
/// board/lease blockers) remain per entity so blocker attribution stays
/// entity-specific.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveSetPlan {
    pub entity_ids: Vec<Uuid>,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub target_workspace_root: PathBuf,
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
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub source_git_worktree_root: PathBuf,
    #[serde(
        serialize_with = "serialize_normalized_path",
        deserialize_with = "deserialize_pathbuf"
    )]
    pub target_git_worktree_root: PathBuf,
    pub git_worktree_topology: GitWorktreeTopology,
    pub target_store_present: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entity_plans: Vec<MovePlan>,
    pub captured_at: chrono::DateTime<Utc>,
}

impl MoveSetPlan {
    /// The set move is supported only when every entity plan has no blockers.
    pub fn supported(&self) -> bool {
        !self.entity_plans.is_empty() && self.entity_plans.iter().all(MovePlan::supported)
    }

    /// All blockers across every entity plan, in `entity_plans` order.
    pub fn blockers(&self) -> Vec<&MoveBlocker> {
        self.entity_plans
            .iter()
            .flat_map(|plan| plan.blockers.iter())
            .collect()
    }
}
