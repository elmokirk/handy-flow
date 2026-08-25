//! DATA-102: empty-database migration test.

use handy_app_lib::storage::migrations::{open_and_migrate, TARGET_VERSION};

#[test]
fn empty_database_migrates_to_target_idempotently() {
    let dir = std::env::temp_dir().join(format!("handy-mig-empty-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("history.db");
    let _ = std::fs::remove_file(&path);

    let (db, report) = open_and_migrate(&path, "0.9.6-test").expect("migrate fresh db");
    assert_eq!(report.from_version, 0);
    assert_eq!(report.to_version, TARGET_VERSION);
    assert_eq!(report.legacy_rows_backfilled, 0);

    let version: i64 = db
        .conn()
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, TARGET_VERSION);

    // Canonical tables exist; content seeded as designed.
    let meta_rows: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM app_meta", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        meta_rows, 3,
        "app_meta must hold schema_version, query_contract_version, created_by_app_version"
    );
    for table in ["captures", "transcription_attempts", "representations"] {
        let count: i64 = db
            .conn()
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "{table} must be empty on fresh install");
    }

    // app_meta seeded.
    let schema_version: String = db
        .conn()
        .query_row(
            "SELECT value FROM app_meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(schema_version, TARGET_VERSION.to_string());
    let created_by: String = db
        .conn()
        .query_row(
            "SELECT value FROM app_meta WHERE key='created_by_app_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(created_by, "0.9.6-test");

    db.integrity_check().unwrap();

    // Second run: no-op, no backup churn, no backfill.
    let second = db.migrate("0.9.6-test", None).unwrap();
    assert_eq!(second.from_version, TARGET_VERSION);
    assert!(second.backup_path.is_none());

    // Verified backup from first run exists on disk.
    assert!(report.backup_path.unwrap().exists());
}
