//! Snippet Tauri commands (SNIP-212 exposure of SNIP-211 domain).

use crate::managers::snippets::SnippetManager;
use crate::storage::database::AppDatabase;
use tauri::AppHandle;

fn manager(app: &AppHandle) -> Result<SnippetManager, String> {
    let dir = crate::portable::app_data_dir(app).map_err(|e| e.to_string())?;
    let db = AppDatabase::open(dir.join("history.db")).map_err(|e| e.to_string())?;
    Ok(SnippetManager::new(db))
}

#[tauri::command]
#[specta::specta]
pub fn snippets_list(
    app: AppHandle,
) -> Result<Vec<crate::storage::repositories::snippets::SnippetRecord>, String> {
    manager(&app)?.list().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn snippets_upsert(
    app: AppHandle,
    trigger: String,
    replacement: String,
    priority: i64,
    enabled: bool,
) -> Result<crate::storage::repositories::snippets::SnippetRecord, String> {
    manager(&app)?
        .upsert(trigger, replacement, priority, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn snippets_delete(app: AppHandle, id: String) -> Result<bool, String> {
    manager(&app)?.delete(&id).map_err(|e| e.to_string())
}

/// Live preview: what would the current snippet set do to this text?
#[tauri::command]
#[specta::specta]
pub fn snippets_test_apply(app: AppHandle, text: String) -> Result<String, String> {
    manager(&app)?
        .apply_snippets(&text)
        .map_err(|e| e.to_string())
}
