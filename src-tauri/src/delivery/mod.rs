//! Transcript delivery routing (PAD-305).
//!
//! Where a finished transcript goes is an explicit, recorded decision.
//! Each destination is a [`DeliverySink`]; [`deliver_with_audit`] delivers
//! once through the chosen sink and appends exactly one audit event —
//! success or failure — via a [`DeliveryAudit`] port.
//!
//! `clipboard.rs` keeps only focused-app paste mechanics; it no longer
//! decides *where* text goes. Adding a destination (Phase 4 KB capture,
//! Phase 5 REST/MCP) means adding a sink, not a branch in the paste path.

use std::fmt;

use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::settings::DeliveryTarget;
use crate::storage::database::AppDatabase;
use crate::storage::models::{DeliveryDestination, DeliverySourceKind};
use crate::storage::repositories::deliveries::{record_delivery, NewDeliveryEvent};
use crate::storage::repositories::notes as noterepo;
use crate::storage::repositories::representations::content_hash;
use crate::NotesManager;

/// Why a delivery did not reach its destination.
#[derive(Clone, Debug)]
pub struct DeliveryError(String);

impl DeliveryError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DeliveryError {}

/// The immutable origin of the delivered text (DATA-107). Never the text
/// itself — only identifiers, so no transcript body is duplicated.
#[derive(Clone, Debug)]
pub struct DeliverySource {
    pub capture_id: Option<String>,
    pub attempt_id: String,
    pub representation_id: Option<String>,
    pub kind: DeliverySourceKind,
}

/// One destination for a finished transcript.
pub trait DeliverySink {
    fn destination(&self) -> DeliveryDestination;
    fn deliver(&self, text: &str) -> Result<(), DeliveryError>;
}

/// Audit port. Implementations observe deliveries; they must never be able
/// to turn a delivered transcript into a failure (see [`deliver_with_audit`]).
pub trait DeliveryAudit {
    fn record(
        &self,
        source: &DeliverySource,
        destination: DeliveryDestination,
        text: &str,
        outcome: Result<(), &DeliveryError>,
    ) -> Result<(), String>;
}

/// Deliver once, then append exactly one event describing what happened.
///
/// Failures are recorded too: DATA-107 requires that a failed delivery can
/// be diagnosed without mutating any source. Retries append further events
/// rather than overwriting.
pub fn deliver_with_audit(
    sink: &dyn DeliverySink,
    audit: &dyn DeliveryAudit,
    source: &DeliverySource,
    text: &str,
) -> Result<(), DeliveryError> {
    let outcome = sink.deliver(text);
    if let Err(e) = audit.record(
        source,
        sink.destination(),
        text,
        outcome.as_ref().map(|_| ()),
    ) {
        // Observer failure: log and carry on. The text either reached the
        // destination or it did not; the audit cannot change that.
        log::error!("Failed to record delivery event: {e}");
    }
    outcome
}

/// SQLite-backed audit (`delivery_events`, append-only).
#[derive(Clone)]
pub struct SqliteDeliveryAudit {
    db: AppDatabase,
}

impl SqliteDeliveryAudit {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }
}

impl DeliveryAudit for SqliteDeliveryAudit {
    fn record(
        &self,
        source: &DeliverySource,
        destination: DeliveryDestination,
        text: &str,
        outcome: Result<(), &DeliveryError>,
    ) -> Result<(), String> {
        let event = NewDeliveryEvent {
            capture_id: source.capture_id.clone(),
            source_attempt_id: source.attempt_id.clone(),
            representation_id: source.representation_id.clone(),
            source_kind: source.kind,
            destination: Some(destination),
            text_sha256: content_hash(text),
            success: outcome.is_ok(),
            error: outcome.err().map(|e| e.message().to_string()),
        };
        record_delivery(&self.db, &event)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

/// Resolve the configured destination into its sink.
///
/// This is the *only* place the destination is decided. Falls back to the
/// historical focused-app paste if the scratchpad database cannot be
/// opened, so a storage problem never silently drops a transcript.
pub fn sink_for(app: &AppHandle) -> Box<dyn DeliverySink> {
    match crate::settings::get_settings(app).delivery_target {
        DeliveryTarget::FocusedApp => Box::new(FocusedAppSink::new(app.clone())),
        DeliveryTarget::Clipboard => Box::new(ClipboardSink::new(app.clone())),
        DeliveryTarget::Scratchpad => match scratchpad_notes(app) {
            Ok(notes) => Box::new(ScratchpadSink::new(notes)),
            Err(e) => {
                log::error!("Scratchpad delivery unavailable ({e}); pasting instead");
                Box::new(FocusedAppSink::new(app.clone()))
            }
        },
    }
}

fn scratchpad_notes(app: &AppHandle) -> Result<NotesManager, String> {
    let dir = crate::portable::app_data_dir(app).map_err(|e| e.to_string())?;
    let db = AppDatabase::open(dir.join("history.db")).map_err(|e| e.to_string())?;
    Ok(NotesManager::new(db))
}

/// Focused-app paste — the historical behaviour, unchanged.
pub struct FocusedAppSink {
    app: AppHandle,
}

impl FocusedAppSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl DeliverySink for FocusedAppSink {
    fn destination(&self) -> DeliveryDestination {
        DeliveryDestination::FocusedApp
    }

    fn deliver(&self, text: &str) -> Result<(), DeliveryError> {
        crate::clipboard::paste(text.to_string(), self.app.clone()).map_err(DeliveryError::new)
    }
}

/// Clipboard-only delivery: the text is made available, no synthetic paste.
pub struct ClipboardSink {
    app: AppHandle,
}

impl ClipboardSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl DeliverySink for ClipboardSink {
    fn destination(&self) -> DeliveryDestination {
        DeliveryDestination::Clipboard
    }

    fn deliver(&self, text: &str) -> Result<(), DeliveryError> {
        self.app
            .clipboard()
            .write_text(text)
            .map_err(|e| DeliveryError::new(format!("Failed to write to clipboard: {e}")))
    }
}

/// Scratchpad delivery: the transcript becomes a new note version with
/// `SOURCE_DICTATION`. Append-only — existing versions are never rewritten.
pub struct ScratchpadSink {
    notes: NotesManager,
}

impl ScratchpadSink {
    pub fn new(notes: NotesManager) -> Self {
        Self { notes }
    }

    /// The note dictation lands in: the most recently touched active note,
    /// or a new one when the scratchpad is empty.
    fn target_note(&self, text: &str) -> Result<Option<String>, rusqlite::Error> {
        if let Some(note) = self.notes.list()?.into_iter().next() {
            return Ok(Some(note.id));
        }
        self.notes.create_dictated(text)?;
        Ok(None)
    }
}

impl DeliverySink for ScratchpadSink {
    fn destination(&self) -> DeliveryDestination {
        DeliveryDestination::Scratchpad
    }

    fn deliver(&self, text: &str) -> Result<(), DeliveryError> {
        let to_error = |e: rusqlite::Error| DeliveryError::new(e.to_string());
        let note_id = match self.target_note(text).map_err(to_error)? {
            // A fresh note already carries the dictated text as version 1.
            None => return Ok(()),
            Some(id) => id,
        };

        let current = self
            .notes
            .current_content(&note_id)
            .map_err(to_error)?
            .unwrap_or_default();
        let appended = if current.trim().is_empty() {
            text.to_string()
        } else {
            format!("{current}\n{text}")
        };

        self.notes
            .save_content(&note_id, &appended, noterepo::SOURCE_DICTATION)
            .map(|_| ())
            .map_err(to_error)
    }
}
