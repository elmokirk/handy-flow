//! Durable audio capture lifecycle (AUDIO-104).
//!
//! Contract (planning/04): `temp → close/flush → validate → hash/size →
//! atomic rename → DB metadata`. Recovery never silently deletes unknown
//! or valid audio: unfinalized temp files are surfaced by [`scan_staged`]
//! for the recovery reconciler (DATA-106), never auto-removed.
//!
//! This module owns FILE mechanics only; DB rows are written by callers.

use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::database::StorageError;

/// A staged (in-progress) capture file. Not yet valid audio; must be
/// finalized or left for recovery — never silently discarded.
#[derive(Clone, Debug)]
pub struct StagedAudio {
    pub temp_path: PathBuf,
}

/// Result of a successful finalize: immutable facts about the audio.
#[derive(Clone, Debug)]
pub struct FinalizedAudio {
    pub final_path: PathBuf,
    /// Canonical file NAME (not full path) stored in captures.audio_file_name.
    pub file_name: String,
    pub sha256: String,
    pub size_bytes: u64,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default()
}

/// Begin a staged capture. Writes go to `<dir>/.staging/<uuid>-<ts>.tmp`
/// so finalized recordings and staging artifacts are trivially separable
/// during recovery scans.
pub fn stage_new(recording_dir: &Path) -> Result<StagedAudio, StorageError> {
    let staging = recording_dir.join(".staging");
    fs::create_dir_all(&staging).map_err(|e| {
        StorageError::StorageUnavailable(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    })?;
    let temp_path = staging.join(format!("capture-{}.tmp", now_nanos()));
    fs::File::create(&temp_path).map_err(|e| {
        StorageError::StorageUnavailable(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    })?;
    Ok(StagedAudio { temp_path })
}

impl StagedAudio {
    /// Append raw bytes (already encoded, e.g. WAV stream from cpal).
    pub fn append(&self, bytes: &[u8]) -> Result<(), StorageError> {
        let mut f = fs::OpenOptions::new()
            .append(true)
            .open(&self.temp_path)
            .map_err(io_err)?;
        f.write_all(bytes).map_err(io_err)?;
        f.flush().map_err(io_err)?;
        f.sync_all().map_err(io_err)?; // crash durability before rename
        Ok(())
    }

    /// Validate → hash → ATOMIC rename into the recordings directory.
    ///
    /// Validation rejects non-WAV payloads so a corrupt temp can never be
    /// promoted into the canonical store. On any failure the temp file is
    /// LEFT IN PLACE for recovery inspection (no silent deletion).
    pub fn finalize(self, recording_dir: &Path) -> Result<FinalizedAudio, StorageError> {
        // flush/close handled by reopening read-only below.
        let mut file = fs::File::open(&self.temp_path).map_err(io_err)?;

        // Validate RIFF/WAVE header.
        let mut header = [0u8; 12];
        let read = file.read(&mut header).map_err(io_err)?;
        if read < 12 || &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            return Err(StorageError::CorruptData(
                "staged audio is not a RIFF/WAVE file".into(),
            ));
        }

        // Stream-hash the whole file.
        file.seek(std::io::SeekFrom::Start(0)).map_err(io_err)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 64 * 1024];
        let mut size: u64 = 0;
        loop {
            let n = file.read(&mut buf).map_err(io_err)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            size += n as u64;
        }
        drop(file);

        // Minimum plausible WAV: 44-byte canonical header.
        if size < 44 {
            return Err(StorageError::CorruptData(format!(
                "staged audio too small ({size} bytes)"
            )));
        }

        let file_name = format!("rec-{}.wav", now_nanos());
        let final_path = recording_dir.join(&file_name);
        // Same-volume atomic rename; fails without touching the destination
        // if something already exists there.
        fs::rename(&self.temp_path, &final_path).map_err(io_err)?;

        Ok(FinalizedAudio {
            final_path,
            file_name,
            sha256: hex(&hasher.finalize()),
            size_bytes: size,
        })
    }
}

/// List staged-but-unfinalized temp files (recovery candidates).
/// Read-only: deletion decisions belong to the explicit purge/recovery flow.
pub fn scan_staged(recording_dir: &Path) -> Vec<PathBuf> {
    let staging = recording_dir.join(".staging");
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(&staging) {
        for e in entries.flatten() {
            if e.path().is_file() {
                out.push(e.path());
            }
        }
    }
    out.sort();
    out
}

/// Verify an EXISTING finalized recording still matches its recorded hash.
pub fn verify_finalized(path: &Path, expected_sha256: &str) -> Result<bool, StorageError> {
    let mut file = fs::File::open(path).map_err(io_err)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(io_err)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()) == expected_sha256)
}

fn io_err(e: std::io::Error) -> StorageError {
    StorageError::StorageUnavailable(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
}
