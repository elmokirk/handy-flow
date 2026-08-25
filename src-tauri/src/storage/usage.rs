//! Storage usage reporting + retention policy surface (DATA-108).
//!
//! Policy: the custom fork PRESERVES raw audio and source text by
//! default. Automatic destructive retention is DISABLED by design —
//! there is intentionally no code path here that deletes anything.
//! Future retention (roadmap) must be a separate, explicit,
//! owner-configured feature and will live behind its own migration +
//! settings keys, never silently inside this module.

use std::path::Path;

use crate::storage::database::AppDatabase;

/// Hard contract for MVP: automatic destructive purge does not exist.
/// A function (not a const) so callers cannot const-fold it into
/// compile-time assumptions; the value is a policy statement, not a flag.
pub fn automatic_retention_enabled() -> bool {
    false
}

#[derive(Clone, Debug, Default)]
pub struct StorageUsage {
    pub database_bytes: u64,
    pub database_wal_bytes: u64,
    pub recording_count: i64,
    pub recording_total_bytes: u64,
    /// Bytes of recordings NOT referenced by any capture row.
    pub unreferenced_recording_bytes: u64,
    pub staged_temp_count: usize,
    pub staged_temp_bytes: u64,
    /// Trashed captures still holding audio (recoverable until purge).
    pub trashed_capture_count: i64,
}

fn dir_stats(dir: &Path) -> (u64, Vec<(std::path::PathBuf, u64)>) {
    let mut total = 0u64;
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_file() {
                let len = p.metadata().map(|m| m.len()).unwrap_or(0);
                total += len;
                files.push((p, len));
            }
        }
    }
    (total, files)
}

pub fn measure(db: &AppDatabase, db_path: &Path, recordings_dir: &Path) -> StorageUsage {
    let conn = db.conn();

    let recording_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM captures WHERE audio_file_name IS NOT NULL AND deleted_at_ms IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Unreferenced = on disk but no active capture row points at it.
    let mut unreferenced = 0u64;
    let (_, files) = dir_stats(recordings_dir);
    let referenced: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT audio_file_name FROM captures WHERE audio_file_name IS NOT NULL",
            )
            .expect("prepare referenced query");
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .expect("query referenced");
        rows.flatten().collect()
    };
    for (path, len) in &files {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !referenced.contains(&name) {
            unreferenced += len;
        }
    }

    let staging_dir = recordings_dir.join(".staging");
    let (staged_bytes, staged_files) = dir_stats(&staging_dir);

    let wal_path = {
        let mut p = db_path.to_path_buf();
        p.set_extension("db-wal");
        p
    };

    StorageUsage {
        database_bytes: db_path.metadata().map(|m| m.len()).unwrap_or(0),
        database_wal_bytes: wal_path.metadata().map(|m| m.len()).unwrap_or(0),
        recording_count,
        recording_total_bytes: files.iter().map(|f| f.1).sum::<u64>()
            - unreferenced.min(files.iter().map(|f| f.1).sum::<u64>()),
        unreferenced_recording_bytes: unreferenced,
        staged_temp_count: staged_files.len(),
        staged_temp_bytes: staged_bytes,
        trashed_capture_count: conn
            .query_row(
                "SELECT COUNT(*) FROM captures WHERE deleted_at_ms IS NOT NULL",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0),
    }
}
