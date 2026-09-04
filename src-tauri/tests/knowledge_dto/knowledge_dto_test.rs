//! KB-401 acceptance: a stable export object that does not leak SQLite.
//!
//! The export DTO is what KB-402 (outbox/idempotency) and KB-403 (Markdown
//! connector) build on, and what UAT-605 checks provenance against. Three
//! things must hold and stay held:
//!
//! 1. the canonical raw text IS `normalized_stt` — never `engine_raw`, never
//!    a post-processed representation;
//! 2. representations travel WITH the item, not as a second export;
//! 3. ids are stable across runs and audio metadata survives.

use handy_app_lib::delivery::pipeline::{
    record_dictation, record_failed_dictation, DictationInput,
};
use handy_app_lib::knowledge::{
    exportable_items, note_item, transcription_item, KnowledgeError, KnowledgeItem, KnowledgeKind,
    KNOWLEDGE_CONTRACT_VERSION,
};
use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::notes::{
    append_version, create_note, trash_note, NewNote, SOURCE_DICTATION, SOURCE_MANUAL_EDIT,
};

fn fresh(tag: &str) -> AppDatabase {
    let dir = std::env::temp_dir().join(format!("handy-kb401-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    db
}

/// A dictation with post-processing: the interesting case, because three
/// different texts exist and only one of them is the canonical raw.
fn seed_dictation(db: &AppDatabase) -> String {
    let source = record_dictation(
        db,
        &DictationInput {
            title: "Standup notes",
            audio_file_name: Some("handy-1.wav"),
            audio_sha256: Some("abc123"),
            audio_size_bytes: Some(4096),
            engine_raw: "um so the api is  down",
            normalized_stt: "So the API is down",
            model_id: Some("whisper-small"),
            language: Some("en"),
            normalizer_version: "1",
            post_processed_text: Some("The API is down."),
            post_process_prompt: Some("Clean up filler words."),
        },
    )
    .unwrap();
    source.capture_id.unwrap()
}

/// 01 — the canonical raw is `normalized_stt`. Exporting `engine_raw` would
/// leak filler words into the knowledge base; exporting the post-processed
/// text would make an LLM rewrite the durable record.
#[test]
fn canonical_raw_is_normalized_stt() {
    let db = fresh("01");
    let capture_id = seed_dictation(&db);

    let item = transcription_item(&db, &capture_id).unwrap();

    assert_eq!(item.raw, "So the API is down");
    assert_ne!(
        item.raw, "um so the api is  down",
        "engine_raw must not leak"
    );
    assert_ne!(item.raw, "The API is down.", "derived text is not the raw");
}

/// 02 — representations travel with the item.
#[test]
fn representations_are_included_with_the_item() {
    let db = fresh("02");
    let capture_id = seed_dictation(&db);

    let item = transcription_item(&db, &capture_id).unwrap();

    assert_eq!(item.representations.len(), 1);
    assert_eq!(item.representations[0].kind, "post_process");
    assert_eq!(item.representations[0].text, "The API is down.");
    assert!(!item.representations[0].id.is_empty());
}

/// 03 — ids are stable: building the same item twice yields the same
/// `export_id`. KB-402's idempotency key depends on this.
#[test]
fn export_id_is_stable_and_names_its_source() {
    let db = fresh("03");
    let capture_id = seed_dictation(&db);

    let first = transcription_item(&db, &capture_id).unwrap();
    let second = transcription_item(&db, &capture_id).unwrap();

    assert_eq!(first.export_id, second.export_id);
    assert_eq!(first.export_id, format!("capture:{capture_id}"));
    assert_eq!(first.source_id, capture_id);
    assert_eq!(first.attempt_id, second.attempt_id);
}

/// 04 — audio metadata survives into the export.
#[test]
fn audio_metadata_is_carried() {
    let db = fresh("04");
    let capture_id = seed_dictation(&db);

    let item = transcription_item(&db, &capture_id).unwrap();

    let audio = item.audio.expect("a capture with a WAV must carry audio");
    assert_eq!(audio.file_name.as_deref(), Some("handy-1.wav"));
    assert_eq!(audio.sha256.as_deref(), Some("abc123"));
    assert_eq!(audio.bytes, Some(4096));
    assert_eq!(audio.integrity_state, "audio_valid");
}

/// 05 — a capture whose transcription failed has no canonical text, so it is
/// NOT exportable. Exporting it would put an empty item in the knowledge base.
#[test]
fn a_capture_without_canonical_attempt_is_not_exportable() {
    let db = fresh("05");
    let capture_id =
        record_failed_dictation(&db, "Failed", Some("h.wav"), Some("x"), Some(1), "boom").unwrap();

    let err = transcription_item(&db, &capture_id).unwrap_err();

    assert!(matches!(err, KnowledgeError::NoCanonicalAttempt));
}

/// 06 — a note exports its CURRENT content, not its first version.
#[test]
fn note_exports_current_content() {
    let db = fresh("06");
    let (note, _) = create_note(
        &db,
        &NewNote {
            title: "Ideas".into(),
            first_version_content: "first draft".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    append_version(&db, &note.id, "second draft", SOURCE_MANUAL_EDIT).unwrap();

    let item = note_item(&db, &note.id).unwrap();

    assert_eq!(item.kind, KnowledgeKind::Note);
    assert_eq!(item.raw, "second draft");
    assert_eq!(item.title, "Ideas");
    assert_eq!(item.export_id, format!("note:{}", note.id));
    assert!(item.audio.is_none());
    assert!(item.representations.is_empty());
}

/// 07 — the DTO serializes to stable snake_case JSON carrying its contract
/// version and RFC3339 timestamps, and round-trips.
#[test]
fn dto_serializes_stably_and_round_trips() {
    let db = fresh("07");
    let capture_id = seed_dictation(&db);
    let item = transcription_item(&db, &capture_id).unwrap();

    let json = serde_json::to_value(&item).unwrap();
    assert_eq!(json["contract_version"], KNOWLEDGE_CONTRACT_VERSION);
    assert_eq!(json["kind"], "transcription");
    assert_eq!(json["raw"], "So the API is down");
    assert!(
        json.get("engine_raw").is_none(),
        "engine_raw is not exported"
    );
    assert!(json["representations"].is_array());

    // RFC3339 UTC, not epoch millis: the export must be readable by tools
    // that never saw our schema.
    let created = json["created_at"].as_str().unwrap();
    assert!(
        created.ends_with('Z'),
        "expected RFC3339 UTC, got {created}"
    );
    chrono::DateTime::parse_from_rfc3339(created).expect("created_at must parse as RFC3339");

    let back: KnowledgeItem = serde_json::from_value(json).unwrap();
    assert_eq!(back.export_id, item.export_id);
    assert_eq!(back.raw, item.raw);
    assert_eq!(back.representations.len(), item.representations.len());
}

/// 08 — the export service lists what is exportable: both kinds, no trashed
/// notes, no capture without canonical text.
#[test]
fn exportable_items_lists_both_kinds_and_skips_the_unexportable() {
    let db = fresh("08");
    let capture_id = seed_dictation(&db);
    record_failed_dictation(&db, "Failed", None, None, None, "boom").unwrap();

    let (kept, _) = create_note(
        &db,
        &NewNote {
            title: "Kept".into(),
            first_version_content: "keep me".into(),
            source: SOURCE_MANUAL_EDIT,
        },
    )
    .unwrap();
    let (trashed, _) = create_note(
        &db,
        &NewNote {
            title: "Trashed".into(),
            first_version_content: "drop me".into(),
            source: SOURCE_MANUAL_EDIT,
        },
    )
    .unwrap();
    trash_note(&db, &trashed.id).unwrap();

    let items = exportable_items(&db, 100).unwrap();

    let ids: Vec<&str> = items.iter().map(|i| i.export_id.as_str()).collect();
    assert!(ids.contains(&format!("capture:{capture_id}").as_str()));
    assert!(ids.contains(&format!("note:{}", kept.id).as_str()));
    assert!(
        !ids.contains(&format!("note:{}", trashed.id).as_str()),
        "trashed notes are not exported"
    );
    assert_eq!(items.len(), 2, "the failed capture must not be exported");
}
