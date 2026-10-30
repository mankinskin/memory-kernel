use super::*;

// ── connection helpers ────────────────────────────────────────────────────────

pub(super) fn read_connection(db_path: &Path) -> Result<Connection, StorageError> {
    // `unlock_notify` feature: if a write lock is briefly held, the read will
    // wait for notification rather than returning SQLITE_BUSY immediately.
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| StorageError::Database(e.to_string()))?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| StorageError::Database(e.to_string()))?;
    Ok(conn)
}

pub(super) fn write_connection(db_path: &Path) -> Result<Connection, StorageError> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| StorageError::Database(e.to_string()))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;",
    )
    .map_err(|e| StorageError::Database(e.to_string()))?;
    Ok(conn)
}

pub(super) fn ensure_tables(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(&format!(
        "BEGIN;
         CREATE TABLE IF NOT EXISTS {TABLE_TICKETS} (
             id   TEXT PRIMARY KEY NOT NULL,
             data BLOB NOT NULL
         );
         CREATE TABLE IF NOT EXISTS {TABLE_WORKFLOW_FACTS} (
             id   TEXT PRIMARY KEY NOT NULL,
             data BLOB NOT NULL
         );
         CREATE TABLE IF NOT EXISTS {TABLE_EDGES} (
             from_id    TEXT NOT NULL,
             to_id      TEXT NOT NULL,
             kind       TEXT NOT NULL,
             created_at TEXT NOT NULL,
             PRIMARY KEY (from_id, to_id, kind)
         );
         CREATE TABLE IF NOT EXISTS {TABLE_SCAN_ROOTS} (
             path            TEXT PRIMARY KEY NOT NULL,
             label           TEXT NOT NULL,
             source          TEXT NOT NULL DEFAULT 'discovered',
             policy_decision TEXT NOT NULL DEFAULT 'included',
             workspace_root  TEXT
         );
         CREATE TABLE IF NOT EXISTS {TABLE_LEASES} (
             id   TEXT PRIMARY KEY NOT NULL,
             data BLOB NOT NULL
         );
         CREATE TABLE IF NOT EXISTS {TABLE_META} (
             key   TEXT PRIMARY KEY NOT NULL,
             value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS {TABLE_BOARD_ENTRIES} (
             id   TEXT PRIMARY KEY NOT NULL,
             data BLOB NOT NULL
         );
         CREATE TABLE IF NOT EXISTS {TABLE_BOARD_ACTIVE_INDEX} (
             key   TEXT PRIMARY KEY NOT NULL,
             value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS {TABLE_BOARD_CONFIG} (
             key  TEXT PRIMARY KEY NOT NULL,
             data BLOB NOT NULL
         );
         COMMIT;"
    ))
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Non-destructive migration: add scan-root auditability columns to indexes
/// created before the metadata schema existed. Existing rows receive safe
/// defaults (`source = 'discovered'`, `policy_decision = 'included'`,
/// `workspace_root = NULL`).
pub(super) fn migrate_scan_roots_metadata(conn: &Connection) -> Result<(), StorageError> {
    let existing: HashSet<String> = {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({TABLE_SCAN_ROOTS})"))
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let names = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| StorageError::Database(e.to_string()))?
            .collect::<Result<HashSet<String>, _>>()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        names
    };

    let additions: [(&str, &str); 3] = [
        (
            "source",
            "ALTER TABLE {t} ADD COLUMN source TEXT NOT NULL DEFAULT 'discovered'",
        ),
        (
            "policy_decision",
            "ALTER TABLE {t} ADD COLUMN policy_decision TEXT NOT NULL DEFAULT 'included'",
        ),
        (
            "workspace_root",
            "ALTER TABLE {t} ADD COLUMN workspace_root TEXT",
        ),
    ];

    for (column, template) in additions {
        if !existing.contains(column) {
            let sql = template.replace("{t}", TABLE_SCAN_ROOTS);
            conn.execute(&sql, [])
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }
    }

    Ok(())
}

pub(super) fn check_or_set_schema_version(conn: &Connection) -> Result<(), StorageError> {
    use crate::storage::schema::ensure_supported_schema_version;
    let existing: Option<String> = conn
        .query_row(
            &format!("SELECT value FROM {TABLE_META} WHERE key = 'schema_version'"),
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| StorageError::Database(e.to_string()))?;
    match existing {
        Some(found) => ensure_supported_schema_version(&found)?,
        None => {
            conn.execute(
                &format!("INSERT INTO {TABLE_META} (key, value) VALUES ('schema_version', ?1)"),
                params![SCHEMA_VERSION],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;
        }
    }
    Ok(())
}

pub(super) fn parse_edge_row(
    from_s: &str,
    to_s: &str,
    kind: &str,
    created_at_s: &str,
) -> Result<EdgeRecord, StorageError> {
    let from: Uuid = from_s
        .parse()
        .map_err(|e: uuid::Error| StorageError::Serialization(e.to_string()))?;
    let to: Uuid = to_s
        .parse()
        .map_err(|e: uuid::Error| StorageError::Serialization(e.to_string()))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(created_at_s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());
    Ok(EdgeRecord {
        from,
        to,
        kind: kind.to_string(),
        created_at,
    })
}
