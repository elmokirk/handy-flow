//! Snippet repository — the ONLY SQL owner for snippets (SNIP-211).

use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use specta::Type;

use crate::storage::database::AppDatabase;
use crate::storage::ids;

#[derive(Clone, Debug)]
pub struct NewSnippet {
    pub trigger: String,
    pub replacement: String,
    pub priority: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Type)]
pub struct SnippetRecord {
    pub id: String,
    pub trigger: String,
    pub replacement: String,
    pub priority: i64,
    pub enabled: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

const COLUMNS: &str = "id, trigger, replacement, priority, enabled, created_at_ms, updated_at_ms";

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<SnippetRecord> {
    Ok(SnippetRecord {
        id: row.get("id")?,
        trigger: row.get("trigger")?,
        replacement: row.get("replacement")?,
        priority: row.get("priority")?,
        enabled: row.get::<_, i64>("enabled")? == 1,
        created_at_ms: row.get("created_at_ms")?,
        updated_at_ms: row.get("updated_at_ms")?,
    })
}

/// Insert or update by trigger (case-insensitive unique).
pub fn upsert_snippet(
    db: &AppDatabase,
    new: &NewSnippet,
) -> Result<SnippetRecord, rusqlite::Error> {
    let ts = now_ms();
    let id = {
        let conn = db.conn();
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM snippets WHERE trigger = ?1 COLLATE NOCASE",
                [&new.trigger],
                |r| r.get(0),
            )
            .optional()?;
        match existing {
            Some(id) => {
                conn.execute(
                    "UPDATE snippets SET replacement = ?2, priority = ?3, enabled = ?4, updated_at_ms = ?5 \
                     WHERE id = ?1",
                    params![id, new.replacement, new.priority, i64::from(new.enabled), ts],
                )?;
                id
            }
            None => {
                let id = ids::new_id();
                conn.execute(
                    "INSERT INTO snippets(id, trigger, replacement, priority, enabled, created_at_ms, updated_at_ms) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                    params![id, new.trigger, new.replacement, new.priority, i64::from(new.enabled), ts],
                )?;
                id
            }
        }
    };
    get_snippet(db, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get_snippet(db: &AppDatabase, id: &str) -> Result<Option<SnippetRecord>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM snippets WHERE id = ?1"),
        [id],
        row_to_record,
    )
    .optional()
}

/// Enabled snippets in matcher tie-break order:
/// (priority DESC, trigger length DESC, id ASC). ADR-020.
pub fn list_enabled_for_match(db: &AppDatabase) -> Result<Vec<SnippetRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM snippets WHERE enabled = 1 \
         ORDER BY priority DESC, LENGTH(trigger) DESC, id ASC"
    ))?;
    let rows = stmt.query_map([], row_to_record)?;
    rows.collect()
}

pub fn list_all(db: &AppDatabase) -> Result<Vec<SnippetRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM snippets ORDER BY trigger COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map([], row_to_record)?;
    rows.collect()
}

pub fn delete_snippet(db: &AppDatabase, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    Ok(conn.execute("DELETE FROM snippets WHERE id = ?1", [id])? == 1)
}
