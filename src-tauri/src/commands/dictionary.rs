//! Dictionary Tauri commands (DICT-202 exposure of DICT-201 domain).

use crate::managers::dictionary::DictionaryManager;
use crate::storage::database::AppDatabase;
use crate::storage::repositories::dictionary::DictionaryEntry;
use tauri::AppHandle;

fn manager(app: &AppHandle) -> Result<DictionaryManager, String> {
    let dir = crate::portable::app_data_dir(app).map_err(|e| e.to_string())?;
    let db = AppDatabase::open(dir.join("history.db")).map_err(|e| e.to_string())?;
    Ok(DictionaryManager::new(db))
}

#[tauri::command]
#[specta::specta]
pub fn dictionary_list(
    app: AppHandle,
) -> Result<Vec<crate::storage::repositories::dictionary::DictionaryEntry>, String> {
    manager(&app)?.list().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn dictionary_upsert(
    app: AppHandle,
    term: String,
    aliases: Vec<String>,
    enabled: bool,
) -> Result<crate::storage::repositories::dictionary::DictionaryEntry, String> {
    manager(&app)?
        .upsert(term, aliases, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn dictionary_delete(app: AppHandle, id: String) -> Result<bool, String> {
    manager(&app)?.delete(&id).map_err(|e| e.to_string())
}

/// Import legacy settings `custom_words` once (idempotent) and report count.
#[tauri::command]
#[specta::specta]
pub fn dictionary_import_legacy(app: AppHandle) -> Result<usize, String> {
    let settings = crate::settings::get_settings(&app);
    manager(&app)?
        .import_legacy_custom_words(&settings.custom_words)
        .map_err(|e| e.to_string())
}

/// Import a CSV (term, alias1;alias2) exported earlier / manually edited.
#[tauri::command]
#[specta::specta]
pub fn dictionary_import_csv(app: AppHandle, csv: String) -> Result<usize, String> {
    let mgr = manager(&app)?;
    let mut count = 0usize;
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split(';');
        let term = match parts.next() {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => continue,
        };
        let aliases: Vec<String> = parts
            .next()
            .map(|a| {
                a.split('|')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        mgr.upsert(term, aliases, true).map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

/// Export as `term;alias1|alias2` lines (CSV-ish, stable order).
#[tauri::command]
#[specta::specta]
pub fn dictionary_export(app: AppHandle) -> Result<String, String> {
    let entries = manager(&app)?.list().map_err(|e| e.to_string())?;
    Ok(entries
        .iter()
        .map(|e| {
            let aliases = e.aliases.join("|");
            if aliases.is_empty() {
                e.term.clone()
            } else {
                format!("{};{}", e.term, aliases)
            }
        })
        .collect::<Vec<_>>()
        .join("\n"))
}
