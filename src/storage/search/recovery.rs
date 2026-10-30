use super::*;

impl TantivySearchIndex {
    pub(super) fn is_retryable_search_error(error: &StorageError) -> bool {
        let StorageError::SearchIndex(message) = error else {
            return false;
        };

        let lower = message.to_ascii_lowercase();
        lower.contains("permissiondenied")
            || lower.contains("os error 5")
            || lower.contains("access is denied")
            || lower.contains("zugriff verweigert")
    }

    pub(super) fn with_retry<T, F>(mut op: F) -> Result<T, StorageError>
    where
        F: FnMut() -> Result<T, StorageError>,
    {
        let mut delay_ms = SEARCH_IO_RETRY_BASE_DELAY_MS;
        for attempt in 0..SEARCH_IO_RETRY_ATTEMPTS {
            match op() {
                Ok(value) => return Ok(value),
                Err(error)
                    if attempt + 1 < SEARCH_IO_RETRY_ATTEMPTS
                        && Self::is_retryable_search_error(&error) =>
                {
                    thread::sleep(Duration::from_millis(delay_ms));
                    delay_ms = (delay_ms * 2).min(500);
                }
                Err(error) => return Err(error),
            }
        }

        unreachable!("retry loop must return or error")
    }

    /// Run a read operation, converting a Tantivy panic into a recoverable
    /// error.
    ///
    /// Tantivy panics (rather than returning an error) on some classes of
    /// on-disk corruption — for example a truncated segment whose slice offset
    /// underflows (`attempt to subtract with overflow`). On the calling thread
    /// such a panic would abort the read, so it is caught here and mapped to a
    /// `SearchIndex` error containing `"panic"`, which
    /// [`Self::should_rebuild_search_index`] and
    /// [`Self::is_rebuildable_read_failure`] both classify as rebuild-worthy.
    /// Catching is safe because the search index is a derived cache that is
    /// rebuilt from the filesystem source of truth on the next operation.
    pub(super) fn catch_index_panic<T, F>(op: F) -> Result<T, StorageError>
    where
        F: FnOnce() -> Result<T, StorageError>,
    {
        // Suppress the default panic hook's stderr backtrace for this scoped
        // read: a caught, recovered-from corruption panic is not a crash.
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(op));
        std::panic::set_hook(previous_hook);

        match result {
            Ok(inner) => inner,
            Err(_) => Err(StorageError::SearchIndex(
                "search index read panicked (corrupt on-disk segment); \
                 rebuild required"
                    .to_string(),
            )),
        }
    }

    pub fn should_rebuild_search_index(error: &StorageError) -> bool {
        let StorageError::SearchIndex(message) = error else {
            return false;
        };

        let lower = message.to_ascii_lowercase();
        lower.contains("index out of bounds")
            || lower.contains("an error occurred in a thread")
            || lower.contains("corrupt")
            || lower.contains("corrupted")
            || lower.contains("panic")
    }

    /// Whether a failure observed while **reading** the index warrants
    /// rebuilding it from the source of truth.
    ///
    /// Segment-content damage (truncated, corrupt, or missing segment files)
    /// keeps `meta.json` valid, so it passes the cheap structural and
    /// completeness checks and only surfaces when the searcher reads a segment.
    /// The error messages vary widely (`File corrupted`, `FileDoesNotExist`,
    /// `UnexpectedEof`, background-thread panics, …), so any search-index error
    /// that is **not** a transient, retryable IO/permission error is treated as
    /// on-disk damage that a rebuild repairs. Rebuilding is always safe because
    /// the filesystem entities are the authoritative source.
    pub fn is_rebuildable_read_failure(error: &StorageError) -> bool {
        matches!(error, StorageError::SearchIndex(_)) && !Self::is_retryable_search_error(error)
    }

    pub fn reset_dir(&self) -> Result<(), StorageError> {
        if self.dir.exists() {
            std::fs::remove_dir_all(&self.dir)?;
        }
        std::fs::create_dir_all(&self.dir)?;
        Ok(())
    }

    /// Validate and heal every [`IndexInvariant`], returning an open [`Index`]
    /// that is guaranteed to satisfy all readiness conditions.
    ///
    /// This is the single proactive gate that every read and write passes
    /// through. It enforces, in order:
    ///
    /// 1. **Directory exists** — created if missing.
    /// 2. **Openable** — a non-empty directory must open as a valid Tantivy
    ///    index; a corrupt/truncated index is rebuilt from the current schema.
    /// 3. **Schema current** — the on-disk schema must match [`build_schema`]
    ///    exactly; a stale layout is rebuilt *before* any document is written,
    ///    so the fast-field writer can never index past its `fast_field_names`
    ///    vector.
    ///
    /// Healing is proactive: schema drift and corruption are repaired here, not
    /// caught downstream after a panic or error. An empty directory is valid
    /// and is created from the current schema.
    pub(super) fn ensure_ready(&self) -> Result<Index, StorageError> {
        // Invariant 1: the directory must exist.
        std::fs::create_dir_all(&self.dir)?;

        let is_empty = self
            .dir
            .read_dir()
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true);
        if is_empty {
            return self.create_fresh_index();
        }

        // Invariant 2: a non-empty directory must open as a valid index.
        let index = match Index::open_in_dir(&self.dir) {
            Ok(index) => index,
            Err(error) => {
                tracing::warn!(
                    dir = %self.dir.display(),
                    %error,
                    "search index unreadable; rebuilding from current schema",
                );
                return self.create_fresh_index();
            }
        };

        // Invariant 3: the on-disk schema must match the current schema.
        let (expected, _) = build_schema();
        if schemas_match(&index.schema(), &expected) {
            return Ok(index);
        }

        drop(index);
        tracing::warn!(
            dir = %self.dir.display(),
            "search index schema is stale; rebuilding from current schema",
        );
        self.create_fresh_index()
    }

    /// Reset the directory and create a brand-new index from the current
    /// schema. Used by [`Self::ensure_ready`] to heal a missing, corrupt, or
    /// stale-schema index.
    pub(super) fn create_fresh_index(&self) -> Result<Index, StorageError> {
        self.reset_dir()?;
        let (schema, _) = build_schema();
        Index::create_in_dir(&self.dir, schema)
            .map_err(|e| StorageError::SearchIndex(e.to_string()))
    }

    /// Validate every [`IndexInvariant`] **without** mutating the index and
    /// return the first violation, if any.
    ///
    /// This is the read-only counterpart to [`Self::ensure_ready`], intended
    /// for health checks and tests. `Ok(None)` means the index is ready (or is
    /// an empty directory that will be created on first use).
    pub fn check_invariants(&self) -> Result<Option<IndexInvariant>, StorageError> {
        if !self.dir.exists() {
            return Ok(Some(IndexInvariant::DirectoryExists));
        }

        let is_empty = self
            .dir
            .read_dir()
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true);
        if is_empty {
            return Ok(None);
        }

        let index = match Index::open_in_dir(&self.dir) {
            Ok(index) => index,
            Err(_) => return Ok(Some(IndexInvariant::Openable)),
        };

        let (expected, _) = build_schema();
        if !schemas_match(&index.schema(), &expected) {
            return Ok(Some(IndexInvariant::SchemaCurrent));
        }

        Ok(None)
    }

    /// Proactively validate and heal all index invariants.
    ///
    /// Thin wrapper over [`Self::ensure_ready`] for callers that want to force
    /// the index into a known-good state without performing an operation (for
    /// example before a bulk reindex). The healed index handle is dropped
    /// immediately so no file handles are retained.
    pub fn ensure_schema_current(&self) -> Result<(), StorageError> {
        self.ensure_ready().map(|_| ())
    }

    /// Proactively validate the **structural** invariants and rebuild the index
    /// when any is violated (missing directory, unopenable/corrupt index, or
    /// stale schema).
    ///
    /// Returns `true` when a rebuild occurred. Because a rebuild produces an
    /// empty index, the caller is then responsible for restoring the
    /// **completeness** invariant — re-indexing every entity from the
    /// filesystem source of truth (the search index cannot do this itself; only
    /// the owning store knows the entities). Returns `false` when the index was
    /// already structurally valid, so no repopulation is needed.
    pub fn heal_if_needed(&self) -> Result<bool, StorageError> {
        std::fs::create_dir_all(&self.dir)?;
        match self.check_invariants()? {
            None => Ok(false),
            Some(_) => {
                self.create_fresh_index()?;
                Ok(true)
            }
        }
    }
}
