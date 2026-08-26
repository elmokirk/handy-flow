//! Prompt profile repository — the ONLY SQL owner for prompt_profiles
//! (PROMPT-221). One domain, two kinds: `style` and `transform`.

use rusqlite::{params, OptionalExtension};

use crate::storage::database::AppDatabase;
use crate::storage::ids;

#[derive(Clone, Debug)]
pub struct NewPromptProfile {
    pub name: String,
    /// `style` or `transform` (DB CHECK enforces).
    pub kind: String,
    pub system_prompt: String,
    /// Template for the user message; `{text}` is the transcript slot.
    pub user_template: String,
    /// Reference into the EXISTING settings provider configuration —
    /// profiles never define their own endpoints or keys.
    pub provider_id: String,
    pub model: String,
    pub temperature: Option<f64>,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct PromptProfileRecord {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub system_prompt: String,
    pub user_template: String,
    pub provider_id: String,
    pub model: String,
    pub temperature: Option<f64>,
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

const COLUMNS: &str = "id, name, kind, system_prompt, user_template, provider_id, model, \
     temperature, enabled, created_at_ms, updated_at_ms";

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptProfileRecord> {
    Ok(PromptProfileRecord {
        id: row.get("id")?,
        name: row.get("name")?,
        kind: row.get("kind")?,
        system_prompt: row.get("system_prompt")?,
        user_template: row.get("user_template")?,
        provider_id: row.get("provider_id")?,
        model: row.get("model")?,
        temperature: row.get("temperature")?,
        enabled: row.get::<_, i64>("enabled")? == 1,
        created_at_ms: row.get("created_at_ms")?,
        updated_at_ms: row.get("updated_at_ms")?,
    })
}

/// Insert or update by (name, kind). Returns the stored row.
pub fn upsert_profile(
    db: &AppDatabase,
    new: &NewPromptProfile,
) -> Result<PromptProfileRecord, rusqlite::Error> {
    let ts = now_ms();
    let id = {
        let conn = db.conn();
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_profiles WHERE name = ?1 AND kind = ?2",
                [&new.name, &new.kind],
                |r| r.get(0),
            )
            .optional()?;
        match existing {
            Some(id) => {
                conn.execute(
                    "UPDATE prompt_profiles SET system_prompt=?3, user_template=?4, provider_id=?5, \
                     model=?6, temperature=?7, enabled=?8, updated_at_ms=?9 WHERE id=?1 AND kind=?2",
                    params![
                        id,
                        new.kind,
                        new.system_prompt,
                        new.user_template,
                        new.provider_id,
                        new.model,
                        new.temperature,
                        i64::from(new.enabled),
                        ts
                    ],
                )?;
                id
            }
            None => {
                let id = ids::new_id();
                conn.execute(
                    "INSERT INTO prompt_profiles(id, name, kind, system_prompt, user_template, \
                     provider_id, model, temperature, enabled, created_at_ms, updated_at_ms) \
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
                    params![
                        id,
                        new.name,
                        new.kind,
                        new.system_prompt,
                        new.user_template,
                        new.provider_id,
                        new.model,
                        new.temperature,
                        i64::from(new.enabled),
                        ts
                    ],
                )?;
                id
            }
        }
    };
    get_profile(db, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get_profile(
    db: &AppDatabase,
    id: &str,
) -> Result<Option<PromptProfileRecord>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM prompt_profiles WHERE id = ?1"),
        [id],
        row_to_record,
    )
    .optional()
}

/// All profiles of one kind (`style` | `transform`), settings-UI order.
pub fn list_by_kind(
    db: &AppDatabase,
    kind: &str,
) -> Result<Vec<PromptProfileRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM prompt_profiles WHERE kind = ?1 ORDER BY name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map([kind], row_to_record)?;
    rows.collect()
}

pub fn delete_profile(db: &AppDatabase, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    Ok(conn.execute("DELETE FROM prompt_profiles WHERE id = ?1", [id])? == 1)
}
