//! Capture repository — the ONLY SQL owner for captures.

use rusqlite::{params, OptionalExtension};

use crate::storage::database::AppDatabase;
use crate::storage::ids;
use crate::storage::models::IntegrityState;

#[derive(Clone, Debug)]
pub struct NewCapture {
    pub audio_file_name: Option<String>,
    pub audio_sha256: Option<String>,
    pub audio_size_bytes: Option<i64>,
    pub title: String,
    pub source_app: Option<String>,
    pub integrity_state: IntegrityState,
}

#[derive(Clone, Debug)]
pub struct CaptureRecord {
    pub id: String,
    pub legacy_history_id: Option<i64>,
    pub audio_file_name: Option<String>,
    pub audio_sha256: Option<String>,
    pub audio_size_bytes: Option<i64>,
    pub title: String,
    pub source_app: Option<String>,
    pub saved: bool,
    pub integrity_state: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub deleted_at_ms: Option<i64>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

const COLUMNS: &str = "id, legacy_history_id, audio_file_name, audio_sha256, audio_size_bytes,
    title, source_app, saved, integrity_state, created_at_ms, updated_at_ms, deleted_at_ms";

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<CaptureRecord> {
    Ok(CaptureRecord {
        id: row.get("id")?,
        legacy_history_id: row.get("legacy_history_id")?,
        audio_file_name: row.get("audio_file_name")?,
        audio_sha256: row.get("audio_sha256")?,
        audio_size_bytes: row.get("audio_size_bytes")?,
        title: row.get("title")?,
        source_app: row.get("source_app")?,
        saved: row.get::<_, i64>("saved")? == 1,
        integrity_state: row.get("integrity_state")?,
        created_at_ms: row.get("created_at_ms")?,
        updated_at_ms: row.get("updated_at_ms")?,
        deleted_at_ms: row.get("deleted_at_ms")?,
    })
}

/// Insert a new active capture (never trashed at creation).
pub fn insert_capture(
    db: &AppDatabase,
    new: &NewCapture,
) -> Result<CaptureRecord, rusqlite::Error> {
    let id = ids::new_id();
    let ts = now_ms();
    {
        // Scope the connection guard: get_capture below re-locks.
        let conn = db.conn();
        conn.execute(
            "INSERT INTO captures(
                id, audio_file_name, audio_sha256, audio_size_bytes, title,
                source_app, saved, integrity_state, created_at_ms, updated_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?8)",
            params![
                id,
                new.audio_file_name,
                new.audio_sha256,
                new.audio_size_bytes,
                new.title,
                new.source_app,
                new.integrity_state.as_str(),
                ts
            ],
        )?;
    }
    get_capture(db, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get_capture(db: &AppDatabase, id: &str) -> Result<Option<CaptureRecord>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM captures WHERE id = ?1"),
        [id],
        row_to_record,
    )
    .optional()
}

/// Explicitly record the integrity verdict after lifecycle events.
pub fn set_integrity_state(
    db: &AppDatabase,
    capture_id: &str,
    state: IntegrityState,
) -> Result<(), rusqlite::Error> {
    let conn = db.conn();
    conn.execute(
        "UPDATE captures SET integrity_state = ?2, updated_at_ms = ?3 WHERE id = ?1",
        params![capture_id, state.as_str(), now_ms()],
    )?;
    Ok(())
}

/// Attach final audio facts after AUDIO-104 finalize.
#[allow(clippy::too_many_arguments)]
pub fn attach_audio(
    db: &AppDatabase,
    capture_id: &str,
    file_name: &str,
    sha256: &str,
    size_bytes: i64,
) -> Result<(), rusqlite::Error> {
    let conn = db.conn();
    conn.execute(
        "UPDATE captures SET audio_file_name = ?2, audio_sha256 = ?3,
            audio_size_bytes = ?4, integrity_state = 'audio_valid', updated_at_ms = ?5
         WHERE id = ?1",
        params![capture_id, file_name, sha256, size_bytes, now_ms()],
    )?;
    Ok(())
}

pub fn list_by_integrity_state(
    db: &AppDatabase,
    state: IntegrityState,
) -> Result<Vec<CaptureRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM captures WHERE integrity_state = ?1 ORDER BY created_at_ms DESC"
    ))?;
    let rows = stmt.query_map([state.as_str()], row_to_record)?;
    rows.collect()
}

/// Final WAV names already referenced by captures (for recovery scans).
pub fn referenced_audio_names(db: &AppDatabase) -> Result<Vec<String>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT DISTINCT audio_file_name FROM captures WHERE audio_file_name IS NOT NULL",
    )?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    rows.collect()
}

/// Deterministic, bounded page over ACTIVE (non-trashed) captures,
/// newest first. Ordering contract: (created_at_ms DESC, id DESC).
pub fn list_active_captures(
    db: &AppDatabase,
    limit: i64,
    offset: i64,
) -> Result<Vec<CaptureRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM captures WHERE deleted_at_ms IS NULL \
         ORDER BY created_at_ms DESC, id DESC LIMIT ?1 OFFSET ?2"
    ))?;
    let rows = stmt.query_map(params![limit.clamp(1, 200), offset.max(0)], row_to_record)?;
    rows.collect()
}

/// Full canonical view of one capture for history/detail surfaces.
pub struct CaptureDetail {
    pub capture: CaptureRecord,
    pub attempts: Vec<crate::storage::repositories::transcriptions::AttemptRecord>,
    pub representations: Vec<crate::storage::repositories::representations::RepresentationRecord>,
}

/// Canonical attempt + derived representations for one capture, in a
/// single transaction-consistent read. `None` when the capture is unknown.
pub fn capture_detail(
    db: &AppDatabase,
    capture_id: &str,
) -> Result<Option<CaptureDetail>, rusqlite::Error> {
    use crate::storage::repositories::representations as reps;
    use crate::storage::repositories::transcriptions as att;

    let Some(capture) = get_capture(db, capture_id)? else {
        return Ok(None);
    };
    let attempts = att::attempts_for_capture(db, capture_id)?;
    let mut all_reps = Vec::new();
    for a in &attempts {
        all_reps.extend(reps::representations_for_attempt(db, &a.id)?);
    }
    Ok(Some(CaptureDetail {
        capture,
        attempts,
        representations: all_reps,
    }))
}

// ---- HIST-109: Trash / Restore / explicit Purge -----------------------

/// Move to trash (soft delete). Returns false when already trashed/unknown.
pub fn trash_capture(db: &AppDatabase, capture_id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let changed = conn.execute(
        "UPDATE captures SET deleted_at_ms = ?2, updated_at_ms = ?2 \
         WHERE id = ?1 AND deleted_at_ms IS NULL",
        params![capture_id, now_ms()],
    )?;
    Ok(changed == 1)
}

/// Restore from trash. Returns false when not trashed/unknown.
pub fn restore_capture(db: &AppDatabase, capture_id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let changed = conn.execute(
        "UPDATE captures SET deleted_at_ms = NULL, updated_at_ms = ?2 \
         WHERE id = ?1 AND deleted_at_ms IS NOT NULL",
        params![capture_id, now_ms()],
    )?;
    Ok(changed == 1)
}

#[derive(Clone, Debug)]
pub struct PurgedCapture {
    pub id: String,
    pub audio_file_name: Option<String>,
    pub legacy_history_id: Option<i64>,
}

/// List ALL trashed captures (purge candidates UI + tests).
pub fn list_trashed(db: &AppDatabase) -> Result<Vec<CaptureRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM captures WHERE deleted_at_ms IS NOT NULL \
         ORDER BY deleted_at_ms DESC"
    ))?;
    let rows = stmt.query_map([], row_to_record)?;
    rows.collect()
}

/// Explicit purge of ONE trashed capture: removes canonical rows in
/// FK-safe order inside ONE transaction and returns the audio file name
/// so the CALLER deletes the file afterwards (repository owns SQL only;
/// filesystem stays outside). Refuses non-trashed captures.
pub fn purge_trashed(db: &AppDatabase, capture_id: &str) -> Result<PurgedCapture, rusqlite::Error> {
    let mut conn = db.conn();
    let tx = conn.transaction()?;
    let (audio, legacy, trashed): (Option<String>, Option<i64>, Option<i64>) = tx.query_row(
        "SELECT audio_file_name, legacy_history_id, deleted_at_ms FROM captures WHERE id = ?1",
        [capture_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let Some(_deleted_at) = trashed else {
        return Err(rusqlite::Error::InvalidQuery);
    };
    // Children first (FK order); delivery events reference attempts too.
    tx.execute(
        "DELETE FROM delivery_events WHERE source_attempt_id IN \
         (SELECT id FROM transcription_attempts WHERE capture_id = ?1)",
        [capture_id],
    )?;
    tx.execute(
        "DELETE FROM representations WHERE attempt_id IN \
         (SELECT id FROM transcription_attempts WHERE capture_id = ?1)",
        [capture_id],
    )?;
    tx.execute(
        "DELETE FROM transcription_attempts WHERE capture_id = ?1",
        [capture_id],
    )?;
    tx.execute("DELETE FROM captures WHERE id = ?1", [capture_id])?;
    tx.commit()?;
    Ok(PurgedCapture {
        id: capture_id.to_string(),
        audio_file_name: audio,
        legacy_history_id: legacy,
    })
}
