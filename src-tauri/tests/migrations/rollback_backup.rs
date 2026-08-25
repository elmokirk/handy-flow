//! DATA-102: rollback backup procedure — verified backup + refusal path.

use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::{create_verified_backup, open_and_migrate};

#[test]
fn failed_migration_leaves_database_untouched_and_backup_verified() {
    let dir = std::env::temp_dir().join(format!("handy-mig-rollback-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("history.db");
    let _ = std::fs::remove_file(&path);

    // Bring the db to a pre-canonical state with legacy data.
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE transcription_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_name TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                saved BOOLEAN NOT NULL DEFAULT 0,
                title TEXT NOT NULL,
                transcription_text TEXT NOT NULL
            );
            INSERT INTO transcription_history(file_name, timestamp, saved, title, transcription_text)
            VALUES ('a.wav', 1700000000, 1, 'A', 'text-a');",
        )
        .unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
    }

    let db = AppDatabase::open(&path).expect("open pre-migration db");

    // Injected failure at version 5 → after backup creation.
    let err = db
        .migrate("0.9.6-test", Some(5))
        .expect_err("injected failure must surface");
    assert!(err.to_string().contains("injected failure"));

    // Database unchanged: still version 1, no canonical tables.
    let version: i64 = db
        .conn()
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 1, "failed migration must not advance user_version");

    let canonical_tables: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN \
             ('captures','transcription_attempts','representations','app_meta')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(canonical_tables, 0, "no partial canonical DDL may survive");

    // Legacy row intact.
    let legacy: String = db
        .conn()
        .query_row(
            "SELECT title FROM transcription_history WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(legacy, "A");

    db.integrity_check().unwrap();

    // Real migration now succeeds and its backup is verified + usable.
    let (db2, report) = open_and_migrate(&path, "0.9.6-test").expect("real migration");
    db2.integrity_check()
        .expect("migrated primary database is intact");
    let backup = report.backup_path.expect("backup must exist");
    assert!(backup.exists());

    let ro = AppDatabase::open_read_only(&backup).expect("backup reopens read-only");
    ro.integrity_check().expect("verified backup integrity");
    let legacy_rows_in_backup: i64 = ro
        .conn()
        .query_row("SELECT COUNT(*) FROM transcription_history", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(legacy_rows_in_backup, 1);
}

#[test]
fn create_verified_backup_rejects_non_database() {
    let dir = std::env::temp_dir().join(format!("handy-mig-bakrej-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("history.db");
    std::fs::write(&path, b"not a database at all").unwrap();

    let conn = rusqlite::Connection::open(&path).unwrap();
    let result = create_verified_backup(&conn);
    assert!(
        result.is_err(),
        "backup of a non-database file must be refused"
    );
}
