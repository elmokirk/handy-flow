//! NOTE-301 acceptance: append-only versions, hash dedupe, restore
//! creates new version, trash/restore, typed source metadata.

use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::notes::{
    append_version, create_note, current_version, list_active_notes, note_versions, restore_note,
    set_pinned, trash_note, NewNote, SOURCE_DICTATION, SOURCE_RESTORE,
};

fn fresh(tag: &str) -> handy_app_lib::storage::database::AppDatabase {
    let dir = std::env::temp_dir().join(format!("handy-notes-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    open_and_migrate(dir.join("history.db"), "test").unwrap().0
}

#[test]
fn fixture_01_create_note_with_first_version() {
    let db = fresh("f01");
    let (note, ver) = create_note(
        &db,
        &NewNote {
            title: "Meeting".into(),
            first_version_content: "erste Notiz".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    assert_eq!(ver.version_no, 1);
    assert_eq!(ver.source, "dictation");
    let cur = current_version(&db, &note.id).unwrap().unwrap();
    assert_eq!(cur.content, "erste Notiz");
}

#[test]
fn fixture_02_append_versions_increase_monotonically() {
    let db = fresh("f02");
    let (note, _) = create_note(
        &db,
        &NewNote {
            title: "t".into(),
            first_version_content: "v1".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    append_version(&db, &note.id, "v2", SOURCE_DICTATION).unwrap();
    append_version(&db, &note.id, "v3", SOURCE_DICTATION).unwrap();
    let versions = note_versions(&db, &note.id).unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0].version_no, 3, "descending order, highest first");
    assert_eq!(
        current_version(&db, &note.id).unwrap().unwrap().content,
        "v3"
    );
}

#[test]
fn fixture_03_hash_dedupe_identical_content() {
    let db = fresh("f03");
    let (note, _) = create_note(
        &db,
        &NewNote {
            title: "t".into(),
            first_version_content: "same".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    let again = append_version(&db, &note.id, "same", SOURCE_DICTATION).unwrap();
    assert!(
        again.is_none(),
        "identical content must not create a version"
    );
    assert_eq!(note_versions(&db, &note.id).unwrap().len(), 1);
}

#[test]
fn fixture_04_restore_creates_new_version_not_rewrite() {
    let db = fresh("f04");
    let (note, _) = create_note(
        &db,
        &NewNote {
            title: "t".into(),
            first_version_content: "original".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    append_version(&db, &note.id, "bearbeitet", SOURCE_DICTATION).unwrap();

    // Restore version 1 content -> becomes version 3, source=restore.
    let restored = handy_app_lib::storage::repositories::notes::append_version(
        &db,
        &note.id,
        "original",
        SOURCE_RESTORE,
    )
    .unwrap()
    .unwrap();
    assert_eq!(restored.version_no, 3);
    assert_eq!(restored.source, "restore");

    // History intact: v1 still "original", v2 still "bearbeitet".
    let versions = note_versions(&db, &note.id).unwrap();
    assert_eq!(versions[2].content, "original");
    assert_eq!(versions[1].content, "bearbeitet");
    assert_eq!(versions[0].content, "original");
}

#[test]
fn fixture_05_trash_restore_roundtrip_excluded_from_active() {
    let db = fresh("f05");
    let (note, _) = create_note(
        &db,
        &NewNote {
            title: "t".into(),
            first_version_content: "c".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    assert!(trash_note(&db, &note.id).unwrap());
    assert!(list_active_notes(&db).unwrap().is_empty());
    assert!(restore_note(&db, &note.id).unwrap());
    assert_eq!(list_active_notes(&db).unwrap().len(), 1);
}

#[test]
fn fixture_06_pinned_notes_sort_first() {
    let db = fresh("f06");
    let (a, _) = create_note(
        &db,
        &NewNote {
            title: "A".into(),
            first_version_content: "a".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    create_note(
        &db,
        &NewNote {
            title: "B".into(),
            first_version_content: "b".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    set_pinned(&db, &a.id, true).unwrap();
    let list = list_active_notes(&db).unwrap();
    assert_eq!(list[0].id, a.id, "pinned first");
    assert!(list[0].pinned);
}

#[test]
fn fixture_07_source_metadata_is_typed_by_check() {
    let db = fresh("f07");
    let (note, _) = create_note(
        &db,
        &NewNote {
            title: "t".into(),
            first_version_content: "c".into(),
            source: SOURCE_DICTATION,
        },
    )
    .unwrap();
    // CHECK constraint rejects unknown sources.
    let bad = db.conn().execute(
        "INSERT INTO note_versions(id, note_id, version_no, content, content_hash_sha256, source, created_at_ms) \
         VALUES ('x', ?1, 99, 'c', 'h', 'magic', 1)",
        [&note.id],
    );
    assert!(bad.is_err(), "unknown source must be rejected");
}
