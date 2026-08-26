//! Prompt profile manager (PROMPT-221).
//!
//! Unifies Styles and Dictation Transforms over the EXISTING Handy LLM
//! transport (`llm_client.rs` stays the only HTTP client — wiring lands
//! with the Phase-2 Integrator bundle). This manager owns profile CRUD,
//! effective-prompt composition and provenance snapshots; a transport
//! trait keeps the domain testable with a mock provider.

use serde::Serialize;
use specta::Type;

use crate::storage::database::AppDatabase;
use crate::storage::repositories::prompt_profiles as repo;

/// Transport-agnostic provenance recorded per representation so outputs
/// stay interpretable after profile edits or deletions (planning/04).
#[derive(Clone, Debug, Serialize, Type)]
pub struct PromptProvenance {
    pub profile_id: String,
    pub profile_name: String,
    pub kind: String,
    pub provider_id: String,
    pub model: String,
    /// The exact prompt sent to the provider.
    pub effective_prompt_snapshot: String,
}

#[allow(dead_code)]
pub const KIND_STYLE: &str = "style";
#[allow(dead_code)]
pub const KIND_TRANSFORM: &str = "transform";

/// Minimal synchronous transport abstraction. The production impl wraps
/// Handy's llm_client; tests inject a mock. Async stays inside the
/// production wrapper so this domain layer keeps no runtime deps.
pub trait LlmTransport: Send + Sync {
    fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
    ) -> Result<String, String>;
}

#[derive(Clone)]
pub struct PromptProfileManager {
    db: AppDatabase,
}

impl PromptProfileManager {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }

    pub fn upsert(
        &self,
        new: &repo::NewPromptProfile,
    ) -> Result<repo::PromptProfileRecord, rusqlite::Error> {
        repo::upsert_profile(&self.db, new)
    }

    pub fn list_by_kind(
        &self,
        kind: &str,
    ) -> Result<Vec<repo::PromptProfileRecord>, rusqlite::Error> {
        repo::list_by_kind(&self.db, kind)
    }

    pub fn delete(&self, id: &str) -> Result<bool, rusqlite::Error> {
        repo::delete_profile(&self.db, id)
    }

    /// Compose the exact user message from the template. `{text}` is the
    /// single supported placeholder; an unknown placeholder is left as-is
    /// (fail-open, deterministic).
    pub fn compose_user_prompt(template: &str, transcript_text: &str) -> String {
        template.replace("{text}", transcript_text)
    }

    /// Build the provenance snapshot for a profile run.
    pub fn build_provenance(
        profile: &repo::PromptProfileRecord,
        user_prompt: &str,
    ) -> PromptProvenance {
        PromptProvenance {
            profile_id: profile.id.clone(),
            profile_name: profile.name.clone(),
            kind: profile.kind.clone(),
            provider_id: profile.provider_id.clone(),
            model: profile.model.clone(),
            effective_prompt_snapshot: format!(
                "SYSTEM:\n{}\n\nUSER:\n{}",
                profile.system_prompt, user_prompt
            ),
        }
    }

    /// Run one enabled profile against a transcript via the given
    /// transport and return output + full provenance. Persistence of the
    /// resulting representation happens in the pipeline layer.
    pub fn run_profile(
        &self,
        profile: &repo::PromptProfileRecord,
        transcript_text: &str,
        transport: &dyn LlmTransport,
    ) -> Result<(String, PromptProvenance), String> {
        if !profile.enabled {
            return Err(format!("profile '{}' is disabled", profile.name));
        }
        let user_prompt = Self::compose_user_prompt(&profile.user_template, transcript_text);
        let provenance = Self::build_provenance(profile, &user_prompt);
        let out = transport.complete(&profile.system_prompt, &user_prompt, &profile.model)?;
        Ok((out, provenance))
    }
}
