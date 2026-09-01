//! Prompt profile Tauri commands (PROMPT-222 exposure of PROMPT-221).
//! Provider/keys stay in existing settings; profiles only reference them.

use crate::managers::prompt_profiles::PromptProfileManager;
use crate::storage::database::AppDatabase;
use crate::storage::repositories::prompt_profiles::{NewPromptProfile, PromptProfileRecord};
use tauri::AppHandle;

fn manager(app: &AppHandle) -> Result<PromptProfileManager, String> {
    let dir = crate::portable::app_data_dir(app).map_err(|e| e.to_string())?;
    let db = AppDatabase::open(dir.join("history.db")).map_err(|e| e.to_string())?;
    Ok(PromptProfileManager::new(db))
}

#[tauri::command]
#[specta::specta]
pub fn prompt_profiles_list(
    app: AppHandle,
    kind: String,
) -> Result<Vec<PromptProfileRecord>, String> {
    manager(&app)?
        .list_by_kind(&kind)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub fn prompt_profiles_upsert(
    app: AppHandle,
    name: String,
    kind: String,
    system_prompt: String,
    user_template: String,
    provider_id: String,
    model: String,
    temperature: Option<f64>,
    enabled: bool,
) -> Result<PromptProfileRecord, String> {
    manager(&app)?
        .upsert(&NewPromptProfile {
            name,
            kind,
            system_prompt,
            user_template,
            provider_id,
            model,
            temperature,
            enabled,
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn prompt_profiles_delete(app: AppHandle, id: String) -> Result<bool, String> {
    manager(&app)?.delete(&id).map_err(|e| e.to_string())
}

/// Run one transform profile against the note's CURRENT content and append
/// the result as a NEW version (source = `transform`). The source version
/// stays untouched and remains restorable (PAD-303, G3).
#[tauri::command]
#[specta::specta]
pub fn notes_transform(
    app: AppHandle,
    note_id: String,
    profile_id: String,
) -> Result<crate::commands::notes::NoteVersionDto, String> {
    use crate::managers::prompt_profiles::PromptProfileManager;

    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = AppDatabase::open(dir.join("history.db")).map_err(|e| e.to_string())?;
    let notes = crate::managers::notes::NotesManager::new(db.clone());
    let profiles = PromptProfileManager::new(db.clone());

    let note_title = notes
        .list()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|n| n.id == note_id)
        .map(|n| n.title)
        .ok_or_else(|| format!("note '{note_id}' not found"))?;
    let _ = note_title;
    let current = notes
        .current_content_version(&note_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("note '{note_id}' has no versions"))?;
    if current.content.trim().is_empty() {
        return Err("note is empty; nothing to transform".to_string());
    }

    // The profile row carries provider_id/model; resolve the concrete
    // provider + key from the existing post-processing settings.
    let _ = &note_title;
    let settings = crate::settings::get_settings(&app);
    let profiles_by_kind = profiles
        .list_by_kind(crate::managers::prompt_profiles::KIND_TRANSFORM)
        .map_err(|e| e.to_string())?;
    let profile = profiles_by_kind
        .into_iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| format!("transform profile '{profile_id}' not found"))?;
    let provider = settings
        .post_process_provider(&profile.provider_id)
        .cloned()
        .ok_or_else(|| format!("provider '{}' not configured", profile.provider_id))?;
    let api_key = settings
        .post_process_api_keys
        .get(&profile.provider_id)
        .cloned()
        .unwrap_or_default();
    let disable_reasoning = matches!(provider.id.as_str(), "custom" | "openrouter");

    let transport = ProviderTransport {
        provider,
        api_key,
        disable_reasoning,
    };
    let (output, _provenance) = profiles.run_profile(&profile, &current.content, &transport)?;

    let created = notes
        .save_content(
            &note_id,
            &output,
            crate::storage::repositories::notes::SOURCE_TRANSFORM,
        )
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "transform produced identical content; no version created".to_string())?;
    Ok(crate::commands::notes::NoteVersionDto {
        id: created.id,
        note_id: created.note_id,
        version_no: created.version_no,
        content: created.content,
        source: created.source,
        created_at_ms: created.created_at_ms,
    })
}

/// Blocking LLM transport bound to the profile's configured provider
/// (PAD-303 note transforms). Reuses the existing `llm_client` HTTP layer;
/// keys stay in the existing settings store.
struct ProviderTransport {
    provider: crate::settings::PostProcessProvider,
    api_key: String,
    disable_reasoning: bool,
}

impl crate::managers::prompt_profiles::LlmTransport for ProviderTransport {
    fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
    ) -> Result<String, String> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| "no tokio runtime on the command thread".to_string())?;
        let provider = self.provider.clone();
        let api_key = self.api_key.clone();
        let model = model.to_string();
        let system = system_prompt.to_string();
        let user = user_prompt.to_string();
        let disable_reasoning = self.disable_reasoning;
        let content = tokio::task::block_in_place(|| {
            rt.block_on(crate::llm_client::send_chat_completion_with_schema(
                &provider,
                api_key,
                &model,
                user,
                Some(system),
                None,
                disable_reasoning,
            ))
        })?;
        content.ok_or_else(|| "LLM response has no content".to_string())
    }
}
