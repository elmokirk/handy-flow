//! Full-text search repository (HIST-231).
//!
//! FTS5 index is DERIVED and repository-managed: no triggers. The index
//! is (re)built from canonical tables inside one transaction and can be
//! truncated + repopulated at any time as a recovery requirement.
//! Queries are bounded and FTS-syntax-safe (user input is quoted).

use rusqlite::params;

use crate::storage::database::AppDatabase;

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
    let count = tx.query_row("SELECT COUNT(*) FROM search_fts", [], |r| r.get(0))?;
    tx.commit()?;
    Ok(count)
}

/// Quote user input so FTS5 operators in it cannot break the query.
fn quote_term(term: &str) -> String {
    format!("\"{}\"", term.replace('"', "\"\""))
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
    let terms: Vec<String> = query
        .split_whitespace()
        .filter(|t| !t.trim_matches('"').is_empty())
        .map(quote_term)
        .collect();
    if terms.is_empty() {
        return Ok(Vec::new());
    }
    // Implicit AND between terms; quoted terms are syntax-safe.
    let match_expr = terms.join(" ");

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
