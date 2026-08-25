//! Representation repository — the ONLY SQL owner for representations.
//!
//! Representations are DERIVED text versions (post_process, snippet,
//! style, transform, manual_edit). They never replace an attempt's
//! normalized_stt; every row records enough provenance to interpret the
//! output after profile edits or deletions (planning/04).

use rusqlite::{params, OptionalExtension};

use crate::storage::database::AppDatabase;
use crate::storage::ids;

#[derive(Clone, Debug)]
pub struct NewRepresentation {
    pub attempt_id: String,
    pub parent_representation_id: Option<String>,
    /// One of: post_process | snippet | style | transform | manual_edit
    /// (DB CHECK enforces the set).
    pub kind: String,
    pub text: String,
    pub processor: String,
    pub processor_version: String,
    pub prompt_profile_id: Option<String>,
    pub effective_prompt_snapshot: Option<String>,
    /// Provider/model snapshot at generation time, transport-agnostic.
    pub provider_snapshot: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RepresentationRecord {
    pub id: String,
    pub attempt_id: String,
    pub parent_representation_id: Option<String>,
    pub kind: String,
    pub text: String,
    pub content_hash_sha256: Option<String>,
    pub processor: Option<String>,
    pub processor_version: Option<String>,
    pub prompt_profile_id: Option<String>,
    pub effective_prompt_snapshot: Option<String>,
    pub provider_snapshot: Option<String>,
    pub status: String,
    pub created_at_ms: i64,
}

/// SHA-256 hex digest of representation content — deterministic identity
/// used by export idempotency later (KB-402).
pub fn content_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

/// Insert a successful derived representation with full provenance.
pub fn insert_representation(
    db: &AppDatabase,
    new: &NewRepresentation,
) -> Result<RepresentationRecord, rusqlite::Error> {
    let conn = db.conn();
    let id = ids::new_id();
    let ts = now_ms();
    let hash = content_hash(&new.text);

    conn.execute(
        "INSERT INTO representations(
            id, attempt_id, parent_representation_id, kind, text,
            content_hash_sha256, processor, processor_version,
            prompt_profile_id, effective_prompt_snapshot, provider_snapshot,
            status, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'success', ?12)",
        params![
            id,
            new.attempt_id,
            new.parent_representation_id,
            new.kind,
            new.text,
            hash,
            new.processor,
            new.processor_version,
            new.prompt_profile_id,
            new.effective_prompt_snapshot,
            new.provider_snapshot,
            ts,
        ],
    )?;

    Ok(RepresentationRecord {
        id,
        attempt_id: new.attempt_id.clone(),
        parent_representation_id: new.parent_representation_id.clone(),
        kind: new.kind.clone(),
        text: new.text.clone(),
        content_hash_sha256: Some(hash),
        processor: Some(new.processor.clone()),
        processor_version: Some(new.processor_version.clone()),
        prompt_profile_id: new.prompt_profile_id.clone(),
        effective_prompt_snapshot: new.effective_prompt_snapshot.clone(),
        provider_snapshot: new.provider_snapshot.clone(),
        status: "success".into(),
        created_at_ms: ts,
    })
}

const REP_COLUMNS: &str = "id, attempt_id, parent_representation_id, kind, text,
    content_hash_sha256, processor, processor_version, prompt_profile_id,
    effective_prompt_snapshot, provider_snapshot, status, created_at_ms";

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<RepresentationRecord> {
    Ok(RepresentationRecord {
        id: row.get("id")?,
        attempt_id: row.get("attempt_id")?,
        parent_representation_id: row.get("parent_representation_id")?,
        kind: row.get("kind")?,
        text: row.get("text")?,
        content_hash_sha256: row.get("content_hash_sha256")?,
        processor: row.get("processor")?,
        processor_version: row.get("processor_version")?,
        prompt_profile_id: row.get("prompt_profile_id")?,
        effective_prompt_snapshot: row.get("effective_prompt_snapshot")?,
        provider_snapshot: row.get("provider_snapshot")?,
        status: row.get("status")?,
        created_at_ms: row.get("created_at_ms")?,
    })
}

pub fn representations_for_attempt(
    db: &AppDatabase,
    attempt_id: &str,
) -> Result<Vec<RepresentationRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {REP_COLUMNS} FROM representations WHERE attempt_id = ?1 ORDER BY created_at_ms ASC"
    ))?;
    let rows = stmt.query_map([attempt_id], row_to_record)?;
    rows.collect()
}

pub fn get_representation(
    db: &AppDatabase,
    id: &str,
) -> Result<Option<RepresentationRecord>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!("SELECT {REP_COLUMNS} FROM representations WHERE id = ?1"),
        [id],
        row_to_record,
    )
    .optional()
}
