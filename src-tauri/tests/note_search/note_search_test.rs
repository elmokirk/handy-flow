//! NOTE-304 acceptance: notes search and restore.
//!
//! `04-DATA_PERSISTENCE.md` freezes the FTS index over "normalized_stt,
//! representations and active notes" — HIST-231 delivered the first two,
//! this ticket adds the third. The index stays DERIVED and
//! repository-managed: canonical note writes keep the FTS document in
//! sync inside the same transaction, and a full rebuild can always
//! repopulate it from the notes tables.
//!
//! Restore stays append-only (ADR-024): restoring an old version appends
//! a NEW version and never rewrites history, and it must not disturb the
//! note's own metadata (pinned/title).

use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::notes as repo;
use handy_app_lib::storage::repositories::search as srepo;
use handy_app_lib::NotesManager;

fn fresh(tag: &str) -> (NotesManager, handy_app_lib::storage::database::AppDatabase) {
    let dir = std::env::temp_dir().join(format!("handy-note304-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    (NotesManager::new(db.clone()), db)
}

/// Note ids returned for a query, in FTS result order.
fn search_ids(notes: &NotesManager, q: &str, limit: i64) -> Vec<String> {
    notes
        .search(q, limit, 0)
        .unwrap()
        .into_iter()
        .map(|n| n.id)
        .collect()
}

#[test]
fn fixture_01_rebuild_indexes_active_notes() {
    let (notes, db) = fresh("rebuild");
    notes
        .create("Rust".into(), "ownership und borrowing".into())
        .unwrap();
    notes
        .create("Einkauf".into(), "milch brot butter".into())
        .unwrap();

    // Rebuild is the recovery path: truncate + repopulate from canonical
    // tables. Both notes must come back.
    let n = srepo::rebuild_search_index(&db).unwrap();
    assert_eq!(n, 2, "two active notes, no transcripts seeded");

    // Idempotent.
    assert_eq!(srepo::rebuild_search_index(&db).unwrap(), 2);

    let hits = srepo::search(&db, "ownership", Some(srepo::REF_NOTE), 50, 0).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].ref_type, srepo::REF_NOTE);
}

#[test]
fn fixture_02_search_finds_notes_without_explicit_rebuild() {
    let (notes, _db) = fresh("live");
    let (a, _) = notes
        .create("Rust".into(), "ownership und borrowing".into())
        .unwrap();
    notes
        .create("Einkauf".into(), "milch brot butter".into())
        .unwrap();

    // No rebuild call: the write path itself keeps the index current.
    let ids = search_ids(&notes, "ownership", 50);
    assert_eq!(ids, vec![a.id]);
}

#[test]
fn fixture_03_appended_version_replaces_old_content_in_index() {
    let (notes, _db) = fresh("append");
    let (note, _) = notes
        .create("Notiz".into(), "quantenverschraenkung".into())
        .unwrap();

    assert_eq!(search_ids(&notes, "quantenverschraenkung", 50).len(), 1);

    notes
        .save_content(&note.id, "gaertnern im herbst", repo::SOURCE_MANUAL_EDIT)
        .unwrap()
        .unwrap();

    // The FTS document tracks CURRENT content: superseded text must not
    // keep returning hits, or search would surface stale versions.
    assert!(
        search_ids(&notes, "quantenverschraenkung", 50).is_empty(),
        "superseded content must leave the index"
    );
    assert_eq!(search_ids(&notes, "gaertnern", 50), vec![note.id]);
}

#[test]
fn fixture_04_trashed_notes_leave_the_index_and_return_on_restore() {
    let (notes, _db) = fresh("trash");
    let (note, _) = notes.create("Geheim".into(), "spargelzeit".into()).unwrap();
    assert_eq!(search_ids(&notes, "spargelzeit", 50).len(), 1);

    assert!(notes.trash(&note.id).unwrap());
    assert!(
        search_ids(&notes, "spargelzeit", 50).is_empty(),
        "only ACTIVE notes are indexed"
    );

    assert!(notes.restore(&note.id).unwrap());
    assert_eq!(search_ids(&notes, "spargelzeit", 50), vec![note.id]);
}

#[test]
fn fixture_05_search_is_bounded_and_syntax_safe() {
    let (notes, _db) = fresh("bounded");
    for i in 0..25 {
        notes
            .create(format!("N{i}"), format!("gemeinsam wort nummer {i}"))
            .unwrap();
    }

    assert_eq!(
        search_ids(&notes, "gemeinsam", 5).len(),
        5,
        "limit honoured"
    );
    // Limit is clamped, never unbounded: 0 and negatives cannot widen it.
    assert_eq!(notes.search("gemeinsam", 0, 0).unwrap().len(), 1);
    assert_eq!(notes.search("gemeinsam", -10, 0).unwrap().len(), 1);
    assert_eq!(notes.search("gemeinsam", 10_000, 0).unwrap().len(), 25);

    // FTS5 operators in user input must not break or widen the query.
    for hostile in ["\"", "NOT gemeinsam", "gemeinsam OR *", "*", "^", "(("] {
        let r = notes.search(hostile, 50, 0);
        assert!(r.is_ok(), "hostile input must not error: {hostile:?}");
    }
    // An all-punctuation query yields nothing rather than everything.
    assert!(notes.search("*", 50, 0).unwrap().is_empty());
}

#[test]
fn fixture_06_restore_version_is_append_only_and_keeps_note_metadata() {
    let (notes, _db) = fresh("restore");
    let (note, _) = notes.create("Titel".into(), "fassung eins".into()).unwrap();
    notes
        .save_content(&note.id, "fassung zwei", repo::SOURCE_MANUAL_EDIT)
        .unwrap()
        .unwrap();
    notes.set_pinned(&note.id, true).unwrap();
    notes.set_title(&note.id, "Wichtig").unwrap();

    let restored = notes.restore_version(&note.id, 1).unwrap().unwrap();

    // Append-only: version 3 carries version 1's content; nothing rewritten.
    assert_eq!(restored.version_no, 3);
    assert_eq!(restored.content, "fassung eins");
    assert_eq!(restored.source, repo::SOURCE_RESTORE);
    let versions = notes.versions(&note.id).unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[2].content, "fassung eins", "v1 still intact");
    assert_eq!(versions[1].content, "fassung zwei", "v2 still intact");

    // Restoring content must not clobber the note's own state.
    let after = notes.get(&note.id).unwrap().unwrap();
    assert!(after.pinned, "pinned state preserved across restore");
    assert_eq!(after.title, "Wichtig", "title preserved across restore");

    // The index follows the restore.
    assert_eq!(search_ids(&notes, "eins", 50), vec![note.id]);
    assert!(search_ids(&notes, "zwei", 50).is_empty());
}

#[test]
fn fixture_07_pinned_notes_rank_first_in_search_results() {
    let (notes, _db) = fresh("pinned");
    let (plain, _) = notes
        .create("A".into(), "gemeinsames stichwort alpha".into())
        .unwrap();
    let (pinned, _) = notes
        .create("B".into(), "gemeinsames stichwort beta".into())
        .unwrap();
    notes.set_pinned(&pinned.id, true).unwrap();

    let hits = notes.search("stichwort", 50, 0).unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(
        hits[0].id, pinned.id,
        "pinned notes surface first, matching list() ordering"
    );
    assert!(hits[0].pinned);
    assert_eq!(hits[1].id, plain.id);
}

#[test]
fn fixture_08_rebuild_recovers_a_corrupted_index() {
    let (notes, db) = fresh("recover");
    let (note, _) = notes
        .create("Notiz".into(), "wiederherstellbar".into())
        .unwrap();

    // Simulate index loss (the recovery requirement in 04-DATA_PERSISTENCE).
    db.conn().execute("DELETE FROM search_fts", []).unwrap();
    assert!(search_ids(&notes, "wiederherstellbar", 50).is_empty());

    srepo::rebuild_search_index(&db).unwrap();
    assert_eq!(search_ids(&notes, "wiederherstellbar", 50), vec![note.id]);
}
