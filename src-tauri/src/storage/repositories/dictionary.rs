//! Dictionary repository — the ONLY SQL owner for dictionary_entries.
//!
//! Contract (DICT-201 / planning/04): durable entries with explicit
//! enabled state and aliases. Dictionary correction is part of the
//! deterministic normalization pipeline; historical attempts are never
//! rewritten after edits (snapshot hash recorded per attempt).

use rusqlite::{params, OptionalExtension};

use crate::storage::database::AppDatabase;
use crate::storage::ids;

#[derive(Clone, Debug)]
pub struct NewDictionaryEntry {
    pub term: String,
    pub aliases: Vec<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct DictionaryEntry {
    pub id: String,
    pub term: String,
    pub aliases: Vec<String>,
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

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<DictionaryEntry> {
    let aliases_json: String = row.get("aliases")?;
    let aliases = serde_json::from_str(&aliases_json).unwrap_or_default();
    Ok(DictionaryEntry {
        id: row.get("id")?,
        term: row.get("term")?,
        aliases,
        enabled: row.get::<_, i64>("enabled")? == 1,
        created_at_ms: row.get("created_at_ms")?,
        updated_at_ms: row.get("updated_at_ms")?,
    })
}

const COLUMNS: &str = "id, term, aliases, enabled, created_at_ms, updated_at_ms";

/// Insert or update by term (case-insensitive unique). Returns the row.
pub fn upsert_entry(
    db: &AppDatabase,
    new: &NewDictionaryEntry,
) -> Result<DictionaryEntry, rusqlite::Error> {
    let ts = now_ms();
    let aliases_json = serde_json::to_string(&new.aliases).unwrap_or_else(|_| "[]".into());
    let id = {
        // Scope the connection guard: get_entry below re-locks.
        let conn = db.conn();
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM dictionary_entries WHERE term = ?1 COLLATE NOCASE",
                [&new.term],
                |r| r.get(0),
            )
            .optional()?;

        match existing {
            Some(id) => {
                conn.execute(
                    "UPDATE dictionary_entries SET aliases = ?2, enabled = ?3, updated_at_ms = ?4 \
                     WHERE id = ?1",
                    params![id, aliases_json, i64::from(new.enabled), ts],
                )?;
                id
            }
            None => {
                let id = ids::new_id();
                conn.execute(
                    "INSERT INTO dictionary_entries(id, term, aliases, enabled, created_at_ms, updated_at_ms) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                    params![id, new.term, aliases_json, i64::from(new.enabled), ts],
                )?;
                id
            }
        }
    };
    get_entry(db, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get_entry(db: &AppDatabase, id: &str) -> Result<Option<DictionaryEntry>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM dictionary_entries WHERE id = ?1"),
        [id],
        row_to_entry,
    )
    .optional()
}

/// All ENABLED entries — the deterministic correction input set.
pub fn list_enabled(db: &AppDatabase) -> Result<Vec<DictionaryEntry>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM dictionary_entries WHERE enabled = 1 ORDER BY term COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map([], row_to_entry)?;
    rows.collect()
}

/// All entries incl. disabled (settings UI).
pub fn list_all(db: &AppDatabase) -> Result<Vec<DictionaryEntry>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM dictionary_entries ORDER BY term COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map([], row_to_entry)?;
    rows.collect()
}

/// Delete one entry (explicit user action; no cascade needed).
pub fn delete_entry(db: &AppDatabase, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    Ok(conn.execute("DELETE FROM dictionary_entries WHERE id = ?1", [id])? == 1)
}

/// Flattened correction terms: term + aliases of enabled entries.
/// This is the exact input shape `apply_custom_words` consumes.
pub fn enabled_correction_terms(db: &AppDatabase) -> Result<Vec<String>, rusqlite::Error> {
    let entries = list_enabled(db)?;
    let mut terms = Vec::with_capacity(entries.len() * 2);
    for e in entries {
        terms.push(e.term);
        terms.extend(e.aliases);
    }
    Ok(terms)
}

/// Deterministic snapshot hash over the enabled correction set — stored
/// per attempt so historical outputs stay interpretable after edits.
pub fn enabled_snapshot_sha256(db: &AppDatabase) -> Result<String, rusqlite::Error> {
    use sha2::{Digest, Sha256};
    let mut terms = enabled_correction_terms(db)?;
    terms.sort();
    let mut hasher = Sha256::new();
    for t in terms {
        hasher.update(t.as_bytes());
        hasher.update([0u8]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
