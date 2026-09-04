//! PAD-306 — the canonical capture/attempt write path for live dictation.
//!
//! DATA-105 built `captures` / `transcription_attempts` / `representations`
//! but nothing in production wrote to them, so `delivery_events` could never
//! satisfy its NOT NULL FK on `transcription_attempts` and PAD-305's audit
//! was unreachable (escalation PAD-305-02, owner decision: keep the FK).
//!
//! This module is the seam `actions.rs` calls: plain values in, canonical
//! rows out, plus the [`DeliverySource`] that [`super::deliver_with_audit`]
//! needs. It contains no transcription logic — the live path keeps deciding
//! WHAT the text is; this only records WHERE it came from.

use crate::delivery::DeliverySource;
use crate::storage::database::{AppDatabase, StorageError};
use crate::storage::models::{DeliverySourceKind, IntegrityState};
use crate::storage::repositories::captures::{insert_capture, NewCapture};
use crate::storage::repositories::representations::{insert_representation, NewRepresentation};
use crate::storage::repositories::transcriptions::{complete_attempt, insert_attempt, NewAttempt};

/// Everything one finished dictation knows about itself.
///
/// Borrowed rather than owned: every field already exists in `actions.rs`
/// scope, and nothing here should force a clone of a transcript.
pub struct DictationInput<'a> {
    pub title: &'a str,
    pub audio_file_name: Option<&'a str>,
    pub audio_sha256: Option<&'a str>,
    pub audio_size_bytes: Option<i64>,
    /// Exact engine output — immutable, never rewritten (STT-103).
    pub engine_raw: &'a str,
    /// Deterministic normalization result — the durable "raw" text.
    pub normalized_stt: &'a str,
    pub model_id: Option<&'a str>,
    pub language: Option<&'a str>,
    pub normalizer_version: &'a str,
    /// LLM post-processing output, if it ran. Stored as a representation
    /// OF the attempt, never as a second transcript.
    pub post_processed_text: Option<&'a str>,
    pub post_process_prompt: Option<&'a str>,
}

/// Audio that never made it to disk still needs a capture: otherwise the
/// transcript would be delivered with nothing to point at.
fn integrity_for(audio_file_name: Option<&str>) -> IntegrityState {
    match audio_file_name {
        Some(_) => IntegrityState::AudioValid,
        None => IntegrityState::AudioMissing,
    }
}

fn new_capture(
    title: &str,
    audio_file_name: Option<&str>,
    audio_sha256: Option<&str>,
    audio_size_bytes: Option<i64>,
) -> NewCapture {
    NewCapture {
        audio_file_name: audio_file_name.map(str::to_string),
        audio_sha256: audio_sha256.map(str::to_string),
        audio_size_bytes,
        title: title.to_string(),
        source_app: None,
        integrity_state: integrity_for(audio_file_name),
    }
}

/// Record a successful dictation and return the source the delivery audit
/// must name.
///
/// `insert_attempt` writes a terminal `success` row and promotes the first
/// one to canonical in the same transaction, so no separate `mark_canonical`
/// call is needed for the live path.
pub fn record_dictation(
    db: &AppDatabase,
    input: &DictationInput<'_>,
) -> Result<DeliverySource, StorageError> {
    let capture = insert_capture(
        db,
        &new_capture(
            input.title,
            input.audio_file_name,
            input.audio_sha256,
            input.audio_size_bytes,
        ),
    )?;

    let attempt = insert_attempt(
        db,
        &NewAttempt {
            capture_id: capture.id.clone(),
            engine_raw: Some(input.engine_raw.to_string()),
            normalized_stt: Some(input.normalized_stt.to_string()),
            model_id: input.model_id.map(str::to_string),
            language: input.language.map(str::to_string),
            normalizer_version: input.normalizer_version.to_string(),
            dictionary_snapshot_sha256: None,
        },
    )?;

    // Only a post-process run produces a derived layer worth naming. The
    // deterministic layers are already inside normalized_stt.
    let representation_id = match input.post_processed_text {
        Some(text) => Some(
            insert_representation(
                db,
                &NewRepresentation {
                    attempt_id: attempt.id.clone(),
                    parent_representation_id: None,
                    kind: "post_process".to_string(),
                    text: text.to_string(),
                    processor: "post_process".to_string(),
                    processor_version: input.normalizer_version.to_string(),
                    prompt_profile_id: None,
                    effective_prompt_snapshot: input.post_process_prompt.map(str::to_string),
                    provider_snapshot: None,
                },
            )?
            .id,
        ),
        None => None,
    };

    let kind = match representation_id {
        Some(_) => DeliverySourceKind::Representation,
        None => DeliverySourceKind::NormalizedStt,
    };

    Ok(DeliverySource {
        capture_id: Some(capture.id),
        attempt_id: attempt.id,
        representation_id,
        kind,
    })
}

/// Record a dictation whose transcription failed.
///
/// Returns the capture id. There is no `DeliverySource`, because nothing is
/// delivered — but the attempt must exist and be terminal so the failure is
/// diagnosable instead of invisible.
pub fn record_failed_dictation(
    db: &AppDatabase,
    title: &str,
    audio_file_name: Option<&str>,
    audio_sha256: Option<&str>,
    audio_size_bytes: Option<i64>,
    error: &str,
) -> Result<String, StorageError> {
    let capture = insert_capture(
        db,
        &new_capture(title, audio_file_name, audio_sha256, audio_size_bytes),
    )?;

    // No normalized_stt => the attempt starts pending and never becomes
    // canonical; complete_attempt then moves it to terminal `failed`.
    let attempt = insert_attempt(
        db,
        &NewAttempt {
            capture_id: capture.id.clone(),
            engine_raw: None,
            normalized_stt: None,
            model_id: None,
            language: None,
            normalizer_version: String::new(),
            dictionary_snapshot_sha256: None,
        },
    )?;
    complete_attempt(db, &attempt.id, None, Some(error), None, None)?;

    Ok(capture.id)
}
