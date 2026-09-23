//! Startup recovery reconciler (DATA-106).
//!
//! Contract: reconcile audio/DB incomplete states WITHOUT silent data
//! loss. Nothing here deletes audio â€” unknown or damaged files are
//! surfaced in the report for the owner/explicit purge flow.

use std::collections::HashSet;
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::storage::audio_files::{scan_staged, StagedAudio};
use crate::storage::database::{AppDatabase, StorageError};
use crate::storage::models::IntegrityState;
use crate::storage::repositories::captures::{
    insert_capture, insert_capture_at, list_by_integrity_state, referenced_audio_names,
    set_integrity_state, NewCapture,
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
    reconcile(db, recording_dir, true)
}

/// History refreshes may run while the microphone is recording, so they
/// adopt old orphan files but must not finalize an active pending capture.
pub fn reconcile_history(
    db: &AppDatabase,
    recording_dir: &Path,
) -> Result<RecoveryReport, StorageError> {
    reconcile(db, recording_dir, false)
}

fn reconcile(
    db: &AppDatabase,
    recording_dir: &Path,
    recover_pending: bool,
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

    // 2. A process can die after creating the capture but before finalizing
    // its directly-written WAV. The last checkpointed header is still useful.
    for capture in if recover_pending {
        list_by_integrity_state(db, IntegrityState::PendingAudio).map_err(StorageError::from)?
    } else {
        Vec::new()
    } {
        let state = match capture.audio_file_name.as_deref() {
            None => IntegrityState::AudioMissing,
            Some(name) if !recording_dir.join(name).is_file() => IntegrityState::AudioMissing,
            Some(name) => match hound::WavReader::open(recording_dir.join(name)) {
                Ok(reader) if reader.duration() > 0 => IntegrityState::RecoveredOrphan,
                _ => IntegrityState::AudioCorrupt,
            },
        };
        set_integrity_state(db, &capture.id, state).map_err(StorageError::from)?;
        if state == IntegrityState::AudioMissing {
            report.marked_audio_missing.push(capture.id.clone());
        }
    }

    // 3. Final recordings with no capture row -> adopt as orphans.
    let mut referenced: HashSet<String> = referenced_audio_names(db)
        .map_err(StorageError::from)?
        .into_iter()
        .collect();
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

            // An old WAV's modification time is its best available end instant.
            // Subtract actual duration; never re-date it to recovery time.
            let metadata = path
                .metadata()
                .map_err(|e| StorageError::CorruptData(e.to_string()))?;
            let end_ms = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or_default();
            let duration_ms = hound::WavReader::open(&path).ok().and_then(|reader| {
                let spec = reader.spec();
                (spec.sample_rate > 0 && reader.duration() > 0).then(|| {
                    (i64::from(reader.duration()) * 1000 + i64::from(spec.sample_rate) - 1)
                        / i64::from(spec.sample_rate)
                })
            });
            let integrity_state = if duration_ms.is_some() {
                IntegrityState::RecoveredOrphan
            } else {
                IntegrityState::AudioCorrupt
            };
            let sha = crate::storage::audio_files::hash_file(&path)?;
            let size = metadata.len() as i64;
            insert_capture_at(
                db,
                &NewCapture {
                    audio_file_name: Some(name.clone()),
                    audio_sha256: Some(sha),
                    audio_size_bytes: Some(size),
                    title: format!("Recovered {name} (estimated time)"),
                    source_app: None,
                    integrity_state,
                },
                end_ms.saturating_sub(duration_ms.unwrap_or_default()),
            )
            .map_err(StorageError::from)?;
            referenced.insert(name.clone());
            report.adopted_orphan_recordings.push(name);
        }
    }

    Ok(report)
}
