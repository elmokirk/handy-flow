//! HIST-231 acceptance: rebuildable FTS over canonical sources, bounded
//! queries, version navigation.

use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::search as srepo;

fn seeded(tag: &str) -> handy_app_lib::storage::database::AppDatabase {
    let dir = std::env::temp_dir().join(format!("handy-search-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();

    let c1 = handy_app_lib::storage::ids::new_id();
    let a1 = handy_app_lib::storage::ids::new_id();
    let r1 = handy_app_lib::storage::ids::new_id();
    db.conn()
        .execute_batch(&format!(
            "INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms)
             VALUES ('{c1}', 'Rust Notizen', 'audio_valid', 1, 1);
            INSERT INTO transcription_attempts(id, capture_id, attempt_number, normalized_stt,
                provenance, status, is_canonical, created_at_ms)
             VALUES ('{a1}', '{c1}', 1, 'rust ownership lernen mit beispielen',
                     'live', 'success', 1, 1);
            INSERT INTO representations(id, attempt_id, kind, text, processor, status, created_at_ms)
             VALUES ('{r1}', '{a1}', 'style', 'Lerne Rust Ownership anhand von Beispielen.',
                     'llm_style', 'success', 2);"
        ))
        .unwrap();
    db
}

#[test]
fn fixture_01_rebuild_populates_from_canonical_sources() {
    let db = seeded("rebuild");
    assert_eq!(srepo::index_size(&db).unwrap(), 0);
    let n = srepo::rebuild_search_index(&db).unwrap();
    assert_eq!(n, 2, "one raw attempt + one representation");
    // Idempotent: rebuild truncates first.
    let n2 = srepo::rebuild_search_index(&db).unwrap();
    assert_eq!(n2, 2);
}

#[test]
fn fixture_02_search_finds_raw_and_representation() {
    let db = seeded("find");
    srepo::rebuild_search_index(&db).unwrap();
    let hits = srepo::search(&db, "ownership", None, 50, 0).unwrap();
    assert_eq!(hits.len(), 2);
    let kinds: Vec<_> = hits.iter().map(|h| h.ref_type.as_str()).collect();
    assert!(kinds.contains(&"raw"));
    assert!(kinds.contains(&"representation"));
}

#[test]
fn fixture_03_ref_filter_bounds_results() {
    let db = seeded("filter");
    srepo::rebuild_search_index(&db).unwrap();
    let raw_only = srepo::search(&db, "ownership", Some("raw"), 50, 0).unwrap();
    assert_eq!(raw_only.len(), 1);
    assert_eq!(raw_only[0].ref_type, "raw");
}

#[test]
fn fixture_04_fts_syntax_in_user_input_is_safe() {
    let db = seeded("safe");
    srepo::rebuild_search_index(&db).unwrap();
    // FTS5 operators / broken syntax in user input must not error.
    for q in ["\"unclosed", "AND OR NOT", "ne* (paren", "'quote"] {
        let res = srepo::search(&db, q, None, 50, 0);
        assert!(res.is_ok(), "query {q:?} must not error");
    }
}

#[test]
fn fixture_05_limit_clamps_and_offset_pages() {
    let db = seeded("limit");
    srepo::rebuild_search_index(&db).unwrap();
    let page = srepo::search(&db, "ownership", None, 1, 0).unwrap();
    assert_eq!(page.len(), 1);
    let huge = srepo::search(&db, "ownership", None, 10_000, 0).unwrap();
    assert_eq!(huge.len(), 2, "clamp to 200 still returns all");
    let empty_query = srepo::search(&db, "   ", None, 50, 0).unwrap();
    assert!(empty_query.is_empty());
}

#[test]
fn fixture_06_capture_link_resolved_for_raw_hits() {
    let db = seeded("link");
    srepo::rebuild_search_index(&db).unwrap();
    let raw_hits = srepo::search(&db, "ownership", Some("raw"), 50, 0).unwrap();
    assert_eq!(raw_hits.len(), 1);
    assert!(
        raw_hits[0].capture_id.is_some(),
        "raw hit resolves capture for UI expansion"
    );
}
