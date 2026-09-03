//! Domain-neutral cross-workspace move kernel.
//!
//! This module owns the *generic* machinery for moving a filesystem-backed
//! entity (a folder identified by a [`Uuid`]) between two stores that may live
//! in the same git worktree or in a parent/submodule relationship:
//!
//! - read-only preflight planning ([`plan_move`]),
//! - journaled / resumable / rollbackable execution
//!   ([`execute_move`], [`resume_move`], [`rollback_move`]),
//! - set-level preflight and execution for an arbitrary, deterministically
//!   normalized selection of entities ([`plan_move_set`],
//!   [`execute_move_set`]), which share store-root/git-topology resolution
//!   and lock acquisition across the whole set instead of once per entity,
//! - git worktree topology classification, tracked-path-reference scanning and
//!   rewriting, dirty-file detection, lock-set management, and journal
//!   persistence.
//!
//! All domain-specific behavior (entity path resolution, edge/reference
//! enumeration, destination visibility, board/lease detection and historical
//! migration, store rescans) is injected through the [`MoveDomain`] trait, so
//! every domain store (ticket, spec, rule, audit, session, feedback) can opt
//! into the same safe move featureset without copying logic.
//!
//! The kernel contains **no** ticket-specific types: identities are bare
//! [`Uuid`]s and board rows are the shared [`BoardEntry`].

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::field::Empty;
use uuid::Uuid;

use crate::storage::board::BoardEntry;

const MOVE_LOCKS_DIR: &str = "move-locks";
const MOVE_JOURNALS_DIR: &str = "move-journals";
const MOVE_TRACE_TARGET: &str = "memory_kernel::storage::move_kernel";
const MOVE_SET_JOURNAL_CHECKPOINT_INTERVAL: usize = 8;

#[path = "move_kernel/internal.rs"]
mod internal;
use internal::*;
pub use internal::{
    collect_lock_paths, load_journal, load_move_set_journal, persist_journal,
    persist_move_set_journal,
};
/// Error type for the move kernel.
///
/// Domain trait implementations map their own storage errors onto
/// [`MoveError::Domain`]; the kernel raises [`MoveError::Io`] for filesystem
/// failures it performs directly (rename, lock files, journal persistence).

#[path = "move_kernel_types.rs"]
mod move_kernel_types;
use move_kernel_types::path_buf_is_empty;
pub use move_kernel_types::*;

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
        crate::workspace::resolve_store_root_from(target_workspace_root, &index_dir);

    let subdir = domain.entity_subdir().to_string();
    let source_entity_path = domain.source_entity_path(entity_id)?;
    let resolved_source_path = source_entity_path
        .clone()
        .unwrap_or_else(|| source_store_root.join(&subdir).join(entity_id.to_string()));
    let destination_entity_path = target_store_root.join(&subdir).join(entity_id.to_string());

    let mut blockers = Vec::new();

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
    if !target_store_present {
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
        crate::workspace::resolve_store_root_from(target_workspace_root, &index_dir);

    let source_git_root = git_toplevel(&source_workspace_root).map_err(MoveError::Domain)?;
    let mut shared_blockers = Vec::new();
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
fn build_entity_plan_with_shared_context<D: MoveDomain + ?Sized>(
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
    if !target_store_present {
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

/// Execute a supported move with a fresh journal.
pub fn execute_move<D: MoveDomain + ?Sized>(
    domain: &D,
    plan: &MovePlan,
) -> MoveResult<MoveOutcome> {
    let _span_guard = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_execute",
        entity_id = %plan.entity_id,
        supported = plan.supported(),
    )
    .entered();
    if !plan.supported() {
        return Err(MoveError::Domain(
            "move preflight contains blockers".to_string(),
        ));
    }
    execute_or_resume(domain, plan, None, None, false, false, false, false)
}

/// Resume an interrupted move identified by its journal id.
pub fn resume_move<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
) -> MoveResult<MoveOutcome> {
    let _span_guard = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_resume",
        journal_id = %journal_id,
    )
    .entered();
    let journal = load_journal(&domain.source_store_root(), journal_id)?;
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        entity_id = %journal.entity_id,
        phase = phase_name(&journal.phase),
        "move_resume_journal_loaded"
    );
    let target_workspace_root = crate::workspace::resolve_workspace_root_from_store_root(
        &journal.target_store_root,
        domain.store_index_dir(),
    );
    let plan = plan_move(domain, &journal.entity_id, &target_workspace_root)?;
    execute_or_resume(domain, &plan, Some(journal), None, true, false, false, false)
}

/// Roll back a move identified by its journal id.
pub fn rollback_move<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
) -> MoveResult<MoveOutcome> {
    rollback_move_within_lock(domain, journal_id, false, false)
}

fn rollback_move_within_lock<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
    skip_lock: bool,
    skip_reconciliation: bool,
) -> MoveResult<MoveOutcome> {
    let source_store_root = domain.source_store_root();
    let mut journal = load_journal(&source_store_root, journal_id)?;
    normalize_journal_entity_paths(domain, &mut journal);
    let span = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_rollback",
        journal_id = %journal.id,
        entity_id = %journal.entity_id,
        resumed = false,
        rolled_back = true,
        phase = Empty,
    );
    let _span_guard = span.enter();
    span.record("phase", phase_name(&journal.phase));
    if journal.lock_paths.is_empty() {
        journal.lock_paths = collect_lock_paths(
            journal.entity_id,
            &journal.source_store_root,
            &journal.target_store_root,
        );
    }
    if !skip_lock {
        acquire_lock_set(&journal.lock_paths)?;
    }

    if journal.destination_entity_path.exists() && !journal.source_entity_path.exists() {
        if let Some(parent) = journal.source_entity_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(
            &journal.destination_entity_path,
            &journal.source_entity_path,
        )?;
    }

    for rewrite in &journal.rewritten_path_files {
        restore_rewritten_path(rewrite)?;
    }

    if !journal.migrated_board_entries.is_empty() {
        let started = Instant::now();
        domain
            .restore_board_history(&journal.target_store_root, &journal.migrated_board_entries)?;
        record_phase_timing(
            &mut journal,
            "rollback_restore_board_history_ms",
            started.elapsed(),
        );
    }

    let source_scan_started = Instant::now();
    if !skip_reconciliation {
        domain.reconcile_store_touched(&journal.source_store_root, &[journal.entity_id])?;
    }
    record_phase_timing(
        &mut journal,
        "rollback_scan_source_ms",
        source_scan_started.elapsed(),
    );

    let target_scan_started = Instant::now();
    if !skip_reconciliation {
        domain.reconcile_store_touched(&journal.target_store_root, &[journal.entity_id])?;
    }
    record_phase_timing(
        &mut journal,
        "rollback_scan_target_ms",
        target_scan_started.elapsed(),
    );

    journal.phase = MoveExecutionPhase::RolledBack;
    journal.updated_at = Utc::now();
    journal
        .steps
        .push("rolled back entity folder to source store".to_string());
    journal.failure = None;
    journal.next_recovery_step = None;
    persist_journal(&source_store_root, &journal)?;
    if !skip_lock {
        release_lock_set(&journal.lock_paths);
    }

    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        journal_id = %journal.id,
        entity_id = %journal.entity_id,
        phase = phase_name(&journal.phase),
        migrated_board_entries = journal.migrated_board_entries.len(),
        rewritten_path_files = journal.rewritten_path_files.len(),
        manual_followups = journal.manual_followups.len(),
        "move_rollback_complete"
    );

    Ok(MoveOutcome {
        journal,
        resumed: false,
        rolled_back: true,
    })
}

fn execute_or_resume<D: MoveDomain + ?Sized>(
    domain: &D,
    plan: &MovePlan,
    existing: Option<MoveJournal>,
    pre_migrated_board_entries: Option<Vec<BoardEntry>>,
    resumed: bool,
    skip_lock: bool,
    skip_reconciliation: bool,
    skip_validation: bool,
) -> MoveResult<MoveOutcome> {
    let journal_root = domain.source_store_root();
    let mut journal = existing.unwrap_or_else(|| MoveJournal {
        id: Uuid::new_v4(),
        entity_id: plan.entity_id,
        source_store_root: plan.source_store_root.clone(),
        target_store_root: plan.target_store_root.clone(),
        source_entity_path: plan.source_entity_path.clone(),
        destination_entity_path: plan.destination_entity_path.clone(),
        phase: MoveExecutionPhase::Planned,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        steps: vec!["created move journal".to_string()],
        rollback_steps: vec![
            "rename destination entity folder back to source path".to_string(),
            "restore migrated board history rows to source store".to_string(),
            "scan source and target stores".to_string(),
        ],
        lock_paths: collect_lock_paths(
            plan.entity_id,
            &plan.source_store_root,
            &plan.target_store_root,
        ),
        migrated_board_entries: Vec::new(),
        rewritten_path_files: Vec::new(),
        manual_followups: Vec::new(),
        phase_timings_ms: std::collections::BTreeMap::new(),
        failure: None,
        next_recovery_step: None,
    });
    normalize_journal_entity_paths(domain, &mut journal);
    if journal.lock_paths.is_empty() {
        journal.lock_paths = collect_lock_paths(
            journal.entity_id,
            &journal.source_store_root,
            &journal.target_store_root,
        );
    }
    persist_journal(&journal_root, &journal)?;

    let span = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_execute_or_resume",
        journal_id = %journal.id,
        entity_id = %journal.entity_id,
        resumed,
        rolled_back = false,
        phase = Empty,
    );
    let _span_guard = span.enter();
    span.record("phase", phase_name(&journal.phase));
    tracing::debug!(
        target: MOVE_TRACE_TARGET,
        phase = phase_name(&journal.phase),
        "move_journal_ready"
    );

    let result: MoveResult<()> = (|| {
        advance_phase_planned(
            &mut journal,
            &journal_root,
            &span,
            skip_lock,
            !skip_reconciliation,
        )?;
        advance_phase_locked(&mut journal, &journal_root, &span)?;
        advance_phase_moved(
            domain,
            plan,
            &mut journal,
            &journal_root,
            &span,
            skip_reconciliation,
            pre_migrated_board_entries,
        )?;
        advance_phase_source_scanned(
            domain,
            &mut journal,
            &journal_root,
            &span,
            skip_reconciliation,
            !skip_reconciliation,
        )?;
        advance_phase_target_scanned(
            domain,
            &mut journal,
            &journal_root,
            &span,
            skip_validation,
            !skip_validation,
        )?;

        Ok(())
    })();

    match result {
        Ok(()) => {
            if !skip_lock {
                release_lock_set(&journal.lock_paths);
            }
            tracing::debug!(
                target: MOVE_TRACE_TARGET,
                journal_id = %journal.id,
                entity_id = %journal.entity_id,
                phase = phase_name(&journal.phase),
                resumed,
                rewritten_path_files = journal.rewritten_path_files.len(),
                manual_followups = journal.manual_followups.len(),
                migrated_board_entries = journal.migrated_board_entries.len(),
                "move_execute_or_resume_complete"
            );
            Ok(MoveOutcome {
                journal,
                resumed,
                rolled_back: false,
            })
        }
        Err(error) => {
            journal.updated_at = Utc::now();
            journal.failure = Some(error.to_string());
            journal.next_recovery_step = Some(recovery_hint_for_phase(&journal.phase).to_string());
            let _ = persist_journal(&journal_root, &journal);
            if !skip_lock {
                release_lock_set(&journal.lock_paths);
            }
            tracing::error!(
                target: MOVE_TRACE_TARGET,
                journal_id = %journal.id,
                entity_id = %journal.entity_id,
                phase = phase_name(&journal.phase),
                resumed,
                error = %error,
                "move_execute_or_resume_failed"
            );
            Err(error)
        }
    }
}

fn journal_from_plan(plan: &MovePlan, journal_id: Uuid) -> MoveJournal {
    MoveJournal {
        id: journal_id,
        entity_id: plan.entity_id,
        source_store_root: plan.source_store_root.clone(),
        target_store_root: plan.target_store_root.clone(),
        source_entity_path: plan.source_entity_path.clone(),
        destination_entity_path: plan.destination_entity_path.clone(),
        phase: MoveExecutionPhase::Planned,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        steps: vec!["created move journal".to_string()],
        rollback_steps: vec![
            "rename destination entity folder back to source path".to_string(),
            "restore migrated board history rows to source store".to_string(),
            "scan source and target stores".to_string(),
        ],
        lock_paths: collect_lock_paths(
            plan.entity_id,
            &plan.source_store_root,
            &plan.target_store_root,
        ),
        migrated_board_entries: Vec::new(),
        rewritten_path_files: Vec::new(),
        manual_followups: Vec::new(),
        phase_timings_ms: std::collections::BTreeMap::new(),
        failure: None,
        next_recovery_step: None,
    }
}

/// Execute one entity's move as part of an enclosing [`execute_move_set`]
/// operation.
///
/// Unlike [`execute_move`], this does not acquire or release the per-entity
/// lock files: the caller already holds the union of lock paths for the
/// whole set for the lifetime of the set operation.
fn execute_move_within_set<D: MoveDomain + ?Sized>(
    domain: &D,
    plan: &MovePlan,
    journal_id: Uuid,
    existing: Option<MoveJournal>,
    pre_migrated_board_entries: Option<Vec<BoardEntry>>,
    resumed: bool,
) -> MoveResult<MoveOutcome> {
    if !plan.supported() {
        return Err(MoveError::Domain(
            "move preflight contains blockers".to_string(),
        ));
    }
    execute_or_resume(
        domain,
        plan,
        Some(existing.unwrap_or_else(|| journal_from_plan(plan, journal_id))),
        pre_migrated_board_entries,
        resumed,
        true,
        true,
        true,
    )
}

fn validate_move_set_outcomes<D: MoveDomain + ?Sized>(
    domain: &D,
    outcomes: &mut [MoveOutcome],
) -> MoveResult<()> {
    let entity_ids = outcomes
        .iter()
        .map(|outcome| outcome.journal.entity_id)
        .collect::<Vec<_>>();
    let source_seen =
        domain.entity_indexed_in_many(&outcomes[0].journal.source_store_root, &entity_ids)?;
    let target_seen =
        domain.entity_indexed_in_many(&outcomes[0].journal.target_store_root, &entity_ids)?;

    for outcome in outcomes {
        let journal = &mut outcome.journal;
        let source_path_exists = journal.source_entity_path.exists();
        let destination_path_exists = journal.destination_entity_path.exists();
        let source_indexed = source_seen
            .get(&journal.entity_id)
            .copied()
            .unwrap_or(false);
        let target_indexed = target_seen
            .get(&journal.entity_id)
            .copied()
            .unwrap_or(false);
        if source_path_exists || !destination_path_exists {
            return Err(build_post_move_validation_error(
                journal,
                source_path_exists,
                destination_path_exists,
                source_indexed,
                target_indexed,
            ));
        }
        journal.phase = MoveExecutionPhase::Validated;
        journal.updated_at = Utc::now();
        journal
            .steps
            .push("validated move ownership in set".to_string());
        journal.failure = None;
        journal.next_recovery_step = None;
        persist_journal(&journal.source_store_root, journal)?;
    }
    Ok(())
}

/// Execute a supported set of moves, holding the union of source/target store
/// locks for the lifetime of the whole set operation rather than acquiring
/// and releasing them once per entity (finding F1/F7's lock-cycling cost).
///
/// A [`MoveSetJournal`] stores the immutable normalized plan and stable
/// per-entity journal identities before mutation. Recovery therefore resumes
/// only unfinished entries from that durable record, without preflight.
pub fn execute_move_set<D: MoveDomain + ?Sized>(
    domain: &D,
    set_plan: &MoveSetPlan,
) -> MoveResult<MoveSetOutcome> {
    let _span_guard = tracing::debug_span!(
        target: MOVE_TRACE_TARGET,
        "move_execute_set",
        entity_count = set_plan.entity_ids.len(),
        supported = set_plan.supported(),
    )
    .entered();
    if !set_plan.supported() {
        return Err(MoveError::Domain(
            "move set preflight contains blockers".to_string(),
        ));
    }

    let mut lock_paths = BTreeSet::new();
    for plan in &set_plan.entity_plans {
        lock_paths.extend(collect_lock_paths(
            plan.entity_id,
            &plan.source_store_root,
            &plan.target_store_root,
        ));
    }
    let mut journal = MoveSetJournal {
        id: Uuid::new_v4(),
        entity_ids: set_plan.entity_ids.clone(),
        source_store_root: set_plan.source_store_root.clone(),
        target_store_root: set_plan.target_store_root.clone(),
        entity_plans: set_plan.entity_plans.clone(),
        entity_journal_ids: set_plan
            .entity_ids
            .iter()
            .map(|id| (*id, Uuid::new_v4()))
            .collect(),
        lock_paths: lock_paths.into_iter().collect(),
        phase: MoveSetExecutionPhase::Planned,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        completed_entity_ids: Vec::new(),
        rollback_completed_entity_ids: Vec::new(),
        entity_errors: std::collections::BTreeMap::new(),
        failure: None,
        next_recovery_step: None,
    };
    persist_move_set_journal(&journal.source_store_root, &journal)?;
    execute_move_set_journal(domain, &mut journal, false)
}

/// Resume a set operation from its immutable persisted plan without re-running
/// [`plan_move`] or [`plan_move_set`].
pub fn resume_move_set<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
) -> MoveResult<MoveSetOutcome> {
    let mut journal = load_move_set_journal(&domain.source_store_root(), journal_id)?;
    execute_move_set_journal(domain, &mut journal, true)
}

fn execute_move_set_journal<D: MoveDomain + ?Sized>(
    domain: &D,
    journal: &mut MoveSetJournal,
    resumed: bool,
) -> MoveResult<MoveSetOutcome> {
    acquire_lock_set(&journal.lock_paths)?;
    journal.phase = MoveSetExecutionPhase::InProgress;
    journal.updated_at = Utc::now();
    persist_move_set_journal(&journal.source_store_root, journal)?;

    let mut entity_outcomes = Vec::with_capacity(journal.entity_plans.len());
    let mut migrated_board_entries_by_entity = domain.migrate_board_history_for_set(
        &journal.target_store_root,
        &journal.entity_ids,
    )?;
    for plan in &journal.entity_plans {
        if journal.completed_entity_ids.contains(&plan.entity_id) {
            let entity_journal_id = journal.entity_journal_ids[&plan.entity_id];
            let completed = load_journal(&journal.source_store_root, entity_journal_id)?;
            if !matches!(
                completed.phase,
                MoveExecutionPhase::SourceScanned
                    | MoveExecutionPhase::TargetScanned
                    | MoveExecutionPhase::Validated
            ) {
                release_lock_set(&journal.lock_paths);
                return Err(MoveError::Domain(format!(
                    "move set journal marks entity {} complete before its journal validated",
                    plan.entity_id
                )));
            }
            entity_outcomes.push(MoveOutcome {
                journal: completed,
                resumed: true,
                rolled_back: false,
            });
            continue;
        }
        let entity_journal_id = journal.entity_journal_ids[&plan.entity_id];
        let existing = load_journal(&journal.source_store_root, entity_journal_id).ok();
        if let Some(existing) = existing.as_ref() {
            if matches!(
                existing.phase,
                MoveExecutionPhase::TargetScanned | MoveExecutionPhase::Validated
            ) {
                journal.completed_entity_ids.push(plan.entity_id);
                entity_outcomes.push(MoveOutcome {
                    journal: existing.clone(),
                    resumed: true,
                    rolled_back: false,
                });
                continue;
            }
        }
        match execute_move_within_set(
            domain,
            plan,
            entity_journal_id,
            existing,
            Some(
                migrated_board_entries_by_entity
                    .remove(&plan.entity_id)
                    .unwrap_or_default(),
            ),
            resumed,
        ) {
            Ok(outcome) => {
                journal.completed_entity_ids.push(plan.entity_id);
                journal.entity_errors.remove(&plan.entity_id);
                entity_outcomes.push(outcome);
                if journal.completed_entity_ids.len() % MOVE_SET_JOURNAL_CHECKPOINT_INTERVAL == 0 {
                    journal.updated_at = Utc::now();
                    persist_move_set_journal(&journal.source_store_root, journal)?;
                }
            }
            Err(error) => {
                journal
                    .entity_errors
                    .insert(plan.entity_id, error.to_string());
                journal.failure = Some(error.to_string());
                journal.next_recovery_step =
                    Some("run resume_move_set or rollback_move_set".to_string());
                journal.updated_at = Utc::now();
                persist_move_set_journal(&journal.source_store_root, journal)?;
                release_lock_set(&journal.lock_paths);
                return Err(error);
            }
        }
    }

    let reconciliation = (|| {
        domain.reconcile_store_set(&journal.source_store_root, &journal.entity_ids)?;
        domain.reconcile_store_set(&journal.target_store_root, &journal.entity_ids)?;
        Ok::<(), MoveError>(())
    })();
    if let Err(error) = reconciliation {
        journal.failure = Some(error.to_string());
        journal.next_recovery_step = Some("run resume_move_set or rollback_move_set".to_string());
        journal.updated_at = Utc::now();
        let _ = persist_move_set_journal(&journal.source_store_root, journal);
        release_lock_set(&journal.lock_paths);
        return Err(error);
    }
    if let Err(error) = validate_move_set_outcomes(domain, &mut entity_outcomes) {
        journal.failure = Some(error.to_string());
        journal.next_recovery_step = Some("run resume_move_set or rollback_move_set".to_string());
        journal.updated_at = Utc::now();
        let _ = persist_move_set_journal(&journal.source_store_root, journal);
        release_lock_set(&journal.lock_paths);
        return Err(error);
    }
    journal.phase = MoveSetExecutionPhase::Validated;
    journal.failure = None;
    journal.next_recovery_step = None;
    journal.updated_at = Utc::now();
    persist_move_set_journal(&journal.source_store_root, journal)?;
    release_lock_set(&journal.lock_paths);
    Ok(MoveSetOutcome {
        journal: journal.clone(),
        entity_ids: journal.entity_ids.clone(),
        entity_outcomes,
    })
}

/// Roll back completed entries of a set operation in reverse normalized order.
/// Repeating the operation skips entries already recorded as restored.
pub fn rollback_move_set<D: MoveDomain + ?Sized>(
    domain: &D,
    journal_id: Uuid,
) -> MoveResult<MoveSetOutcome> {
    let mut journal = load_move_set_journal(&domain.source_store_root(), journal_id)?;
    acquire_lock_set(&journal.lock_paths)?;
    let mut entity_outcomes = Vec::new();
    for entity_id in journal.entity_ids.iter().rev() {
        if journal.rollback_completed_entity_ids.contains(entity_id) {
            continue;
        }
        let entity_journal_id = journal.entity_journal_ids[entity_id];
        let Ok(entity_journal) = load_journal(&journal.source_store_root, entity_journal_id) else {
            continue;
        };
        if entity_journal.phase == MoveExecutionPhase::Planned {
            continue;
        }
        match rollback_move_within_lock(domain, entity_journal_id, true, true) {
            Ok(outcome) => {
                journal.rollback_completed_entity_ids.push(*entity_id);
                entity_outcomes.push(outcome);
                if journal.rollback_completed_entity_ids.len()
                    % MOVE_SET_JOURNAL_CHECKPOINT_INTERVAL
                    == 0
                {
                    journal.updated_at = Utc::now();
                    persist_move_set_journal(&journal.source_store_root, &journal)?;
                }
            }
            Err(error) => {
                journal.entity_errors.insert(*entity_id, error.to_string());
                journal.failure = Some(error.to_string());
                journal.next_recovery_step =
                    Some("run rollback_move_set again after resolving the error".to_string());
                journal.updated_at = Utc::now();
                persist_move_set_journal(&journal.source_store_root, &journal)?;
                release_lock_set(&journal.lock_paths);
                return Err(error);
            }
        }
    }
    let reconciliation = (|| {
        domain.reconcile_store_set(&journal.source_store_root, &journal.entity_ids)?;
        domain.reconcile_store_set(&journal.target_store_root, &journal.entity_ids)?;
        Ok::<(), MoveError>(())
    })();
    if let Err(error) = reconciliation {
        journal.failure = Some(error.to_string());
        journal.next_recovery_step =
            Some("run rollback_move_set again after resolving the error".to_string());
        journal.updated_at = Utc::now();
        let _ = persist_move_set_journal(&journal.source_store_root, &journal);
        release_lock_set(&journal.lock_paths);
        return Err(error);
    }
    journal.phase = MoveSetExecutionPhase::RolledBack;
    journal.failure = None;
    journal.next_recovery_step = None;
    journal.updated_at = Utc::now();
    persist_move_set_journal(&journal.source_store_root, &journal)?;
    release_lock_set(&journal.lock_paths);
    let entity_ids = journal.entity_ids.clone();
    Ok(MoveSetOutcome {
        journal,
        entity_ids,
        entity_outcomes,
    })
}

#[cfg(test)]
mod move_set_tests {
    use super::*;
    use std::cell::Cell;

    /// Minimal [`MoveDomain`] fixture backed by plain folders under a
    /// `.fixture` store marker: an entity "exists" iff
    /// `<store_root>/entities/<id>` is a directory. No edges, board, or
    /// leases — those hooks use the trait's empty defaults.
    struct FixtureDomain {
        source_store_root: PathBuf,
        fail_scan_number: Cell<Option<usize>>,
        scan_calls: Cell<usize>,
    }

    impl MoveDomain for FixtureDomain {
        fn entity_subdir(&self) -> &str {
            "entities"
        }

        fn store_index_dir(&self) -> &str {
            ".fixture"
        }

        fn source_store_root(&self) -> PathBuf {
            self.source_store_root.clone()
        }

        fn source_entity_path(&self, entity_id: &Uuid) -> MoveResult<Option<PathBuf>> {
            let path = self
                .source_store_root
                .join("entities")
                .join(entity_id.to_string());
            Ok(path.is_dir().then_some(path))
        }

        fn related_entities(&self, _entity_id: &Uuid) -> MoveResult<MoveReferences> {
            Ok(MoveReferences::default())
        }

        fn target_store_present(&self, target_store_root: &Path) -> MoveResult<bool> {
            Ok(target_store_root.is_dir())
        }

        fn entity_indexed_in(&self, store_root: &Path, entity_id: &Uuid) -> MoveResult<bool> {
            Ok(store_root
                .join("entities")
                .join(entity_id.to_string())
                .is_dir())
        }

        fn scan_store(&self, _store_root: &Path) -> MoveResult<()> {
            let scan_number = self.scan_calls.get() + 1;
            self.scan_calls.set(scan_number);
            if self.fail_scan_number.get() == Some(scan_number) {
                return Err(MoveError::Domain("fixture scan failure".to_string()));
            }
            Ok(())
        }
    }

    fn init_git_repo(root: &Path) {
        let status = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(root)
            .status()
            .expect("git init must run");
        assert!(status.success(), "git init failed");
    }

    fn make_entity(store_root: &Path, entity_id: Uuid) {
        fs::create_dir_all(store_root.join("entities").join(entity_id.to_string()))
            .expect("create entity folder");
    }

    /// One git repo containing both `source/` and `target/` workspaces, so
    /// git worktree topology resolves to `Same` without a second repo.
    fn setup_repo() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let repo = tempfile::tempdir().expect("temp repo");
        init_git_repo(repo.path());
        let source_ws = repo.path().join("source");
        let target_ws = repo.path().join("target");
        fs::create_dir_all(source_ws.join(".fixture").join("entities"))
            .expect("create source store");
        fs::create_dir_all(target_ws.join(".fixture")).expect("create target store");
        (repo, source_ws, target_ws)
    }

    #[test]
    fn normalize_entity_selection_sorts_and_dedupes() {
        let a = Uuid::from_u128(3);
        let b = Uuid::from_u128(1);
        let c = Uuid::from_u128(2);
        let normalized = normalize_entity_selection(&[a, b, c, b, a]);
        assert_eq!(normalized, vec![b, c, a]);
    }

    #[test]
    fn plan_move_set_matches_plan_move_for_single_entity() {
        let (_repo, source_ws, target_ws) = setup_repo();
        let domain = FixtureDomain {
            source_store_root: source_ws.join(".fixture"),
            fail_scan_number: Cell::new(None),
            scan_calls: Cell::new(0),
        };
        let entity_id = Uuid::new_v4();
        make_entity(&domain.source_store_root, entity_id);

        let single_plan = plan_move(&domain, &entity_id, &target_ws).expect("plan_move");
        let set_plan = plan_move_set(&domain, &[entity_id], &target_ws).expect("plan_move_set");

        assert_eq!(set_plan.entity_ids, vec![entity_id]);
        assert_eq!(set_plan.entity_plans.len(), 1);
        let entity_plan = &set_plan.entity_plans[0];
        assert_eq!(entity_plan.entity_id, single_plan.entity_id);
        assert_eq!(
            entity_plan.source_entity_path,
            single_plan.source_entity_path
        );
        assert_eq!(
            entity_plan.destination_entity_path,
            single_plan.destination_entity_path
        );
        assert_eq!(
            entity_plan.git_worktree_topology,
            single_plan.git_worktree_topology
        );
        assert!(entity_plan.supported());
        assert!(single_plan.supported());
        assert!(set_plan.supported());
    }

    #[test]
    fn plan_move_set_batches_tracked_path_references_per_entity() {
        let (repo, source_ws, target_ws) = setup_repo();
        let domain = FixtureDomain {
            source_store_root: source_ws.join(".fixture"),
            fail_scan_number: Cell::new(None),
            scan_calls: Cell::new(0),
        };
        let entity_a = Uuid::new_v4();
        let entity_b = Uuid::new_v4();
        make_entity(&domain.source_store_root, entity_a);
        make_entity(&domain.source_store_root, entity_b);
        let entity_a_path = domain
            .source_store_root
            .join("entities")
            .join(entity_a.to_string());
        let entity_b_path = domain
            .source_store_root
            .join("entities")
            .join(entity_b.to_string());
        fs::write(
            repo.path().join("references.txt"),
            format!(
                "{}\n{}\n",
                entity_a_path.to_string_lossy().replace('\\', "/"),
                entity_b_path.to_string_lossy().replace('\\', "/")
            ),
        )
        .expect("write tracked path references");
        let status = Command::new("git")
            .args(["add", "references.txt"])
            .current_dir(repo.path())
            .status()
            .expect("git add must run");
        assert!(status.success(), "git add failed");

        let set_plan =
            plan_move_set(&domain, &[entity_a, entity_b], &target_ws).expect("plan_move_set");
        let reference_file = repo.path().join("references.txt");
        for entity_id in [entity_a, entity_b] {
            let plan = set_plan
                .entity_plans
                .iter()
                .find(|plan| plan.entity_id == entity_id)
                .expect("entity plan");
            assert_eq!(plan.path_reference_files, vec![reference_file.clone()]);
        }
    }

    #[test]
    fn plan_move_set_propagates_missing_source_blocker_per_entity() {
        let (_repo, source_ws, target_ws) = setup_repo();
        let domain = FixtureDomain {
            source_store_root: source_ws.join(".fixture"),
            fail_scan_number: Cell::new(None),
            scan_calls: Cell::new(0),
        };
        let present_id = Uuid::new_v4();
        let missing_id = Uuid::new_v4();
        make_entity(&domain.source_store_root, present_id);

        let set_plan =
            plan_move_set(&domain, &[present_id, missing_id], &target_ws).expect("plan_move_set");

        assert!(!set_plan.supported());
        let present_plan = set_plan
            .entity_plans
            .iter()
            .find(|plan| plan.entity_id == present_id)
            .expect("present entity plan");
        assert!(present_plan.supported());

        let missing_plan = set_plan
            .entity_plans
            .iter()
            .find(|plan| plan.entity_id == missing_id)
            .expect("missing entity plan");
        assert!(
            missing_plan.blockers.iter().any(|blocker| matches!(
                blocker,
                MoveBlocker::MissingSourceEntity { entity_id }
                    if *entity_id == missing_id
            )),
            "expected MissingSourceEntity blocker, got {:?}",
            missing_plan.blockers
        );
    }

    #[test]
    fn execute_move_set_moves_all_entities_and_supports_per_entity_rollback() {
        let (_repo, source_ws, target_ws) = setup_repo();
        let domain = FixtureDomain {
            source_store_root: source_ws.join(".fixture"),
            fail_scan_number: Cell::new(None),
            scan_calls: Cell::new(0),
        };
        let entity_a = Uuid::new_v4();
        let entity_b = Uuid::new_v4();
        make_entity(&domain.source_store_root, entity_a);
        make_entity(&domain.source_store_root, entity_b);

        let set_plan =
            plan_move_set(&domain, &[entity_a, entity_b], &target_ws).expect("plan_move_set");
        assert!(set_plan.supported());

        let outcome = execute_move_set(&domain, &set_plan).expect("execute_move_set");
        assert_eq!(outcome.entity_ids, set_plan.entity_ids);
        assert_eq!(outcome.entity_outcomes.len(), 2);
        assert_eq!(domain.scan_calls.get(), 2);
        let loaded_set_journal =
            load_move_set_journal(&domain.source_store_root, outcome.journal.id)
                .expect("set journal must round-trip");
        assert_eq!(loaded_set_journal.entity_ids, set_plan.entity_ids);
        assert_eq!(loaded_set_journal.phase, MoveSetExecutionPhase::Validated);
        assert!(loaded_set_journal.entity_plans.is_empty());
        let persisted_payload = fs::read_to_string(move_set_journal_path(
            &domain.source_store_root,
            outcome.journal.id,
        ))
        .expect("persisted set journal");
        assert!(!persisted_payload.contains("entity_plans"));
        for entity_outcome in &outcome.entity_outcomes {
            assert_eq!(entity_outcome.journal.phase, MoveExecutionPhase::Validated);
            assert!(!entity_outcome.journal.source_entity_path.exists());
            assert!(entity_outcome.journal.destination_entity_path.exists());
        }

        // Recovery path: an individual entity's journal produced by the set
        // operation remains independently rollback-able via the existing
        // single-entity `rollback_move`.
        let target_store_root = target_ws.join(".fixture");
        let entity_a_outcome = outcome
            .entity_outcomes
            .iter()
            .find(|entity_outcome| entity_outcome.journal.entity_id == entity_a)
            .expect("entity_a outcome");
        let rolled_back = rollback_move(&domain, entity_a_outcome.journal.id)
            .expect("rollback_move on set-produced journal");
        assert_eq!(rolled_back.journal.phase, MoveExecutionPhase::RolledBack);
        assert!(
            domain
                .source_entity_path(&entity_a)
                .expect("source_entity_path")
                .is_some(),
            "entity_a must be restored to the source store after rollback"
        );
        assert!(
            !target_store_root
                .join("entities")
                .join(entity_a.to_string())
                .exists()
        );

        // entity_b remains moved (untouched by entity_a's rollback).
        assert!(
            target_store_root
                .join("entities")
                .join(entity_b.to_string())
                .exists()
        );
    }

    #[test]
    fn resume_move_set_skips_completed_entities() {
        let (_repo, source_ws, target_ws) = setup_repo();
        let domain = FixtureDomain {
            source_store_root: source_ws.join(".fixture"),
            fail_scan_number: Cell::new(Some(2)),
            scan_calls: Cell::new(0),
        };
        let entity_a = Uuid::new_v4();
        let entity_b = Uuid::new_v4();
        make_entity(&domain.source_store_root, entity_a);
        make_entity(&domain.source_store_root, entity_b);
        let set_plan =
            plan_move_set(&domain, &[entity_a, entity_b], &target_ws).expect("plan_move_set");

        execute_move_set(&domain, &set_plan)
            .expect_err("batched target reconciliation must interrupt the set move");
        let journal_path = fs::read_dir(domain.source_store_root.join(MOVE_JOURNALS_DIR))
            .expect("set journal directory")
            .map(|entry| entry.expect("journal entry").path())
            .find(|path| path.to_string_lossy().ends_with(".set.json"))
            .expect("set journal path");
        let journal_id = Uuid::parse_str(
            journal_path
                .file_name()
                .expect("name")
                .to_string_lossy()
                .trim_end_matches(".set.json"),
        )
        .expect("set journal id");
        let interrupted = load_move_set_journal(&domain.source_store_root, journal_id)
            .expect("interrupted set journal");
        assert_eq!(interrupted.completed_entity_ids, set_plan.entity_ids);

        domain.fail_scan_number.set(None);
        let resumed = resume_move_set(&domain, journal_id).expect("resume only unfinished entity");
        assert_eq!(resumed.entity_outcomes.len(), 2);
        assert!(
            resumed
                .entity_outcomes
                .iter()
                .any(|outcome| { outcome.journal.entity_id == entity_a && outcome.resumed })
        );
        let completed = load_move_set_journal(&domain.source_store_root, journal_id)
            .expect("completed set journal");
        assert_eq!(completed.completed_entity_ids, set_plan.entity_ids);
    }

    #[test]
    fn rollback_move_set_is_idempotent() {
        let (_repo, source_ws, target_ws) = setup_repo();
        let domain = FixtureDomain {
            source_store_root: source_ws.join(".fixture"),
            fail_scan_number: Cell::new(None),
            scan_calls: Cell::new(0),
        };
        let entity_a = Uuid::new_v4();
        let entity_b = Uuid::new_v4();
        make_entity(&domain.source_store_root, entity_a);
        make_entity(&domain.source_store_root, entity_b);
        let plan =
            plan_move_set(&domain, &[entity_a, entity_b], &target_ws).expect("plan_move_set");
        let moved = execute_move_set(&domain, &plan).expect("execute_move_set");

        let rolled_back = rollback_move_set(&domain, moved.journal.id).expect("first rollback");
        assert_eq!(rolled_back.entity_outcomes.len(), 2);
        assert!(domain.source_entity_path(&entity_a).unwrap().is_some());
        assert!(domain.source_entity_path(&entity_b).unwrap().is_some());

        let repeated = rollback_move_set(&domain, moved.journal.id).expect("idempotent rollback");
        assert!(repeated.entity_outcomes.is_empty());
        assert_eq!(repeated.journal.phase, MoveSetExecutionPhase::RolledBack);
    }
}
