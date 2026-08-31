//! HIST-107 acceptance: canonical history queries with raw/derived
//! distinction, preserved audio references and visible legacy rows.

use handy_app_lib::storage::migrations::open_and_migrate;

/// Build a manager-like fixture WITHOUT a Tauri AppHandle: HistoryManager
/// needs an AppHandle at construction, so these tests exercise the
/// canonical query layer through the same repository calls the manager
/// composes (list_active_captures / capture_detail), plus the migration
/// unification path via open_and_migrate on the same db filename.
fn seeded(
    tag: &str,
) -> (
    handy_app_lib::storage::database::AppDatabase,
    std::path::PathBuf,
) {
    let dir = std::env::temp_dir().join(format!("handy-hist-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("history.db");

    // Legacy row (pre-canonical era) at upstream's user_version=4 shape.
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
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
            INSERT INTO transcription_history(file_name, timestamp, saved, title, transcription_text)
             VALUES ('legacy.wav', 1700000000, 1, 'Legacy note', 'alter legacytext');",
        )
        .unwrap();
    }

    let (db, _) = open_and_migrate(&db_path, "test").unwrap();

    // A live capture with canonical attempt (raw) + one style representation.
    let cap = handy_app_lib::storage::ids::new_id();
    let att = handy_app_lib::storage::ids::new_id();
    db.conn()
        .execute_batch(&format!(
            "INSERT INTO captures(id, title, audio_file_name, integrity_state, created_at_ms, updated_at_ms)
             VALUES ('{cap}', 'Live note', 'live.wav', 'audio_valid', 1700001000000, 1700001000000);
            INSERT INTO transcription_attempts(id, capture_id, attempt_number, engine_raw,
                normalized_stt, model_id, language, normalizer_version,
                provenance, status, is_canonical, created_at_ms, completed_at_ms)
             VALUES ('{att}', '{cap}', 1, '  roh text mit fuellwoertern halt ',
                     'roh text', 'whisper-small', 'de', '1',
                     'live', 'success', 1, 1700001000000, 1700001000500);
            INSERT INTO representations(id, attempt_id, kind, text, processor,
                effective_prompt_snapshot, status, created_at_ms)
             VALUES ('{rep}', '{att}', 'style', 'stilisierter text', 'llm_style',
                     'prompt-snapshot', 'success', 1700001001000);",
            rep = handy_app_lib::storage::ids::new_id()
        ))
        .unwrap();
    (db, dir)
}

#[test]
fn raw_and_derived_are_distinguishable_per_entry() {
    use handy_app_lib::storage::repositories::captures::{capture_detail, list_active_captures};

    let (db, _dir) = seeded("rawderived");
    let page = list_active_captures(&db, 50, 0).unwrap();
    assert_eq!(page.len(), 2, "legacy backfill + live capture both visible");

    // Deterministic order: newest first.
    assert_eq!(page[0].title, "Live note");
    assert!(
        page[1].legacy_history_id.is_some(),
        "legacy entry visible in canonical list"
    );

    for capture in &page {
        let d = capture_detail(&db, &capture.id).unwrap().unwrap();
        let canonical = d
            .attempts
            .iter()
            .find(|a| a.is_canonical)
            .expect("canonical attempt");
        let raw = canonical
            .normalized_stt
            .as_deref()
            .expect("raw text present");

        match d.capture.legacy_history_id {
            None => {
                // Live entry: raw differs from derived, engine_raw preserved.
                assert_eq!(raw, "roh text");
                assert_ne!(
                    canonical.engine_raw.as_deref(),
                    Some(raw),
                    "engine_raw keeps the exact pre-cleanup output"
                );
                assert_eq!(d.representations.len(), 1);
                assert_eq!(d.representations[0].kind, "style");
                assert_ne!(
                    d.representations[0].text, *raw,
                    "derived text is distinct from raw"
                );
            }
            Some(_) => {
                // Legacy entry: normalized only; no live engine_raw existed.
                assert_eq!(raw, "alter legacytext");
                assert!(canonical.engine_raw.is_none());
                assert_eq!(canonical.provenance, "legacy_migration");
            }
        }
        // Audio reference survives for playback.
        assert!(d.capture.audio_file_name.is_some());
    }
}

#[test]
fn pagination_is_bounded_and_ordered() {
    use handy_app_lib::storage::repositories::captures::list_active_captures;

    let (db, _dir) = seeded("paging");
    // clamp(1..=200) + deterministic ordering contract.
    let huge = list_active_captures(&db, 10_000, 0).unwrap();
    assert_eq!(huge.len(), 2);
    assert!(huge
        .windows(2)
        .all(|w| w[0].created_at_ms >= w[1].created_at_ms));
}
