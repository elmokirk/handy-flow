//! KB-401 — the knowledge export object.
//!
//! One stable shape for everything that leaves this app as knowledge: a
//! dictation or a note. It is deliberately independent of SQLite — no row
//! ids beyond the domain ids, no epoch millis, no nullable columns leaking
//! through. KB-402 builds its idempotency key on [`KnowledgeItem::export_id`],
//! KB-403 renders these into Markdown, and UAT-605 checks provenance here.
//!
//! Three rules the tests pin and later phases must not quietly bend:
//!
//! * **The canonical raw is `normalized_stt`.** Not `engine_raw` (that would
//!   push filler words and mis-hearings into a permanent knowledge base) and
//!   not a post-processed representation (that would let an LLM rewrite the
//!   durable record). `engine_raw` is forensic detail and stays out of the
//!   export entirely.
//! * **Representations travel with the item.** A derived text is a layer of
//!   one capture, never a second knowledge item.
//! * **Ids are stable.** Building the same item twice yields the same
//!   `export_id`, or re-export would duplicate instead of update.

use serde::{Deserialize, Serialize};

use crate::storage::database::{AppDatabase, StorageError};
use crate::storage::repositories::captures::{get_capture, list_active_captures, CaptureRecord};
use crate::storage::repositories::notes as noterepo;
use crate::storage::repositories::representations::representations_for_attempt;
use crate::storage::repositories::transcriptions::canonical_attempt;

/// Bumped only on a breaking change to the exported shape. Consumers that
/// see an unknown version must refuse rather than guess.
pub const KNOWLEDGE_CONTRACT_VERSION: &str = "1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeKind {
    Transcription,
    Note,
}

/// Why an item could not be exported. Distinguished from a storage failure
/// because "nothing to export" is a normal outcome, not an error condition.
#[derive(Debug)]
pub enum KnowledgeError {
    /// The capture or note does not exist (or is trashed).
    NotFound,
    /// The capture exists but has no successful transcription, so there is
    /// no canonical text to export.
    NoCanonicalAttempt,
    Storage(StorageError),
}

impl std::fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnowledgeError::NotFound => f.write_str("not found"),
            KnowledgeError::NoCanonicalAttempt => {
                f.write_str("capture has no canonical attempt to export")
            }
            KnowledgeError::Storage(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for KnowledgeError {}

impl From<rusqlite::Error> for KnowledgeError {
    fn from(e: rusqlite::Error) -> Self {
        KnowledgeError::Storage(StorageError::from(e))
    }
}

/// Audio facts, carried so an export can be traced back to the recording
/// even when the WAV itself is not copied (KB-404 audio modes).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeAudio {
    pub file_name: Option<String>,
    pub sha256: Option<String>,
    pub bytes: Option<i64>,
    /// One of the frozen `IntegrityState` values, as text.
    pub integrity_state: String,
}

/// A derived text layer of the canonical attempt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeRepresentation {
    pub id: String,
    pub kind: String,
    pub text: String,
    pub processor: Option<String>,
    pub processor_version: Option<String>,
    pub created_at: String,
}

/// The export object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub contract_version: String,
    pub kind: KnowledgeKind,
    /// Stable, deterministic identity: `capture:<id>` or `note:<id>`.
    pub export_id: String,
    /// The domain id without the kind prefix.
    pub source_id: String,
    /// The immutable attempt the raw text came from. `None` for notes.
    pub attempt_id: Option<String>,
    pub title: String,
    /// The canonical text. `normalized_stt` for a dictation, current content
    /// for a note.
    pub raw: String,
    pub language: Option<String>,
    pub model_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub audio: Option<KnowledgeAudio>,
    pub representations: Vec<KnowledgeRepresentation>,
}

/// Epoch millis → RFC3339 UTC. The export must be readable by tools that
/// have never seen our schema, so timestamps leave as text, not integers.
fn rfc3339(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .unwrap_or(chrono::DateTime::UNIX_EPOCH)
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn audio_of(capture: &CaptureRecord) -> Option<KnowledgeAudio> {
    // A capture with no audio at all still exports — it just says so
    // through `integrity_state` rather than pretending a file exists.
    capture.audio_file_name.as_ref()?;
    Some(KnowledgeAudio {
        file_name: capture.audio_file_name.clone(),
        sha256: capture.audio_sha256.clone(),
        bytes: capture.audio_size_bytes,
        integrity_state: capture.integrity_state.clone(),
    })
}

fn build_transcription(
    db: &AppDatabase,
    capture: &CaptureRecord,
) -> Result<KnowledgeItem, KnowledgeError> {
    let attempt = canonical_attempt(db, &capture.id)?.ok_or(KnowledgeError::NoCanonicalAttempt)?;
    // A canonical attempt without normalized_stt cannot exist by contract
    // (insert_attempt only promotes successful ones), but an empty export
    // would be worse than a clear refusal.
    let raw = attempt
        .normalized_stt
        .clone()
        .ok_or(KnowledgeError::NoCanonicalAttempt)?;

    let representations = representations_for_attempt(db, &attempt.id)?
        .into_iter()
        .map(|r| KnowledgeRepresentation {
            id: r.id,
            kind: r.kind,
            text: r.text,
            processor: r.processor,
            processor_version: r.processor_version,
            created_at: rfc3339(r.created_at_ms),
        })
        .collect();

    Ok(KnowledgeItem {
        contract_version: KNOWLEDGE_CONTRACT_VERSION.to_string(),
        kind: KnowledgeKind::Transcription,
        export_id: format!("capture:{}", capture.id),
        source_id: capture.id.clone(),
        attempt_id: Some(attempt.id),
        title: capture.title.clone(),
        raw,
        language: attempt.language,
        model_id: attempt.model_id,
        created_at: rfc3339(capture.created_at_ms),
        updated_at: rfc3339(capture.updated_at_ms),
        audio: audio_of(capture),
        representations,
    })
}

/// Build the export object for one dictation.
pub fn transcription_item(
    db: &AppDatabase,
    capture_id: &str,
) -> Result<KnowledgeItem, KnowledgeError> {
    let capture = get_capture(db, capture_id)?.ok_or(KnowledgeError::NotFound)?;
    if capture.deleted_at_ms.is_some() {
        return Err(KnowledgeError::NotFound);
    }
    build_transcription(db, &capture)
}

/// Build the export object for one note.
///
/// A note exports its CURRENT content — the highest version — because that
/// is what the user considers the note. History stays in `note_versions`.
pub fn note_item(db: &AppDatabase, note_id: &str) -> Result<KnowledgeItem, KnowledgeError> {
    let note = noterepo::get_note(db, note_id)?.ok_or(KnowledgeError::NotFound)?;
    if note.deleted_at_ms.is_some() {
        return Err(KnowledgeError::NotFound);
    }
    let version = noterepo::current_version(db, note_id)?.ok_or(KnowledgeError::NotFound)?;

    Ok(KnowledgeItem {
        contract_version: KNOWLEDGE_CONTRACT_VERSION.to_string(),
        kind: KnowledgeKind::Note,
        export_id: format!("note:{}", note.id),
        source_id: note.id.clone(),
        attempt_id: None,
        title: note.title,
        raw: version.content,
        language: None,
        model_id: None,
        created_at: rfc3339(note.created_at_ms),
        updated_at: rfc3339(note.updated_at_ms),
        audio: None,
        representations: Vec::new(),
    })
}

/// Everything currently exportable, newest captures first, then notes.
///
/// Captures without a canonical attempt are skipped rather than reported:
/// a dictation that failed to transcribe is not an export error, it simply
/// has nothing to say. Trashed items are excluded by the repositories.
pub fn exportable_items(
    db: &AppDatabase,
    limit: i64,
) -> Result<Vec<KnowledgeItem>, KnowledgeError> {
    let mut items = Vec::new();

    for capture in list_active_captures(db, limit, 0)? {
        match build_transcription(db, &capture) {
            Ok(item) => items.push(item),
            Err(KnowledgeError::NoCanonicalAttempt) => continue,
            Err(e) => return Err(e),
        }
    }

    for note in noterepo::list_active_notes(db)? {
        match note_item(db, &note.id) {
            Ok(item) => items.push(item),
            // A note whose versions vanished is not worth failing the whole
            // export over.
            Err(KnowledgeError::NotFound) => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(items)
}
