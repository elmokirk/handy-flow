//! DATA-107 acceptance: append-only delivery audit with hash identity.

use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::models::{DeliveryDestination, DeliverySourceKind};
use handy_app_lib::storage::repositories::deliveries::{
    events_for_capture, get_event, record_delivery, NewDeliveryEvent,
};
use handy_app_lib::storage::repositories::representations::content_hash;

fn fresh_db(tag: &str) -> AppDatabase {
    let dir = std::env::temp_dir().join(format!("handy-delivery-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    open_and_migrate(dir.join("history.db"), "test")
        .expect("db")
        .0
}

#[test]
fn delivery_audit_records_success_failure_and_hash_identity() {
    let db = fresh_db("audit");

    // Seed capture + attempt + representation to reference.
    let capture_id = handy_app_lib::storage::ids::new_id();
    let attempt_id = handy_app_lib::storage::ids::new_id();
    let rep_id = handy_app_lib::storage::ids::new_id();
    db.conn()
        .execute_batch(&format!(
            "INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms)
             VALUES ('{capture_id}', 't', 'audio_valid', 1, 1);
            INSERT INTO transcription_attempts(id, capture_id, attempt_number, normalized_stt,
                provenance, status, is_canonical, created_at_ms)
             VALUES ('{attempt_id}', '{capture_id}', 1, 'text', 'live', 'success', 1, 1);
            INSERT INTO representations(id, attempt_id, kind, text, status, created_at_ms)
             VALUES ('{rep_id}', '{attempt_id}', 'style', 'styled', 'success', 1);"
        ))
        .unwrap();

    // The delivered text IS the canonical source text (paste path).
    let delivered_text = "text";
    let hash = content_hash(delivered_text);

    // Successful focused-app delivery of the NORMALIZED source.
    let ok_event = record_delivery(
        &db,
        &NewDeliveryEvent {
            capture_id: Some(capture_id.clone()),
            source_attempt_id: attempt_id.clone(),
            representation_id: None,
            source_kind: DeliverySourceKind::NormalizedStt,
            destination: Some(DeliveryDestination::FocusedApp),
            text_sha256: hash.clone(),
            success: true,
            error: None,
        },
    )
    .unwrap();

    assert!(ok_event.success);
    assert_eq!(ok_event.text_sha256, hash);

    // Failed delivery of the REPRESENTATION (e.g. paste injection error).
    let fail_event = record_delivery(
        &db,
        &NewDeliveryEvent {
            capture_id: Some(capture_id.clone()),
            source_attempt_id: attempt_id.clone(),
            representation_id: Some(rep_id),
            source_kind: DeliverySourceKind::Representation,
            destination: Some(DeliveryDestination::FocusedApp),
            text_sha256: content_hash("styled"),
            success: false,
            error: Some("paste window lost focus".into()),
        },
    )
    .unwrap();
    assert!(!fail_event.success);

    // Diagnosability: failure reason retrievable WITHOUT touching sources.
    let loaded = get_event(&db, &fail_event.id).unwrap().unwrap();
    assert_eq!(loaded.error.as_deref(), Some("paste window lost focus"));

    // Hash matches referenced immutable source text.
    let stored_normalized: String = db
        .conn()
        .query_row(
            "SELECT normalized_stt FROM transcription_attempts WHERE id = ?1",
            [&attempt_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        get_event(&db, &ok_event.id).unwrap().unwrap().text_sha256,
        content_hash(&stored_normalized)
    );

    let events = events_for_capture(&db, &capture_id).unwrap();
    assert_eq!(events.len(), 2, "append-only: every delivery recorded once");
}
