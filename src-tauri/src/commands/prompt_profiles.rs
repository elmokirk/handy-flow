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
