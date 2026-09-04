//! Notes Tauri commands (PAD-302/303 + NOTE-304 exposure of NOTE-301).
//!
//! Thin command layer over `NotesManager`. DTOs live here so the
//! repository records stay free of specta derives.

use crate::managers::notes::NotesManager;
use crate::storage::database::AppDatabase;
use crate::storage::repositories::notes as repo;
use serde::Serialize;
use specta::Type;
use tauri::AppHandle;

#[derive(Clone, Debug, Serialize, Type)]
pub struct NoteDto {
    pub id: String,
    pub title: String,
    pub pinned: bool,
    pub trashed: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Type)]
pub struct NoteVersionDto {
    pub id: String,
    pub note_id: String,
    pub version_no: i64,
    pub content: String,
    pub source: String,
    pub created_at_ms: i64,
}

fn note_dto(n: repo::NoteRecord) -> NoteDto {
    NoteDto {
        id: n.id,
        title: n.title,
        pinned: n.pinned,
        trashed: n.deleted_at_ms.is_some(),
        created_at_ms: n.created_at_ms,
        updated_at_ms: n.updated_at_ms,
    }
}

fn version_dto(v: repo::NoteVersionRecord) -> NoteVersionDto {
    NoteVersionDto {
        id: v.id,
        note_id: v.note_id,
        version_no: v.version_no,
        content: v.content,
        source: v.source,
        created_at_ms: v.created_at_ms,
    }
}

/// Only user-facing sources may come from the frontend; `transform` and
/// `restore` are backend-owned and recorded through dedicated paths.
fn frontend_source(source: &str) -> Result<&'static str, String> {
    match source {
        "manual_edit" => Ok(repo::SOURCE_MANUAL_EDIT),
        "dictation" => Ok(repo::SOURCE_DICTATION),
        other => Err(format!("unsupported note source '{other}'")),
    }
}

fn manager_with_db(
    app: &AppHandle,
) -> Result<(NotesManager, crate::storage::database::AppDatabase), String> {
    let dir = crate::portable::app_data_dir(app).map_err(|e| e.to_string())?;
    let db = AppDatabase::open(dir.join("history.db")).map_err(|e| e.to_string())?;
    Ok((NotesManager::new(db.clone()), db))
}

fn manager(app: &AppHandle) -> Result<NotesManager, String> {
    Ok(manager_with_db(app)?.0)
}

#[tauri::command]
#[specta::specta]
pub fn notes_list(app: AppHandle) -> Result<Vec<NoteDto>, String> {
    let (mgr, _) = manager_with_db(&app)?;
    mgr.list()
        .map(|notes| notes.into_iter().map(note_dto).collect())
        .map_err(|e| e.to_string())
}

/// Bounded full-text search over active notes (NOTE-304). `limit` is
/// clamped 1..=200 downstream, so a frontend bug cannot request an
/// unbounded scan.
#[tauri::command]
#[specta::specta]
pub fn notes_search(
    app: AppHandle,
    query: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<NoteDto>, String> {
    let (mgr, _) = manager_with_db(&app)?;
    mgr.search(&query, limit, offset)
        .map(|notes| notes.into_iter().map(note_dto).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn notes_create(app: AppHandle, title: String, content: String) -> Result<NoteDto, String> {
    let (mgr, _) = manager_with_db(&app)?;
    let (note, _) = mgr.create(title, content).map_err(|e| e.to_string())?;
    Ok(note_dto(note))
}

#[tauri::command]
#[specta::specta]
pub fn notes_current(app: AppHandle, note_id: String) -> Result<Option<NoteVersionDto>, String> {
    manager(&app)?
        .current_content_version(&note_id)
        .map(|v| v.map(version_dto))
        .map_err(|e| e.to_string())
}

/// Autosave entry point: append a version when content changed (hash
/// dedupe makes identical saves a no-op). `source` must be a
/// frontend-allowed source (`manual_edit` | `dictation`).
#[tauri::command]
#[specta::specta]
pub fn notes_append(
    app: AppHandle,
    note_id: String,
    content: String,
    source: String,
) -> Result<Option<NoteVersionDto>, String> {
    let source = frontend_source(&source)?;
    manager(&app)?
        .save_content(&note_id, &content, source)
        .map(|v| v.map(version_dto))
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn notes_versions(app: AppHandle, note_id: String) -> Result<Vec<NoteVersionDto>, String> {
    manager(&app)?
        .versions(&note_id)
        .map(|v| v.into_iter().map(version_dto).collect())
        .map_err(|e| e.to_string())
}

/// Restore an older version as a NEW version (append-only; never rewrites
/// history).
#[tauri::command]
#[specta::specta]
pub fn notes_restore_version(
    app: AppHandle,
    note_id: String,
    version_no: i64,
) -> Result<Option<NoteVersionDto>, String> {
    manager(&app)?
        .restore_version(&note_id, version_no)
        .map(|v| v.map(version_dto))
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn notes_set_title(app: AppHandle, note_id: String, title: String) -> Result<(), String> {
    manager(&app)?
        .set_title(&note_id, &title)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn notes_set_pinned(app: AppHandle, note_id: String, pinned: bool) -> Result<(), String> {
    manager(&app)?
        .set_pinned(&note_id, pinned)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn notes_trash(app: AppHandle, note_id: String) -> Result<bool, String> {
    manager(&app)?.trash(&note_id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn notes_restore(app: AppHandle, note_id: String) -> Result<bool, String> {
    manager(&app)?.restore(&note_id).map_err(|e| e.to_string())
}
