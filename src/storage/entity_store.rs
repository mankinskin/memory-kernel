use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    error::StorageError,
    model::{
        edge::EdgeRecord,
        filesystem::{ParseDiagnostic, ScanRoot},
        query::parse_query,
        schema_registry::SchemaRegistry,
    },
    storage::{
        entity_fs::{EntityFs, EntityScanEntry},
        index::RedbIndexStore,
        indexed::IndexedEntity,
        local_root::ensure_sqlite_index_root,
        search::{SearchResult, TantivySearchIndex},
    },
};

/// Result of a full scan across all registered roots.
pub struct ScanReport {
    pub integrated: usize,
    pub pruned: usize,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// Convenience facade composing all three storage layers:
/// [`RedbIndexStore`] (metadata index), [`EntityFs`] (filesystem),
/// and [`TantivySearchIndex`] (full-text search).
///
/// Downstream crates can use this as a single entry point instead
/// of managing the three stores individually.
pub struct EntityStore {
    pub index: RedbIndexStore,
    pub fs: EntityFs,
    pub search: TantivySearchIndex,
    pub schema_registry: SchemaRegistry,
    pub index_root: PathBuf,
}

impl EntityStore {
    /// Open (or create) an entity store rooted at `index_root`.
    ///
    /// `index_root` is the directory for SQLite + Tantivy index files.
    /// `fs` provides the filesystem layout configuration for entity folders.
    pub fn open(index_root: &Path, fs: EntityFs) -> Result<Self, StorageError> {
        Self::open_with(index_root, fs, SchemaRegistry::new())
    }

    /// Open with a custom schema registry.
    pub fn open_with(
        index_root: &Path,
        fs: EntityFs,
        schema_registry: SchemaRegistry,
    ) -> Result<Self, StorageError> {
        ensure_sqlite_index_root(index_root, "entities.db", &["search_index/"])?;
        let db_path = index_root.join("entities.db");
        let search_dir = index_root.join("search_index");

        let index = RedbIndexStore::open(&db_path)?;
        let search = TantivySearchIndex::open_or_create(&search_dir)?;

        Ok(Self {
            index,
            fs,
            search,
            schema_registry,
            index_root: index_root.to_path_buf(),
        })
    }

    pub fn schema_registry(&self) -> &SchemaRegistry {
        &self.schema_registry
    }

    // ── Scan-root management ────────────────────────────────────────

    pub fn add_scan_root(&self, root: ScanRoot) -> Result<(), StorageError> {
        self.index.add_scan_root(&root)
    }

    pub fn list_scan_roots(&self) -> Result<Vec<ScanRoot>, StorageError> {
        self.index.list_scan_roots()
    }

    // ── Index queries ───────────────────────────────────────────────

    pub fn get_indexed(&self, id: &Uuid) -> Result<Option<IndexedEntity>, StorageError> {
        self.index.get_ticket(id)
    }

    pub fn list_indexed(&self) -> Result<Vec<IndexedEntity>, StorageError> {
        self.index.list_tickets()
    }

    // ── Full-text search ────────────────────────────────────────────

    pub fn search(
        &self,
        query_expr: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, StorageError> {
        // Proactively guarantee the search index is valid and complete before a
        // read, healing structural corruption and repopulating from disk if the
        // index is empty or partial.
        self.ensure_search_ready()?;
        let expr = parse_query(query_expr)?;

        match self.search.search(&expr, limit) {
            Ok(results) => Ok(results),
            // Deep segment-content corruption keeps `meta.json` valid, so it
            // passes the cheap structural and completeness checks above and only
            // surfaces here when the searcher reads a segment. This is the one
            // class of damage that cannot be detected without reading every
            // segment, so it is the sole case repaired reactively: rebuild from
            // the filesystem source of truth and retry the read once.
            Err(error) if TantivySearchIndex::is_rebuildable_read_failure(&error) => {
                self.search.reset_dir()?;
                self.scan_once(true)?;
                self.search.search(&expr, limit)
            }
            Err(error) => Err(error),
        }
    }

    /// Enforce the search-index readiness invariants before a read.
    ///
    /// Structural invariants (directory present, index openable, schema
    /// current) are healed inside [`TantivySearchIndex::num_docs`] /
    /// [`TantivySearchIndex::open_index`]. The **completeness** invariant —
    /// every entity in the metadata index has a search document — is then
    /// checked by comparing document counts; a mismatch (for example an empty
    /// index left by a structural rebuild) is repaired by re-indexing from the
    /// filesystem source of truth.
    fn ensure_search_ready(&self) -> Result<(), StorageError> {
        if self.search_needs_rebuild()? {
            self.scan_once(true)?;
        }
        Ok(())
    }

    /// Whether the search index must be rebuilt before it can be trusted.
    ///
    /// Returns `true` when the search index cannot be opened/counted (structural
    /// or segment-content corruption — [`TantivySearchIndex::num_docs`] errors)
    /// or when its document count differs from the metadata index (the
    /// filesystem-backed source of truth, which survives Tantivy corruption).
    /// Calling this also heals the cheap structural invariants, so a stale or
    /// empty index reports as needing a rebuild.
    fn search_needs_rebuild(&self) -> Result<bool, StorageError> {
        let indexed = self.index.list_tickets()?.len() as u64;
        match self.search.num_docs() {
            Ok(docs) => Ok(docs != indexed),
            Err(_) => Ok(true),
        }
    }

    // ── Edge management ─────────────────────────────────────────────

    pub fn add_edge(&self, edge: EdgeRecord) -> Result<(), StorageError> {
        // Enforce acyclicity when the schema says so.
        let is_acyclic = self
            .schema_registry
            .type_ids()
            .filter_map(|tid| self.schema_registry.get(tid))
            .filter_map(|s| s.edge_rules.get(&edge.kind))
            .any(|r| r.acyclic_enforced);

        if is_acyclic && self.index.is_reachable(&edge.to, &edge.from)? {
            return Err(StorageError::DependencyCycle);
        }

        self.index.insert_edge(&edge)
    }

    pub fn remove_edge(&self, edge: EdgeRecord) -> Result<(), StorageError> {
        self.index.delete_edge(&edge)
    }

    pub fn edges_from(&self, id: &Uuid) -> Result<Vec<EdgeRecord>, StorageError> {
        self.index.edges_from(id)
    }

    pub fn list_all_edges(&self) -> Result<Vec<EdgeRecord>, StorageError> {
        self.index.list_all_edges()
    }

    // ── Scan / reconcile ────────────────────────────────────────────

    /// Scan all registered roots (plus the default entities dir under
    /// `index_root`) and reconcile the index + search stores.
    ///
    /// When `reindex` is `true` (or the search index is missing, partial, or
    /// corrupt), the search directory is **reset** and rebuilt from scratch
    /// before scanning, and stale SQLite entries are pruned.
    ///
    /// Resetting the directory (rather than clearing documents from the existing
    /// index) makes a forced rebuild robust against any on-disk corruption —
    /// including a stale Tantivy schema whose fast-field layout would otherwise
    /// panic the writer, or truncated/missing segment files that cannot be
    /// opened. The index is then repopulated from the filesystem source of
    /// truth, so the completeness invariant is restored.
    pub fn scan(&self, reindex: bool) -> Result<ScanReport, StorageError> {
        // Proactively enforce all search-index invariants before writing. The
        // rebuild check heals structural corruption (via `num_docs`) and detects
        // an empty/partial/unreadable index; either condition forces a full
        // rebuild from the filesystem.
        let force = reindex || self.search_needs_rebuild()?;
        self.scan_once(force)
    }

    fn scan_once(&self, reindex: bool) -> Result<ScanReport, StorageError> {
        if reindex {
            // Reset the directory instead of clearing documents: a forced
            // rebuild must not depend on opening the (possibly corrupt) existing
            // index. The next upsert recreates a fresh index from the current
            // schema.
            self.search.reset_dir()?;
        }

        let roots = self.index.list_scan_roots()?;
        let default_root = ScanRoot {
            path: self.index_root.join("entities"),
            label: "default".into(),
        };
        let all_roots: Vec<&ScanRoot> =
            std::iter::once(&default_root).chain(roots.iter()).collect();

        // On an incremental (non-reindex) scan, load the per-entity manifest
        // fingerprints recorded by the previous scan so unchanged entities can
        // skip the manifest read + metadata re-integration entirely. A forced
        // reindex ignores the previous fingerprints and re-integrates every
        // entity from scratch.
        let previous_fingerprints = if reindex {
            std::collections::HashMap::new()
        } else {
            self.load_scan_fingerprints()
        };
        let mut current_fingerprints = std::collections::HashMap::new();

        let mut integrated = 0usize;
        let mut diagnostics = Vec::new();
        let mut disk_ids = std::collections::HashSet::new();

        for root in all_roots {
            if !root.path.exists() {
                continue;
            }
            let (candidates, diags) = self.fs.scan_root_candidates(&root.path)?;
            diagnostics.extend(diags);

            for candidate in candidates {
                disk_ids.insert(candidate.id);

                let unchanged = !reindex
                    && candidate.manifest_mtime.is_some()
                    && previous_fingerprints.get(&candidate.id)
                        == candidate.manifest_mtime.as_ref()
                    && self.index.get_ticket(&candidate.id)?.is_some();

                if unchanged {
                    if let Some(mtime) = candidate.manifest_mtime {
                        current_fingerprints.insert(candidate.id, mtime);
                    }
                    continue;
                }

                match self
                    .fs
                    .load_scan_entry(candidate.path.clone(), candidate.id)
                {
                    Ok(Some(entry)) => {
                        self.integrate_entry(entry, reindex)?;
                        integrated += 1;
                        if let Some(mtime) = candidate.manifest_mtime {
                            current_fingerprints.insert(candidate.id, mtime);
                        }
                    }
                    Ok(None) => {}
                    Err(diag) => diagnostics.push(diag),
                }
            }
        }

        let mut pruned = 0usize;
        if reindex {
            let indexed = self.index.list_tickets()?;
            for entity in indexed {
                if !disk_ids.contains(&entity.id) {
                    self.index.remove_ticket(&entity.id)?;
                    pruned += 1;
                }
            }
        }

        self.save_scan_fingerprints(&current_fingerprints);

        Ok(ScanReport {
            integrated,
            pruned,
            diagnostics,
        })
    }

    fn scan_fingerprint_path(&self) -> PathBuf {
        self.index_root.join("scan_fingerprints.json")
    }

    /// Load the per-entity manifest fingerprints recorded by the previous scan.
    /// A missing or unreadable/corrupt sidecar yields an empty map, which is
    /// safe: every entity is then treated as changed and re-integrated.
    fn load_scan_fingerprints(&self) -> std::collections::HashMap<Uuid, u128> {
        let Ok(content) = std::fs::read_to_string(self.scan_fingerprint_path()) else {
            return std::collections::HashMap::new();
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    /// Persist the per-entity manifest fingerprints observed in this scan.
    /// Best-effort: a write failure only forfeits the next scan's skip
    /// optimization, so it must never fail the scan itself.
    fn save_scan_fingerprints(&self, fingerprints: &std::collections::HashMap<Uuid, u128>) {
        let Ok(serialized) = serde_json::to_string(fingerprints) else {
            return;
        };
        let path = self.scan_fingerprint_path();
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, serialized).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }

    /// Re-integrate a single entity folder into the metadata index **and** the
    /// full-text search index.
    ///
    /// This is the per-entry counterpart to [`Self::scan`]: instead of walking
    /// every scan root, it reconciles just the entity located at `path`. It is
    /// used by the filesystem watcher so any change to a filesystem entry
    /// immediately refreshes that entry's search-index document.
    ///
    /// Returns `Ok(true)` when the entity was integrated, or `Ok(false)` when
    /// `path` is not a valid entity folder (non-UUID name or unreadable
    /// manifest).
    pub fn integrate_orphan(&self, path: &Path) -> Result<bool, StorageError> {
        let id: Uuid = match path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.parse().ok())
        {
            Some(id) => id,
            None => return Ok(false),
        };

        let manifest = match self.fs.read(path) {
            Ok(manifest) => manifest,
            Err(_) => return Ok(false),
        };

        // Proactively heal the search index before a single-entity write so a
        // corrupt/stale/empty index does not lose the other entities'
        // documents. If the index needed a rebuild, the full repopulation
        // already integrated this entity.
        if self.search_needs_rebuild()? {
            self.scan_once(true)?;
            return Ok(true);
        }

        let entry = EntityScanEntry {
            id,
            path: path.to_path_buf(),
            manifest,
        };
        self.integrate_entry(entry, true)?;
        Ok(true)
    }

    /// Remove a single entity from the metadata index and the search index.
    ///
    /// Used by the watcher when an entity folder is deleted from disk.
    pub fn remove_entity(&self, id: &Uuid) -> Result<(), StorageError> {
        self.index.remove_ticket(id)?;
        self.search.remove(id)?;
        Ok(())
    }

    fn integrate_entry(
        &self,
        entry: EntityScanEntry,
        update_search: bool,
    ) -> Result<(), StorageError> {
        let type_id = entry
            .manifest
            .extra
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let title = entry
            .manifest
            .extra
            .get("title")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let state = entry
            .manifest
            .extra
            .get("state")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let now = Utc::now();

        let indexed = match self.index.get_ticket(&entry.id)? {
            Some(mut existing) => {
                existing.path = entry.path.clone();
                existing.type_id = type_id.clone();
                existing.updated_at = now;
                existing.title = title.clone();
                existing.state = state.clone();
                existing
            }
            None => IndexedEntity {
                id: entry.id,
                path: entry.path.clone(),
                type_id: type_id.clone(),
                title: title.clone(),
                state: state.clone(),
                created_at: entry.manifest.created_at,
                updated_at: now,
            },
        };
        self.index.insert_ticket(&indexed)?;

        if update_search {
            let body = self.fs.read_description(&entry.path);
            let created_at_str = indexed.created_at.to_rfc3339();
            let effort_str = entry.manifest.extra.get("effort").and_then(|v| match v {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                _ => None,
            });
            self.search.upsert(
                &entry.id,
                title.as_deref(),
                body.as_deref(),
                state.as_deref(),
                Some(&type_id),
                Some(&created_at_str),
                effort_str.as_deref(),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
