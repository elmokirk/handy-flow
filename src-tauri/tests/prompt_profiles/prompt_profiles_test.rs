//! PROMPT-221 domain tests: unified style/transform profiles over the
//! existing LLM transport abstraction, provenance snapshots, mock provider.

use handy_app_lib::storage::migrations::open_and_migrate;
use handy_app_lib::storage::repositories::prompt_profiles as repo;
use handy_app_lib::{
    LlmTransport, PromptProfileManager, PromptProvenance, KIND_STYLE, KIND_TRANSFORM,
};

fn fresh(tag: &str) -> PromptProfileManager {
    let dir = std::env::temp_dir().join(format!("handy-prompt-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (db, _) = open_and_migrate(dir.join("history.db"), "test").unwrap();
    PromptProfileManager::new(db)
}

struct MockLlm {
    response: &'static str,
    seen: std::sync::Mutex<Vec<(String, String, String)>>,
}

impl MockLlm {
    fn new(response: &'static str) -> Self {
        Self {
            response,
            seen: std::sync::Mutex::new(Vec::new()),
        }
    }
    fn calls(&self) -> Vec<(String, String, String)> {
        self.seen.lock().unwrap().clone()
    }
}

impl LlmTransport for MockLlm {
    fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
    ) -> Result<String, String> {
        self.seen
            .lock()
            .unwrap()
            .push((system_prompt.into(), user_prompt.into(), model.into()));
        Ok(self.response.to_string())
    }
}

fn profile(mgr: &PromptProfileManager, kind: &str, name: &str) -> repo::PromptProfileRecord {
    mgr.upsert(&repo::NewPromptProfile {
        name: name.into(),
        kind: kind.into(),
        system_prompt: "Du bist ein Formulierungsassistent.".into(),
        user_template: "Formuliere um:\n{text}".into(),
        provider_id: "openai-compatible-local".into(),
        model: "llama3:8b".into(),
        temperature: Some(0.4),
        enabled: true,
    })
    .unwrap()
}

// ---- Persistence / kind unification ----------------------------------------

#[test]
fn fixture_01_style_and_transform_share_one_domain() {
    let m = fresh("f01");
    profile(&m, KIND_STYLE, "Professionell");
    profile(&m, KIND_TRANSFORM, "Aufzählung");
    assert_eq!(m.list_by_kind(KIND_STYLE).unwrap().len(), 1);
    assert_eq!(m.list_by_kind(KIND_TRANSFORM).unwrap().len(), 1);
}

#[test]
fn fixture_02_name_unique_within_kind_only() {
    let m = fresh("f02");
    profile(&m, KIND_STYLE, "Clean");
    // Same name, different kind: allowed (separate list entry).
    profile(&m, KIND_TRANSFORM, "Clean");
    // Same name+kind again: upsert updates, no duplicate.
    profile(&m, KIND_STYLE, "Clean");
    assert_eq!(m.list_by_kind(KIND_STYLE).unwrap().len(), 1);
    assert_eq!(m.list_by_kind(KIND_TRANSFORM).unwrap().len(), 1);
}

#[test]
fn fixture_03_invalid_kind_rejected_by_db_check() {
    let m = fresh("f03");
    let res = m.upsert(&repo::NewPromptProfile {
        name: "bad".into(),
        kind: "wizard".into(),
        system_prompt: "s".into(),
        user_template: "{text}".into(),
        provider_id: "p".into(),
        model: "m".into(),
        temperature: None,
        enabled: true,
    });
    assert!(res.is_err(), "CHECK constraint must reject unknown kinds");
}

#[test]
fn fixture_04_delete_profile() {
    let m = fresh("f04");
    let p = profile(&m, KIND_STYLE, "Temp");
    assert!(m.delete(&p.id).unwrap());
    assert!(m.list_by_kind(KIND_STYLE).unwrap().is_empty());
}

// ---- Prompt composition + provenance + mock transport ----------------------

#[test]
fn fixture_05_user_template_placeholder_substitution() {
    let m = fresh("f05");
    let p = profile(&m, KIND_STYLE, "T");
    let user = PromptProfileManager::compose_user_prompt(&p.user_template, "hallo welt");
    assert_eq!(user, "Formuliere um:\nhallo welt");
}

#[test]
fn fixture_06_provenance_snapshot_captures_everything() {
    let m = fresh("f06");
    let p = profile(&m, KIND_STYLE, "Snap");
    let user = PromptProfileManager::compose_user_prompt(&p.user_template, "text");
    let prov: PromptProvenance = PromptProfileManager::build_provenance(&p, &user);
    assert_eq!(prov.profile_id, p.id);
    assert_eq!(prov.provider_id, "openai-compatible-local");
    assert_eq!(prov.model, "llama3:8b");
    assert!(prov.effective_prompt_snapshot.contains("SYSTEM:\nDu bist"));
    assert!(prov
        .effective_prompt_snapshot
        .contains("USER:\nFormuliere um:\ntext"));
}

#[test]
fn fixture_07_mock_provider_roundtrip_records_calls() {
    let m = fresh("f07");
    let p = profile(&m, KIND_STYLE, "Mock");
    let mock = MockLlm::new("umgeformter text");
    let (out, prov) = m.run_profile(&p, "eingabe text", &mock).unwrap();
    assert_eq!(out, "umgeformter text");
    assert_eq!(prov.profile_id, p.id);

    let calls = mock.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, p.system_prompt);
    assert_eq!(calls[0].1, "Formuliere um:\neingabe text");
    assert_eq!(calls[0].2, "llama3:8b", "model comes from profile");
}

#[test]
fn fixture_08_disabled_profile_refuses_to_run() {
    let m = fresh("f08");
    let mut p = profile(&m, KIND_TRANSFORM, "Off");
    p.enabled = false;
    let mock = MockLlm::new("x");
    let res = m.run_profile(&p, "t", &mock);
    assert!(res.is_err(), "disabled profiles must not run");
}
