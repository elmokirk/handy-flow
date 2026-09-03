//! Full-text search repository (HIST-231).
//!
//! FTS5 index is DERIVED and repository-managed: no triggers. The index
//! is (re)built from canonical tables inside one transaction and can be
//! truncated + repopulated at any time as a recovery requirement.
//! Queries are bounded and FTS-syntax-safe (user input is quoted).

use rusqlite::{params, Transaction};

use crate::storage::database::AppDatabase;

/// `ref_type` for note documents (NOTE-304). Notes are the third canonical
/// source named in `04-DATA_PERSISTENCE.md` alongside raw and representation.
pub const REF_NOTE: &str = "note";

#[derive(Clone, Debug)]
pub struct SearchHit {
    pub ref_type: String, // raw | representation
    pub ref_id: String,   // attempt_id | representation_id
    /// Optional context: capture id when resolvable in this query.
    pub capture_id: Option<String>,
}

/// Rebuild the whole index from canonical tables. Deterministic and
/// idempotent; safe to run at any time (startup recovery or manual).
pub fn rebuild_search_index(db: &AppDatabase) -> Result<usize, rusqlite::Error> {
    let mut conn = db.conn();
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM search_fts", [])?;
    tx.execute(
        "INSERT INTO search_fts(body, ref_type, ref_id)
         SELECT a.normalized_stt, 'raw', a.id
         FROM transcription_attempts a
         WHERE a.normalized_stt IS NOT NULL AND a.normalized_stt != ''",
        [],
    )?;
    tx.execute(
        "INSERT INTO search_fts(body, ref_type, ref_id)
         SELECT r.text, 'representation', r.id
         FROM representations r
         WHERE r.status = 'success' AND r.text != ''",
        [],
    )?;
    // NOTE-304: active notes, indexed on their CURRENT version only.
    // Trashed notes are excluded so search never resurfaces them.
    tx.execute(
        "INSERT INTO search_fts(body, ref_type, ref_id)
         SELECT n.title || ' ' || v.content, ?1, n.id
         FROM notes n
         JOIN note_versions v ON v.note_id = n.id
         WHERE n.deleted_at_ms IS NULL
           AND v.version_no = (SELECT MAX(v2.version_no) FROM note_versions v2
                               WHERE v2.note_id = n.id)",
        params![REF_NOTE],
    )?;
    let count = tx.query_row("SELECT COUNT(*) FROM search_fts", [], |r| r.get(0))?;
    tx.commit()?;
    Ok(count)
}

/// Replace a note's FTS document inside the caller's transaction, so the
/// index can never diverge from a committed note write. Called by the note
/// repository (the only SQL owner for notes) on every content/title change.
pub(crate) fn upsert_note_document(
    tx: &Transaction<'_>,
    note_id: &str,
    body: &str,
) -> Result<(), rusqlite::Error> {
    delete_note_document(tx, note_id)?;
    tx.execute(
        "INSERT INTO search_fts(body, ref_type, ref_id) VALUES (?1, ?2, ?3)",
        params![body, REF_NOTE, note_id],
    )?;
    Ok(())
}

/// Drop a note's FTS document (trashing a note, or clearing before reinsert).
pub(crate) fn delete_note_document(
    tx: &Transaction<'_>,
    note_id: &str,
) -> Result<(), rusqlite::Error> {
    tx.execute(
        "DELETE FROM search_fts WHERE ref_type = ?1 AND ref_id = ?2",
        params![REF_NOTE, note_id],
    )?;
    Ok(())
}

/// Quote user input so FTS5 operators in it cannot break the query.
fn quote_term(term: &str) -> String {
    format!("\"{}\"", term.replace('"', "\"\""))
}

/// Build a syntax-safe FTS5 MATCH expression from raw user input.
/// Returns None when the query carries no searchable term, so callers
/// return "no results" instead of matching everything.
pub(crate) fn to_match_expr(query: &str) -> Option<String> {
    let terms: Vec<String> = query
        .split_whitespace()
        .filter(|t| !t.trim_matches(|c: char| !c.is_alphanumeric()).is_empty())
        .map(quote_term)
        .collect();
    if terms.is_empty() {
        return None;
    }
    Some(terms.join(" "))
}

/// Bounded full-text search over the derived index. `ref_filter`
/// optionally narrows to `raw` or `representation`. Limit clamps 1..=200.
pub fn search(
    db: &AppDatabase,
    query: &str,
    ref_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<SearchHit>, rusqlite::Error> {
    // Implicit AND between terms; quoted terms are syntax-safe.
    let Some(match_expr) = to_match_expr(query) else {
        return Ok(Vec::new());
    };

    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT f.ref_type, f.ref_id,
                COALESCE((SELECT c.id FROM captures c
                          JOIN transcription_attempts t2 ON t2.capture_id = c.id
                          WHERE t2.id = f.ref_id), NULL) AS capture_id
         FROM search_fts f
         WHERE search_fts MATCH ?1 AND (?2 IS NULL OR f.ref_type = ?2)
         ORDER BY rank
         LIMIT ?3 OFFSET ?4",
    )?;
    let rows = stmt.query_map(
        params![match_expr, ref_filter, limit.clamp(1, 200), offset.max(0)],
        |r| {
            Ok(SearchHit {
                ref_type: r.get(0)?,
                ref_id: r.get(1)?,
                capture_id: r.get(2)?,
            })
        },
    )?;
    rows.collect()
}

/// Index size (diagnostics + rebuild verification).
pub fn index_size(db: &AppDatabase) -> Result<i64, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row("SELECT COUNT(*) FROM search_fts", [], |r| r.get(0))
}
