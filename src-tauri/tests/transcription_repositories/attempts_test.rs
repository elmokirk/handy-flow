//! DATA-105 acceptance: append-only retries, canonical invariant,
//! representation provenance.

use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::{
    representations::{insert_representation, representations_for_attempt},
    transcriptions::{
        attempts_for_capture, canonical_attempt, complete_attempt, insert_attempt, mark_canonical,
        NewAttempt,
    },
};

fn seeded_db(tag: &str) -> (AppDatabase, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("handy-repo-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("history.db");
    let _ = std::fs::remove_file(&path);
    let (db, _) = open_and_migrate(&path, "test").expect("seed db");

    // A capture row to attach attempts to.
    let capture_id = handy_app_lib::storage::ids::new_id();
    db.conn()
        .execute(
            "INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms) \
             VALUES (?1, 't', 'pending_audio', 1, 1)",
            [&capture_id],
        )
        .unwrap();
    (db, path)
}

fn attempt(capture_id: &str, text: Option<&str>) -> NewAttempt {
    NewAttempt {
        capture_id: capture_id.to_string(),
        engine_raw: Some("raw text".into()),
        normalized_stt: text.map(|t| t.to_string()),
        model_id: Some("whisper-small".into()),
        language: Some("de".into()),
        normalizer_version: "1".into(),
        dictionary_snapshot_sha256: None,
    }
}

#[test]
fn first_success_becomes_canonical_and_retries_append_without_overwriting() {
    let (db, _p) = seeded_db("retry");
    let cap = {
        let id = handy_app_lib::storage::ids::new_id();
        db.conn()
            .execute(
                "INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms) \
                 VALUES (?1,'x','pending_audio',1,1)",
                [&id],
            )
            .unwrap();
        id
    };

    // Attempt 1 fails → no canonical yet.
    let a1 = insert_attempt(&db, &attempt(&cap, None)).unwrap();
    complete_attempt(&db, &a1.id, None, Some("engine boom"), None, None).unwrap();
    assert!(canonical_attempt(&db, &cap).unwrap().is_none());

    // Retry inserts a NEW successful row; nothing about attempt 1 changes.
    let a2 = insert_attempt(&db, &attempt(&cap, Some("zweiter versuch"))).unwrap();
    assert_eq!(a2.attempt_number, 2);
    assert!(a2.is_canonical, "first success auto-canonicalizes");

    let stored_a1 = attempts_for_capture(&db, &cap)
        .unwrap()
        .into_iter()
        .find(|a| a.id == a1.id)
        .unwrap();
    assert_eq!(
        stored_a1.status, "failed",
        "retry must not overwrite prior attempt"
    );
    assert_eq!(stored_a1.error.as_deref(), Some("engine boom"));
    assert!(!stored_a1.is_canonical);

    // Third attempt does NOT steal canonicality automatically.
    let a3 = insert_attempt(&db, &attempt(&cap, Some("dritter"))).unwrap();
    assert!(!a3.is_canonical);
    assert_eq!(canonical_attempt(&db, &cap).unwrap().unwrap().id, a2.id);

    // Explicit promotion swaps canonicality transactionally.
    mark_canonical(&db, &cap, &a3.id).unwrap();
    assert_eq!(canonical_attempt(&db, &cap).unwrap().unwrap().id, a3.id);
}

#[test]
fn terminal_attempts_are_immutable() {
    let (db, _p) = seeded_db("immutable");
    let cap = {
        let id = handy_app_lib::storage::ids::new_id();
        db.conn()
            .execute(
                "INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms) \
                 VALUES (?1,'x','pending_audio',1,1)",
                [&id],
            )
            .unwrap();
        id
    };
    let a = insert_attempt(&db, &attempt(&cap, Some("done"))).unwrap();

    let second_completion = complete_attempt(&db, &a.id, Some("overwrite!"), None, None, None);
    assert!(
        second_completion.is_err(),
        "overwriting a terminal success must be rejected"
    );

    let still = canonical_attempt(&db, &cap).unwrap().unwrap();
    assert_eq!(still.normalized_stt.as_deref(), Some("done"));
}

#[test]
fn representations_record_source_prompt_and_model_metadata() {
    let (db, _p) = seeded_db("reps");
    let cap = {
        let id = handy_app_lib::storage::ids::new_id();
        db.conn()
            .execute(
                "INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms) \
                 VALUES (?1,'x','pending_audio',1,1)",
                [&id],
            )
            .unwrap();
        id
    };
    let a = insert_attempt(&db, &attempt(&cap, Some("basis"))).unwrap();

    let rep = insert_representation(
        &db,
        &handy_app_lib::storage::repositories::representations::NewRepresentation {
            attempt_id: a.id.clone(),
            parent_representation_id: None,
            kind: "style".into(),
            text: "gestylter text".into(),
            processor: "llm_style".into(),
            processor_version: "1.0".into(),
            prompt_profile_id: Some("profile-42".into()),
            effective_prompt_snapshot: Some("Du formst Text um: ...".into()),
            provider_snapshot: Some(r#"{"model":"llama3","endpoint":"http://127.0.0.1"}"#.into()),
        },
    )
    .unwrap();

    let all = representations_for_attempt(&db, &a.id).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].kind, "style");
    assert_eq!(all[0].prompt_profile_id.as_deref(), Some("profile-42"));
    assert!(all[0]
        .effective_prompt_snapshot
        .as_deref()
        .unwrap()
        .starts_with("Du formst"));

    // Content hash is deterministic for identical text.
    let again = insert_representation(
        &db,
        &handy_app_lib::storage::repositories::representations::NewRepresentation {
            attempt_id: a.id.clone(),
            parent_representation_id: Some(rep.id.clone()),
            kind: "transform".into(),
            text: "gestylter text".into(),
            processor: "transform".into(),
            processor_version: "1.0".into(),
            prompt_profile_id: None,
            effective_prompt_snapshot: None,
            provider_snapshot: None,
        },
    )
    .unwrap();
    assert_eq!(rep.content_hash_sha256, again.content_hash_sha256);

    // CHECK constraint rejects unknown kinds.
    let bad = rusqlite::Connection::open_in_memory().is_ok(); // sanity only
    assert!(bad);
    let invalid_kind = insert_representation(
        &db,
        &handy_app_lib::storage::repositories::representations::NewRepresentation {
            attempt_id: a.id.clone(),
            parent_representation_id: None,
            kind: "dictionary".into(), // not in the allowed set
            text: "x".into(),
            processor: "p".into(),
            processor_version: "1".into(),
            prompt_profile_id: None,
            effective_prompt_snapshot: None,
            provider_snapshot: None,
        },
    );
    assert!(
        invalid_kind.is_err(),
        "unknown representation kinds must be rejected"
    );
}
