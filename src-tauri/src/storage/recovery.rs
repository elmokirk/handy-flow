//! Startup recovery reconciler (DATA-106).
//!
//! Contract: reconcile audio/DB incomplete states WITHOUT silent data
//! loss. Nothing here deletes audio â€” unknown or damaged files are
//! surfaced in the report for the owner/explicit purge flow.

use std::path::Path;

use crate::storage::audio_files::{scan_staged, StagedAudio};
use crate::storage::database::{AppDatabase, StorageError};
use crate::storage::models::IntegrityState;
use crate::storage::repositories::captures::{
    insert_capture, list_by_integrity_state, referenced_audio_names, set_integrity_state,
    NewCapture,
};

#[derive(Clone, Debug, Default)]
pub struct RecoveryReport {
    /// Staged temps that finalized cleanly -> recovered as orphan captures.
    pub recovered_from_staging: Vec<String>,
    /// Staged temps that failed validation (left on disk untouched).
    pub corrupt_staging: Vec<String>,
    /// pending_audio captures whose temp vanished -> marked audio_missing.
    pub marked_audio_missing: Vec<String>,
    /// Final WAVs on disk with no capture row -> captured as orphans.
    pub adopted_orphan_recordings: Vec<String>,
}

/// Run all startup reconciliations. Idempotent and safe to re-run.
pub fn reconcile_startup(
    db: &AppDatabase,
    recording_dir: &Path,
) -> Result<RecoveryReport, StorageError> {
    let mut report = RecoveryReport::default();

    // 1. Unfinalized staging temps: try to promote valid ones.
    for temp in scan_staged(recording_dir) {
        // Re-stage through the same validate/hash/rename path by moving
        // the temp under a fresh StagedAudio handle.
        let staged = StagedAudio {
            temp_path: temp.clone(),
        };
        match staged.finalize(recording_dir) {
            Ok(finalized) => {
                insert_capture(
                    db,
                    &NewCapture {
                        audio_file_name: Some(finalized.file_name.clone()),
                        audio_sha256: Some(finalized.sha256),
                        audio_size_bytes: Some(finalized.size_bytes as i64),
                        title: format!("Recovered {}", finalized.file_name),
                        source_app: None,
                        integrity_state: IntegrityState::RecoveredOrphan,
                    },
                )
                .map_err(StorageError::from)?;
                report.recovered_from_staging.push(
                    temp.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                );
            }
            Err(_) => {
                report.corrupt_staging.push(
                    temp.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }

    // 2. Captures still pending_audio whose staged temp is gone.
    for capture in
        list_by_integrity_state(db, IntegrityState::PendingAudio).map_err(StorageError::from)?
    {
        if capture.audio_file_name.is_none() {
            set_integrity_state(db, &capture.id, IntegrityState::AudioMissing)
                .map_err(StorageError::from)?;
            report.marked_audio_missing.push(capture.id.clone());
        }
    }

    // 3. Final recordings with no capture row -> adopt as orphans.
    let referenced = referenced_audio_names(db).map_err(StorageError::from)?;
    if let Ok(entries) = std::fs::read_dir(recording_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_wav = path.extension().map(|e| e == "wav").unwrap_or(false);
            if !is_wav || !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if referenced.contains(&name) {
                continue;
            }

            // Hash the orphan so the new capture carries verifiable facts.
            let sha = crate::storage::audio_files::hash_file(&path)?;
            let size = path.metadata().map(|m| m.len() as i64).unwrap_or(0);
            insert_capture(
                db,
                &NewCapture {
                    audio_file_name: Some(name.clone()),
                    audio_sha256: Some(sha),
                    audio_size_bytes: Some(size),
                    title: format!("Adopted {name}"),
                    source_app: None,
                    integrity_state: IntegrityState::RecoveredOrphan,
                },
            )
            .map_err(StorageError::from)?;
            report.adopted_orphan_recordings.push(name);
        }
    }

    Ok(report)
}
