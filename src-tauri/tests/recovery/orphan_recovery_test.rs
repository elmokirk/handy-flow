//! DATA-106 + DATA-108 acceptance: startup reconciliation without data
//! loss, and storage usage reporting with preservation defaults.

use handy_app_lib::storage::audio_files::{stage_new, verify_finalized};
use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::recovery::reconcile_startup;
use handy_app_lib::storage::usage::{automatic_retention_enabled, measure};

fn workspace(tag: &str) -> (AppDatabase, std::path::PathBuf, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("handy-recovery-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("history.db");
    let recordings = dir.join("recordings");
    std::fs::create_dir_all(&recordings).unwrap();
    let (db, _) = open_and_migrate(&db_path, "test").expect("db");
    (db, db_path, recordings)
}

fn minimal_wav() -> Vec<u8> {
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&36u32.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&[1, 0, 1, 0]);
    wav.extend_from_slice(&16000u32.to_le_bytes());
    wav.extend_from_slice(&32000u32.to_le_bytes());
    wav.extend_from_slice(&[2, 0, 16, 0]);
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&8u32.to_le_bytes());
    wav.extend_from_slice(&[0u8; 8]);
    wav
}

#[test]
fn orphan_wavs_are_adopted_and_staging_is_reconciled_without_deletion() {
    let (db, _db_path, recordings) = workspace("orphans");

    // A crash artifact: staged temp with VALID wav content.
    let staged = stage_new(&recordings).unwrap();
    staged.append(&minimal_wav()).unwrap();

    // An already-finalized orphan wav with no capture row.
    let orphan_final = recordings.join("rec-orphan.wav");
    std::fs::write(&orphan_final, minimal_wav()).unwrap();
    // Corrupt staging candidate: must SURVIVE untouched.
    let corrupt = stage_new(&recordings).unwrap();
    corrupt.append(b"not audio at all").unwrap();
    let corrupt_path = corrupt.temp_path.clone();

    let report = reconcile_startup(&db, &recordings).expect("reconcile");

    assert_eq!(
        report.recovered_from_staging.len(),
        1,
        "valid temp promoted"
    );
    assert_eq!(report.corrupt_staging.len(), 1, "corrupt temp surfaced");
    assert!(corrupt_path.exists(), "nothing is ever silently deleted");
    assert_eq!(
        report.adopted_orphan_recordings,
        vec!["rec-orphan.wav".to_string()]
    );

    // Both recovered sources became captures marked recovered_orphan.
    use handy_app_lib::storage::models::IntegrityState;
    use handy_app_lib::storage::repositories::captures::list_by_integrity_state;
    let orphans = list_by_integrity_state(&db, IntegrityState::RecoveredOrphan).unwrap();
    assert_eq!(orphans.len(), 2, "staged-promoted + adopted final");

    for capture in &orphans {
        if let Some(name) = &capture.audio_file_name {
            let path = recordings.join(name);
            assert!(path.exists());
            assert!(verify_finalized(&path, capture.audio_sha256.as_ref().unwrap()).unwrap());
        }
    }

    // Idempotent second run: nothing new adopted, nothing deleted.
    let report2 = reconcile_startup(&db, &recordings).unwrap();
    assert!(report2.recovered_from_staging.is_empty());
    assert_eq!(report2.adopted_orphan_recordings.len(), 0);
    assert_eq!(
        list_by_integrity_state(&db, IntegrityState::RecoveredOrphan)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn usage_reports_preservation_state_and_policy_is_off() {
    let (db, db_path, recordings) = workspace("usage");

    // One finalized recording + one capture referencing it + one trashed.
    let staged = stage_new(&recordings).unwrap();
    staged.append(&minimal_wav()).unwrap();
    let finalized = staged.finalize(&recordings).unwrap();

    let id = handy_app_lib::storage::ids::new_id();
    let trash_id = handy_app_lib::storage::ids::new_id();
    db.conn()
        .execute_batch(&format!(
            "INSERT INTO captures(id, title, integrity_state, audio_file_name,
                created_at_ms, updated_at_ms)
             VALUES ('{id}', 'keep', 'audio_valid', '{name}', 1, 1);
            INSERT INTO captures(id, title, integrity_state, audio_file_name,
                created_at_ms, updated_at_ms, deleted_at_ms)
             VALUES ('{trash_id}', 'trashed', 'audio_valid', '{name}', 1, 1, 999);",
            name = finalized.file_name
        ))
        .unwrap();

    // An unreferenced wav on disk.
    std::fs::write(recordings.join("rec-unref.wav"), b"RIFF----WAVE-junk").unwrap();

    let usage = measure(&db, &db_path, &recordings);
    assert_eq!(
        usage.recording_count, 1,
        "active recording count excludes trash"
    );
    assert_eq!(
        usage.trashed_capture_count, 1,
        "trash remains visible/recoverable"
    );
    assert!(
        usage.unreferenced_recording_bytes > 0,
        "unreferenced bytes are visible"
    );
    assert_eq!(usage.staged_temp_count, 0);

    // The core acceptance: automatic destructive retention is OFF and
    // this module exposes no delete path at all.
    assert!(!automatic_retention_enabled());
}
