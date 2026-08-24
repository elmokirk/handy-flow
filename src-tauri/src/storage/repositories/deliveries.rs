//! Delivery event repository — append-only audit (DATA-107).
//!
//! Every successful paste/scratchpad delivery records ONE event naming
//! the exact immutable source (attempt or representation) plus a sha256
//! of the delivered text. Failed deliveries are recorded too so they
//! can be diagnosed without mutating any source. No transcript bodies
//! are duplicated here.

use rusqlite::{params, OptionalExtension};

use crate::storage::database::AppDatabase;
use crate::storage::ids;
use crate::storage::models::{DeliveryDestination, DeliverySourceKind};

#[derive(Clone, Debug)]
pub struct NewDeliveryEvent {
    pub capture_id: Option<String>,
    pub source_attempt_id: String,
    pub representation_id: Option<String>,
    pub source_kind: DeliverySourceKind,
    pub destination: Option<DeliveryDestination>,
    /// Hash of the exact text handed to the destination.
    pub text_sha256: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DeliveryEventRecord {
    pub id: String,
    pub capture_id: Option<String>,
    pub source_attempt_id: String,
    pub representation_id: Option<String>,
    pub source_kind: String,
    pub destination: Option<String>,
    pub text_sha256: String,
    pub success: bool,
    pub error: Option<String>,
    pub created_at_ms: i64,
}

/// Append one audit event. Never updates existing rows.
pub fn record_delivery(
    db: &AppDatabase,
    event: &NewDeliveryEvent,
) -> Result<DeliveryEventRecord, rusqlite::Error> {
    let conn = db.conn();
    let id = ids::new_id();
    let created_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();

    conn.execute(
        "INSERT INTO delivery_events(
            id, capture_id, source_attempt_id, representation_id,
            source_kind, destination, text_sha256, success, error, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            event.capture_id,
            event.source_attempt_id,
            event.representation_id,
            event.source_kind.as_str(),
            event.destination.map(|d| d.as_str().to_string()),
            event.text_sha256,
            i64::from(event.success),
            event.error,
            created_at_ms,
        ],
    )?;

    Ok(DeliveryEventRecord {
        id,
        capture_id: event.capture_id.clone(),
        source_attempt_id: event.source_attempt_id.clone(),
        representation_id: event.representation_id.clone(),
        source_kind: event.source_kind.as_str().into(),
        destination: event.destination.map(|d| d.as_str().into()),
        text_sha256: event.text_sha256.clone(),
        success: event.success,
        error: event.error.clone(),
        created_at_ms,
    })
}

const EVENT_COLUMNS: &str = "id, capture_id, source_attempt_id, representation_id,
    source_kind, destination, text_sha256, success, error, created_at_ms";

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeliveryEventRecord> {
    Ok(DeliveryEventRecord {
        id: row.get("id")?,
        capture_id: row.get("capture_id")?,
        source_attempt_id: row.get("source_attempt_id")?,
        representation_id: row.get("representation_id")?,
        source_kind: row.get("source_kind")?,
        destination: row.get("destination")?,
        text_sha256: row.get("text_sha256")?,
        success: row.get::<_, i64>("success")? == 1,
        error: row.get("error")?,
        created_at_ms: row.get("created_at_ms")?,
    })
}

pub fn events_for_capture(
    db: &AppDatabase,
    capture_id: &str,
) -> Result<Vec<DeliveryEventRecord>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(&format!(
        "SELECT {EVENT_COLUMNS} FROM delivery_events \
         WHERE capture_id = ?1 ORDER BY created_at_ms ASC"
    ))?;
    rows_collect(&mut stmt, [capture_id])
}

pub fn get_event(
    db: &AppDatabase,
    id: &str,
) -> Result<Option<DeliveryEventRecord>, rusqlite::Error> {
    let conn = db.conn();
    conn.query_row(
        &format!("SELECT {EVENT_COLUMNS} FROM delivery_events WHERE id = ?1"),
        [id],
        row_to_record,
    )
    .optional()
}

fn rows_collect<P>(
    stmt: &mut rusqlite::Statement<'_>,
    params: P,
) -> Result<Vec<DeliveryEventRecord>, rusqlite::Error>
where
    P: rusqlite::Params,
{
    let rows = stmt.query_map(params, row_to_record)?;
    rows.collect()
}
