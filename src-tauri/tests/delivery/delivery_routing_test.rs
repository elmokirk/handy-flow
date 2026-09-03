//! PAD-305 acceptance: transcript delivery routing through sinks.
//!
//! `DeliveryDestination` and `record_delivery` were frozen in DATA-107 but
//! were called from nowhere. This suite covers the seam that wires them:
//! a `DeliverySink` per destination, and a router that delivers once and
//! records exactly one audit event per attempt.
//!
//! Per DATA-107 ("failed delivery can be diagnosed without mutating
//! source") FAILURES are recorded too — one event either way.
//!
//! `FocusedAppSink`/`ClipboardSink` need a live `AppHandle` and are not
//! constructible here; the trait is exercised through fakes plus the real
//! `ScratchpadSink`, which depends only on `NotesManager`.

use handy_app_lib::delivery::{
    deliver_with_audit, DeliveryError, DeliverySink, DeliverySource, ScratchpadSink,
    SqliteDeliveryAudit,
};
use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::models::{DeliveryDestination, DeliverySourceKind};
use handy_app_lib::storage::repositories::deliveries::events_for_capture;
use handy_app_lib::storage::repositories::notes as noterepo;
use handy_app_lib::storage::repositories::representations::content_hash;
use handy_app_lib::NotesManager;
use std::cell::RefCell;

fn fresh(tag: &str) -> (AppDatabase, NotesManager) {
    let dir = std::env::temp_dir().join(format!("handy-pad305-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    let notes = NotesManager::new(db.clone());
    (db, notes)
}

/// Seed capture + attempt so delivery events satisfy the NOT NULL FK on
/// `source_attempt_id`.
fn seed_source(db: &AppDatabase) -> DeliverySource {
    let capture_id = handy_app_lib::storage::ids::new_id();
    let attempt_id = handy_app_lib::storage::ids::new_id();
    db.conn()
        .execute_batch(&format!(
            "INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms)
             VALUES ('{capture_id}', 't', 'audio_valid', 1, 1);
            INSERT INTO transcription_attempts(id, capture_id, attempt_number, normalized_stt,
                provenance, status, is_canonical, created_at_ms)
             VALUES ('{attempt_id}', '{capture_id}', 1, 'text', 'live', 'success', 1, 1);"
        ))
        .unwrap();
    DeliverySource {
        capture_id: Some(capture_id),
        attempt_id,
        representation_id: None,
        kind: DeliverySourceKind::NormalizedStt,
    }
}

/// Records what it was asked to deliver; optionally fails.
struct FakeSink {
    destination: DeliveryDestination,
    fail_with: Option<String>,
    delivered: RefCell<Vec<String>>,
}

impl FakeSink {
    fn ok(destination: DeliveryDestination) -> Self {
        Self {
            destination,
            fail_with: None,
            delivered: RefCell::new(Vec::new()),
        }
    }
    fn failing(destination: DeliveryDestination, msg: &str) -> Self {
        Self {
            destination,
            fail_with: Some(msg.into()),
            delivered: RefCell::new(Vec::new()),
        }
    }
}

impl DeliverySink for FakeSink {
    fn destination(&self) -> DeliveryDestination {
        self.destination
    }
    fn deliver(&self, text: &str) -> Result<(), DeliveryError> {
        self.delivered.borrow_mut().push(text.to_string());
        match &self.fail_with {
            Some(m) => Err(DeliveryError::new(m)),
            None => Ok(()),
        }
    }
}

#[test]
fn fixture_01_scratchpad_sink_appends_a_dictation_version() {
    let (_db, notes) = fresh("pad-append");
    let (note, _) = notes.create("Pad".into(), "erste zeile".into()).unwrap();

    let sink = ScratchpadSink::new(notes.clone());
    sink.deliver("diktierter satz").unwrap();

    assert_eq!(sink.destination(), DeliveryDestination::Scratchpad);
    let versions = notes.versions(&note.id).unwrap();
    assert_eq!(versions.len(), 2, "delivery appends, never overwrites");
    assert_eq!(versions[0].source, noterepo::SOURCE_DICTATION);
    assert!(
        versions[0].content.contains("diktierter satz"),
        "dictated text reaches the note"
    );
    assert_eq!(
        versions[1].content, "erste zeile",
        "previous version untouched"
    );
}

#[test]
fn fixture_02_scratchpad_sink_creates_a_note_when_none_exists() {
    let (_db, notes) = fresh("pad-create");
    assert!(notes.list().unwrap().is_empty());

    ScratchpadSink::new(notes.clone())
        .deliver("erster diktierter satz")
        .unwrap();

    let all = notes.list().unwrap();
    assert_eq!(all.len(), 1, "dictating with no note creates exactly one");
    let versions = notes.versions(&all[0].id).unwrap();
    assert_eq!(versions[0].source, noterepo::SOURCE_DICTATION);
    assert_eq!(versions[0].content, "erster diktierter satz");
}

#[test]
fn fixture_03_successful_delivery_records_exactly_one_event() {
    let (db, _notes) = fresh("audit-ok");
    let source = seed_source(&db);
    let audit = SqliteDeliveryAudit::new(db.clone());
    let sink = FakeSink::ok(DeliveryDestination::FocusedApp);

    deliver_with_audit(&sink, &audit, &source, "hallo welt").unwrap();

    assert_eq!(sink.delivered.borrow().as_slice(), ["hallo welt"]);
    let events = events_for_capture(&db, source.capture_id.as_ref().unwrap()).unwrap();
    assert_eq!(events.len(), 1, "exactly one event per delivery");
    let e = &events[0];
    assert!(e.success);
    assert_eq!(e.destination.as_deref(), Some("focused_app"));
    assert_eq!(e.source_attempt_id, source.attempt_id);
    assert_eq!(
        e.text_sha256,
        content_hash("hallo welt"),
        "hash identifies the exact delivered text"
    );
    assert!(e.error.is_none());
}

#[test]
fn fixture_04_failed_delivery_is_recorded_for_diagnosis() {
    // DATA-107: "failed delivery can be diagnosed without mutating source".
    let (db, _notes) = fresh("audit-fail");
    let source = seed_source(&db);
    let audit = SqliteDeliveryAudit::new(db.clone());
    let sink = FakeSink::failing(DeliveryDestination::FocusedApp, "target refused paste");

    let err = deliver_with_audit(&sink, &audit, &source, "text").unwrap_err();
    assert!(err.to_string().contains("target refused paste"));

    let events = events_for_capture(&db, source.capture_id.as_ref().unwrap()).unwrap();
    assert_eq!(events.len(), 1, "a failure still records exactly one event");
    assert!(!events[0].success);
    assert_eq!(
        events[0].error.as_deref(),
        Some("target refused paste"),
        "the reason is preserved for diagnosis"
    );
    assert_eq!(events[0].destination.as_deref(), Some("focused_app"));
}

#[test]
fn fixture_05_destination_is_recorded_per_sink_without_touching_the_paste_path() {
    // Switching destination = swapping the sink. Nothing else changes.
    let (db, _notes) = fresh("audit-dest");
    let audit = SqliteDeliveryAudit::new(db.clone());

    for (dest, expected) in [
        (DeliveryDestination::FocusedApp, "focused_app"),
        (DeliveryDestination::Scratchpad, "scratchpad"),
        (DeliveryDestination::Clipboard, "clipboard"),
    ] {
        let source = seed_source(&db);
        deliver_with_audit(&FakeSink::ok(dest), &audit, &source, "t").unwrap();
        let events = events_for_capture(&db, source.capture_id.as_ref().unwrap()).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].destination.as_deref(), Some(expected));
    }
}

#[test]
fn fixture_06_retries_append_rather_than_overwrite() {
    // DATA-107: "retries create new events rather than overwrite".
    let (db, _notes) = fresh("audit-retry");
    let source = seed_source(&db);
    let audit = SqliteDeliveryAudit::new(db.clone());

    let failing = FakeSink::failing(DeliveryDestination::FocusedApp, "busy");
    assert!(deliver_with_audit(&failing, &audit, &source, "text").is_err());
    deliver_with_audit(
        &FakeSink::ok(DeliveryDestination::FocusedApp),
        &audit,
        &source,
        "text",
    )
    .unwrap();

    let events = events_for_capture(&db, source.capture_id.as_ref().unwrap()).unwrap();
    assert_eq!(events.len(), 2, "retry adds an event, never mutates");
    assert!(!events[0].success);
    assert!(events[1].success);
}

#[test]
fn fixture_07_scratchpad_delivery_records_a_scratchpad_event() {
    // End-to-end for the dictation target: text lands in a note version AND
    // the audit names the scratchpad.
    let (db, notes) = fresh("e2e");
    let source = seed_source(&db);
    let audit = SqliteDeliveryAudit::new(db.clone());
    let sink = ScratchpadSink::new(notes.clone());

    deliver_with_audit(&sink, &audit, &source, "diktat ins notizfeld").unwrap();

    let all = notes.list().unwrap();
    assert_eq!(all.len(), 1);
    let versions = notes.versions(&all[0].id).unwrap();
    assert_eq!(versions[0].source, noterepo::SOURCE_DICTATION);
    assert!(versions[0].content.contains("diktat ins notizfeld"));

    let events = events_for_capture(&db, source.capture_id.as_ref().unwrap()).unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].success);
    assert_eq!(events[0].destination.as_deref(), Some("scratchpad"));
    assert_eq!(events[0].text_sha256, content_hash("diktat ins notizfeld"));
}

#[test]
fn fixture_08_audit_failure_does_not_swallow_a_successful_delivery() {
    // The audit is an observer: if the event cannot be written, the text was
    // still delivered and the caller must not be told otherwise.
    struct BrokenAudit;
    impl handy_app_lib::delivery::DeliveryAudit for BrokenAudit {
        fn record(
            &self,
            _source: &DeliverySource,
            _destination: DeliveryDestination,
            _text: &str,
            _outcome: Result<(), &DeliveryError>,
        ) -> Result<(), String> {
            Err("audit db is gone".into())
        }
    }

    let (db, _notes) = fresh("audit-broken");
    let source = seed_source(&db);
    let sink = FakeSink::ok(DeliveryDestination::FocusedApp);

    let result = deliver_with_audit(&sink, &BrokenAudit, &source, "text");
    assert!(
        result.is_ok(),
        "a broken audit must not turn a delivered transcript into an error"
    );
    assert_eq!(sink.delivered.borrow().len(), 1);
}
