use super::*;

/// Domain-specific hooks the kernel needs to plan and execute a move.
///
/// Implementors are thin adapters over a concrete domain store. Every method is
/// expressed in domain-neutral terms ([`Uuid`] identities, [`BoardEntry`] rows,
/// store-root [`Path`]s) so the kernel never sees ticket-specific types.
pub trait MoveDomain {
    /// Subdirectory under a store root that holds entity folders, e.g.
    /// `"tickets"` or `"specs"`.
    fn entity_subdir(&self) -> &str;

    /// Store index directory name used for workspace<->store-root resolution,
    /// e.g. `".ticket"` or `".spec"`.
    fn store_index_dir(&self) -> &str;

    /// The source store's index/store root.
    fn source_store_root(&self) -> PathBuf;

    /// On-disk path of the entity in the source store, or `None` if the source
    /// store does not currently index it.
    fn source_entity_path(&self, entity_id: &Uuid) -> MoveResult<Option<PathBuf>>;

    fn source_entity_paths_for_set(
        &self,
        entity_ids: &[Uuid],
    ) -> MoveResult<std::collections::BTreeMap<Uuid, PathBuf>> {
        let mut result = std::collections::BTreeMap::new();
        for entity_id in entity_ids {
            if let Some(path) = self.source_entity_path(entity_id)? {
                result.insert(*entity_id, path);
            }
        }
        Ok(result)
    }

    /// Inbound and outbound related entity ids (graph edges). Domains without an
    /// edge model return [`MoveReferences::default`].
    fn related_entities(&self, entity_id: &Uuid) -> MoveResult<MoveReferences>;

    /// Whether the destination store exists at `target_store_root`.
    fn target_store_present(&self, target_store_root: &Path) -> MoveResult<bool>;

    /// Whether execution may initialize a missing canonical destination store.
    fn can_initialize_target_store(&self) -> bool {
        false
    }

    /// Initialize a missing canonical destination store before execution.
    fn initialize_target_store(&self, _target_store_root: &Path) -> MoveResult<()> {
        Err(MoveError::Domain(
            "destination store initialization is not supported by this domain".to_string(),
        ))
    }

    /// Whether `entity_id` is indexed by the store rooted at `store_root`.
    fn entity_indexed_in(&self, store_root: &Path, entity_id: &Uuid) -> MoveResult<bool>;

    /// Board rows for the entity. Domains without a board return
    /// [`MoveBoardState::default`] (the default implementation).
    fn board_state(&self, _entity_id: &Uuid) -> MoveResult<MoveBoardState> {
        Ok(MoveBoardState::default())
    }

    /// Active leases for the entity. Domains without leases return an empty vec
    /// (the default implementation).
    fn active_leases(&self, _entity_id: &Uuid) -> MoveResult<Vec<MoveLeaseBlock>> {
        Ok(Vec::new())
    }

    /// Migrate historical board rows from the source store to the target store,
    /// returning the migrated rows. Domains without a board return an empty vec
    /// (the default implementation).
    ///
    /// Implementations must fail if any active/stale row is encountered.
    fn migrate_board_history(
        &self,
        _target_store_root: &Path,
        _entity_id: &Uuid,
    ) -> MoveResult<Vec<BoardEntry>> {
        Ok(Vec::new())
    }

    /// Migrate historical board rows for a complete set in one store
    /// operation, keyed by entity id. The default preserves compatibility by
    /// delegating to the single-entity hook.
    fn migrate_board_history_for_set(
        &self,
        target_store_root: &Path,
        entity_ids: &[Uuid],
    ) -> MoveResult<std::collections::BTreeMap<Uuid, Vec<BoardEntry>>> {
        let mut result = std::collections::BTreeMap::new();
        for entity_id in entity_ids {
            let entries = self.migrate_board_history(target_store_root, entity_id)?;
            if !entries.is_empty() {
                result.insert(*entity_id, entries);
            }
        }
        Ok(result)
    }

    /// Restore previously migrated board rows back to the source store (rollback).
    fn restore_board_history(
        &self,
        _target_store_root: &Path,
        _entries: &[BoardEntry],
    ) -> MoveResult<()> {
        Ok(())
    }

    /// Force a full rescan of the store rooted at `store_root`.
    fn scan_store(&self, store_root: &Path) -> MoveResult<()>;

    /// Reconcile only a known touched subset when the caller already knows the
    /// affected ids (for example move execution for a single entity). Domains
    /// that do not support targeted reconciliation can fall back to `scan_store`.
    fn reconcile_store_touched(
        &self,
        store_root: &Path,
        touched_entity_ids: &[Uuid],
    ) -> MoveResult<()> {
        let _ = touched_entity_ids;
        self.scan_store(store_root)
    }

    /// Reconcile one complete set in a single store operation.
    fn reconcile_store_set(
        &self,
        store_root: &Path,
        touched_entity_ids: &[Uuid],
    ) -> MoveResult<()> {
        self.reconcile_store_touched(store_root, touched_entity_ids)
    }

    /// Related entities for every id in `entity_ids`, keyed by entity id.
    ///
    /// **Compatibility fallback**: the default implementation loops
    /// [`MoveDomain::related_entities`] once per id, which is exactly the
    /// per-entity edge rescan documented as finding F2 in the Waypoint A
    /// benchmark review. Domains that can answer the whole set from one bulk
    /// edge-table query (e.g. `edges_from_ids` / `edges_to_ids`) should
    /// override this hook to remove the repeated scan; [`plan_move_set`]
    /// calls this hook exactly once for the whole move set regardless of
    /// which implementation runs.
    fn related_entities_for_set(
        &self,
        entity_ids: &[Uuid],
    ) -> MoveResult<std::collections::BTreeMap<Uuid, MoveReferences>> {
        let mut result = std::collections::BTreeMap::new();
        for entity_id in entity_ids {
            result.insert(*entity_id, self.related_entities(entity_id)?);
        }
        Ok(result)
    }

    /// Whether each of `entity_ids` is indexed by the store rooted at
    /// `store_root`, keyed by entity id.
    ///
    /// **Compatibility fallback**: the default implementation loops
    /// [`MoveDomain::entity_indexed_in`] once per id, reopening whatever
    /// store/connection that hook opens each time — the repeated
    /// destination-store open documented as finding F3. Domains that can open
    /// one destination read context and answer membership for every id in one
    /// query should override this hook.
    fn entity_indexed_in_many(
        &self,
        store_root: &Path,
        entity_ids: &[Uuid],
    ) -> MoveResult<std::collections::BTreeMap<Uuid, bool>> {
        let mut result = std::collections::BTreeMap::new();
        for entity_id in entity_ids {
            result.insert(*entity_id, self.entity_indexed_in(store_root, entity_id)?);
        }
        Ok(result)
    }

    /// Board rows for every id in `entity_ids`, keyed by entity id.
    ///
    /// **Compatibility fallback**: the default implementation loops
    /// [`MoveDomain::board_state`] once per id, the repeated full
    /// active/history board scan documented as finding F4. Domains with a
    /// board should override this with one targeted/bulk read for the whole
    /// set (e.g. one query filtered to `entity_ids`).
    fn board_state_for_set(
        &self,
        entity_ids: &[Uuid],
    ) -> MoveResult<std::collections::BTreeMap<Uuid, MoveBoardState>> {
        let mut result = std::collections::BTreeMap::new();
        for entity_id in entity_ids {
            result.insert(*entity_id, self.board_state(entity_id)?);
        }
        Ok(result)
    }

    /// Active leases for every id in `entity_ids`, keyed by entity id.
    ///
    /// **Compatibility fallback**: the default implementation loops
    /// [`MoveDomain::active_leases`] once per id, the repeated full lease-table
    /// scan documented as finding F4. Domains with a lease table should
    /// override this with one targeted/bulk read for the whole set.
    fn active_leases_for_set(
        &self,
        entity_ids: &[Uuid],
    ) -> MoveResult<std::collections::BTreeMap<Uuid, Vec<MoveLeaseBlock>>> {
        let mut result = std::collections::BTreeMap::new();
        for entity_id in entity_ids {
            result.insert(*entity_id, self.active_leases(entity_id)?);
        }
        Ok(result)
    }

    /// Discover tracked path-reference files for every existing entity in a
    /// set. The default implementation batches candidates into one Git query
    /// per distinct repository; domains may override the hook when they have a
    /// more direct reference index.
    fn path_reference_files_for_set(
        &self,
        entity_paths: &std::collections::BTreeMap<Uuid, PathBuf>,
        source_git_root: &Path,
        target_git_root: &Path,
        source_store_root: &Path,
        target_store_root: &Path,
        subdir: &str,
    ) -> MoveResult<MovePathReferenceSet> {
        Ok(super::internal::collect_plan_path_reference_files_for_set(
            entity_paths,
            source_git_root,
            target_git_root,
            source_store_root,
            target_store_root,
            subdir,
        ))
    }
}
