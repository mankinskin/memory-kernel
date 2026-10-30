use super::*;

fn test_fs() -> EntityFs {
    EntityFs::new("entity.toml", "entity.lock")
}

#[test]
fn test_entity_store_open() {
    let tmp = tempfile::tempdir().unwrap();
    let store = EntityStore::open(tmp.path(), test_fs()).unwrap();
    assert!(store.index_root.exists());
}

#[test]
fn test_empty_list_and_search() {
    let tmp = tempfile::tempdir().unwrap();
    let store = EntityStore::open(tmp.path(), test_fs()).unwrap();

    let indexed = store.list_indexed().unwrap();
    assert!(indexed.is_empty());

    let results = store.search("nonexistent", 10).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_scan_empty_roots() {
    let tmp = tempfile::tempdir().unwrap();
    let store = EntityStore::open(tmp.path(), test_fs()).unwrap();
    let report = store.scan(false).unwrap();
    assert_eq!(report.integrated, 0);
    assert_eq!(report.pruned, 0);
}

#[test]
fn scan_reindex_self_heals_stale_search_index_schema() {
    use crate::model::entity::EntityManifest;
    use serde_json::json;
    use tantivy::schema::{FAST, STORED, STRING, Schema, TEXT};

    let tmp = tempfile::tempdir().unwrap();
    let store = EntityStore::open(tmp.path(), test_fs()).unwrap();
    let entity_dir = tmp.path().join("entities");
    std::fs::create_dir_all(&entity_dir).unwrap();

    let id = Uuid::new_v4();
    let mut manifest = EntityManifest::new(id, Utc::now());
    manifest
        .extra
        .insert("type".to_string(), json!("rule-entry"));
    manifest
        .extra
        .insert("title".to_string(), json!("Stale schema heals"));
    manifest.extra.insert("state".to_string(), json!("ready"));
    store
        .fs
        .create(&manifest, &entity_dir, Some("searchable body text"))
        .unwrap();

    // Replace the freshly-built (current-schema) index with one built from
    // an OLDER 5-field schema (before `created_at` and `effort` were added).
    // Writing a document that references the newer field ids would make the
    // Tantivy fast-field writer panic on a background thread; the scan must
    // detect this, reset the search dir, and rebuild from the current schema.
    let search_dir = tmp.path().join("search_index");
    std::fs::remove_dir_all(&search_dir).unwrap();
    std::fs::create_dir_all(&search_dir).unwrap();
    let mut builder = Schema::builder();
    builder.add_text_field("id", STRING | STORED);
    builder.add_text_field("title", TEXT | STORED);
    builder.add_text_field("body", TEXT | STORED);
    builder.add_text_field("state", STRING | STORED | FAST);
    builder.add_text_field("ticket_type", STRING | STORED | FAST);
    tantivy::Index::create_in_dir(&search_dir, builder.build()).unwrap();

    // Must not panic; self-heals and indexes the entity.
    store.scan(true).unwrap();

    let results = store.search("searchable", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id);
}

/// A read must proactively heal and repopulate the search index across the
/// full corruption matrix — without any manual `scan` — because the search
/// index is a derived cache rebuildable from the filesystem source of truth.
#[test]
fn search_heals_every_corruption_mode_without_manual_scan() {
    use crate::model::entity::EntityManifest;
    use serde_json::json;

    let tmp = tempfile::tempdir().unwrap();
    let store = EntityStore::open(tmp.path(), test_fs()).unwrap();
    let entity_dir = tmp.path().join("entities");
    std::fs::create_dir_all(&entity_dir).unwrap();

    // Seed three searchable entities.
    let mut ids = Vec::new();
    for (n, body) in [
        ("needle alpha document", "alpha"),
        ("needle beta document", "beta"),
        ("needle gamma document", "gamma"),
    ] {
        let id = Uuid::new_v4();
        let mut manifest = EntityManifest::new(id, Utc::now());
        manifest.extra.insert("type".into(), json!("rule-entry"));
        manifest.extra.insert("title".into(), json!(n));
        manifest.extra.insert("state".into(), json!("ready"));
        store.fs.create(&manifest, &entity_dir, Some(body)).unwrap();
        ids.push(id);
    }
    store.scan(true).unwrap();
    assert_eq!(store.search("needle", 10).unwrap().len(), 3);

    let search_dir = tmp.path().join("search_index");
    let list_files = |ext: &str| -> Vec<std::path::PathBuf> {
        std::fs::read_dir(&search_dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some(ext))
            .collect()
    };

    // 1. Corrupt meta.json (unopenable index).
    std::fs::write(search_dir.join("meta.json"), b"not valid json").unwrap();
    assert_eq!(store.search("needle", 10).unwrap().len(), 3);

    // 2. Wipe the whole index directory.
    std::fs::remove_dir_all(&search_dir).unwrap();
    assert_eq!(store.search("needle", 10).unwrap().len(), 3);

    // 3. Truncate a segment store file (segment-content corruption that
    //    keeps meta.json valid — only surfaces on read).
    for f in list_files("store") {
        std::fs::write(&f, b"x").unwrap();
    }
    assert_eq!(store.search("needle", 10).unwrap().len(), 3);

    // 4. Delete the term dictionary files.
    for f in list_files("term") {
        std::fs::remove_file(&f).unwrap();
    }
    assert_eq!(store.search("needle", 10).unwrap().len(), 3);

    // 5. Overwrite a fast-field file with garbage.
    for f in list_files("fast") {
        std::fs::write(&f, b"zzzzz").unwrap();
    }
    assert_eq!(store.search("needle", 10).unwrap().len(), 3);

    // The index is healthy and every seeded entity is searchable again.
    let healed: std::collections::HashSet<_> = store
        .search("needle", 10)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect();
    for id in ids {
        assert!(healed.contains(&id));
    }
}

/// A forced reindex must succeed even when the existing index is corrupt,
/// because a rebuild resets the directory rather than opening the index.
#[test]
fn scan_reindex_succeeds_over_corrupt_index() {
    use crate::model::entity::EntityManifest;
    use serde_json::json;

    let tmp = tempfile::tempdir().unwrap();
    let store = EntityStore::open(tmp.path(), test_fs()).unwrap();
    let entity_dir = tmp.path().join("entities");
    std::fs::create_dir_all(&entity_dir).unwrap();

    let id = Uuid::new_v4();
    let mut manifest = EntityManifest::new(id, Utc::now());
    manifest.extra.insert("type".into(), json!("rule-entry"));
    manifest
        .extra
        .insert("title".into(), json!("Corrupt reindex"));
    store
        .fs
        .create(&manifest, &entity_dir, Some("searchable body"))
        .unwrap();
    store.scan(true).unwrap();

    // Corrupt every segment file, then force a reindex.
    let search_dir = tmp.path().join("search_index");
    for entry in std::fs::read_dir(&search_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            std::fs::write(&path, b"corrupt").unwrap();
        }
    }
    store.scan(true).unwrap();

    let results = store.search("searchable", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id);
}

#[test]
fn test_query_ordering_comparisons() {
    use crate::model::entity::EntityManifest;
    use chrono::TimeZone;
    use serde_json::json;

    let tmp = tempfile::tempdir().unwrap();
    let store = EntityStore::open(tmp.path(), test_fs()).unwrap();
    let entity_dir = tmp.path().join("entities");
    std::fs::create_dir_all(&entity_dir).unwrap();

    let id1 = Uuid::new_v4();
    let date1 = Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap();
    let mut manifest1 = EntityManifest::new(id1, date1);
    manifest1
        .extra
        .insert("type".to_string(), json!("tracker-improvement"));
    manifest1
        .extra
        .insert("title".to_string(), json!("Low effort, early date"));
    manifest1.extra.insert("state".to_string(), json!("ready"));
    manifest1.extra.insert("effort".to_string(), json!("3"));
    store
        .fs
        .create(&manifest1, &entity_dir, Some("Description 1"))
        .unwrap();

    let id2 = Uuid::new_v4();
    let date2 = Utc.with_ymd_and_hms(2026, 6, 5, 12, 0, 0).unwrap();
    let mut manifest2 = EntityManifest::new(id2, date2);
    manifest2
        .extra
        .insert("type".to_string(), json!("tracker-improvement"));
    manifest2
        .extra
        .insert("title".to_string(), json!("Medium effort, mid date"));
    manifest2
        .extra
        .insert("state".to_string(), json!("in-implementation"));
    manifest2.extra.insert("effort".to_string(), json!("5"));
    store
        .fs
        .create(&manifest2, &entity_dir, Some("Description 2"))
        .unwrap();

    let id3 = Uuid::new_v4();
    let date3 = Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap();
    let mut manifest3 = EntityManifest::new(id3, date3);
    manifest3
        .extra
        .insert("type".to_string(), json!("tracker-improvement"));
    manifest3
        .extra
        .insert("title".to_string(), json!("High effort, late date"));
    manifest3.extra.insert("state".to_string(), json!("done"));
    manifest3.extra.insert("effort".to_string(), json!("8"));
    store
        .fs
        .create(&manifest3, &entity_dir, Some("Description 3"))
        .unwrap();

    // Scan and index them all!
    store.scan(true).unwrap();

    // Query 1: effort gt 3 (should return id2 and id3)
    let results = store.search("effort:>3", 10).unwrap();
    assert_eq!(results.len(), 2);
    let ids: Vec<Uuid> = results.iter().map(|r| r.id).collect();
    assert!(ids.contains(&id2));
    assert!(ids.contains(&id3));

    // Query 2: effort lte 5 (should return id1 and id2)
    let results = store.search("effort:<=5", 10).unwrap();
    assert_eq!(results.len(), 2);
    let ids: Vec<Uuid> = results.iter().map(|r| r.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));

    // Query 3: effort range [4 TO 8] (should return id2 and id3)
    let results = store.search("effort:[4 TO 8]", 10).unwrap();
    assert_eq!(results.len(), 2);
    let ids: Vec<Uuid> = results.iter().map(|r| r.id).collect();
    assert!(ids.contains(&id2));
    assert!(ids.contains(&id3));

    // Query 4: created_at gt mid date (should return id3)
    let results = store
        .search("created_at:>2026-06-05T12:00:00Z", 10)
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id3);

    // Query 5: created_at range (should return id1 and id2)
    let results = store
        .search(
            "created_at:[2026-06-01T00:00:00Z TO 2026-06-06T00:00:00Z]",
            10,
        )
        .unwrap();
    assert_eq!(results.len(), 2);
    let ids: Vec<Uuid> = results.iter().map(|r| r.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn scan_skips_unchanged_entities_across_consecutive_scans() {
    use crate::model::entity::EntityManifest;
    use serde_json::json;

    let tmp = tempfile::tempdir().unwrap();
    let store = EntityStore::open(tmp.path(), test_fs()).unwrap();
    let entity_dir = tmp.path().join("entities");
    std::fs::create_dir_all(&entity_dir).unwrap();

    let mut ids = Vec::new();
    for n in 0..3 {
        let id = Uuid::new_v4();
        let mut manifest = EntityManifest::new(id, Utc::now());
        manifest.extra.insert("type".into(), json!("rule-entry"));
        manifest
            .extra
            .insert("title".into(), json!(format!("Entity {n}")));
        manifest.extra.insert("state".into(), json!("ready"));
        store
            .fs
            .create(&manifest, &entity_dir, Some("body"))
            .unwrap();
        ids.push(id);
    }

    // Forced reindex integrates every entity and records fingerprints.
    let first = store.scan(true).unwrap();
    assert_eq!(first.integrated, 3);

    // A subsequent incremental scan with no filesystem changes must not
    // re-integrate any entity (AC5: no per-entity re-integration).
    let second = store.scan(false).unwrap();
    assert_eq!(
        second.integrated, 0,
        "unchanged entities must be skipped on repeat scan"
    );

    // Touching one manifest re-integrates exactly that entity.
    let changed_manifest = entity_dir.join(ids[0].to_string()).join("entity.toml");
    // Guarantee a distinct mtime even on coarse-resolution filesystems.
    std::thread::sleep(std::time::Duration::from_millis(20));
    let body = std::fs::read_to_string(&changed_manifest).unwrap();
    std::fs::write(&changed_manifest, format!("{body}\n")).unwrap();

    let third = store.scan(false).unwrap();
    assert_eq!(third.integrated, 1, "only the changed entity re-integrates");
}
