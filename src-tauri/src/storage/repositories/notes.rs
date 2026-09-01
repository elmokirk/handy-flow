//! Note repository — the ONLY SQL owner for notes/note_versions
//! (NOTE-301). Append-only versions; current = highest version_no.

use rusqlite::{params, OptionalExtension};

use crate::storage::database::AppDatabase;
use crate::storage::ids;

pub const SOURCE_DICTATION: &str = "dictation";
pub const SOURCE_MANUAL_EDIT: &str = "manual_edit";
pub const SOURCE_TRANSFORM: &str = "transform";
pub const SOURCE_RESTORE: &str = "restore";

#[derive(Clone, Debug)]
pub struct NewNote {
    pub title: String,
    pub first_version_content: String,
    pub source: &'static str,
}

#[derive(Clone, Debug)]
pub struct NoteRecord {
    pub id: String,
    pub title: String,
    pub pinned: bool,
    pub deleted_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug)]
pub struct NoteVersionRecord {
    pub id: String,
    pub note_id: String,
    pub version_no: i64,
    pub content: String,
    pub content_hash_sha256: String,
    pub source: String,
    pub created_at_ms: i64,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

pub fn content_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

const NOTE_COLS: &str = "id, title, pinned, deleted_at_ms, created_at_ms, updated_at_ms";
const VER_COLS: &str =
    "id, note_id, version_no, content, content_hash_sha256, source, created_at_ms";

fn row_to_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteRecord> {
    Ok(NoteRecord {
        id: row.get("id")?,
        title: row.get("title")?,
        pinned: row.get::<_, i64>("pinned")? == 1,
        deleted_at_ms: row.get("deleted_at_ms")?,
        created_at_ms: row.get("created_at_ms")?,
        updated_at_ms: row.get("updated_at_ms")?,
    })
}

fn row_to_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteVersionRecord> {
    Ok(NoteVersionRecord {
        id: row.get("id")?,
        note_id: row.get("note_id")?,
        version_no: row.get("version_no")?,
        content: row.get("content")?,
        content_hash_sha256: row.get("content_hash_sha256")?,
        source: row.get("source")?,
        created_at_ms: row.get("created_at_ms")?,
    })
}

/// Create a note with its first version. Returns (note, version).
pub fn create_note(
    db: &AppDatabase,
    new: &NewNote,
) -> Result<(NoteRecord, NoteVersionRecord), rusqlite::Error> {
    let ver_id: String;
    let note_id: String;
    {
        // Scope the connection guard: get_note re-locks after commit.
        let mut conn = db.conn();
        let tx = conn.transaction()?;
        note_id = ids::new_id();
        let ts = now_ms();
        conn2_insert_note(&tx, &note_id, &new.title, ts)?;
        ver_id =
            conn2_insert_version(&tx, &note_id, 1, &new.first_version_content, new.source, ts)?;
        tx.commit()?;
    }
    let note = get_note(db, &note_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let ver = tx_get_version_after_commit(db, &ver_id)?;
    Ok((note, ver))
}

fn tx_get_version_after_commit(
    db: &AppDatabase,
    ver_id: &str,
) -> Result<NoteVersionRecord, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!("SELECT {VER_COLS} FROM note_versions WHERE id = ?1"),
        [ver_id],
        row_to_version,
    )
}

fn conn2_insert_note(
    tx: &rusqlite::Transaction<'_>,
    id: &str,
    title: &str,
    ts: i64,
) -> Result<(), rusqlite::Error> {
    tx.execute(
        "INSERT INTO notes(id, title, pinned, created_at_ms, updated_at_ms) \
         VALUES (?1, ?2, 0, ?3, ?3)",
        params![id, title, ts],
    )?;
    Ok(())
}

fn conn2_insert_version(
    tx: &rusqlite::Transaction<'_>,
    note_id: &str,
    version_no: i64,
    content: &str,
    source: &str,
    ts: i64,
) -> Result<String, rusqlite::Error> {
    let ver_id = ids::new_id();
    tx.execute(
        "INSERT INTO note_versions(id, note_id, version_no, content, content_hash_sha256, source, created_at_ms) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![ver_id, note_id, version_no, content, content_hash(content), source, ts],
    )?;
    Ok(ver_id)
}

pub fn get_note(db: &AppDatabase, id: &str) -> Result<Option<NoteRecord>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!("SELECT {NOTE_COLS} FROM notes WHERE id = ?1"),
        [id],
        row_to_note,
    )
    .optional()
}

/// Append a new version if content differs from the latest (content-hash
/// dedupe). Returns None when identical to the current version.
pub fn append_version(
    db: &AppDatabase,
    note_id: &str,
    content: &str,
    source: &'static str,
) -> Result<Option<NoteVersionRecord>, rusqlite::Error> {
    let mut conn = db.conn();
    let tx = conn.transaction()?;
    let next_no: i64 = tx.query_row(
        "SELECT COALESCE(MAX(version_no), 0) + 1 FROM note_versions WHERE note_id = ?1",
        [note_id],
        |r| r.get(0),
    )?;
    let current_hash: Option<String> = tx
        .query_row(
            "SELECT content_hash_sha256 FROM note_versions \
             WHERE note_id = ?1 ORDER BY version_no DESC LIMIT 1",
            [note_id],
            |r| r.get(0),
        )
        .optional()?;
    let hash = content_hash(content);
    if current_hash.as_deref() == Some(hash.as_str()) {
        return Ok(None); // dedupe: unchanged content creates no version
    }
    let ts = now_ms();
    let ver_id = conn2_insert_version(&tx, note_id, next_no, content, source, ts)?;
    tx.execute(
        "UPDATE notes SET updated_at_ms = ?2 WHERE id = ?1",
        params![note_id, ts],
    )?;
    let created = tx.query_row(
        &format!("SELECT {VER_COLS} FROM note_versions WHERE id = ?1"),
        [&ver_id],
        row_to_version,
    )?;
    tx.commit()?;
    Ok(Some(created))
}

/// Active (non-trashed) notes, pinned first then newest updated.
pub fn list_active_notes(db: &AppDatabase) -> Result<Vec<NoteRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {NOTE_COLS} FROM notes WHERE deleted_at_ms IS NULL \
         ORDER BY pinned DESC, updated_at_ms DESC"
    ))?;
    let rows = stmt.query_map([], row_to_note)?;
    rows.collect()
}

pub fn note_versions(
    db: &AppDatabase,
    note_id: &str,
) -> Result<Vec<NoteVersionRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {VER_COLS} FROM note_versions WHERE note_id = ?1 ORDER BY version_no DESC"
    ))?;
    let rows = stmt.query_map([note_id], row_to_version)?;
    rows.collect()
}

/// Current content = highest version (derived, never stored twice).
pub fn current_version(
    db: &AppDatabase,
    note_id: &str,
) -> Result<Option<NoteVersionRecord>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!(
            "SELECT {VER_COLS} FROM note_versions WHERE note_id = ?1 \
             ORDER BY version_no DESC LIMIT 1"
        ),
        [note_id],
        row_to_version,
    )
    .optional()
}

pub fn set_pinned(db: &AppDatabase, note_id: &str, pinned: bool) -> Result<(), rusqlite::Error> {
    let conn = db.conn();
    conn.execute(
        "UPDATE notes SET pinned = ?2, updated_at_ms = ?3 WHERE id = ?1",
        params![note_id, i64::from(pinned), now_ms()],
    )?;
    Ok(())
}

pub fn set_title(db: &AppDatabase, note_id: &str, title: &str) -> Result<(), rusqlite::Error> {
    let conn = db.conn();
    conn.execute(
        "UPDATE notes SET title = ?2, updated_at_ms = ?3 WHERE id = ?1",
        params![note_id, title, now_ms()],
    )?;
    Ok(())
}

/// Trash / restore per DATA_STATE_MACHINES (no purge here).
pub fn trash_note(db: &AppDatabase, note_id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    Ok(conn.execute(
        "UPDATE notes SET deleted_at_ms = ?2, updated_at_ms = ?2 \
         WHERE id = ?1 AND deleted_at_ms IS NULL",
        params![note_id, now_ms()],
    )? == 1)
}

pub fn restore_note(db: &AppDatabase, note_id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    Ok(conn.execute(
        "UPDATE notes SET deleted_at_ms = NULL, updated_at_ms = ?2 \
         WHERE id = ?1 AND deleted_at_ms IS NOT NULL",
        params![note_id, now_ms()],
    )? == 1)
}
