//! KB-402 — the export outbox. The ONLY SQL owner for export targets/jobs.
//!
//! An export leaves the app and writes into a filesystem someone else owns,
//! so two questions must be answerable from the database alone:
//!
//! * *did we already do this?* — the `idempotency_key` is
//!   target + item + content hash. Same vault, same item, same bytes: one
//!   job, ever. Change the content and the key changes, so updates still
//!   flow. That is why the hash is part of the key rather than a separate
//!   "dirty" flag someone has to remember to clear.
//! * *what did we start and never finish?* — a job is never deleted.
//!   A crash leaves it `running`; [`requeue_stale_running`] brings it back.
//!   Exhausted retries leave it `failed_permanent`, still queryable. There
//!   is no path on which an export disappears quietly.
//!
//! State machine, frozen in `orchestration/DATA_STATE_MACHINES.md`:
//! `pending → running → succeeded | retry_wait | failed_permanent`,
//! plus `retry_wait → running`.

use rusqlite::{params, OptionalExtension};

use crate::storage::database::AppDatabase;
use crate::storage::ids;

/// How often one job is retried before it is parked as
/// `failed_permanent`. A bounded budget is the point: an unbounded retry
/// loop hides a broken vault instead of surfacing it.
pub const MAX_ATTEMPTS: i64 = 5;

/// First retry delay; doubles per attempt.
const BASE_BACKOFF_MS: i64 = 30_000;

/// A `running` job older than this is assumed stranded by a crash.
const STALE_RUNNING_MS: i64 = 60_000;

#[derive(Clone, Debug)]
pub struct NewExportTarget {
    /// Frozen to `markdown` in the MVP (DB CHECK enforces it).
    pub kind: String,
    pub root_path: String,
}

#[derive(Clone, Debug)]
pub struct ExportTargetRecord {
    pub id: String,
    pub kind: String,
    pub root_path: String,
    pub enabled: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug)]
pub struct ExportJobRecord {
    pub id: String,
    pub target_id: String,
    pub export_id: String,
    pub content_hash_sha256: String,
    pub idempotency_key: String,
    pub status: String,
    pub attempt_count: i64,
    pub next_attempt_at_ms: Option<i64>,
    pub last_error: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub completed_at_ms: Option<i64>,
}

/// What an enqueue actually did. The caller needs the difference: queueing
/// work, finding work already queued, and finding the work already done are
/// three different situations.
#[derive(Debug)]
pub enum EnqueueOutcome {
    Enqueued(ExportJobRecord),
    AlreadyQueued(ExportJobRecord),
    AlreadyExported(ExportJobRecord),
}

const TARGET_COLS: &str = "id, kind, root_path, enabled, created_at_ms, updated_at_ms";
const JOB_COLS: &str = "id, target_id, export_id, content_hash_sha256, idempotency_key, status, \
     attempt_count, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms, completed_at_ms";

fn row_to_target(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExportTargetRecord> {
    Ok(ExportTargetRecord {
        id: row.get("id")?,
        kind: row.get("kind")?,
        root_path: row.get("root_path")?,
        enabled: row.get::<_, i64>("enabled")? == 1,
        created_at_ms: row.get("created_at_ms")?,
        updated_at_ms: row.get("updated_at_ms")?,
    })
}

fn row_to_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExportJobRecord> {
    Ok(ExportJobRecord {
        id: row.get("id")?,
        target_id: row.get("target_id")?,
        export_id: row.get("export_id")?,
        content_hash_sha256: row.get("content_hash_sha256")?,
        idempotency_key: row.get("idempotency_key")?,
        status: row.get("status")?,
        attempt_count: row.get("attempt_count")?,
        next_attempt_at_ms: row.get("next_attempt_at_ms")?,
        last_error: row.get("last_error")?,
        created_at_ms: row.get("created_at_ms")?,
        updated_at_ms: row.get("updated_at_ms")?,
        completed_at_ms: row.get("completed_at_ms")?,
    })
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

/// A target is identified by what it points at, so re-registering the same
/// vault returns the existing row instead of forking the outbox.
pub fn upsert_target(
    db: &AppDatabase,
    new: &NewExportTarget,
) -> Result<ExportTargetRecord, rusqlite::Error> {
    let conn = db.conn();
    let existing = conn
        .query_row(
            &format!("SELECT {TARGET_COLS} FROM export_targets WHERE kind = ?1 AND root_path = ?2"),
            params![new.kind, new.root_path],
            row_to_target,
        )
        .optional()?;
    if let Some(target) = existing {
        return Ok(target);
    }

    let id = ids::new_id();
    let ts = now_ms();
    conn.execute(
        "INSERT INTO export_targets(id, kind, root_path, enabled, created_at_ms, updated_at_ms)
         VALUES (?1, ?2, ?3, 1, ?4, ?4)",
        params![id, new.kind, new.root_path, ts],
    )?;
    conn.query_row(
        &format!("SELECT {TARGET_COLS} FROM export_targets WHERE id = ?1"),
        [&id],
        row_to_target,
    )
}

pub fn list_targets(db: &AppDatabase) -> Result<Vec<ExportTargetRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {TARGET_COLS} FROM export_targets ORDER BY created_at_ms, id"
    ))?;
    let rows = stmt.query_map([], row_to_target)?;
    rows.collect()
}

fn idempotency_key(target_id: &str, export_id: &str, content_hash: &str) -> String {
    format!("{target_id}|{export_id}|{content_hash}")
}

/// Queue one item for one target, or report that there is nothing to do.
pub fn enqueue(
    db: &AppDatabase,
    target_id: &str,
    export_id: &str,
    content_hash: &str,
    now: i64,
) -> Result<EnqueueOutcome, rusqlite::Error> {
    let key = idempotency_key(target_id, export_id, content_hash);
    let mut conn = db.conn();
    let tx = conn.transaction()?;

    let existing = tx
        .query_row(
            &format!("SELECT {JOB_COLS} FROM export_jobs WHERE idempotency_key = ?1"),
            [&key],
            row_to_job,
        )
        .optional()?;

    if let Some(job) = existing {
        tx.commit()?;
        return Ok(match job.status.as_str() {
            "succeeded" => EnqueueOutcome::AlreadyExported(job),
            _ => EnqueueOutcome::AlreadyQueued(job),
        });
    }

    let id = ids::new_id();
    tx.execute(
        "INSERT INTO export_jobs(
            id, target_id, export_id, content_hash_sha256, idempotency_key,
            status, attempt_count, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, 'pending', 0, ?6, ?6)",
        params![id, target_id, export_id, content_hash, key, now],
    )?;
    let job = tx.query_row(
        &format!("SELECT {JOB_COLS} FROM export_jobs WHERE id = ?1"),
        [&id],
        row_to_job,
    )?;
    tx.commit()?;
    Ok(EnqueueOutcome::Enqueued(job))
}

/// Take the next due job and mark it `running` in ONE transaction, so two
/// workers can never write the same file at once.
pub fn claim_next(db: &AppDatabase, now: i64) -> Result<Option<ExportJobRecord>, rusqlite::Error> {
    let mut conn = db.conn();
    let tx = conn.transaction()?;

    let candidate: Option<String> = tx
        .query_row(
            "SELECT id FROM export_jobs
             WHERE status = 'pending'
                OR (status = 'retry_wait' AND next_attempt_at_ms <= ?1)
             ORDER BY created_at_ms, id LIMIT 1",
            [now],
            |r| r.get(0),
        )
        .optional()?;

    let Some(id) = candidate else {
        tx.commit()?;
        return Ok(None);
    };

    tx.execute(
        "UPDATE export_jobs SET status = 'running', claimed_at_ms = ?2, updated_at_ms = ?2
         WHERE id = ?1 AND status IN ('pending', 'retry_wait')",
        params![id, now],
    )?;
    let job = tx.query_row(
        &format!("SELECT {JOB_COLS} FROM export_jobs WHERE id = ?1"),
        [&id],
        row_to_job,
    )?;
    tx.commit()?;
    Ok(Some(job))
}

pub fn mark_succeeded(db: &AppDatabase, job_id: &str, now: i64) -> Result<(), rusqlite::Error> {
    let conn = db.conn();
    conn.execute(
        "UPDATE export_jobs SET status = 'succeeded', last_error = NULL,
            next_attempt_at_ms = NULL, completed_at_ms = ?2, updated_at_ms = ?2
         WHERE id = ?1",
        params![job_id, now],
    )?;
    Ok(())
}

/// Record a failed attempt: schedule a retry, or park the job permanently
/// once the budget is spent. Returns the stored state so the caller does not
/// have to guess which of the two happened.
pub fn mark_failed(
    db: &AppDatabase,
    job_id: &str,
    error: &str,
    now: i64,
) -> Result<ExportJobRecord, rusqlite::Error> {
    let mut conn = db.conn();
    let tx = conn.transaction()?;

    let attempts: i64 = tx.query_row(
        "SELECT attempt_count FROM export_jobs WHERE id = ?1",
        [job_id],
        |r| r.get(0),
    )?;
    let attempts = attempts + 1;

    if attempts >= MAX_ATTEMPTS {
        tx.execute(
            "UPDATE export_jobs SET status = 'failed_permanent', attempt_count = ?2,
                last_error = ?3, next_attempt_at_ms = NULL, completed_at_ms = ?4,
                updated_at_ms = ?4
             WHERE id = ?1",
            params![job_id, attempts, error, now],
        )?;
    } else {
        // Exponential, deterministic: 30s, 60s, 120s, 240s. No jitter — a
        // single-user desktop app has no thundering herd to spread out, and
        // determinism keeps the behaviour testable.
        let delay = BASE_BACKOFF_MS.saturating_mul(1 << (attempts - 1).min(20));
        tx.execute(
            "UPDATE export_jobs SET status = 'retry_wait', attempt_count = ?2,
                last_error = ?3, next_attempt_at_ms = ?4, updated_at_ms = ?5
             WHERE id = ?1",
            params![job_id, attempts, error, now + delay, now],
        )?;
    }

    let job = tx.query_row(
        &format!("SELECT {JOB_COLS} FROM export_jobs WHERE id = ?1"),
        [job_id],
        row_to_job,
    )?;
    tx.commit()?;
    Ok(job)
}

/// Bring back jobs left `running` by a crash.
///
/// The attempt count is deliberately NOT raised: the process dying is not
/// evidence that the export would fail, and charging it an attempt would
/// burn the retry budget on our own bug.
pub fn requeue_stale_running(db: &AppDatabase, now: i64) -> Result<usize, rusqlite::Error> {
    let conn = db.conn();
    let changed = conn.execute(
        "UPDATE export_jobs SET status = 'pending', claimed_at_ms = NULL, updated_at_ms = ?1
         WHERE status = 'running' AND claimed_at_ms IS NOT NULL AND claimed_at_ms <= ?2",
        params![now, now - STALE_RUNNING_MS],
    )?;
    Ok(changed)
}

pub fn jobs_for_export(
    db: &AppDatabase,
    export_id: &str,
) -> Result<Vec<ExportJobRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {JOB_COLS} FROM export_jobs WHERE export_id = ?1 ORDER BY created_at_ms, id"
    ))?;
    let rows = stmt.query_map([export_id], row_to_job)?;
    rows.collect()
}

/// Everything that will never be retried without a human. The UI must be
/// able to show this — a failed export the user cannot see is a lost export.
pub fn failed_permanent(db: &AppDatabase) -> Result<Vec<ExportJobRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {JOB_COLS} FROM export_jobs WHERE status = 'failed_permanent' \
         ORDER BY updated_at_ms DESC, id"
    ))?;
    let rows = stmt.query_map([], row_to_job)?;
    rows.collect()
}
