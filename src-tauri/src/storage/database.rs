//! `AppDatabase` — connection policy over the existing physical database.
//!
//! Policy (frozen, planning/04-DATA_PERSISTENCE):
//! - physical filename stays `history.db`; no rename, ever;
//! - WAL journal mode, `foreign_keys = ON`, bounded busy timeout,
//!   short transactions owned by repositories;
//! - connectors later open read-only and fail closed on incompatible
//!   schema versions (enforced in later tickets via `app_meta`).
//!
//! Threading model: `rusqlite::Connection` is `Send` but not `Sync`, so
//! each [`AppDatabase`] guards exactly one connection behind a mutex and
//! is freely shareable (`Clone`). True reader/writer parallelism comes
//! from using SEPARATE connections (WAL allows one writer plus many
//! readers), not from sharing one.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

/// Bounded busy timeout for writers contending under WAL.
pub const BUSY_TIMEOUT_MS: u64 = 5_000;

#[derive(Debug)]
pub enum StorageError {
    /// Opening/using the database failed at the SQLite level.
    StorageUnavailable(rusqlite::Error),
    /// The file exists but is not a usable SQLite database.
    CorruptData(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::StorageUnavailable(e) => write!(f, "storage unavailable: {e}"),
            StorageError::CorruptData(detail) => write!(f, "corrupt data: {detail}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::SqliteFailure(ffi, _)
                if ffi.code == rusqlite::ErrorCode::NotADatabase =>
            {
                StorageError::CorruptData("file is not a SQLite database".into())
            }
            other => StorageError::StorageUnavailable(other),
        }
    }
}

/// Shareable handle around one SQLite connection with the custom
/// connection policy. Repositories run their SQL through short
/// transactions obtained via [`AppDatabase::conn`].
#[derive(Clone, Debug)]
pub struct AppDatabase {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl AppDatabase {
    /// Open (creating if needed) a read/write application connection.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open(path)?;
        apply_write_policy(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open an immutable read-only connection.
    ///
    /// Later REST/MCP companions reuse this policy; they must never be
    /// able to write even if a future bug tried to.
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
                | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )?;
        // Read-only connections still honor busy_timeout for readers that
        // race a checkpointing writer under WAL.
        conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Lock the connection for one short transaction / statement batch.
    ///
    /// The guard derefs to `&Connection`, so repositories keep natural
    /// rusqlite ergonomics: `db.conn().execute(...)` etc.
    pub fn conn(&self) -> MutexGuard<'_, rusqlite::Connection> {
        self.conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// `PRAGMA integrity_check` — must return exactly one row, `ok`.
    pub fn integrity_check(&self) -> Result<(), StorageError> {
        let result: String = self
            .conn()
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if result == "ok" {
            Ok(())
        } else {
            Err(StorageError::CorruptData(format!(
                "integrity_check reported: {result}"
            )))
        }
    }
}

fn apply_write_policy(conn: &rusqlite::Connection) -> Result<(), StorageError> {
    conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    // NORMAL under WAL is the durable-and-fast recommended pairing; it is
    // crash-safe for application crashes and consistent with the recovery
    // reconciler design (power-loss safety comes from WAL + fsync policy).
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}
