pub mod audio;
pub mod dictionary;
pub mod history;
pub mod models;
pub mod notes;
pub mod prompt_profiles;
pub mod snippets;
pub mod transcription;

use crate::storage::database::AppDatabase;

// Canonical history exposure (HIST-231/HIST-107 UI wiring).
#[tauri::command]
#[specta::specta]
pub async fn canonical_history_entries(
    app: tauri::AppHandle,
    limit: i64,
    offset: i64,
) -> Result<Vec<crate::managers::history::CanonicalEntry>, String> {
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = AppDatabase::open(dir.join("history.db")).map_err(|e| e.to_string())?;
    let captures = crate::storage::repositories::captures::list_active_captures(&db, limit, offset)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(captures.len());
    for c in &captures {
        out.push(HistoryAdapter::to_canonical_entry(&db, c).map_err(|e| e.to_string())?);
    }
    Ok(out)
}

#[tauri::command]
#[specta::specta]
pub async fn history_search(
    app: tauri::AppHandle,
    query: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<crate::managers::history::CanonicalEntry>, String> {
    use crate::storage::repositories::search as srepo;
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = AppDatabase::open(dir.join("history.db")).map_err(|e| e.to_string())?;
    if srepo::index_size(&db).unwrap_or(0) == 0 {
        srepo::rebuild_search_index(&db).map_err(|e| e.to_string())?;
    }
    let hits = srepo::search(&db, &query, None, limit, offset).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(hits.len());
    for hit in hits {
        if let Some(cid) = hit.capture_id {
            if let Some(c) = crate::storage::repositories::captures::get_capture(&db, &cid)
                .map_err(|e| e.to_string())?
            {
                out.push(HistoryAdapter::to_canonical_entry(&db, &c).map_err(|e| e.to_string())?);
            }
        }
    }
    Ok(out)
}

/// Thin adapter so commands reuse the manager's entry mapping without a
/// Tauri AppHandle (manager methods need one for settings).
struct HistoryAdapter;

impl HistoryAdapter {
    fn to_canonical_entry(
        db: &AppDatabase,
        c: &crate::storage::repositories::captures::CaptureRecord,
    ) -> anyhow::Result<crate::managers::history::CanonicalEntry> {
        use crate::storage::repositories::{representations as reps, transcriptions as att};
        let raw = att::canonical_attempt(db, &c.id)?.and_then(|a| a.normalized_stt);
        let derived = match att::canonical_attempt(db, &c.id)? {
            Some(a) => reps::representations_for_attempt(db, &a.id)?
                .into_iter()
                .map(|r| crate::managers::history::DerivedTextSummary {
                    representation_id: r.id,
                    kind: r.kind,
                    text: r.text,
                    created_at_ms: r.created_at_ms,
                })
                .collect(),
            None => Vec::new(),
        };
        Ok(crate::managers::history::CanonicalEntry {
            capture_id: c.id.clone(),
            legacy_history_id: c.legacy_history_id,
            title: c.title.clone(),
            created_at_ms: c.created_at_ms,
            integrity_state: c.integrity_state.clone(),
            trashed: c.deleted_at_ms.is_some(),
            audio_file_name: c.audio_file_name.clone(),
            raw_text: raw,
            derived,
        })
    }
}

use crate::settings::{get_settings, write_settings, AppSettings, LogLevel};
use crate::utils::cancel_current_operation;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
#[specta::specta]
pub fn cancel_operation(app: AppHandle) {
    cancel_current_operation(&app);
}

#[tauri::command]
#[specta::specta]
pub fn is_portable() -> bool {
    crate::portable::is_portable()
}

#[tauri::command]
#[specta::specta]
pub fn get_app_dir_path(app: AppHandle) -> Result<String, String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(app_data_dir.to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    Ok(get_settings(&app))
}

#[tauri::command]
#[specta::specta]
pub fn get_default_settings() -> Result<AppSettings, String> {
    Ok(crate::settings::get_default_settings())
}

#[tauri::command]
#[specta::specta]
pub fn get_log_dir_path(app: AppHandle) -> Result<String, String> {
    let log_dir = crate::portable::app_log_dir(&app)
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    Ok(log_dir.to_string_lossy().to_string())
}

#[specta::specta]
#[tauri::command]
pub fn set_log_level(app: AppHandle, level: LogLevel) -> Result<(), String> {
    let tauri_log_level: tauri_plugin_log::LogLevel = level.into();
    let log_level: log::Level = tauri_log_level.into();
    // Update the file log level atomic so the filter picks up the new level
    crate::FILE_LOG_LEVEL.store(
        log_level.to_level_filter() as u8,
        std::sync::atomic::Ordering::Relaxed,
    );

    let mut settings = get_settings(&app);
    settings.log_level = level;
    write_settings(&app, settings);

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_recordings_folder(app: AppHandle) -> Result<(), String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let recordings_dir = app_data_dir.join("recordings");

    let path = recordings_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open recordings folder: {}", e))?;

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_log_dir(app: AppHandle) -> Result<(), String> {
    let log_dir = crate::portable::app_log_dir(&app)
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    let path = log_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open log directory: {}", e))?;

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_app_data_dir(app: AppHandle) -> Result<(), String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let path = app_data_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open app data directory: {}", e))?;

    Ok(())
}

/// Check if Apple Intelligence is available on this device.
/// Called by the frontend when the user selects Apple Intelligence provider.
#[specta::specta]
#[tauri::command]
pub fn check_apple_intelligence_available() -> bool {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        crate::apple_intelligence::check_apple_intelligence_availability()
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        false
    }
}

/// Try to initialize Enigo (keyboard/mouse simulation).
/// On macOS, this will return an error if accessibility permissions are not granted.
#[specta::specta]
#[tauri::command]
pub fn initialize_enigo(app: AppHandle) -> Result<(), String> {
    use crate::input::EnigoState;

    // Check if already initialized
    if app.try_state::<EnigoState>().is_some() {
        log::debug!("Enigo already initialized");
        return Ok(());
    }

    // Try to initialize
    match EnigoState::new() {
        Ok(enigo_state) => {
            app.manage(enigo_state);
            log::info!("Enigo initialized successfully after permission grant");
            Ok(())
        }
        Err(e) => {
            if cfg!(target_os = "macos") {
                log::warn!(
                    "Failed to initialize Enigo: {} (accessibility permissions may not be granted)",
                    e
                );
            } else {
                log::warn!("Failed to initialize Enigo: {}", e);
            }
            Err(format!("Failed to initialize input system: {}", e))
        }
    }
}

/// Marker state to track if shortcuts have been initialized.
pub struct ShortcutsInitialized;

/// Initialize keyboard shortcuts.
/// On macOS, this should be called after accessibility permissions are granted.
/// This is idempotent - calling it multiple times is safe.
#[specta::specta]
#[tauri::command]
pub fn initialize_shortcuts(app: AppHandle) -> Result<(), String> {
    // Check if already initialized
    if app.try_state::<ShortcutsInitialized>().is_some() {
        log::debug!("Shortcuts already initialized");
        return Ok(());
    }

    // Initialize shortcuts
    crate::shortcut::init_shortcuts(&app);

    // Mark as initialized before reconciling the macOS Secure Input fallback.
    app.manage(ShortcutsInitialized);
    crate::secure_input::reconcile_fallback(&app);

    log::info!("Shortcuts initialized successfully");
    Ok(())
}
