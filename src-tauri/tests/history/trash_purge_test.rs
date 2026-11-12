//! HIST-109 acceptance: Trash → Restore → explicit Purge with
//! repository-controlled order and no automatic deletion anywhere.

use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::models::IntegrityState;
use handy_app_lib::storage::repositories::captures::{
    capture_detail, get_capture, insert_capture, list_active_captures, list_trashed, purge_trashed,
    restore_capture, trash_capture, NewCapture,
};

fn seeded(tag: &str) -> (AppDatabase, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("handy-trash-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    (db, dir)
}

fn capture_with_audio(db: &AppDatabase, title: &str, wav: &str) -> String {
    let id = handy_app_lib::storage::ids::new_id();
    let att = handy_app_lib::storage::ids::new_id();
    db.conn()
        .execute_batch(&format!(
            "INSERT INTO captures(id, title, audio_file_name, integrity_state, created_at_ms, updated_at_ms)
             VALUES ('{id}', '{title}', '{wav}', 'audio_valid', 1, 1);
            INSERT INTO transcription_attempts(id, capture_id, attempt_number, normalized_stt,
                provenance, status, is_canonical, created_at_ms)
             VALUES ('{att}', '{id}', 1, 'text', 'live', 'success', 1, 1);
            INSERT INTO representations(id, attempt_id, kind, text, status, created_at_ms)
             VALUES ('{rep}', '{att}', 'manual_edit', 'edited', 'success', 2);
            INSERT INTO delivery_events(id, capture_id, source_attempt_id, source_kind,
                text_sha256, success, created_at_ms)
             VALUES ('{ev}', '{id}', '{att}', 'normalized_stt', 'abc', 1, 3);",
            rep = handy_app_lib::storage::ids::new_id(),
            ev = handy_app_lib::storage::ids::new_id(),
        ))
        .unwrap();
    id
}

#[test]
fn trash_restore_roundtrip_excludes_from_active_queries() {
    let (db, _d) = seeded("roundtrip");
    let a = insert_capture(
        &db,
        &NewCapture {
            audio_file_name: Some("a.wav".into()),
            audio_sha256: None,
            audio_size_bytes: None,
            title: "A".into(),
            source_app: None,
            integrity_state: IntegrityState::AudioValid,
        },
    )
    .unwrap();

    assert!(trash_capture(&db, &a.id).unwrap(), "first trash succeeds");
    assert!(
        !trash_capture(&db, &a.id).unwrap(),
        "double trash is a no-op"
    );
    assert!(list_active_captures(&db, 50, 0).unwrap().is_empty());
    assert_eq!(list_trashed(&db).unwrap().len(), 1);
    assert!(get_capture(&db, &a.id)
        .unwrap()
        .unwrap()
        .deleted_at_ms
        .is_some());

    assert!(restore_capture(&db, &a.id).unwrap());
    assert!(!restore_capture(&db, &a.id).unwrap());
    assert_eq!(list_active_captures(&db, 50, 0).unwrap().len(), 1);
    assert!(get_capture(&db, &a.id)
        .unwrap()
        .unwrap()
        .deleted_at_ms
        .is_none());
}

#[test]
fn explicit_purge_removes_rows_in_order_but_never_audio_files() {
    let (db, dir) = seeded("purge");
    let recordings = dir.join("recordings");
    std::fs::create_dir_all(&recordings).unwrap();
    let wav_path = recordings.join("keepme.wav");
    std::fs::write(&wav_path, b"RIFF....WAVEdata").unwrap();

    let cap = capture_with_audio(&db, "Purge me", "keepme.wav");
    trash_capture(&db, &cap).unwrap();

    // Repository purge returns the audio reference; FILE deletion is the
    // caller's explicit job — the storage layer itself never deletes.
    let purged = purge_trashed(&db, &cap).unwrap();
    assert_eq!(purged.audio_file_name.as_deref(), Some("keepme.wav"));
    assert!(
        wav_path.exists(),
        "repository must not touch the filesystem"
    );

    // All canonical rows for the capture are gone.
    assert!(get_capture(&db, &cap).unwrap().is_none());
    assert!(capture_detail(&db, &cap).unwrap().is_none());

    // Purging an ACTIVE (non-trashed) capture is refused.
    let active = insert_capture(
        &db,
        &NewCapture {
            audio_file_name: None,
            audio_sha256: None,
            audio_size_bytes: None,
            title: "active".into(),
            source_app: None,
            integrity_state: IntegrityState::PendingAudio,
        },
    )
    .unwrap();
    assert!(purge_trashed(&db, &active.id).is_err());
}

#[test]
fn stor260_trash_pages_are_bounded_and_restore_is_visible() {
    use handy_app_lib::storage::repositories::captures::{count_trashed, list_trashed_page};
    let (db, _) = seeded("pages260");
    {
        let conn = db.conn();
        for i in 0..3000 {
            conn.execute("INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms, deleted_at_ms) VALUES (?1, 'entry', 'audio_missing', ?2, ?2, ?2)", rusqlite::params![format!("cap-{i:04}"), i]).unwrap();
        }
    }
    assert_eq!(count_trashed(&db).unwrap(), 3000);
    let first = list_trashed_page(&db, 1000, 0).unwrap();
    let second = list_trashed_page(&db, 200, 200).unwrap();
    assert_eq!(first.len(), 200);
    assert_eq!(second.len(), 200);
    assert_ne!(first[0].id, second[0].id);
    assert!(restore_capture(&db, &first[0].id).unwrap());
    assert_eq!(count_trashed(&db).unwrap(), 2999);
    assert!(list_active_captures(&db, 200, 0)
        .unwrap()
        .iter()
        .any(|c| c.id == first[0].id));
}

#[test]
fn stor260_purge_failure_keeps_reference_and_missing_file_retry_succeeds() {
    use handy_app_lib::storage::repositories::captures::purge_trashed_with_audio;
    let (db, dir) = seeded("purge260");
    let recordings = dir.join("recordings");
    std::fs::create_dir_all(recordings.join("blocked.wav")).unwrap();
    let id = capture_with_audio(&db, "blocked", "blocked.wav");
    trash_capture(&db, &id).unwrap();
    assert!(purge_trashed_with_audio(&db, &id, &recordings).is_err());
    assert!(get_capture(&db, &id)
        .unwrap()
        .unwrap()
        .deleted_at_ms
        .is_some());
    std::fs::remove_dir(recordings.join("blocked.wav")).unwrap();
    purge_trashed_with_audio(&db, &id, &recordings).unwrap();
    assert!(get_capture(&db, &id).unwrap().is_none());
}

#[cfg(windows)]
#[test]
fn stor260_locked_audio_cannot_be_orphaned_and_retry_works() {
    use handy_app_lib::storage::repositories::captures::purge_trashed_with_audio;
    use std::os::windows::fs::OpenOptionsExt;
    let (db, dir) = seeded("locked260");
    let recordings = dir.join("recordings");
    std::fs::create_dir_all(&recordings).unwrap();
    let path = recordings.join("locked.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).unwrap();
    writer.write_sample::<i16>(1).unwrap();
    writer.finalize().unwrap();
    let id = capture_with_audio(&db, "locked", "locked.wav");
    trash_capture(&db, &id).unwrap();
    let locked = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();
    assert!(purge_trashed_with_audio(&db, &id, &recordings).is_err());
    assert!(get_capture(&db, &id)
        .unwrap()
        .unwrap()
        .deleted_at_ms
        .is_some());
    drop(locked);
    let report = handy_app_lib::storage::recovery::reconcile_history(&db, &recordings).unwrap();
    assert!(report.adopted_orphan_recordings.is_empty());
    purge_trashed_with_audio(&db, &id, &recordings).unwrap();
    assert!(!path.exists());
}
