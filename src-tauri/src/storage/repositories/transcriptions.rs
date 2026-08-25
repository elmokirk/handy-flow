//! Attempt repository — the ONLY SQL owner for transcription_attempts.
//!
//! Contract (DATA-105 / planning/04):
//! - append-only: a retry inserts a NEW attempt row; nothing is ever
//!   overwritten;
//! - terminal payloads are immutable: pending→running→(success|failed)
//!   transitions happen at most once per row;
//! - at most ONE canonical attempt per capture, enforced by the partial
//!   unique index plus [`mark_canonical`] transactional swap.

use rusqlite::{params, OptionalExtension};

use crate::storage::database::AppDatabase;
use crate::storage::ids;

/// Where an attempt came from (mirrors storage::models for repo-local use).
pub const PROVENANCE_LIVE: &str = "live";

#[derive(Clone, Debug)]
pub struct NewAttempt {
    pub capture_id: String,
    pub engine_raw: Option<String>,
    pub normalized_stt: Option<String>,
    pub model_id: Option<String>,
    pub language: Option<String>,
    pub normalizer_version: String,
    pub dictionary_snapshot_sha256: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AttemptRecord {
    pub id: String,
    pub capture_id: String,
    pub attempt_number: i64,
    pub engine_raw: Option<String>,
    pub normalized_stt: Option<String>,
    pub model_id: Option<String>,
    pub language: Option<String>,
    pub normalizer_version: Option<String>,
    pub dictionary_snapshot_sha256: Option<String>,
    pub provenance: String,
    pub status: String,
    pub error: Option<String>,
    pub is_canonical: bool,
    pub created_at_ms: i64,
    pub completed_at_ms: Option<i64>,
}

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<AttemptRecord> {
    Ok(AttemptRecord {
        id: row.get("id")?,
        capture_id: row.get("capture_id")?,
        attempt_number: row.get("attempt_number")?,
        engine_raw: row.get("engine_raw")?,
        normalized_stt: row.get("normalized_stt")?,
        model_id: row.get("model_id")?,
        language: row.get("language")?,
        normalizer_version: row.get("normalizer_version")?,
        dictionary_snapshot_sha256: row.get("dictionary_snapshot_sha256")?,
        provenance: row.get("provenance")?,
        status: row.get("status")?,
        error: row.get("error")?,
        is_canonical: row.get::<_, i64>("is_canonical")? == 1,
        created_at_ms: row.get("created_at_ms")?,
        completed_at_ms: row.get("completed_at_ms")?,
    })
}

const ATTEMPT_COLUMNS: &str = "id, capture_id, attempt_number, engine_raw, normalized_stt,
    model_id, language, normalizer_version, dictionary_snapshot_sha256,
    provenance, status, error, is_canonical, created_at_ms, completed_at_ms";

/// Insert the next attempt for a capture. The attempt number is derived
/// inside the same transaction so concurrent writers cannot collide.
/// A first successful attempt automatically becomes canonical when the
/// capture has none yet.
pub fn insert_attempt(
    db: &AppDatabase,
    new: &NewAttempt,
) -> Result<AttemptRecord, rusqlite::Error> {
    let mut conn = db.conn();
    let tx = conn.transaction()?;

    let number: i64 = tx.query_row(
        "SELECT COALESCE(MAX(attempt_number), 0) + 1 FROM transcription_attempts \
             WHERE capture_id = ?1",
        [&new.capture_id],
        |r| r.get(0),
    )?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();

    let has_canonical: i64 = tx.query_row(
        "SELECT COUNT(*) FROM transcription_attempts WHERE capture_id = ?1 AND is_canonical = 1",
        [&new.capture_id],
        |r| r.get(0),
    )?;
    let is_success = new.normalized_stt.is_some();
    let make_canonical = is_success && has_canonical == 0;

    let id = ids::new_id();
    let initial_status = if new.normalized_stt.is_some() {
        "success"
    } else {
        "pending"
    };

    tx.execute(
        "INSERT INTO transcription_attempts(
            id, capture_id, attempt_number, engine_raw, normalized_stt,
            model_id, language, normalizer_version, dictionary_snapshot_sha256,
            provenance, status, is_canonical, created_at_ms, completed_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'live', ?10, ?11, ?12, ?13)",
        params![
            id,
            new.capture_id,
            number,
            new.engine_raw,
            new.normalized_stt,
            new.model_id,
            new.language,
            new.normalizer_version,
            new.dictionary_snapshot_sha256,
            initial_status,
            i64::from(make_canonical),
            now_ms,
            if new.normalized_stt.is_some() {
                Some(now_ms)
            } else {
                None
            },
        ],
    )?;

    tx.commit()?;
    Ok(AttemptRecord {
        id,
        capture_id: new.capture_id.clone(),
        attempt_number: number,
        engine_raw: new.engine_raw.clone(),
        normalized_stt: new.normalized_stt.clone(),
        model_id: new.model_id.clone(),
        language: new.language.clone(),
        normalizer_version: Some(new.normalizer_version.clone()),
        dictionary_snapshot_sha256: new.dictionary_snapshot_sha256.clone(),
        provenance: PROVENANCE_LIVE.into(),
        status: initial_status.into(),
        error: None,
        is_canonical: make_canonical,
        created_at_ms: now_ms,
        completed_at_ms: if new.normalized_stt.is_some() {
            Some(now_ms)
        } else {
            None
        },
    })
}

/// Transition a pending/running attempt to a terminal state exactly once.
/// Returns the immutable stored values; attempting to overwrite an
/// already-terminal attempt is rejected (append-only contract).
pub fn complete_attempt(
    db: &AppDatabase,
    attempt_id: &str,
    normalized_stt: Option<&str>,
    error: Option<&str>,
    model_id: Option<&str>,
    language: Option<&str>,
) -> Result<(), super::super::database::StorageError> {
    use super::super::database::StorageError;
    let conn = db.conn();
    let current: Option<String> = conn
        .query_row(
            "SELECT status FROM transcription_attempts WHERE id = ?1",
            [attempt_id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(status) = current else {
        return Err(StorageError::CorruptData(format!(
            "attempt {attempt_id} does not exist"
        )));
    };
    if status == "success" || status == "failed" {
        return Err(StorageError::CorruptData(format!(
            "attempt {attempt_id} is already terminal ({status}); retries must insert new attempts"
        )));
    }

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();
    let new_status = if normalized_stt.is_some() {
        "success"
    } else {
        "failed"
    };

    let changed = conn.execute(
        "UPDATE transcription_attempts SET
            status = ?2, normalized_stt = COALESCE(?3, normalized_stt),
            error = ?4, model_id = COALESCE(?5, model_id), language = COALESCE(?6, language),
            completed_at_ms = ?7
         WHERE id = ?1 AND status IN ('pending','running')",
        params![
            attempt_id,
            new_status,
            normalized_stt,
            error,
            model_id,
            language,
            now_ms
        ],
    )?;
    if changed == 0 {
        return Err(StorageError::CorruptData(format!(
            "attempt {attempt_id} transition raced and was refused"
        )));
    }
    Ok(())
}

/// Promote one attempt to canonical, demoting the previous holder in the
/// SAME transaction so the partial unique index is never violated mid-flight.
pub fn mark_canonical(
    db: &AppDatabase,
    capture_id: &str,
    attempt_id: &str,
) -> Result<(), rusqlite::Error> {
    let mut conn = db.conn();
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE transcription_attempts SET is_canonical = 0 \
         WHERE capture_id = ?1 AND is_canonical = 1",
        [capture_id],
    )?;
    let changed = tx.execute(
        "UPDATE transcription_attempts SET is_canonical = 1 \
         WHERE id = ?1 AND capture_id = ?2 AND status = 'success'",
        [attempt_id, capture_id],
    )?;
    if changed != 1 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    tx.commit()
}

pub fn canonical_attempt(
    db: &AppDatabase,
    capture_id: &str,
) -> Result<Option<AttemptRecord>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!(
            "SELECT {ATTEMPT_COLUMNS} FROM transcription_attempts \
             WHERE capture_id = ?1 AND is_canonical = 1"
        ),
        [capture_id],
        row_to_record,
    )
    .optional()
}

pub fn attempts_for_capture(
    db: &AppDatabase,
    capture_id: &str,
) -> Result<Vec<AttemptRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {ATTEMPT_COLUMNS} FROM transcription_attempts \
         WHERE capture_id = ?1 ORDER BY attempt_number ASC"
    ))?;
    let rows = stmt.query_map([capture_id], row_to_record)?;
    rows.collect()
}
