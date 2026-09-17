//! Wispr Flow one-time import (IMP-001).
//!
//! Source DB is opened STRICTLY read-only (`mode=ro`); nothing in the
//! Wispr directory is ever written or deleted (owner cleans up after
//! uninstalling — AGENTS.md Rule 5). Idempotent via
//! `captures.import_ref = wispr:<transcriptEntityId>`.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension};

use crate::storage::database::{AppDatabase, StorageError};
use crate::storage::ids;
use crate::storage::models::IntegrityState;
use crate::storage::repositories::captures::{insert_capture, NewCapture};
use crate::storage::repositories::dictionary as dict_repo;
use crate::storage::repositories::snippets as snip_repo;

#[derive(Clone, Debug, Default, serde::Serialize, specta::Type)]
pub struct ImportReport {
    pub history_imported: usize,
    pub history_skipped: usize,
    pub history_invalid_timestamp: usize,
    pub audio_extracted: usize,
    pub dictionary_imported: usize,
    pub snippet_triggers_imported: usize,
    pub polish_imported: usize,
    pub errors: Vec<String>,
}

fn open_wispr_ro(path: &Path) -> Result<Connection, rusqlite::Error> {
    // Open read-only via a plain path with flags (no URI: avoids Windows
    // path/drive quirks with spaces in %TEMP%). `SQLITE_OPEN_READ_ONLY`
    // is the hard guarantee — writes to the Wispr file fail at the
    // SQLite level regardless of caller bugs.
    Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
}

/// Wispr stores chrono strings (`2026-08-27 18:22:20.079 +00:00`).
/// Parse the original instant directly so fractional milliseconds cannot drift.
fn parse_wispr_time(raw: &Option<String>) -> Result<i64, String> {
    let ts = raw
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "missing timestamp".to_string())?;
    chrono::DateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S%.f %:z")
        .or_else(|_| chrono::DateTime::parse_from_rfc3339(ts))
        .map(|value| value.timestamp_millis())
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(ts, "%Y-%m-%d").map(|date| {
                date.and_hms_opt(0, 0, 0)
                    .expect("midnight is valid")
                    .and_utc()
                    .timestamp_millis()
            })
        })
        .map_err(|_| format!("invalid timestamp: {ts}"))
}

fn first_line(s: &str) -> String {
    s.lines()
        .next()
        .unwrap_or("Imported entry")
        .chars()
        .take(60)
        .collect()
}

fn extract_audio(dir: &Path, blob: &[u8]) -> Result<(String, String, i64), String> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let name = format!("wispr-{ts}.wav");
    let path: PathBuf = dir.join(&name);
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    std::fs::write(&path, blob).map_err(|e| e.to_string())?;
    let hash = crate::storage::audio_files::hash_file(&path).map_err(|e| e.to_string())?;
    Ok((name, hash, blob.len() as i64))
}

/// Count-only dry run (no writes anywhere).
pub fn dry_run(wispr_db: &Path, target: &AppDatabase) -> Result<ImportReport, String> {
    let con = open_wispr_ro(wispr_db).map_err(|e| format!("cannot open Wispr DB: {e}"))?;
    let mut report = ImportReport::default();
    let mut stmt = con
        .prepare(
            "SELECT transcriptEntityId, timestamp,
                    CASE WHEN audio IS NOT NULL AND length(audio) > 0 THEN 1 ELSE 0 END
             FROM History",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let entity_id: String = row.get(0).map_err(|e| e.to_string())?;
        let timestamp: Option<String> = row.get(1).unwrap_or(None);
        let has_audio: bool = row.get::<_, i64>(2).unwrap_or(0) == 1;
        if parse_wispr_time(&timestamp).is_err() {
            report.history_invalid_timestamp += 1;
            continue;
        }
        let import_ref = format!("wispr:{entity_id}");
        let exists: i64 = target
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM captures WHERE import_ref = ?1",
                [&import_ref],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists > 0 {
            report.history_skipped += 1;
        } else {
            report.history_imported += 1;
            if has_audio {
                report.audio_extracted += 1;
            }
        }
    }
    report.dictionary_imported = con
        .query_row(
            "SELECT COUNT(*) FROM Dictionary WHERE isDeleted = 0",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n as usize)
        .map_err(|e| e.to_string())?;
    Ok(report)
}

/// Full import. Returns the assembled report.
pub fn run_import(
    wispr_db: &Path,
    target: &AppDatabase,
    recordings_dir: Option<&Path>,
) -> Result<ImportReport, String> {
    let con = open_wispr_ro(wispr_db).map_err(|e| format!("cannot open Wispr DB: {e}"))?;
    let mut report = ImportReport::default();

    // ---- History → captures + attempts + representations ----
    let mut stmt = con
        .prepare(
            "SELECT transcriptEntityId, timestamp, asrText, formattedText, editedText, \
                    app, audio FROM History ORDER BY timestamp ASC",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let entity_id: String = row.get(0).map_err(|e| e.to_string())?;
        let import_ref = format!("wispr:{entity_id}");

        let exists: i64 = {
            let conn = target.conn();
            conn.query_row(
                "SELECT COUNT(*) FROM captures WHERE import_ref = ?1",
                [&import_ref],
                |r| r.get(0),
            )
            .unwrap_or(0)
        };
        if exists > 0 {
            report.history_skipped += 1;
            continue;
        }

        let timestamp_s: Option<String> = row.get(1).unwrap_or(None);
        let created_ms = match parse_wispr_time(&timestamp_s) {
            Ok(created_ms) => created_ms,
            Err(error) => {
                report.history_invalid_timestamp += 1;
                report.errors.push(format!("{entity_id}: {error}"));
                continue;
            }
        };
        let asr: Option<String> = row.get(2).unwrap_or(None);
        let formatted: Option<String> = row.get(3).unwrap_or(None);
        let edited: Option<String> = row.get(4).unwrap_or(None);
        let app: Option<String> = row.get(5).unwrap_or(None);
        let audio: Option<Vec<u8>> = row.get(6).unwrap_or(None);

        let mut audio_file_name = None;
        let mut audio_sha = None;
        let mut audio_size = None;
        let mut integrity = IntegrityState::RecoveredOrphan;
        if let (Some(dir), Some(blob)) = (recordings_dir, audio.as_deref()) {
            match extract_audio(dir, blob) {
                Ok((name, sha, size)) => {
                    audio_file_name = Some(name);
                    audio_sha = Some(sha);
                    audio_size = Some(size);
                    integrity = IntegrityState::AudioValid;
                    report.audio_extracted += 1;
                }
                Err(e) => report.errors.push(format!("{entity_id}: audio: {e}")),
            }
        }

        let body = asr
            .as_deref()
            .or(formatted.as_deref())
            .unwrap_or("Imported Wispr entry");
        let capture = insert_capture(
            target,
            &NewCapture {
                audio_file_name: audio_file_name.clone(),
                audio_sha256: audio_sha.clone(),
                audio_size_bytes: audio_size,
                title: first_line(body),
                source_app: app.clone(),
                integrity_state: integrity,
            },
        )
        .map_err(|e| e.to_string())?;

        {
            let conn = target.conn();
            conn.execute(
                "UPDATE captures SET import_ref = ?2, created_at_ms = ?3, updated_at_ms = ?3 WHERE id = ?1",
                rusqlite::params![capture.id, import_ref, created_ms],
            )
            .map_err(|e| e.to_string())?;
            let attempt_id = ids::new_id();
            conn.execute(
                "INSERT INTO transcription_attempts(id, capture_id, attempt_number, normalized_stt, \
                 provenance, status, is_canonical, created_at_ms, completed_at_ms) \
                 VALUES (?1, ?2, 1, ?3, 'legacy_migration', 'success', 1, ?4, ?4)",
                rusqlite::params![attempt_id, capture.id, asr, created_ms],
            )
            .map_err(|e| e.to_string())?;

            for (text, kind) in [
                (formatted.as_deref(), "post_process"),
                (edited.as_deref(), "manual_edit"),
            ] {
                let Some(t) = text else { continue };
                if t.is_empty() {
                    continue;
                }
                conn.execute(
                    "INSERT INTO representations(id, attempt_id, parent_representation_id, kind, text, \
                     processor, status, created_at_ms) \
                     VALUES (?1, ?2, NULL, ?3, ?4, 'legacy_migration', 'success', ?5)",
                    rusqlite::params![ids::new_id(), attempt_id, kind, t, created_ms],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        report.history_imported += 1;
    }

    // ---- Dictionary: semantic split (ADR-020 compatible) ----
    let mut stmt = con
        .prepare(
            "SELECT phrase, replacement FROM Dictionary WHERE isDeleted = 0 \
             ORDER BY modifiedAt ASC",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let phrase: String = row.get(0).map_err(|e| e.to_string())?;
        let replacement: Option<String> = row.get(1).unwrap_or(None);
        match replacement {
            Some(r) if !r.trim().is_empty() => {
                snip_repo::upsert_snippet(
                    target,
                    &snip_repo::NewSnippet {
                        trigger: phrase,
                        replacement: r,
                        priority: 0,
                        enabled: true,
                    },
                )
                .map_err(|e| e.to_string())?;
                report.snippet_triggers_imported += 1;
            }
            _ => {
                dict_repo::upsert_entry(
                    target,
                    &dict_repo::NewDictionaryEntry {
                        term: phrase,
                        aliases: vec![],
                        enabled: true,
                    },
                )
                .map_err(|e| e.to_string())?;
                report.dictionary_imported += 1;
            }
        }
    }

    // ---- Polish → style representations (provenance preserved) ----
    let mut stmt = con
        .prepare(
            "SELECT p.polishedText, p.instruction, p.modelVersion, p.createdAt, \
                    h.transcriptEntityId \
             FROM Polish p JOIN History h ON h.transcriptEntityId = p.id \
             WHERE p.polishedText IS NOT NULL ORDER BY p.createdAt ASC",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let polished: Option<String> = row.get(0).unwrap_or(None);
        let instruction: Option<String> = row.get(1).unwrap_or(None);
        let model_version: Option<String> = row.get(2).unwrap_or(None);
        let created: Option<String> = row.get(3).unwrap_or(None);
        let entity_id: String = row.get(4).map_err(|e| e.to_string())?;

        let import_ref = format!("wispr:{entity_id}");
        let attempt_id: Option<String> = {
            let conn = target.conn();
            conn.query_row(
                "SELECT a.id FROM transcription_attempts a JOIN captures c ON c.id = a.capture_id \
                 WHERE c.import_ref = ?1 LIMIT 1",
                [&import_ref],
                |r| r.get(0),
            )
            .optional()
            .unwrap_or(None)
        };
        let Some(attempt_id) = attempt_id else {
            continue;
        };
        let Some(text) = polished else { continue };
        let created_ms = match parse_wispr_time(&created) {
            Ok(created_ms) => created_ms,
            Err(error) => {
                report.errors.push(format!("{entity_id}: polish {error}"));
                continue;
            }
        };
        let prompt = instruction.unwrap_or_default();
        let exists: i64 = target
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM representations\
                 WHERE attempt_id = ?1 AND processor = 'wispr_polish'\
                   AND text = ?2 AND created_at_ms = ?3\
                   AND COALESCE(effective_prompt_snapshot, '') = ?4\
                   AND COALESCE(provider_snapshot, '') = COALESCE(?5, '')",
                rusqlite::params![attempt_id, text, created_ms, prompt, model_version],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists > 0 {
            continue;
        }

        target
            .conn()
            .execute(
                "INSERT INTO representations(id, attempt_id, parent_representation_id, kind, text, \
                 processor, effective_prompt_snapshot, provider_snapshot, status, created_at_ms) \
                 VALUES (?1, ?2, NULL, 'style', ?3, 'wispr_polish', ?4, ?5, 'success', ?6)",
                rusqlite::params![
                    ids::new_id(),
                    attempt_id,
                    text,
                    prompt,
                    model_version,
                    created_ms
                ],
            )
            .map_err(|e| e.to_string())?;
        report.polish_imported += 1;
    }

    Ok(report)
}

/// Full import convenience for commands (reports counts).
pub fn run_import_reported(
    wispr_db: &Path,
    target: &AppDatabase,
    recordings_dir: Option<&Path>,
) -> Result<ImportReport, String> {
    run_import(wispr_db, target, recordings_dir)
}

/// Touch for ids import (kept minimal to satisfy lints).
#[allow(dead_code)]
fn _touch() -> String {
    ids::new_id()
}

#[cfg(test)]
mod tests {
    use super::parse_wispr_time;

    #[test]
    fn preserves_milliseconds_and_rejects_missing_time() {
        assert_eq!(
            parse_wispr_time(&Some("2026-08-27 18:22:20.079 +00:00".to_string())).unwrap(),
            1_787_854_940_079,
        );
        assert!(parse_wispr_time(&None).is_err());
        assert!(parse_wispr_time(&Some("not a timestamp".to_string())).is_err());
        assert_eq!(
            parse_wispr_time(&Some("2026-08-27".to_string())).unwrap(),
            1_787_788_800_000,
        );
    }
}
