use crate::managers::{
    history::{HistoryManager, PaginatedHistory},
    transcription::TranscriptionManager,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Clone, Debug, serde::Deserialize, specta::Type)]
pub struct CanonicalHistoryFilter {
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    /// `handy`, `wispr`, or `manual`; absent means every origin.
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
    pub attempt_status: String,
    pub attempt_error: Option<String>,
    pub audio_duration_ms: Option<i64>,
    pub completed_samples: i64,
    pub completed_chunks: i64,
    pub review_seams: i64,
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
        r#"SELECT c.id, c.created_at_ms, c.title,
                  COALESCE(a.normalized_stt, (
                      SELECT r.text FROM representations r
                      WHERE r.attempt_id = a.id ORDER BY r.created_at_ms ASC LIMIT 1
                  ), ''),
                  c.saved, c.audio_file_name, c.source_app, c.integrity_state,
                  CASE WHEN c.import_ref LIKE 'wispr:%' THEN 'wispr'
                       WHEN c.import_ref LIKE 'manual:%' THEN 'manual'
                       ELSE 'handy' END,
                  COALESCE((SELECT latest.status FROM transcription_attempts latest
                    WHERE latest.capture_id = c.id ORDER BY latest.attempt_number DESC LIMIT 1), 'none'),
                  (SELECT latest.error FROM transcription_attempts latest
                    WHERE latest.capture_id = c.id ORDER BY latest.attempt_number DESC LIMIT 1),
                  c.audio_duration_ms,
                  COALESCE((SELECT MAX(ch.end_sample) FROM transcription_chunks ch
                    WHERE ch.attempt_id = (SELECT latest.id FROM transcription_attempts latest
                    WHERE latest.capture_id = c.id ORDER BY latest.attempt_number DESC LIMIT 1)), 0),
                  (SELECT COUNT(*) FROM transcription_chunks ch
                    WHERE ch.attempt_id = (SELECT latest.id FROM transcription_attempts latest
                    WHERE latest.capture_id = c.id ORDER BY latest.attempt_number DESC LIMIT 1)),
                  (SELECT COUNT(*) FROM transcription_chunks ch
                    JOIN transcription_attempts reviewed ON reviewed.id = ch.attempt_id
                    WHERE reviewed.capture_id = c.id AND reviewed.is_canonical = 1
                      AND ch.seam_uncertain = 1)
           FROM captures c
           LEFT JOIN transcription_attempts a
             ON a.capture_id = c.id AND a.is_canonical = 1
           WHERE c.deleted_at_ms IS NULL"#,
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
        Some("manual") => sql.push_str(" AND c.import_ref LIKE 'manual:%'"),
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
                attempt_status: row.get(9)?,
                attempt_error: row.get(10)?,
                audio_duration_ms: row.get(11)?,
                completed_samples: row.get(12)?,
                completed_chunks: row.get(13)?,
                review_seams: row.get(14)?,
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
    crate::storage::recovery::reconcile_history(&db, &dir.join("recordings"))
        .map_err(|e| e.to_string())?;
    canonical_page(&db, &filter, cursor.as_deref(), limit)
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct CanonicalSeamDetail {
    pub position_ms: i64,
    pub left: String,
    pub right: String,
}

#[tauri::command]
#[specta::specta]
pub fn canonical_seam_details(
    app: AppHandle,
    capture_id: String,
) -> Result<Vec<CanonicalSeamDetail>, String> {
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    let conn = db.conn();
    let mut stmt = conn
        .prepare(
            "SELECT ch.start_sample, ch.seam_left, ch.seam_right
         FROM transcription_chunks ch JOIN transcription_attempts a ON a.id = ch.attempt_id
         WHERE a.capture_id = ?1 AND a.is_canonical = 1 AND ch.seam_uncertain = 1
         ORDER BY ch.chunk_index",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([capture_id], |row| {
            Ok(CanonicalSeamDetail {
                position_ms: row.get::<_, i64>(0)? * 1000 / 16_000,
                left: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                right: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
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
    _transcription_manager: State<'_, Arc<TranscriptionManager>>,
    capture_id: String,
) -> Result<(), String> {
    queue_retry(&app, capture_id)
}

fn queue_retry(app: &AppHandle, capture_id: String) -> Result<(), String> {
    use crate::storage::repositories::transcriptions as attempts;
    let dir = crate::portable::app_data_dir(app).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    let capture = crate::storage::repositories::captures::get_capture(&db, &capture_id)
        .map_err(|e| e.to_string())?
        .ok_or("history entry not found")?;
    if !matches!(
        capture.integrity_state.as_str(),
        "audio_valid" | "recovered_orphan"
    ) {
        return Err("Recording audio is missing or corrupt".into());
    }
    let name = capture
        .audio_file_name
        .ok_or("history entry has no audio")?;
    if std::path::Path::new(&name)
        .file_name()
        .and_then(|n| n.to_str())
        != Some(name.as_str())
    {
        return Err("invalid recording filename".into());
    }
    if !dir.join("recordings").join(&name).is_file() {
        return Err("Original audio file is missing".into());
    }
    if attempts::attempts_for_capture(&db, &capture_id)
        .map_err(|e| e.to_string())?
        .last()
        .is_some_and(|attempt| matches!(attempt.status.as_str(), "pending" | "running"))
    {
        app.state::<crate::long_audio::TranscriptionQueue>().wake();
        return Ok(());
    }
    attempts::insert_attempt(
        &db,
        &attempts::NewAttempt {
            capture_id,
            engine_raw: None,
            normalized_stt: None,
            model_id: Some(crate::settings::get_settings(app).selected_model),
            language: None,
            normalizer_version: crate::NORMALIZER_VERSION.to_string(),
            dictionary_snapshot_sha256: None,
        },
    )
    .map_err(|e| e.to_string())?;
    app.state::<crate::long_audio::TranscriptionQueue>().wake();
    let _ = app.emit("canonical-history-changed", ());
    Ok(())
}

fn import_local_audio_file(app: AppHandle, source_path: String) -> Result<String, String> {
    use crate::storage::models::IntegrityState;
    use crate::storage::repositories::{captures, transcriptions};

    let source = std::fs::canonicalize(&source_path).map_err(|e| e.to_string())?;
    let metadata = std::fs::metadata(&source).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err("Choose a non-empty audio file".into());
    }
    let extension = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "wav" && extension != "mp3" {
        return Err("Only WAV and MP3 files are supported".into());
    }
    let title = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid audio filename")?
        .to_string();
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let recordings = dir.join("recordings");
    std::fs::create_dir_all(&recordings).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    let unique = crate::storage::ids::new_id();
    let file_name = format!("manual-{unique}.{extension}");
    let capture = captures::insert_capture_at_with_ref(
        &db,
        &captures::NewCapture {
            audio_file_name: Some(file_name.clone()),
            audio_sha256: None,
            audio_size_bytes: None,
            title,
            source_app: None,
            integrity_state: IntegrityState::PendingAudio,
        },
        chrono::Utc::now().timestamp_millis(),
        Some(&format!("manual:{unique}")),
    )
    .map_err(|e| e.to_string())?;
    let _ = app.emit("canonical-history-changed", ());
    let target = recordings.join(&file_name);
    let staging_dir = recordings.join(".processing");
    let staging = staging_dir.join(format!("{}.import.part", capture.id));
    let mut decode_started = false;
    let mut audio_decoded = false;
    let prepared = (|| -> Result<(), String> {
        std::fs::create_dir_all(&staging_dir).map_err(|e| e.to_string())?;
        let original_hash =
            crate::storage::audio_files::hash_file(&source).map_err(|e| e.to_string())?;
        std::fs::copy(&source, &staging).map_err(|e| e.to_string())?;
        std::fs::File::open(&staging)
            .and_then(|file| file.sync_all())
            .map_err(|e| e.to_string())?;
        let copied_hash =
            crate::storage::audio_files::hash_file(&staging).map_err(|e| e.to_string())?;
        if original_hash != copied_hash {
            return Err("Audio source changed during import; copied file was not accepted".into());
        }
        std::fs::rename(&staging, &target).map_err(|e| e.to_string())?;
        captures::attach_audio(
            &db,
            &capture.id,
            &file_name,
            &copied_hash,
            metadata.len() as i64,
        )
        .map_err(|e| e.to_string())?;
        decode_started = true;
        let working = crate::long_audio::prepare_working_wav(
            &target,
            &recordings.join(".processing"),
            &capture.id,
        )?;
        let samples = crate::long_audio::inspect_wav(&working)?;
        audio_decoded = true;
        captures::set_audio_duration(&db, &capture.id, (samples * 1000 / 16_000) as i64)
            .map_err(|e| e.to_string())?;
        transcriptions::insert_attempt(
            &db,
            &transcriptions::NewAttempt {
                capture_id: capture.id.clone(),
                engine_raw: None,
                normalized_stt: None,
                model_id: Some(crate::settings::get_settings(&app).selected_model),
                language: None,
                normalizer_version: crate::NORMALIZER_VERSION.to_string(),
                dictionary_snapshot_sha256: None,
            },
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = prepared {
        let _ = std::fs::remove_file(&staging);
        let _ = std::fs::remove_file(staging_dir.join(format!("{}.part", capture.id)));
        if !target.is_file() || (decode_started && !audio_decoded) {
            let state = if target.is_file() {
                IntegrityState::AudioCorrupt
            } else {
                IntegrityState::AudioMissing
            };
            let _ = captures::set_integrity_state(&db, &capture.id, state);
        }
        if let Ok(attempt) = transcriptions::insert_attempt(
            &db,
            &transcriptions::NewAttempt {
                capture_id: capture.id.clone(),
                engine_raw: None,
                normalized_stt: None,
                model_id: None,
                language: None,
                normalizer_version: crate::NORMALIZER_VERSION.to_string(),
                dictionary_snapshot_sha256: None,
            },
        ) {
            let _ =
                transcriptions::complete_attempt(&db, &attempt.id, None, Some(&error), None, None);
        }
        let _ = app.emit("canonical-history-changed", ());
        return Err(error);
    }
    app.state::<crate::long_audio::TranscriptionQueue>().wake();
    let _ = app.emit("canonical-history-changed", ());
    Ok(capture.id)
}

#[tauri::command]
#[specta::specta]
pub async fn import_local_audio(app: AppHandle, source_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || import_local_audio_file(app, source_path))
        .await
        .map_err(|e| e.to_string())?
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
    _history_manager: State<'_, Arc<HistoryManager>>,
    _transcription_manager: State<'_, Arc<TranscriptionManager>>,
    id: i64,
) -> Result<(), String> {
    let dir = crate::portable::app_data_dir(&app).map_err(|e| e.to_string())?;
    let db = crate::storage::database::AppDatabase::open(dir.join("history.db"))
        .map_err(|e| e.to_string())?;
    let capture_id: String = db
        .conn()
        .query_row(
            "SELECT id FROM captures WHERE legacy_history_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| "Legacy entry has no canonical capture".to_string())?;
    queue_retry(&app, capture_id)
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
