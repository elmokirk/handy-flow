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

#[derive(Clone, Debug)]
pub struct ChunkRecord {
    pub chunk_index: i64,
    pub start_sample: i64,
    pub end_sample: i64,
    pub window_samples: i64,
    pub payload_json: String,
    pub seam_uncertain: bool,
    pub seam_left: Option<String>,
    pub seam_right: Option<String>,
}

#[derive(Clone, Debug)]
pub struct IncompleteJob {
    pub attempt_id: String,
    pub capture_id: String,
    pub audio_file_name: String,
    pub model_id: Option<String>,
    pub audio_duration_ms: Option<i64>,
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
    complete_attempt_with_raw(
        db,
        attempt_id,
        None,
        normalized_stt,
        error,
        model_id,
        language,
    )
}

pub fn complete_attempt_with_raw(
    db: &AppDatabase,
    attempt_id: &str,
    engine_raw: Option<&str>,
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
            status = ?2, engine_raw = COALESCE(?3, engine_raw),
            normalized_stt = COALESCE(?4, normalized_stt),
            error = ?5, model_id = COALESCE(?6, model_id), language = COALESCE(?7, language),
            completed_at_ms = ?8
         WHERE id = ?1 AND status IN ('pending','running')",
        params![
            attempt_id,
            new_status,
            engine_raw,
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

/// Final text and canonical selection become visible together. A crash
/// between separate writes must not leave a successful invisible attempt.
pub fn complete_and_promote(
    db: &AppDatabase,
    capture_id: &str,
    attempt_id: &str,
    engine_raw: &str,
    normalized_stt: &str,
    model_id: Option<&str>,
    language: Option<&str>,
) -> Result<(), rusqlite::Error> {
    let mut conn = db.conn();
    let tx = conn.transaction()?;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();
    let changed = tx.execute(
        "UPDATE transcription_attempts SET status = 'success', engine_raw = ?3,
         normalized_stt = ?4, model_id = COALESCE(?5, model_id),
         language = COALESCE(?6, language), error = NULL, completed_at_ms = ?7
         WHERE id = ?1 AND capture_id = ?2 AND status IN ('pending', 'running')
           AND EXISTS (SELECT 1 FROM captures WHERE id = ?2 AND deleted_at_ms IS NULL)",
        params![
            attempt_id,
            capture_id,
            engine_raw,
            normalized_stt,
            model_id,
            language,
            now_ms
        ],
    )?;
    if changed != 1 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    tx.execute(
        "UPDATE transcription_attempts SET is_canonical = 0
         WHERE capture_id = ?1 AND is_canonical = 1",
        [capture_id],
    )?;
    tx.execute(
        "UPDATE transcription_attempts SET is_canonical = 1 WHERE id = ?1",
        [attempt_id],
    )?;
    tx.execute(
        "DELETE FROM transcription_chunks WHERE attempt_id = ?1 AND seam_uncertain = 0",
        [attempt_id],
    )?;
    tx.execute(
        "UPDATE transcription_chunks SET payload_json = '' WHERE attempt_id = ?1 AND seam_uncertain = 1",
        [attempt_id],
    )?;
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

pub fn incomplete_jobs(db: &AppDatabase) -> Result<Vec<IncompleteJob>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT a.id, c.id, c.audio_file_name, a.model_id, c.audio_duration_ms
         FROM transcription_attempts a JOIN captures c ON c.id = a.capture_id
         WHERE a.status IN ('pending', 'running') AND c.deleted_at_ms IS NULL
           AND c.audio_file_name IS NOT NULL
           AND c.integrity_state IN ('audio_valid', 'recovered_orphan')
         ORDER BY c.created_at_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(IncompleteJob {
            attempt_id: row.get(0)?,
            capture_id: row.get(1)?,
            audio_file_name: row.get(2)?,
            model_id: row.get(3)?,
            audio_duration_ms: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn mark_running(db: &AppDatabase, attempt_id: &str) -> Result<(), rusqlite::Error> {
    db.conn().execute(
        "UPDATE transcription_attempts SET status = 'running'
         WHERE id = ?1 AND status IN ('pending', 'running')",
        [attempt_id],
    )?;
    Ok(())
}

pub fn chunks_for_attempt(
    db: &AppDatabase,
    attempt_id: &str,
) -> Result<Vec<ChunkRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT chunk_index, start_sample, end_sample, window_samples, payload_json, seam_uncertain, seam_left, seam_right
         FROM transcription_chunks WHERE attempt_id = ?1 ORDER BY chunk_index",
    )?;
    let rows = stmt.query_map([attempt_id], |row| {
        Ok(ChunkRecord {
            chunk_index: row.get(0)?,
            start_sample: row.get(1)?,
            end_sample: row.get(2)?,
            window_samples: row.get(3)?,
            payload_json: row.get(4)?,
            seam_uncertain: row.get::<_, i64>(5)? != 0,
            seam_left: row.get(6)?,
            seam_right: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn insert_chunk(
    db: &AppDatabase,
    attempt_id: &str,
    chunk: &ChunkRecord,
) -> Result<(), rusqlite::Error> {
    db.conn().execute(
        "INSERT INTO transcription_chunks
         (attempt_id, chunk_index, start_sample, end_sample, window_samples, payload_json, seam_uncertain, seam_left, seam_right)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![attempt_id, chunk.chunk_index, chunk.start_sample, chunk.end_sample,
            chunk.window_samples, chunk.payload_json, i64::from(chunk.seam_uncertain), chunk.seam_left, chunk.seam_right],
    )?;
    Ok(())
}

/// Startup repair: valid-audio attempts are resumed by the background queue;
/// only jobs with no usable audio become terminal failures.
pub fn fail_interrupted_attempts(db: &AppDatabase) -> Result<usize, rusqlite::Error> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();
    db.conn().execute(
        "UPDATE transcription_attempts SET status = 'failed',
         error = 'Transcription interrupted by app shutdown', completed_at_ms = ?1
         WHERE status IN ('pending', 'running')
           AND NOT EXISTS (SELECT 1 FROM captures c
                           WHERE c.id = transcription_attempts.capture_id
                             AND c.integrity_state IN ('audio_valid', 'recovered_orphan'))",
        [now_ms],
    )
}
