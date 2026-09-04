//! PAD-306 acceptance: the canonical capture/attempt pipeline is the LIVE
//! write path, so a delivered transcript names an immutable source.
//!
//! DATA-105 delivered `insert_capture` / `insert_attempt` /
//! `complete_attempt` / `mark_canonical` at repository level with no
//! production caller. PAD-305 built the delivery audit, but
//! `delivery_events.source_attempt_id` is NOT NULL against
//! `transcription_attempts`, so no live delivery could ever be audited
//! (escalation PAD-305-02, owner decision: keep the FK, wire the pipeline).
//!
//! `actions.rs` runs inside a Tauri task needing an `AppHandle`, a loaded
//! model and real audio. The recorder tested here is the seam it calls:
//! plain values in, canonical rows written, `DeliverySource` out.

use handy_app_lib::delivery::pipeline::{
    record_dictation, record_failed_dictation, DictationInput,
};
use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::models::DeliverySourceKind;
use handy_app_lib::storage::repositories::captures::get_capture;
use handy_app_lib::storage::repositories::representations::representations_for_attempt;
use handy_app_lib::storage::repositories::transcriptions::{
    attempts_for_capture, canonical_attempt,
};

fn fresh(tag: &str) -> AppDatabase {
    let dir = std::env::temp_dir().join(format!("handy-pad306-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    db
}

fn plain_input<'a>() -> DictationInput<'a> {
    DictationInput {
        title: "2026-09-04 10:00",
        audio_file_name: Some("handy-1.wav"),
        audio_sha256: Some("abc123"),
        audio_size_bytes: Some(4096),
        engine_raw: "hello  world",
        normalized_stt: "Hello world",
        model_id: Some("whisper-small"),
        language: Some("en"),
        normalizer_version: "1",
        post_processed_text: None,
        post_process_prompt: None,
    }
}

/// 01 — the core acceptance: one capture, one successful attempt, canonical.
#[test]
fn successful_dictation_writes_capture_and_canonical_attempt() {
    let db = fresh("01");

    let source = record_dictation(&db, &plain_input()).expect("recording must succeed");

    let capture_id = source.capture_id.clone().expect("capture id must be set");
    let capture = get_capture(&db, &capture_id).unwrap().expect("capture row");
    assert_eq!(capture.audio_file_name.as_deref(), Some("handy-1.wav"));
    assert_eq!(capture.audio_sha256.as_deref(), Some("abc123"));
    assert_eq!(capture.audio_size_bytes, Some(4096));

    let attempts = attempts_for_capture(&db, &capture_id).unwrap();
    assert_eq!(attempts.len(), 1, "exactly one attempt per live dictation");
    assert_eq!(attempts[0].status, "success");
    assert_eq!(attempts[0].id, source.attempt_id);

    let canonical = canonical_attempt(&db, &capture_id)
        .unwrap()
        .expect("first successful attempt must be canonical");
    assert_eq!(canonical.id, source.attempt_id);
}

/// 02 — engine_raw stays exactly what the engine produced (STT-103).
#[test]
fn engine_raw_and_normalized_are_stored_verbatim() {
    let db = fresh("02");

    let source = record_dictation(&db, &plain_input()).unwrap();

    let attempt = canonical_attempt(&db, source.capture_id.as_ref().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(attempt.engine_raw.as_deref(), Some("hello  world"));
    assert_eq!(attempt.normalized_stt.as_deref(), Some("Hello world"));
    assert_eq!(attempt.model_id.as_deref(), Some("whisper-small"));
    assert_eq!(attempt.language.as_deref(), Some("en"));
    assert_eq!(attempt.id, source.attempt_id);
}

/// 03 — without post-processing the delivered text IS normalized_stt.
#[test]
fn without_post_processing_the_source_is_normalized_stt() {
    let db = fresh("03");

    let source = record_dictation(&db, &plain_input()).unwrap();

    assert!(matches!(source.kind, DeliverySourceKind::NormalizedStt));
    assert!(source.representation_id.is_none());
    let reps = representations_for_attempt(&db, &source.attempt_id).unwrap();
    assert!(reps.is_empty(), "no derived layer means no representation");
}

/// 04 — post-processed output is a REPRESENTATION of the attempt, not a
/// second transcript. `engine_raw` must survive untouched.
#[test]
fn post_processed_text_is_stored_as_a_representation() {
    let db = fresh("04");
    let mut input = plain_input();
    input.post_processed_text = Some("Hello, world!");
    input.post_process_prompt = Some("Fix punctuation.");

    let source = record_dictation(&db, &input).unwrap();

    let reps = representations_for_attempt(&db, &source.attempt_id).unwrap();
    assert_eq!(reps.len(), 1, "exactly one post-process representation");
    assert_eq!(reps[0].kind, "post_process");
    assert_eq!(reps[0].text, "Hello, world!");
    assert_eq!(
        reps[0].effective_prompt_snapshot.as_deref(),
        Some("Fix punctuation.")
    );

    assert!(matches!(source.kind, DeliverySourceKind::Representation));
    assert_eq!(
        source.representation_id.as_deref(),
        Some(reps[0].id.as_str())
    );

    let attempt = canonical_attempt(&db, source.capture_id.as_ref().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(attempt.engine_raw.as_deref(), Some("hello  world"));
    assert_eq!(attempt.normalized_stt.as_deref(), Some("Hello world"));
}

/// 05 — a failed transcription is representable: terminal attempt carrying
/// the error, and NO canonical attempt (nothing was transcribed).
#[test]
fn failed_transcription_writes_terminal_attempt_without_canonical() {
    let db = fresh("05");

    let capture_id = record_failed_dictation(
        &db,
        "2026-09-04 10:05",
        Some("handy-2.wav"),
        Some("def456"),
        Some(2048),
        "engine exploded",
    )
    .expect("a failed dictation must still be recorded");

    let attempts = attempts_for_capture(&db, &capture_id).unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].status, "failed");
    assert_eq!(attempts[0].error.as_deref(), Some("engine exploded"));
    assert!(!attempts[0].is_canonical);

    assert!(
        canonical_attempt(&db, &capture_id).unwrap().is_none(),
        "a failed dictation has no canonical text"
    );
}

/// 06 — the returned source satisfies the delivery_events FK. This is the
/// whole point of the ticket: PAD-305's audit becomes reachable.
#[test]
fn recorded_source_can_carry_a_delivery_event() {
    use handy_app_lib::delivery::{DeliveryAudit, SqliteDeliveryAudit};
    use handy_app_lib::storage::models::DeliveryDestination;
    use handy_app_lib::storage::repositories::deliveries::events_for_capture;

    let db = fresh("06");
    let source = record_dictation(&db, &plain_input()).unwrap();

    let audit = SqliteDeliveryAudit::new(db.clone());
    audit
        .record(
            &source,
            DeliveryDestination::FocusedApp,
            "Hello world",
            Ok(()),
        )
        .expect("the FK must resolve against the attempt just written");

    let events = events_for_capture(&db, source.capture_id.as_ref().unwrap()).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source_attempt_id, source.attempt_id);
    assert!(events[0].success);
}

/// 07 — audio is optional. A dictation whose WAV failed to save must still
/// produce a source, otherwise the transcript would be delivered unaudited.
#[test]
fn dictation_without_audio_still_yields_a_source() {
    let db = fresh("07");
    let mut input = plain_input();
    input.audio_file_name = None;
    input.audio_sha256 = None;
    input.audio_size_bytes = None;

    let source = record_dictation(&db, &input).unwrap();

    let capture = get_capture(&db, source.capture_id.as_ref().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(capture.audio_file_name, None);
    assert_eq!(capture.integrity_state, "audio_missing");
    assert!(!source.attempt_id.is_empty());
}
