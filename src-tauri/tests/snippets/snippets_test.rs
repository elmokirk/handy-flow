//! SNIP-211 fixtures: deterministic snippet matching per ADR-020 plus
//! persistence and conflict cases (15+ required).

use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::SnippetManager;

fn fresh(tag: &str) -> SnippetManager {
    let dir = std::env::temp_dir().join(format!("handy-snip-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    SnippetManager::new(db)
}

fn add(m: &SnippetManager, trigger: &str, replacement: &str, priority: i64) {
    m.upsert(trigger.into(), replacement.into(), priority, true)
        .unwrap();
}

// ---- Persistence -----------------------------------------------------------

#[test]
fn fixture_01_upsert_and_list_roundtrip() {
    let m = fresh("f01");
    add(&m, "adresse", "Musterstraße 12", 0);
    let all = m.list().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].trigger, "adresse");
    assert_eq!(all[0].replacement, "Musterstraße 12");
}

#[test]
fn fixture_02_trigger_case_insensitive_unique() {
    let m = fresh("f02");
    add(&m, "Adresse", "A1", 0);
    add(&m, "adresse", "A2", 0);
    let all = m.list().unwrap();
    assert_eq!(all.len(), 1, "case variants merge");
    assert_eq!(all[0].replacement, "A2", "upsert overwrites");
}

#[test]
fn fixture_03_delete_removes() {
    let m = fresh("f03");
    let s = m
        .upsert("kunde".into(), "Acme GmbH".into(), 0, true)
        .unwrap();
    assert!(m.delete(&s.id).unwrap());
    assert!(m.list().unwrap().is_empty());
}

// ---- Matching semantics (ADR-020) ------------------------------------------

#[test]
fn fixture_04_single_word_exact_replacement() {
    let m = fresh("f04");
    add(&m, "email", "max@beispiel.de", 0);
    assert_eq!(
        m.apply_snippets("schick mir deine email bitte").unwrap(),
        "schick mir deine max@beispiel.de bitte"
    );
}

#[test]
fn fixture_05_multi_word_phrase_trigger() {
    let m = fresh("f05");
    add(&m, "meine adresse", "Hauptplatz 1, 1010 Wien", 0);
    assert_eq!(
        m.apply_snippets("sende meine adresse an tom").unwrap(),
        "sende Hauptplatz 1, 1010 Wien an tom"
    );
}

#[test]
fn fixture_06_case_insensitive_match_replacement_verbatim() {
    let m = fresh("f06");
    add(&m, "Gruesse", "Beste Grüße", 0);
    assert_eq!(
        m.apply_snippets("GRUESSE an alle").unwrap(),
        "Beste Grüße an alle"
    );
}

#[test]
fn fixture_07_punctuation_attached_is_preserved() {
    let m = fresh("f07");
    add(&m, "adresse", "Hauptstraße 5", 0);
    // Punctuation glued to the token survives around the replacement.
    assert_eq!(
        m.apply_snippets("meine adresse, ok?").unwrap(),
        "meine Hauptstraße 5, ok?"
    );
}

#[test]
fn fixture_08_no_substring_matches_only_whole_phrases() {
    let m = fresh("f08");
    add(&m, "mail", "Mail", 0);
    // "emails" contains "mail" but is a DIFFERENT token -> untouched.
    assert_eq!(
        m.apply_snippets("zwei emails erhalten").unwrap(),
        "zwei emails erhalten"
    );
}

#[test]
fn fixture_09_longest_match_wins_over_shorter_prefix() {
    let m = fresh("f09");
    add(&m, "sig", "--- Signatur ---", 0);
    add(&m, "lange sig", "[vollständige Signatur]", 0);
    assert_eq!(
        m.apply_snippets("das ist meine lange sig Ende").unwrap(),
        "das ist meine [vollständige Signatur] Ende"
    );
}

#[test]
fn fixture_10_conflict_same_length_priority_decides() {
    let m = fresh("f10");
    add(&m, "kunde", "Niedrig", 0);
    add(&m, "KUNDE", "Hoch", 5); // same length, higher priority
    assert_eq!(m.apply_snippets("hallo kunde").unwrap(), "hallo Hoch");
}

#[test]
fn fixture_11_conflict_equal_priority_id_breaks_tie_deterministically() {
    // Two distinct same-length triggers, equal priority: repository order
    // (id ASC = insertion order for UUIDv7) decides deterministically.
    let m = fresh("f11");
    add(&m, "aaa", "Erste", 0);
    add(&m, "bbb", "Zweite", 0);
    assert_eq!(m.apply_snippets("aaa").unwrap(), "Erste");
}

#[test]
fn fixture_12_disabled_snippet_ignored() {
    let m = fresh("f12");
    m.upsert("adresse".into(), "X".into(), 0, false).unwrap();
    assert_eq!(m.apply_snippets("meine adresse").unwrap(), "meine adresse");
}

#[test]
fn fixture_13_replacement_not_rescanned_single_pass() {
    let m = fresh("f13");
    add(&m, "a", "b a b", 0); // replacement contains the trigger itself
    add(&m, "b", "B!", 0);
    // Single pass: replacement tokens are never matched again.
    assert_eq!(m.apply_snippets("x a y").unwrap(), "x b a b y");
}

#[test]
fn fixture_14_multiple_occurrences_all_replaced() {
    let m = fresh("f14");
    add(&m, "email", "max@x.de", 0);
    assert_eq!(
        m.apply_snippets("email eins, email zwei").unwrap(),
        "max@x.de eins, max@x.de zwei"
    );
}

#[test]
fn fixture_15_unicode_triggers_work() {
    let m = fresh("f15");
    add(&m, "müll", "Müllabfuhr ✓", 0);
    assert_eq!(
        m.apply_snippets("heute ist müll tag").unwrap(),
        "heute ist Müllabfuhr ✓ tag"
    );
}

#[test]
fn fixture_16_empty_input_and_no_snippets_identity() {
    let m = fresh("f16");
    assert_eq!(m.apply_snippets("").unwrap(), "");
    assert_eq!(m.apply_snippets("normaler text").unwrap(), "normaler text");
}

#[test]
fn fixture_17_conflict_overlap_middle_position_longest_wins() {
    let m = fresh("f17");
    add(&m, "bitte", "BITTE-KURZ", 9); // higher priority but SHORTER
    add(&m, "ganz bitte sehr", "LANG-GEWANN", 0);
    assert_eq!(
        m.apply_snippets("und ganz bitte sehr danke").unwrap(),
        "und LANG-GEWANN danke"
    );
}
