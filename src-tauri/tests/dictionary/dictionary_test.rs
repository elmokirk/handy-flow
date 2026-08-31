//! DICT-201 acceptance: durable dictionary persistence, aliases,
//! legacy import and regression tests for the existing fuzzy matcher.

use handy_app_lib::audio_toolkit::text::apply_custom_words;
use handy_app_lib::storage::database::AppDatabase;
use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::DictionaryManager;

fn fresh(tag: &str) -> (DictionaryManager, AppDatabase) {
    let dir = std::env::temp_dir().join(format!("handy-dict-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    (DictionaryManager::new(db.clone()), db)
}

// ---- Persistence fixtures -------------------------------------------------

#[test]
fn fixture_01_upsert_and_list_roundtrip() {
    let (mgr, _db) = fresh("f01");
    mgr.upsert("Handy".into(), vec![], true).unwrap();
    let all = mgr.list().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].term, "Handy");
    assert!(all[0].enabled);
}

#[test]
fn fixture_02_term_is_case_insensitive_unique() {
    let (mgr, _db) = fresh("f02");
    mgr.upsert("Handy".into(), vec![], true).unwrap();
    mgr.upsert("handy".into(), vec!["händy".into()], true)
        .unwrap();
    let all = mgr.list().unwrap();
    assert_eq!(all.len(), 1, "case variants must merge into one entry");
    assert_eq!(all[0].aliases, vec!["händy".to_string()]);
}

#[test]
fn fixture_03_aliases_roundtrip() {
    let (mgr, _db) = fresh("f03");
    mgr.upsert(
        "Wispr".into(),
        vec!["wisper".into(), "whisper".into()],
        true,
    )
    .unwrap();
    let entry = &mgr.list().unwrap()[0];
    assert_eq!(entry.aliases.len(), 2);
}

#[test]
fn fixture_04_disabled_entries_are_excluded_from_correction() {
    let (mgr, _db) = fresh("f04");
    mgr.upsert("aktiv".into(), vec![], true).unwrap();
    mgr.upsert("inaktiv".into(), vec![], false).unwrap();
    let terms = mgr.correction_terms().unwrap();
    assert_eq!(terms, vec!["aktiv".to_string()]);
    assert_eq!(mgr.list().unwrap().len(), 2, "settings list shows both");
}

#[test]
fn fixture_05_correction_terms_flatten_term_and_aliases() {
    let (mgr, _db) = fresh("f05");
    mgr.upsert("Handy".into(), vec!["händy".into()], true)
        .unwrap();
    let mut terms = mgr.correction_terms().unwrap();
    terms.sort();
    assert_eq!(terms, vec!["Handy".to_string(), "händy".to_string()]);
}

#[test]
fn fixture_06_snapshot_hash_is_stable_and_changes_on_edit() {
    let (mgr, _db) = fresh("f06");
    mgr.upsert("Handy".into(), vec![], true).unwrap();
    let h1 = mgr.snapshot_sha256().unwrap();
    let h2 = mgr.snapshot_sha256().unwrap();
    assert_eq!(h1, h2, "hash must be deterministic");
    mgr.upsert("Wispr".into(), vec![], true).unwrap();
    assert_ne!(h1, mgr.snapshot_sha256().unwrap(), "edit changes snapshot");
}

#[test]
fn fixture_07_legacy_import_is_idempotent() {
    let (mgr, _db) = fresh("f07");
    let legacy = vec!["Handy".to_string(), "cjpais".to_string()];
    assert_eq!(mgr.import_legacy_custom_words(&legacy).unwrap(), 2);
    assert_eq!(mgr.import_legacy_custom_words(&legacy).unwrap(), 2);
    assert_eq!(mgr.list().unwrap().len(), 2, "no duplicates on re-import");
}

#[test]
fn fixture_08_delete_removes_entry() {
    let (mgr, _db) = fresh("f08");
    let e = mgr.upsert("weg".into(), vec![], true).unwrap();
    assert!(mgr.delete(&e.id).unwrap());
    assert!(!mgr.delete(&e.id).unwrap());
    assert!(mgr.list().unwrap().is_empty());
}

// ---- Matching regression fixtures -----------------------------------------
// These PIN the actual upstream matcher semantics (Levenshtein + Soundex
// boost + n-gram consumption + case-pattern preservation), including its
// known aggressive false positives. They are regression guards, not
// statements about desired product behavior.

#[test]
fn fixture_09_exact_match_is_replaced() {
    let out = apply_custom_words("ich nutze handy", &["Handy".into()], 0.8);
    assert_eq!(out, "ich nutze Handy");
}

#[test]
fn fixture_10_case_pattern_is_preserved_and_soundex_can_overreach() {
    // Documents current behavior: ALL-CAPS source keeps its pattern, and
    // the Soundex-boosted matcher pulls the neighbouring token into a
    // 2-gram replacement ("ist" consumed alongside "HANDY").
    let out = apply_custom_words("HANDY ist gut", &["Handy".into(), "Wispr".into()], 0.8);
    assert_eq!(out, "HANDY Wispr gut");
}

#[test]
fn fixture_11_non_ascii_candidates_are_skipped_by_the_fallback() {
    // The Soundex/ASCII fallback intentionally ignores non-ASCII words
    // like "händy"; they pass through untouched.
    let out = apply_custom_words("ich mag händy", &["Handy".into()], 0.8);
    assert_eq!(out, "ich mag händy");
}

#[test]
fn fixture_12_soundex_boost_can_false_positive_on_unrelated_words() {
    // Documents current behavior: "anderes" phonetically collides with
    // "Handy" once the Soundex boost scales its edit distance down.
    let out = apply_custom_words(
        "komplett anderes wort",
        &["Handy".into(), "Wispr".into()],
        0.8,
    );
    assert_eq!(out, "komplett Handy wort");
}

#[test]
fn fixture_13_empty_dictionary_is_identity() {
    let text = "irgendein text";
    assert_eq!(apply_custom_words(text, &[], 0.8), text);
}

#[test]
fn fixture_14_multiple_terms_all_applied() {
    // Documents current behavior of the same aggressive matching on a
    // two-term set ("und" is consumed into the HANDY replacement).
    let out = apply_custom_words("handy und wispr", &["Handy".into(), "Wispr".into()], 0.8);
    assert_eq!(out, "Handy Handy Wispr");
}

#[test]
fn fixture_15_multi_word_alias_flows_via_ngram_matching() {
    let (mgr, _db) = fresh("f15");
    mgr.upsert("SecondBrain".into(), vec!["second brain".into()], true)
        .unwrap();
    let terms = mgr.correction_terms().unwrap();
    let out = apply_custom_words("mein second brain eintrag", &terms, 0.8);
    assert_eq!(out, "mein SecondBrain eintrag");
}

#[test]
fn fixture_16_disabled_term_does_not_correct() {
    let (mgr, _db) = fresh("f16");
    mgr.upsert("Handy".into(), vec![], false).unwrap();
    let terms = mgr.correction_terms().unwrap();
    let text = "ich mag handy";
    assert_eq!(apply_custom_words(text, &terms, 0.8), text);
}
