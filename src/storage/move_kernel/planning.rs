use super::*;

/// Build a read-only preflight plan for moving `entity_id` to
/// `target_workspace_root`.
pub fn plan_move<D: MoveDomain + ?Sized>(
    domain: &D,
    entity_id: &Uuid,
    target_workspace_root: &Path,
) -> MoveResult<MovePlan> {
    let index_dir = domain.store_index_dir().to_string();
    let source_store_root = domain.source_store_root();
    let source_workspace_root =
        crate::workspace::resolve_workspace_root_from_store_root(&source_store_root, &index_dir);
    let target_store_root =
        crate::workspace::canonical_store_root(target_workspace_root, &index_dir);

    let subdir = domain.entity_subdir().to_string();
    let source_entity_path = domain.source_entity_path(entity_id)?;
    let resolved_source_path = source_entity_path
        .clone()
        .unwrap_or_else(|| source_store_root.join(&subdir).join(entity_id.to_string()));
    let destination_entity_path = target_store_root.join(&subdir).join(entity_id.to_string());

    let mut blockers = Vec::new();

    if !target_store_root.starts_with(target_workspace_root) {
        blockers.push(MoveBlocker::TargetStoreOutsideWorkspace {
            target_workspace_root: target_workspace_root.to_path_buf(),
            target_store_root: target_store_root.clone(),
        });
    }

    let source_git_root = git_toplevel(&source_workspace_root).map_err(MoveError::Domain)?;
    let target_git_root =
        resolve_target_git_root_or_block(target_workspace_root, &source_git_root, &mut blockers);

    let git_worktree_topology = classify_git_worktree_topology(&source_git_root, &target_git_root);
    if git_worktree_topology == GitWorktreeTopology::Unrelated {
        blockers.push(MoveBlocker::DifferentGitWorktree {
            source_worktree_root: source_git_root.clone(),
            target_worktree_root: target_git_root.clone(),
        });
    }

    let target_store_present = domain.target_store_present(&target_store_root)?;
    if !target_store_present && !domain.can_initialize_target_store() {
        blockers.push(MoveBlocker::MissingTargetStore {
            target_store_root: target_store_root.clone(),
        });
    }

    if source_entity_path.is_none() {
        blockers.push(MoveBlocker::MissingSourceEntity {
            entity_id: *entity_id,
        });
    }

    let references = domain.related_entities(entity_id)?;
    let inbound: BTreeSet<Uuid> = references.inbound.into_iter().collect();
    let outbound: BTreeSet<Uuid> = references.outbound.into_iter().collect();

    let reference_visibility = build_reference_visibility(
        domain,
        &target_store_root,
        target_store_present,
        &inbound,
        &outbound,
    )?;

    let board_state = domain.board_state(entity_id)?;
    for entry in &board_state.active_entries {
        blockers.push(MoveBlocker::ActiveOrStaleBoardEntry {
            entry_id: entry.entry_id,
            status: format!("{:?}", entry.status),
            agent_id: entry.agent_id.clone(),
        });
    }

    let active_leases = domain.active_leases(entity_id)?;
    for lease in &active_leases {
        blockers.push(MoveBlocker::ActiveLease {
            entity_id: lease.entity_id,
            working_by: lease.working_by.clone(),
        });
    }

    let path_reference_files = collect_plan_path_reference_files(
        source_entity_path.is_some(),
        &source_git_root,
        &target_git_root,
        &resolved_source_path,
        &source_store_root,
        &target_store_root,
        &subdir,
        &mut blockers,
    );

    Ok(MovePlan {
        entity_id: *entity_id,
        source_workspace_root,
        target_workspace_root: target_workspace_root.to_path_buf(),
        source_store_root,
        target_store_root,
        source_git_worktree_root: source_git_root,
        target_git_worktree_root: target_git_root,
        git_worktree_topology,
        source_entity_path: resolved_source_path,
        destination_entity_path,
        inbound_related_entity_ids: inbound.into_iter().collect(),
        outbound_related_entity_ids: outbound.into_iter().collect(),
        reference_visibility,
        active_board_entries: board_state.active_entries,
        historical_board_entries: board_state.historical_entries,
        active_leases,
        path_reference_files,
        blockers,
        captured_at: Utc::now(),
    })
}

/// Build a read-only preflight plan for moving a normalized set of entities
/// to `target_workspace_root`.
///
/// Deduplicates and sorts `entity_ids` via [`normalize_entity_selection`],
/// resolves store roots and git worktree topology once for the whole set,
/// and calls the [`MoveDomain`] bulk set hooks
/// ([`MoveDomain::related_entities_for_set`],
/// [`MoveDomain::entity_indexed_in_many`],
/// [`MoveDomain::board_state_for_set`], [`MoveDomain::active_leases_for_set`])
/// exactly once for the whole set rather than once per entity, then builds
/// each entity's [`MovePlan`] from that shared context.
///
/// This is a separate implementation from [`plan_move`] rather than a loop
/// that calls it: `plan_move` recomputes git topology and store-root
/// resolution per call, which is exactly the repeated work this function
/// exists to remove. `plan_move` itself is left unchanged as the stable
/// single-entity primitive; see [`execute_move_set`] for the equivalent
/// execution-side design note.
pub fn plan_move_set<D: MoveDomain + ?Sized>(
    domain: &D,
    entity_ids: &[Uuid],
    target_workspace_root: &Path,
) -> MoveResult<MoveSetPlan> {
    let entity_ids = normalize_entity_selection(entity_ids);
    if entity_ids.is_empty() {
        return Err(MoveError::Domain(
            "move set selection is empty after normalization".to_string(),
        ));
    }

    let index_dir = domain.store_index_dir().to_string();
    let source_store_root = domain.source_store_root();
    let source_workspace_root =
        crate::workspace::resolve_workspace_root_from_store_root(&source_store_root, &index_dir);
    let target_store_root =
        crate::workspace::canonical_store_root(target_workspace_root, &index_dir);

    let source_git_root = git_toplevel(&source_workspace_root).map_err(MoveError::Domain)?;
    let mut shared_blockers = Vec::new();
    if !target_store_root.starts_with(target_workspace_root) {
        shared_blockers.push(MoveBlocker::TargetStoreOutsideWorkspace {
            target_workspace_root: target_workspace_root.to_path_buf(),
            target_store_root: target_store_root.clone(),
        });
    }
    let target_git_root = resolve_target_git_root_or_block(
        target_workspace_root,
        &source_git_root,
        &mut shared_blockers,
    );
    let git_worktree_topology = classify_git_worktree_topology(&source_git_root, &target_git_root);
    let target_store_present = domain.target_store_present(&target_store_root)?;

    // Bulk context computed once for the whole set (findings F2-F4).
    let mut related_by_entity = domain.related_entities_for_set(&entity_ids)?;
    let mut all_related: BTreeSet<Uuid> = BTreeSet::new();
    for references in related_by_entity.values() {
        all_related.extend(references.inbound.iter().copied());
        all_related.extend(references.outbound.iter().copied());
    }
    let all_related: Vec<Uuid> = all_related.into_iter().collect();
    let visibility_by_id = if target_store_present {
        domain.entity_indexed_in_many(&target_store_root, &all_related)?
    } else {
        std::collections::BTreeMap::new()
    };
    let mut board_state_by_entity = domain.board_state_for_set(&entity_ids)?;
    let mut active_leases_by_entity = domain.active_leases_for_set(&entity_ids)?;

    let mut source_entity_paths = std::collections::BTreeMap::new();
    for entity_id in &entity_ids {
        if let Some(path) = domain.source_entity_path(entity_id)? {
            source_entity_paths.insert(*entity_id, path);
        }
    }
    let path_reference_set = domain.path_reference_files_for_set(
        &source_entity_paths,
        &source_git_root,
        &target_git_root,
        &source_store_root,
        &target_store_root,
        domain.entity_subdir(),
    )?;

    let mut entity_plans = Vec::with_capacity(entity_ids.len());
    for entity_id in &entity_ids {
        let references = related_by_entity.remove(entity_id).unwrap_or_default();
        let board_state = board_state_by_entity.remove(entity_id).unwrap_or_default();
        let active_leases = active_leases_by_entity
            .remove(entity_id)
            .unwrap_or_default();
        entity_plans.push(build_entity_plan_with_shared_context(
            domain,
            entity_id,
            target_workspace_root,
            &source_workspace_root,
            &source_store_root,
            &target_store_root,
            &source_git_root,
            &target_git_root,
            &git_worktree_topology,
            target_store_present,
            &shared_blockers,
            references,
            &visibility_by_id,
            board_state,
            active_leases,
            source_entity_paths.get(entity_id).cloned(),
            path_reference_set
                .files_by_entity
                .get(entity_id)
                .cloned()
                .unwrap_or_default(),
            &path_reference_set.blockers,
        )?);
    }

    Ok(MoveSetPlan {
        entity_ids,
        target_workspace_root: target_workspace_root.to_path_buf(),
        source_store_root,
        target_store_root,
        source_git_worktree_root: source_git_root,
        target_git_worktree_root: target_git_root,
        git_worktree_topology,
        target_store_present,
        entity_plans,
        captured_at: Utc::now(),
    })
}

/// Build one entity's [`MovePlan`] from precomputed set-level shared context.
///
/// Mirrors [`plan_move`]'s blocker construction exactly (git-worktree
/// mismatch, missing target store, missing source entity, active board
/// entries, active leases, path-reference scan), but reads shared
/// git-topology/store-root/bulk-hook results instead of recomputing them.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_entity_plan_with_shared_context<D: MoveDomain + ?Sized>(
    domain: &D,
    entity_id: &Uuid,
    target_workspace_root: &Path,
    source_workspace_root: &Path,
    source_store_root: &Path,
    target_store_root: &Path,
    source_git_root: &Path,
    target_git_root: &Path,
    git_worktree_topology: &GitWorktreeTopology,
    target_store_present: bool,
    shared_blockers: &[MoveBlocker],
    references: MoveReferences,
    visibility_by_id: &std::collections::BTreeMap<Uuid, bool>,
    board_state: MoveBoardState,
    active_leases: Vec<MoveLeaseBlock>,
    source_entity_path: Option<PathBuf>,
    path_reference_files: Vec<PathBuf>,
    path_reference_blockers: &[MoveBlocker],
) -> MoveResult<MovePlan> {
    let subdir = domain.entity_subdir().to_string();
    let resolved_source_path = source_entity_path
        .clone()
        .unwrap_or_else(|| source_store_root.join(&subdir).join(entity_id.to_string()));
    let destination_entity_path = target_store_root.join(&subdir).join(entity_id.to_string());

    let mut blockers = shared_blockers.to_vec();
    blockers.extend(path_reference_blockers.iter().cloned());
    if *git_worktree_topology == GitWorktreeTopology::Unrelated {
        blockers.push(MoveBlocker::DifferentGitWorktree {
            source_worktree_root: source_git_root.to_path_buf(),
            target_worktree_root: target_git_root.to_path_buf(),
        });
    }
    if !target_store_present && !domain.can_initialize_target_store() {
        blockers.push(MoveBlocker::MissingTargetStore {
            target_store_root: target_store_root.to_path_buf(),
        });
    }
    if source_entity_path.is_none() {
        blockers.push(MoveBlocker::MissingSourceEntity {
            entity_id: *entity_id,
        });
    }

    let inbound: BTreeSet<Uuid> = references.inbound.into_iter().collect();
    let outbound: BTreeSet<Uuid> = references.outbound.into_iter().collect();

    let mut reference_visibility = Vec::new();
    for related_entity_id in inbound.iter().chain(outbound.iter()).copied() {
        let visible_from_destination =
            target_store_present && *visibility_by_id.get(&related_entity_id).unwrap_or(&false);
        let direction = if outbound.contains(&related_entity_id) {
            MoveReferenceDirection::Outbound
        } else {
            MoveReferenceDirection::Inbound
        };
        reference_visibility.push(MoveReferenceVisibility {
            related_entity_id,
            direction,
            visible_from_destination,
        });
    }

    for entry in &board_state.active_entries {
        blockers.push(MoveBlocker::ActiveOrStaleBoardEntry {
            entry_id: entry.entry_id,
            status: format!("{:?}", entry.status),
            agent_id: entry.agent_id.clone(),
        });
    }

    for lease in &active_leases {
        blockers.push(MoveBlocker::ActiveLease {
            entity_id: lease.entity_id,
            working_by: lease.working_by.clone(),
        });
    }

    Ok(MovePlan {
        entity_id: *entity_id,
        source_workspace_root: source_workspace_root.to_path_buf(),
        target_workspace_root: target_workspace_root.to_path_buf(),
        source_store_root: source_store_root.to_path_buf(),
        target_store_root: target_store_root.to_path_buf(),
        source_git_worktree_root: source_git_root.to_path_buf(),
        target_git_worktree_root: target_git_root.to_path_buf(),
        git_worktree_topology: git_worktree_topology.clone(),
        source_entity_path: resolved_source_path,
        destination_entity_path,
        inbound_related_entity_ids: inbound.into_iter().collect(),
        outbound_related_entity_ids: outbound.into_iter().collect(),
        reference_visibility,
        active_board_entries: board_state.active_entries,
        historical_board_entries: board_state.historical_entries,
        active_leases,
        path_reference_files,
        blockers,
        captured_at: Utc::now(),
    })
}
