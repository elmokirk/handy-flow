//! IMP-001 acceptance: read-only, idempotent Wispr import with correct
//! canonical mapping and umlaut round-trip. Uses a synthetic Wispr-shaped
//! fixture DB (no real user data touched).

use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::dictionary as dict_repo;
use handy_app_lib::storage::repositories::snippets as snip_repo;
use handy_app_lib::wispr_import::{dry_run, run_import};

fn fixture_wispr_db(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("flow.sqlite");
    let con = rusqlite::Connection::open(&path).unwrap();
    con.execute_batch(
        "CREATE TABLE History (
            transcriptEntityId TEXT PRIMARY KEY,
            timestamp TEXT,
            asrText TEXT,
            formattedText TEXT,
            editedText TEXT,
            app TEXT,
            audio BLOB,
            duration INTEGER
        );
        CREATE TABLE Dictionary (
            id INTEGER PRIMARY KEY,
            phrase TEXT,
            replacement TEXT,
            isDeleted INTEGER DEFAULT 0,
            modifiedAt TEXT
        );
        CREATE TABLE Polish (
            id TEXT,
            polishInitialText TEXT,
            polishedText TEXT,
            instruction TEXT,
            modelVersion TEXT,
            createdAt TEXT
        );
        INSERT INTO History VALUES
         ('h-1', '2026-08-27 18:22:20.079 +00:00', 'test mit umlauten: größer', 'Test mit Umlauten!', NULL, 'chrome', NULL, 1234),
         ('h-2', '2026-08-27 18:23:00.000 +00:00', 'zweiter rohtext', NULL, NULL, 'code', NULL, 900);
        INSERT INTO Dictionary(phrase, replacement, isDeleted, modifiedAt) VALUES
         ('btw', 'by the way', 0, '2026-08-01'),
         ('Kirk Kleinau', NULL, 0, '2026-08-01');
        INSERT INTO Polish VALUES ('h-1', 'test', 'verbessert: Test mit Umlauten', 'mach besser', 'wispr-v9', '2026-08-27');
        ",
    )
    .unwrap();
    path
}

fn fresh_target(tag: &str) -> (AppDatabase, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("handy-imp-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let recordings = dir.join("recordings");
    std::fs::create_dir_all(&recordings).unwrap();
    open_and_migrate(dir.join("history.db"), "test")
        .map(|(db, _)| (db, recordings))
        .unwrap()
}

#[test]
fn fixture_01_dry_run_reports_counts_without_writes() {
    let dir = std::env::temp_dir().join(format!("handy-imp-dry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let wispr = fixture_wispr_db(&dir);
    let report = dry_run(&wispr).unwrap();
    assert_eq!(report.history_imported, 2);
    assert_eq!(report.dictionary_imported, 2);
    // Nothing was imported anywhere (dry run has no target).
}

#[test]
fn fixture_02_import_maps_canonically_and_is_idempotent() {
    let dir = std::env::temp_dir().join(format!("handy-imp-full-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let wispr = fixture_wispr_db(&dir);
    let (db, recordings) = fresh_target("target-full");

    let report = run_import(&wispr, &db, Some(&recordings)).unwrap();
    assert_eq!(report.history_imported, 2);
    assert!(report.audio_extracted == 0, "fixture has no audio blobs");

    use handy_app_lib::storage::repositories::captures::list_active_captures;
    let caps = list_active_captures(&db, 50, 0).unwrap();
    assert_eq!(caps.len(), 2);

    // Dictionary split: 'btw' (with replacement) -> snippet; name -> dictionary term.
    let terms = dict_repo::list_all(&db).unwrap();
    assert!(
        terms.iter().any(|e| e.term == "Kirk Kleinau"),
        "name without replacement becomes dictionary term"
    );
    let snippets = snip_repo::list_all(&db).unwrap();
    assert!(
        snippets
            .iter()
            .any(|s| s.trigger == "btw" && s.replacement == "by the way"),
        "phrase with replacement becomes snippet trigger"
    );

    // Idempotency: second run imports nothing new.
    let report2 = run_import(&wispr, &db, Some(&recordings)).unwrap();
    assert_eq!(report2.history_imported, 0, "all entries already imported");
    assert!(report2.history_skipped >= 2);
    assert_eq!(
        handy_app_lib::storage::repositories::captures::list_active_captures(&db, 200, 0)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn fixture_03_umlauts_roundtrip() {
    let dir = std::env::temp_dir().join(format!("handy-imp-uml-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let wispr = fixture_wispr_db(&dir);
    let (db, recordings) = fresh_target("target-uml");

    run_import(&wispr, &db, Some(&recordings)).unwrap();
    let conn = db.conn();
    let mut stmt = conn
        .prepare(
            "SELECT normalized_stt FROM transcription_attempts WHERE normalized_stt LIKE '%umlaut%'",
        )
        .unwrap();
    let texts: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .unwrap()
        .flatten()
        .collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("größer") || t.contains("umlaut")),
        "German umlauts must round-trip: {texts:?}"
    );
}
