//! Canonical schema migrations with verified-backup safety (DATA-102).
//!
//! Version history (`user_version` pragma) — MUST stay aligned with
//! `managers/history.rs` until HIST-107 unifies the runners:
//!   1  upstream legacy create transcription_history
//!   2  + post_processed_text
//!   3  + post_process_prompt
//!   4  + post_process_requested   (upstream latest)
//!   5  canonical schema: app_meta, captures, transcription_attempts,
//!      representations + legacy data backfill.
//!
//! Safety gate: before raising the version, an online backup is created
//! and verified (`PRAGMA integrity_check = ok` on the reopened copy).
//! Migration is refused when a verified backup cannot be produced.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::database::{AppDatabase, StorageError};
use super::ids;

pub const TARGET_VERSION: i64 = 11;
/// `query_contract_version` advertised to REST/MCP companions later on.
pub const QUERY_CONTRACT_VERSION: i64 = 1;

/// Upstream legacy statements, verbatim from managers/history.rs V1..V4.
const LEGACY_V1_CREATE: &str = "CREATE TABLE IF NOT EXISTS transcription_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_name TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            saved BOOLEAN NOT NULL DEFAULT 0,
            title TEXT NOT NULL,
            transcription_text TEXT NOT NULL
        );";
const LEGACY_V2_ALTER: &str =
    "ALTER TABLE transcription_history ADD COLUMN post_processed_text TEXT;";
const LEGACY_V3_ALTER: &str =
    "ALTER TABLE transcription_history ADD COLUMN post_process_prompt TEXT;";
const LEGACY_V4_UPSTREAM_ALTER: &str =
    "ALTER TABLE transcription_history ADD COLUMN post_process_requested BOOLEAN NOT NULL DEFAULT 0;";

const CANONICAL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS app_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS captures (
    id TEXT PRIMARY KEY,
    legacy_history_id INTEGER UNIQUE,
    audio_file_name TEXT,
    audio_sha256 TEXT,
    audio_size_bytes INTEGER,
    audio_duration_ms INTEGER,
    title TEXT NOT NULL,
    source_app TEXT,
    saved INTEGER NOT NULL DEFAULT 1 CHECK (saved IN (0, 1)),
    integrity_state TEXT NOT NULL CHECK (integrity_state IN (
        'pending_audio', 'audio_valid', 'audio_missing',
        'audio_corrupt', 'recovered_orphan'
    )),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    deleted_at_ms INTEGER
);

CREATE INDEX idx_captures_created ON captures (created_at_ms DESC, id DESC);
CREATE INDEX idx_captures_trash ON captures (deleted_at_ms) WHERE deleted_at_ms IS NOT NULL;

CREATE TABLE IF NOT EXISTS transcription_attempts (
    id TEXT PRIMARY KEY,
    capture_id TEXT NOT NULL REFERENCES captures(id),
    attempt_number INTEGER NOT NULL,
    engine_raw TEXT,
    normalized_stt TEXT,
    model_id TEXT,
    language TEXT,
    normalizer_version TEXT,
    dictionary_snapshot_sha256 TEXT,
    provenance TEXT NOT NULL CHECK (provenance IN ('live', 'legacy_migration')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'success', 'failed')),
    error TEXT,
    is_canonical INTEGER NOT NULL DEFAULT 0 CHECK (is_canonical IN (0, 1)),
    created_at_ms INTEGER NOT NULL,
    completed_at_ms INTEGER
);

CREATE UNIQUE INDEX idx_attempts_capture_number
    ON transcription_attempts (capture_id, attempt_number);
CREATE UNIQUE INDEX idx_attempts_one_canonical
    ON transcription_attempts (capture_id) WHERE is_canonical = 1;

CREATE TABLE IF NOT EXISTS representations (
    id TEXT PRIMARY KEY,
    attempt_id TEXT NOT NULL REFERENCES transcription_attempts(id),
    parent_representation_id TEXT REFERENCES representations(id),
    kind TEXT NOT NULL CHECK (kind IN ('post_process', 'snippet', 'style', 'transform', 'manual_edit')),
    text TEXT NOT NULL,
    content_hash_sha256 TEXT,
    processor TEXT,
    processor_version TEXT,
    prompt_profile_id TEXT,
    effective_prompt_snapshot TEXT,
    provider_snapshot TEXT,
    status TEXT NOT NULL CHECK (status IN ('success', 'failed')),
    error TEXT,
    created_at_ms INTEGER NOT NULL
);

CREATE INDEX idx_representations_attempt ON representations (attempt_id);
"#;

#[derive(Clone, Debug)]
pub struct MigrationReport {
    pub from_version: i64,
    pub to_version: i64,
    pub legacy_rows_backfilled: usize,
    pub backup_path: Option<PathBuf>,
}

impl AppDatabase {
    /// Run pending canonical migrations with verified backup.
    ///
    /// Test hook `fail_at` injects a failure after backup creation to
    /// prove rollback safety without touching real files.
    pub fn migrate(
        &self,
        app_version: &str,
        fail_at: Option<i64>,
    ) -> Result<MigrationReport, StorageError> {
        let mut conn = self.conn();
        let from = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if from > TARGET_VERSION {
            return Err(StorageError::CorruptData(format!(
                "database user_version {from} is newer than supported {TARGET_VERSION}"
            )));
        }
        if from == TARGET_VERSION {
            return Ok(MigrationReport {
                from_version: from,
                to_version: TARGET_VERSION,
                legacy_rows_backfilled: 0,
                backup_path: None,
            });
        }

        // Verified online backup before ANY schema change.
        let backup_path = create_verified_backup(&conn)?;

        let result = apply_migrations(&mut conn, from, TARGET_VERSION, app_version, fail_at);
        match result {
            Ok(backfilled) => Ok(MigrationReport {
                from_version: from,
                to_version: TARGET_VERSION,
                legacy_rows_backfilled: backfilled,
                backup_path: Some(backup_path),
            }),
            Err(e) => {
                // Rollback contract: the transaction wrapper in
                // apply_migrations reverts partial DDL/DML; the pre-migration
                // file remains untouched because every step runs inside one
                // transaction. The verified backup stays on disk for owner
                // recovery either way.
                Err(e)
            }
        }
    }
}

fn apply_migrations(
    conn: &mut Connection,
    from: i64,
    to: i64,
    app_version: &str,
    fail_at: Option<i64>,
) -> Result<usize, StorageError> {
    let tx = conn.transaction()?;
    let mut backfilled = 0usize;

    for v in (from + 1)..=to {
        if let Some(fail) = fail_at {
            if v == fail {
                return Err(StorageError::CorruptData(format!(
                    "injected failure at version {v} (rollback test)"
                )));
            }
        }
        match v {
            1 => tx.execute_batch(LEGACY_V1_CREATE)?,
            2 => tx.execute_batch(LEGACY_V2_ALTER)?,
            3 => tx.execute_batch(LEGACY_V3_ALTER)?,
            4 => {
                // Guarded for databases that already carry the column via
                // another runner path; upstream numbering keeps V4 = this.
                let has_requested = tx
                    .query_row(
                        "SELECT COUNT(*) FROM pragma_table_info('transcription_history') \
                         WHERE name = 'post_process_requested'",
                        [],
                        |r| r.get::<_, i64>(0),
                    )
                    .unwrap_or(0);
                if has_requested == 0 {
                    tx.execute_batch(LEGACY_V4_UPSTREAM_ALTER)?;
                }
            }
            5 => {
                tx.execute_batch(CANONICAL_SCHEMA)?;
                backfilled = backfill_legacy_entries(&tx)?;
                seed_app_meta(&tx, app_version)?;
            }
            6 => {
                // DATA-107: append-only delivery audit. Events reference
                // immutable sources by id + content hash, never bodies.
                tx.execute_batch(
                    "CREATE TABLE IF NOT EXISTS delivery_events (
                        id TEXT PRIMARY KEY,
                        capture_id TEXT REFERENCES captures(id),
                        source_attempt_id TEXT NOT NULL
                            REFERENCES transcription_attempts(id),
                        representation_id TEXT REFERENCES representations(id),
                        source_kind TEXT NOT NULL CHECK (source_kind IN (
                            'normalized_stt', 'representation'
                        )),
                        destination TEXT CHECK (destination IN (
                            'focused_app', 'scratchpad', 'clipboard'
                        )),
                        text_sha256 TEXT NOT NULL,
                        success INTEGER NOT NULL CHECK (success IN (0, 1)),
                        error TEXT,
                        created_at_ms INTEGER NOT NULL
                    );
                    CREATE INDEX idx_delivery_events_capture
                        ON delivery_events (capture_id);
                    CREATE INDEX idx_delivery_events_created
                        ON delivery_events (created_at_ms DESC);",
                )?;
            }
            7 => {
                // DICT-201: durable dictionary entries with aliases and an
                // explicit enabled state. Terms are unique (case-insensitive
                // via COLLATE NOCASE) so legacy custom-word imports stay
                // idempotent.
                tx.execute_batch(
                    "CREATE TABLE IF NOT EXISTS dictionary_entries (
                        id TEXT PRIMARY KEY,
                        term TEXT NOT NULL COLLATE NOCASE UNIQUE,
                        aliases TEXT NOT NULL DEFAULT '[]',
                        enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
                        created_at_ms INTEGER NOT NULL,
                        updated_at_ms INTEGER NOT NULL
                    );
                    CREATE INDEX idx_dictionary_enabled
                        ON dictionary_entries (enabled);",
                )?;
            }
            8 => {
                // SNIP-211 + PROMPT-221: spoken-snippet triggers and the
                // unified Styles/Transforms prompt profile domain.
                tx.execute_batch(
                    "CREATE TABLE IF NOT EXISTS snippets (
                        id TEXT PRIMARY KEY,
                        trigger TEXT NOT NULL COLLATE NOCASE UNIQUE,
                        replacement TEXT NOT NULL,
                        priority INTEGER NOT NULL DEFAULT 0,
                        enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
                        created_at_ms INTEGER NOT NULL,
                        updated_at_ms INTEGER NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS prompt_profiles (
                        id TEXT PRIMARY KEY,
                        name TEXT NOT NULL,
                        kind TEXT NOT NULL CHECK (kind IN ('style', 'transform')),
                        system_prompt TEXT NOT NULL,
                        user_template TEXT NOT NULL,
                        provider_id TEXT NOT NULL,
                        model TEXT NOT NULL,
                        temperature REAL,
                        enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
                        created_at_ms INTEGER NOT NULL,
                        updated_at_ms INTEGER NOT NULL,
                        UNIQUE (name, kind)
                    );",
                )?;
            }
            9 => {
                // HIST-231: rebuildable full-text index over canonical text
                // sources. Repository-managed (no triggers); rebuild truncates
                // and repopulates from canonical tables in one transaction.
                tx.execute_batch(
                    "CREATE VIRTUAL TABLE IF NOT EXISTS search_fts USING fts5(
                        body,
                        ref_type UNINDEXED,
                        ref_id UNINDEXED
                    );",
                )?;
            }
            10 => {
                // NOTE-301: one Note domain with append-only versions.
                // Current content = highest version number (derived, ADR-024):
                // no mutable current_version_id that could drift.
                tx.execute_batch(
                    "CREATE TABLE IF NOT EXISTS notes (
                        id TEXT PRIMARY KEY,
                        title TEXT NOT NULL DEFAULT 'Note',
                        pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)),
                        deleted_at_ms INTEGER,
                        created_at_ms INTEGER NOT NULL,
                        updated_at_ms INTEGER NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS note_versions (
                        id TEXT PRIMARY KEY,
                        note_id TEXT NOT NULL REFERENCES notes(id),
                        version_no INTEGER NOT NULL,
                        content TEXT NOT NULL,
                        content_hash_sha256 TEXT NOT NULL,
                        source TEXT NOT NULL CHECK (source IN (
                            'dictation', 'manual_edit', 'transform', 'restore', 'wispr_import'
                        )),
                        created_at_ms INTEGER NOT NULL,
                        UNIQUE (note_id, version_no)
                    );
                    CREATE INDEX idx_note_versions_note
                        ON note_versions (note_id, version_no DESC);
                    CREATE INDEX idx_notes_active
                        ON notes (deleted_at_ms) WHERE deleted_at_ms IS NULL;",
                )?;
            }
            11 => {
                // IMP-001: generic import reference for foreign sources
                // (Wispr Flow transcriptEntityId). SQLite cannot ADD a
                // UNIQUE column, so uniqueness is enforced by the partial
                // unique index below; imports stay idempotent.
                tx.execute_batch(
                    "ALTER TABLE captures ADD COLUMN import_ref TEXT;
                     CREATE UNIQUE INDEX idx_captures_import_ref
                        ON captures (import_ref) WHERE import_ref IS NOT NULL;",
                )?;
            }
            other => {
                return Err(StorageError::CorruptData(format!(
                    "unknown migration step {other}"
                )))
            }
        }
        tx.pragma_update(None, "user_version", v)?;
    }

    tx.commit()?;
    Ok(backfilled)
}

fn seed_app_meta(tx: &rusqlite::Transaction<'_>, app_version: &str) -> Result<(), rusqlite::Error> {
    let existing_schema_version: i64 = tx
        .query_row(
            "SELECT value FROM app_meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if existing_schema_version == 0 {
        tx.execute(
            "INSERT INTO app_meta(key, value) VALUES ('schema_version', ?1)",
            [TARGET_VERSION.to_string()],
        )?;
        tx.execute(
            "INSERT INTO app_meta(key, value) VALUES ('created_by_app_version', ?1)",
            [app_version],
        )?;
    }
    tx.execute(
        "INSERT INTO app_meta(key, value) VALUES ('query_contract_version', ?1) \
         ON CONFLICT(key) DO NOTHING",
        [QUERY_CONTRACT_VERSION.to_string()],
    )?;
    Ok(())
}

/// Map legacy rows into canonical tables. Idempotent: rows whose
/// `legacy_history_id` already exists are skipped.
fn backfill_legacy_entries(tx: &rusqlite::Transaction<'_>) -> Result<usize, rusqlite::Error> {
    let legacy_exists: i64 = tx.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transcription_history'",
        [],
        |r| r.get(0),
    )?;
    if legacy_exists == 0 {
        return Ok(0);
    }

    let mut stmt = tx.prepare(
        "SELECT id, file_name, timestamp, saved, title, transcription_text,
                post_processed_text, post_process_prompt, post_process_requested
         FROM transcription_history",
    )?;
    let mut rows = stmt.query([])?;

    let mut migrated = 0usize;
    while let Some(row) = rows.next()? {
        let legacy_id: i64 = row.get("id")?;
        let already: i64 = tx.query_row(
            "SELECT COUNT(*) FROM captures WHERE legacy_history_id = ?1",
            [legacy_id],
            |r| r.get(0),
        )?;
        if already > 0 {
            continue;
        }

        let file_name: String = row.get("file_name")?;
        let timestamp_s: i64 = row.get("timestamp")?;
        let saved: bool = row.get("saved")?;
        let title: String = row.get("title")?;
        let transcription_text: String = row.get("transcription_text")?;
        let post_processed_text: Option<String> = row.get("post_processed_text")?;
        let post_process_prompt: Option<String> = row.get("post_process_prompt")?;
        let post_process_requested: bool = row.get("post_process_requested")?;

        // Legacy timestamps are SECONDS (chrono Utc::now().timestamp()).
        let created_at_ms = timestamp_s.saturating_mul(1000);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(created_at_ms);

        let capture_id = ids::new_id();
        tx.execute(
            "INSERT INTO captures(
                id, legacy_history_id, audio_file_name, title, saved,
                integrity_state, created_at_ms, updated_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'audio_valid', ?6, ?6)",
            rusqlite::params![
                capture_id,
                legacy_id,
                file_name,
                title,
                i64::from(saved),
                created_at_ms
            ],
        )?;

        // Legacy text maps to normalized_stt; engine_raw is unknown and
        // stays NULL forever (immutable), provenance marks the import.
        let attempt_id = ids::new_id();
        tx.execute(
            "INSERT INTO transcription_attempts(
                id, capture_id, attempt_number, engine_raw, normalized_stt,
                provenance, status, is_canonical, created_at_ms, completed_at_ms
             ) VALUES (?1, ?2, 1, NULL, ?3, 'legacy_migration', 'success', 1, ?4, ?4)",
            rusqlite::params![attempt_id, capture_id, transcription_text, created_at_ms],
        )?;

        // Legacy post-processing survives as a representation snapshot.
        if post_process_requested || post_processed_text.is_some() {
            let rep_id = ids::new_id();
            let (text, status): (String, &str) = match post_processed_text {
                Some(t) => (t, "success"),
                None => (String::new(), "failed"),
            };
            tx.execute(
                "INSERT INTO representations(
                    id, attempt_id, kind, text, processor,
                    effective_prompt_snapshot, provider_snapshot,
                    status, created_at_ms
                 ) VALUES (?1, ?2, 'post_process', ?3, 'upstream_post_process', ?4, NULL, ?5, ?6)",
                rusqlite::params![
                    rep_id,
                    attempt_id,
                    text,
                    post_process_prompt.unwrap_or_default(),
                    status,
                    now_ms
                ],
            )?;
        }

        migrated += 1;
    }
    Ok(migrated)
}

/// Create an ONLINE backup (WAL-safe, no filesystem copy of a hot db),
/// reopen it read-only and verify integrity. Returns the backup path.
pub fn create_verified_backup(src: &Connection) -> Result<PathBuf, StorageError> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let src_path =
        PathBuf::from(src.path().ok_or_else(|| {
            StorageError::CorruptData("source connection has no file path".into())
        })?);
    let backup_path = src_path.with_extension(format!("bak-{stamp}"));

    let mut dst = Connection::open(&backup_path)?;
    {
        let backup = rusqlite::backup::Backup::new(src, &mut dst)?;
        // Single-step full copy is fine at this scale; run to completion.
        backup.run_to_completion(64, std::time::Duration::from_millis(5), None)?;
    }
    dst.close().map_err(|(_, e)| StorageError::from(e))?;

    // Reopen the copy and prove it is usable before trusting it.
    let verify = AppDatabase::open_read_only(&backup_path)?;
    verify.integrity_check()?;
    Ok(backup_path)
}

/// Convenience for tests and tooling: open + migrate in one call.
pub fn open_and_migrate(
    path: impl AsRef<Path>,
    app_version: &str,
) -> Result<(AppDatabase, MigrationReport), StorageError> {
    let _ = Path::new(path.as_ref()); // ensure parent exists for fresh installs
    if let Some(parent) = path.as_ref().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let db = AppDatabase::open(path)?;
    let report = db.migrate(app_version, None)?;
    Ok((db, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_statements_match_upstream_shape() {
        // Guard against drift between the two runners' version numbering.
        assert!(LEGACY_V1_CREATE.contains("transcription_history"));
        assert!(LEGACY_V2_ALTER.contains("post_processed_text"));
        assert!(LEGACY_V3_ALTER.contains("post_process_prompt"));
        assert!(LEGACY_V4_UPSTREAM_ALTER.contains("post_process_requested"));
    }
}
