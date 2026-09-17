use crate::actions::process_transcription_output;
use crate::managers::{
    history::{HistoryManager, PaginatedHistory},
    transcription::TranscriptionManager,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, Debug, serde::Deserialize, specta::Type)]
pub struct CanonicalHistoryFilter {
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    /// `handy` or `wispr`; absent means every origin.
    pub origin: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct CanonicalHistoryEntry {
    pub capture_id: String,
    pub created_at_ms: i64,
    pub title: String,
    pub text: String,
    pub saved: bool,
    pub audio_file_name: Option<String>,
    pub source_app: Option<String>,
    pub origin: String,
    pub integrity_state: String,
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct CanonicalHistoryPage {
    pub entries: Vec<CanonicalHistoryEntry>,
    pub next_cursor: Option<String>,
}

fn parse_cursor(cursor: Option<&str>) -> Result<Option<(i64, String)>, String> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let (ms, id) = cursor
        .split_once(':')
        .ok_or_else(|| "invalid history cursor".to_string())?;
    let ms = ms
        .parse::<i64>()
        .map_err(|_| "invalid history cursor timestamp".to_string())?;
    if id.is_empty() {
        return Err("invalid history cursor id".to_string());
    }
    Ok(Some((ms, id.to_string())))
}

fn canonical_page(
    db: &crate::storage::database::AppDatabase,
    filter: &CanonicalHistoryFilter,
    cursor: Option<&str>,
    limit: i64,
) -> Result<CanonicalHistoryPage, String> {
    use rusqlite::{params_from_iter, types::ToSql};

    let mut sql = String::from(
        "SELECT c.id, c.created_at_ms, c.title,\
                COALESCE(a.normalized_stt, (\
                    SELECT r.text FROM representations r\
                    WHERE r.attempt_id = a.id ORDER BY r.created_at_ms ASC LIMIT 1\
                ), ''),\
                c.saved, c.audio_file_name, c.source_app, c.integrity_state,\
                CASE WHEN c.import_ref LIKE 'wispr:%' THEN 'wispr' ELSE 'handy' END\
         FROM captures c\
         LEFT JOIN transcription_attempts a\
           ON a.capture_id = c.id AND a.is_canonical = 1\
         WHERE c.deleted_at_ms IS NULL",
    );
    let mut values: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(from) = filter.from_ms {
        sql.push_str(" AND c.created_at_ms >= ?");
        values.push(Box::new(from));
    }
    if let Some(to) = filter.to_ms {
        sql.push_str(" AND c.created_at_ms < ?");
        values.push(Box::new(to));
    }
    match filter.origin.as_deref() {
        None | Some("all") => {}
        Some("wispr") => sql.push_str(" AND c.import_ref LIKE 'wispr:%'"),
        Some("handy") => sql.push_str(" AND c.import_ref IS NULL"),
        Some(_) => return Err("invalid history origin".to_string()),
    }
    if let Some((created_at_ms, id)) = parse_cursor(cursor)? {
        sql.push_str(" AND (c.created_at_ms < ? OR (c.created_at_ms = ? AND c.id < ?))");
        values.push(Box::new(created_at_ms));
        values.push(Box::new(created_at_ms));
        values.push(Box::new(id));
    }
    sql.push_str(" ORDER BY c.created_at_ms DESC, c.id DESC LIMIT ?");
    let requested = limit.clamp(1, 200);
    values.push(Box::new(requested + 1));

    let conn = db.conn();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(values.iter().map(|v| v.as_ref())), |row| {
            Ok(CanonicalHistoryEntry {
                capture_id: row.get(0)?,
                created_at_ms: row.get(1)?,
                title: row.get(2)?,
                text: row.get(3)?,
                saved: row.get::<_, i64>(4)? == 1,
                audio_file_name: row.get(5)?,
                source_app: row.get(6)?,
                integrity_state: row.get(7)?,
                origin: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut entries: Vec<_> = rows
        .collect::<Result<_, _>>()
        .map_err(|e: rusqlite::Error| e.to_string())?;
    let has_more = entries.len() > requested as usize;
    if has_more {
        entries.pop();
    }
    let next_cursor = has_more.then(|| {
        let last = entries.last().expect("page with more entries is non-empty");
        format!("{}:{}", last.created_at_ms, last.capture_id)
    });
    Ok(CanonicalHistoryPage {
        entries,
        next_cursor,
    })
}

/// Canonical history page used by the app UI. Filtering happens in SQLite
/// before the stable `(created_at_ms, id)` cursor is applied.
#[tauri::command]
#[specta::specta]
pub fn canonical_history_page(
    app: AppHandle,
    filter: CanonicalHistoryFilter,
    limit: i64,
    cursor: Option<String>,
) -> Result<CanonicalHistoryPage, String> {
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    canonical_page(&db, &filter, cursor.as_deref(), limit)
}

#[tauri::command]
#[specta::specta]
pub fn toggle_canonical_history_saved(app: AppHandle, capture_id: String) -> Result<(), String> {
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();
    let changed = db
        .conn()
        .execute(
            "UPDATE captures SET saved = CASE saved WHEN 1 THEN 0 ELSE 1 END,\
             updated_at_ms = ?2\
             WHERE id = ?1 AND deleted_at_ms IS NULL",
            rusqlite::params![capture_id, now_ms],
        )
        .map_err(|e| e.to_string())?;
    if changed != 1 {
        return Err("history entry not found".to_string());
    }
    let _ = app.emit("canonical-history-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn canonical_audio_file_path(app: AppHandle, capture_id: String) -> Result<String, String> {
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    let name: Option<String> = db
        .conn()
        .query_row(
            "SELECT audio_file_name FROM captures WHERE id = ?1 AND deleted_at_ms IS NULL",
            [&capture_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let name = name.ok_or_else(|| "history entry has no audio".to_string())?;
    if std::path::Path::new(&name)
        .file_name()
        .and_then(|n| n.to_str())
        != Some(name.as_str())
    {
        return Err("invalid recording filename".to_string());
    }
    Ok(dir
        .join("recordings")
        .join(name)
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
#[specta::specta]
pub fn trash_canonical_history_entry(app: AppHandle, capture_id: String) -> Result<(), String> {
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    if !crate::storage::repositories::captures::trash_capture(&db, &capture_id)
        .map_err(|e| e.to_string())?
    {
        return Err("history entry not found".to_string());
    }
    let _ = app.emit("canonical-history-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn retry_canonical_history_entry(
    app: AppHandle,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    capture_id: String,
) -> Result<(), String> {
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    let capture = crate::storage::repositories::captures::get_capture(&db, &capture_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "history entry not found".to_string())?;
    let name = capture
        .audio_file_name
        .ok_or_else(|| "history entry has no audio".to_string())?;
    let samples = crate::audio_toolkit::read_wav_samples(&dir.join("recordings").join(name))
        .map_err(|e| format!("Failed to load audio: {e}"))?;
    if samples.is_empty() {
        return Err("Recording has no audio samples".to_string());
    }
    transcription_manager.initiate_model_load();
    let tm = Arc::clone(&transcription_manager);
    let transcription = tauri::async_runtime::spawn_blocking(move || tm.transcribe(samples))
        .await
        .map_err(|e| format!("Transcription task panicked: {e}"))?
        .map_err(|e| e.to_string())?;
    if transcription.is_empty() {
        return Err("Recording contains no speech".to_string());
    }
    let processed = process_transcription_output(&app, &transcription, false).await;
    use crate::storage::repositories::transcriptions as attempts;
    let attempt = attempts::insert_attempt(
        &db,
        &attempts::NewAttempt {
            capture_id: capture_id.clone(),
            engine_raw: None,
            normalized_stt: Some(transcription),
            model_id: None,
            language: None,
            normalizer_version: crate::NORMALIZER_VERSION.to_string(),
            dictionary_snapshot_sha256: None,
        },
    )
    .map_err(|e| e.to_string())?;
    attempts::mark_canonical(&db, &capture_id, &attempt.id).map_err(|e| e.to_string())?;
    if let Some(text) = processed.post_processed_text {
        crate::storage::repositories::representations::insert_representation(
            &db,
            &crate::storage::repositories::representations::NewRepresentation {
                attempt_id: attempt.id,
                parent_representation_id: None,
                kind: "post_process".to_string(),
                text,
                processor: "retry".to_string(),
                processor_version: crate::NORMALIZER_VERSION.to_string(),
                prompt_profile_id: None,
                effective_prompt_snapshot: processed.post_process_prompt,
                provider_snapshot: None,
            },
        )
        .map_err(|e| e.to_string())?;
    }
    let _ = app.emit("canonical-history-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_history_entries(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    cursor: Option<i64>,
    limit: Option<usize>,
) -> Result<PaginatedHistory, String> {
    history_manager
        .get_history_entries(cursor, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_history_entry_saved(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<(), String> {
    history_manager
        .toggle_saved_status(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_audio_file_path(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    file_name: String,
) -> Result<String, String> {
    let path = history_manager.get_audio_file_path(&file_name);
    path.to_str()
        .ok_or_else(|| "Invalid file path".to_string())
        .map(|s| s.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_history_entry(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<(), String> {
    history_manager
        .delete_entry(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn retry_history_entry_transcription(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    id: i64,
) -> Result<(), String> {
    let entry = history_manager
        .get_entry_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("History entry {} not found", id))?;

    let audio_path = history_manager.get_audio_file_path(&entry.file_name);
    let samples = crate::audio_toolkit::read_wav_samples(&audio_path)
        .map_err(|e| format!("Failed to load audio: {}", e))?;

    if samples.is_empty() {
        return Err("Recording has no audio samples".to_string());
    }

    transcription_manager.initiate_model_load();

    let tm = Arc::clone(&transcription_manager);
    let transcription = tauri::async_runtime::spawn_blocking(move || tm.transcribe(samples))
        .await
        .map_err(|e| format!("Transcription task panicked: {}", e))?
        .map_err(|e| e.to_string())?;

    if transcription.is_empty() {
        return Err("Recording contains no speech".to_string());
    }

    let processed =
        process_transcription_output(&app, &transcription, entry.post_process_requested).await;
    history_manager
        .update_transcription(
            id,
            transcription,
            processed.post_processed_text,
            processed.post_process_prompt,
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn update_history_limit(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    limit: usize,
) -> Result<(), String> {
    let mut settings = crate::settings::get_settings(&app);
    settings.history_limit = limit;
    crate::settings::write_settings(&app, settings);

    history_manager
        .cleanup_canonical_entries()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_recording_retention_period(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    period: String,
) -> Result<(), String> {
    use crate::settings::RecordingRetentionPeriod;

    let retention_period = match period.as_str() {
        "never" => RecordingRetentionPeriod::Never,
        "preserve_limit" => RecordingRetentionPeriod::PreserveLimit,
        "days3" => RecordingRetentionPeriod::Days3,
        "weeks2" => RecordingRetentionPeriod::Weeks2,
        "months3" => RecordingRetentionPeriod::Months3,
        _ => return Err(format!("Invalid retention period: {}", period)),
    };

    let mut settings = crate::settings::get_settings(&app);
    settings.recording_retention_period = retention_period;
    crate::settings::write_settings(&app, settings);

    history_manager
        .cleanup_canonical_entries()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod canonical_history_tests {
    use super::*;
    use crate::storage::migrations::open_and_migrate;

    fn seeded_db() -> crate::storage::database::AppDatabase {
        let dir = tempfile::tempdir().unwrap();
        // Keep the file after TempDir drops: AppDatabase owns the open handle.
        let path = dir.keep().join("history.db");
        let (db, _) = open_and_migrate(path, "test").unwrap();
        db.conn()
            .execute_batch(
                "INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms, import_ref)
                   VALUES ('wispr-old', 'Wispr', 'audio_missing', 1000, 1000, 'wispr:h-1');
                 INSERT INTO captures(id, title, integrity_state, created_at_ms, updated_at_ms)
                   VALUES ('handy-new', 'Handy', 'audio_missing', 2000, 2000);
                 INSERT INTO transcription_attempts(id, capture_id, attempt_number, normalized_stt, provenance, status, is_canonical, created_at_ms)
                   VALUES ('a-wispr', 'wispr-old', 1, 'older Wispr', 'legacy_migration', 'success', 1, 1000);
                 INSERT INTO transcription_attempts(id, capture_id, attempt_number, normalized_stt, provenance, status, is_canonical, created_at_ms)
                   VALUES ('a-handy', 'handy-new', 1, 'newer Handy', 'live', 'success', 1, 2000);",
            )
            .unwrap();
        db
    }

    #[test]
    fn filters_in_sql_preserve_chronology_and_origin() {
        let db = seeded_db();
        let all = canonical_page(
            &db,
            &CanonicalHistoryFilter {
                from_ms: None,
                to_ms: None,
                origin: None,
            },
            None,
            30,
        )
        .unwrap();
        assert_eq!(all.entries[0].capture_id, "handy-new");
        assert_eq!(all.entries[1].capture_id, "wispr-old");

        let wispr = canonical_page(
            &db,
            &CanonicalHistoryFilter {
                from_ms: None,
                to_ms: None,
                origin: Some("wispr".to_string()),
            },
            None,
            30,
        )
        .unwrap();
        assert_eq!(wispr.entries.len(), 1);
        assert_eq!(wispr.entries[0].text, "older Wispr");

        let recent = canonical_page(
            &db,
            &CanonicalHistoryFilter {
                from_ms: Some(1500),
                to_ms: None,
                origin: None,
            },
            None,
            30,
        )
        .unwrap();
        assert_eq!(recent.entries.len(), 1);
        assert_eq!(recent.entries[0].capture_id, "handy-new");
    }
}
