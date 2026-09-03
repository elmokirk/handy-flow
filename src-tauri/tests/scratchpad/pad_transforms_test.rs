//! PAD-303 acceptance: dictation/transform routing into Note versions.
//!
//! The production transform path appends the LLM output as a NEW note
//! version (source = `transform`) while the source version stays
//! untouched and restorable. The LLM boundary is exercised with the same
//! mock-transport pattern as PROMPT-221 (`LlmTransport` trait).

use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::notes as repo;
use handy_app_lib::storage::repositories::prompt_profiles as pprepo;
use handy_app_lib::{LlmTransport, PromptProfileManager, PromptProfileRecord, KIND_TRANSFORM};

fn fresh_notes(tag: &str) -> (handy_app_lib::NotesManager, PromptProfileManager) {
    let dir = std::env::temp_dir().join(format!("handy-pad303-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    (
        handy_app_lib::NotesManager::new(db.clone()),
        PromptProfileManager::new(db),
    )
}

struct MockLlm {
    response: String,
}

impl LlmTransport for MockLlm {
    fn complete(
        &self,
        _system_prompt: &str,
        user_prompt: &str,
        _model: &str,
    ) -> Result<String, String> {
        // Mirror the transform semantics: echo a derived line.
        Ok(format!(
            "{} -> {}",
            self.response,
            user_prompt.lines().next().unwrap_or("")
        ))
    }
}

fn transform_profile(mgr: &PromptProfileManager, name: &str) -> PromptProfileRecord {
    mgr.upsert(&pprepo::NewPromptProfile {
        name: name.into(),
        kind: KIND_TRANSFORM.into(),
        system_prompt: "Du bist ein Assistent.".into(),
        user_template: "Formuliere um:\n{text}".into(),
        provider_id: "openai-compatible-local".into(),
        model: "llama3:8b".into(),
        temperature: Some(0.2),
        enabled: true,
    })
    .unwrap()
}

/// Mirrors the command-layer flow of `notes_transform`: run profile over
/// the note's CURRENT content and append the output with SOURCE_TRANSFORM.
fn run_transform(
    notes: &handy_app_lib::NotesManager,
    profiles: &PromptProfileManager,
    note_id: &str,
    transport: &dyn LlmTransport,
) -> Result<repo::NoteVersionRecord, String> {
    let profile = profiles
        .list_by_kind(KIND_TRANSFORM)
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .ok_or_else(|| "no transform profile".to_string())?;
    let current = notes
        .current_version_of(note_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "note has no versions".to_string())?;
    let (output, _prov) = profiles.run_profile(&profile, &current.content, transport)?;
    notes
        .save_content(
            note_id,
            &output,
            handy_app_lib::storage::repositories::notes::SOURCE_TRANSFORM,
        )
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "transform produced identical content".to_string())
}

#[test]
fn fixture_01_transform_appends_new_version_and_keeps_source() {
    let (notes, profiles) = fresh_notes("f01");
    let (note, _) = notes
        .create("Meeting".into(), "roher diktattext".into())
        .unwrap();
    // Route the first version through the dictation source explicitly
    // (create() records manual_edit for the initial version).
    let dictated = notes
        .save_content(&note.id, "roher diktattext", repo::SOURCE_DICTATION)
        .unwrap();
    assert!(dictated.is_none(), "identical content is deduped");

    let _profile = transform_profile(&profiles, "Aufzählung");
    let mock = MockLlm {
        response: "aufgezählt".into(),
    };
    let created = run_transform(&notes, &profiles, &note.id, &mock).unwrap();

    assert_eq!(created.version_no, 2, "transform appends, never rewrites");
    assert_eq!(created.source, repo::SOURCE_TRANSFORM);

    let versions = notes.versions(&note.id).unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[1].content, "roher diktattext");
    assert_eq!(versions[1].version_no, 1, "source version stays intact");
    assert_eq!(versions[0].content, created.content);
}

#[test]
fn fixture_02_source_version_restorable_after_transform() {
    let (notes, profiles) = fresh_notes("f02");
    let (note, _) = notes.create("t".into(), "original text".into()).unwrap();
    let _profile = transform_profile(&profiles, "Clean");
    let mock = MockLlm {
        response: "transformiert".into(),
    };
    let _ = run_transform(&notes, &profiles, &note.id, &mock).unwrap();

    // Restoring the source version appends a NEW version with the old
    // content (append-only contract, NOTE-301/PAD-303).
    let restored = notes.restore_version(&note.id, 1).unwrap().unwrap();
    assert_eq!(restored.content, "original text");
    assert_eq!(restored.source, repo::SOURCE_RESTORE);
    let versions = notes.versions(&note.id).unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0].content, "original text");
    assert_eq!(versions[0].version_no, 3);
}

#[test]
fn fixture_03_dictation_and_manual_edits_interleave_with_transforms() {
    let (notes, profiles) = fresh_notes("f03");
    // create() seeds version 1 with manual_edit; the first dictation update
    // appends version 2, the manual edit version 3, the transform version 4.
    let (note, _) = notes.create("Log".into(), "d1".into()).unwrap();
    notes
        .save_content(&note.id, "d1\nm2", repo::SOURCE_MANUAL_EDIT)
        .unwrap()
        .unwrap();
    let _profile = transform_profile(&profiles, "T");
    let mock = MockLlm {
        response: "out".into(),
    };
    let transformed = run_transform(&notes, &profiles, &note.id, &mock).unwrap();

    // A later dictation appends after the transform; version order stays
    // monotonic with insertion order.
    let later = notes
        .save_content(&note.id, "d1\nm2\nd3", repo::SOURCE_DICTATION)
        .unwrap()
        .unwrap();
    assert_eq!(transformed.version_no, 3);
    assert_eq!(later.version_no, 4);
    let versions = notes.versions(&note.id).unwrap();
    let sources: Vec<&str> = versions.iter().map(|v| v.source.as_str()).collect();
    assert_eq!(
        sources,
        vec![
            repo::SOURCE_DICTATION,
            repo::SOURCE_TRANSFORM,
            repo::SOURCE_MANUAL_EDIT,
            repo::SOURCE_MANUAL_EDIT,
        ]
    );
}
