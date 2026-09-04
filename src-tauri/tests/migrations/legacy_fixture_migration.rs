//! DATA-102: legacy fixture migration — the acceptance-critical mapping.

use handy_app_lib::storage::migrations::{open_and_migrate, TARGET_VERSION};
use handy_app_lib::storage::models::{AttemptProvenance, AttemptStatus};

fn seed_legacy_fixture(path: &std::path::Path) {
    let conn = rusqlite::Connection::open(path).unwrap();
    // Exact upstream shape AFTER its own chain (user_version = 4): the
    // most common real-world starting point for the canonical migration.
    conn.execute_batch(
        "CREATE TABLE transcription_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_name TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            saved BOOLEAN NOT NULL DEFAULT 0,
            title TEXT NOT NULL,
            transcription_text TEXT NOT NULL
        );
        ALTER TABLE transcription_history ADD COLUMN post_processed_text TEXT;
        ALTER TABLE transcription_history ADD COLUMN post_process_prompt TEXT;
        ALTER TABLE transcription_history ADD COLUMN post_process_requested BOOLEAN NOT NULL DEFAULT 0;
        PRAGMA user_version = 4;

        INSERT INTO transcription_history(
            file_name, timestamp, saved, title, transcription_text
        ) VALUES (
            'rec-1700000000.wav', 1700000000, 1, 'Plain note',
            'einfach nur diktiert'
        );
        INSERT INTO transcription_history(
            file_name, timestamp, saved, title, transcription_text,
            post_processed_text, post_process_prompt, post_process_requested
        ) VALUES (
            'rec-1700000100.wav', 1700000100, 1, 'Post processed',
            'roher transkriptionstext',
            'aufbereiteter transkriptionstext',
            'Fasse den Text zusammen', 1
        );
        INSERT INTO transcription_history(
            file_name, timestamp, saved, title, transcription_text,
            post_process_prompt, post_process_requested
        ) VALUES (
            'rec-1700000200.wav', 1700000200, 1, 'Requested but failed',
            'zweiter rohtext',
            'Fasse den Text zusammen', 1
        );",
    )
    .unwrap();
}

#[test]
fn legacy_fixture_maps_to_canonical_tables() {
    let dir = std::env::temp_dir().join(format!("handy-mig-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("history.db");
    let _ = std::fs::remove_file(&path);
    seed_legacy_fixture(&path);

    let (db, report) = open_and_migrate(&path, "0.9.6-test").expect("migrate legacy fixture");
    assert_eq!(report.from_version, 4, "upstream-migrated db starts at 4");
    // Bound to TARGET_VERSION, not a literal: the assertion means "migrated
    // to the current target", so a schema step does not need this file edited.
    assert_eq!(report.to_version, TARGET_VERSION);
    assert_eq!(report.legacy_rows_backfilled, 3);

    // Acceptance: legacy table preserved untouched.
    let legacy_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM transcription_history", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(legacy_count, 3, "legacy table must be preserved");

    // One canonical capture per legacy row.
    let capture_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM captures", [], |r| r.get(0))
        .unwrap();
    assert_eq!(capture_count, 3);

    // Counts + audio references validated.
    let with_audio: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM captures WHERE audio_file_name IS NOT NULL \
             AND integrity_state = 'audio_valid'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(with_audio, 3, "every legacy capture references its wav");

    // Legacy seconds → canonical ms.
    let created_ms: i64 = db
        .conn()
        .query_row(
            "SELECT created_at_ms FROM captures WHERE legacy_history_id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(created_ms, 1_700_000_000_000, "seconds must become ms");

    // Acceptance: legacy transcription text maps to normalized_stt.
    let (normalized, provenance, status, engine_raw): (String, String, String, Option<String>) = db
        .conn()
        .query_row(
            "SELECT a.normalized_stt, a.provenance, a.status, a.engine_raw
             FROM transcription_attempts a JOIN captures c ON c.id = a.capture_id
             WHERE c.legacy_history_id = 2",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(normalized, "roher transkriptionstext");
    assert_eq!(
        provenance,
        AttemptProvenance::LegacyMigration.as_str(),
        "imported attempts are marked as such"
    );
    assert_eq!(status, AttemptStatus::Success.as_str());
    assert!(engine_raw.is_none(), "legacy engine_raw is unknown → NULL");

    let is_canonical: i64 = db
        .conn()
        .query_row(
            "SELECT a.is_canonical FROM transcription_attempts a
             JOIN captures c ON c.id = a.capture_id WHERE c.legacy_history_id = 2",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(is_canonical, 1, "single legacy attempt becomes canonical");

    // Acceptance: legacy post-process preserved as representation.
    let (rep_kind, rep_text, rep_status, prompt_snapshot): (String, String, String, String) = db
        .conn()
        .query_row(
            "SELECT r.kind, r.text, r.status, r.effective_prompt_snapshot
             FROM representations r
             JOIN transcription_attempts a ON a.id = r.attempt_id
             JOIN captures c ON c.id = a.capture_id
             WHERE c.legacy_history_id = 2",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(rep_kind, "post_process");
    assert_eq!(rep_text, "aufbereiteter transkriptionstext");
    assert_eq!(rep_status, "success");
    assert_eq!(prompt_snapshot, "Fasse den Text zusammen");

    // Requested-but-failed post-processing becomes a FAILED representation.
    let failed_rep: (String, i64) = db
        .conn()
        .query_row(
            "SELECT r.status, COUNT(*) FROM representations r
             JOIN transcription_attempts a ON a.id = r.attempt_id
             JOIN captures c ON c.id = a.capture_id
             WHERE c.legacy_history_id = 3",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(failed_rep, ("failed".into(), 1));

    // Plain entry without post-processing has NO representation.
    let reps_for_plain: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM representations r
             JOIN transcription_attempts a ON a.id = r.attempt_id
             JOIN captures c ON c.id = a.capture_id WHERE c.legacy_history_id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(reps_for_plain, 0);

    db.integrity_check().unwrap();

    // Re-run: idempotent backfill, no duplicates.
    let again = db.migrate("0.9.6-test", None).unwrap();
    assert_eq!(again.from_version, TARGET_VERSION);
}
