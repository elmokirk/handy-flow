//! Bounded, history-only acceptance route through the production importer.
use crate::{cli::CliArgs, commands::history, managers::history::HistoryManager};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

/// Validate before portable initialization or any plugin/manager can write.
pub fn validate(args: &CliArgs) -> Result<(), String> {
    let Some(root) = &args.canonical_sandbox else {
        return Ok(());
    };
    if args.transcription_worker
        || args.transcribe_file.is_some()
        || args.list_devices
        || args.list_models
        || args.toggle_transcription
        || args.toggle_post_process
        || args.cancel
        || args.start_hidden
        || args.no_tray
        || args.debug
        || args.model.is_some()
        || args.device_index.is_some()
        || args.repeat.is_some()
        || args.json
    {
        return Err("canonical sandbox conflicts with other CLI modes".into());
    }
    let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let exe = std::fs::canonicalize(std::env::current_exe().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    validate_paths(
        &root,
        &exe,
        args.canonical_import
            .as_deref()
            .ok_or("canonical import required")?,
    )
}

fn validate_paths(root: &Path, exe: &Path, input: &Path) -> Result<(), String> {
    if !root
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with("handy-up700-canonical-"))
        || exe.parent() != Some(root)
        || !matches!(
            std::fs::read_to_string(root.join("portable")).as_deref(),
            Ok("Handy Portable Mode")
        )
        || !matches!(
            std::fs::read_to_string(root.join("UP700-SANDBOX")).as_deref(),
            Ok("synthetic-only")
        )
    {
        return Err("explicit private portable sandbox required".into());
    }
    // Reject reparse points/symlinks in every existing sandbox component.
    for entry in walk(root)? {
        let metadata = std::fs::symlink_metadata(&entry).map_err(|e| e.to_string())?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err("sandbox reparse point rejected".into());
            }
        }
        if metadata.file_type().is_symlink() {
            return Err("sandbox symlink rejected".into());
        }
    }
    let input = std::fs::canonicalize(input).map_err(|e| e.to_string())?;
    if !input.starts_with(root.join("inputs")) || !input.is_file() {
        return Err("input must be a sandbox input file".into());
    }
    for name in ["TEMP", "TMP", "HF_HOME", "HF_HUB_CACHE"] {
        let value = std::env::var_os(name).ok_or_else(|| format!("{name} required"))?;
        let path = std::fs::canonicalize(value).map_err(|e| e.to_string())?;
        if !path.starts_with(root) {
            return Err(format!("{name} escapes sandbox"));
        }
    }
    if std::env::var("HF_HUB_OFFLINE").as_deref() != Ok("1") {
        return Err("offline mode required".into());
    }
    Ok(())
}

fn walk(root: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut entries = vec![root.to_path_buf()];
    let mut index = 0;
    while index < entries.len() {
        let path = &entries[index];
        let metadata = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err("sandbox reparse point rejected".into());
            }
        }
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            let children = std::fs::read_dir(path)
                .map_err(|e| e.to_string())?
                .map(|e| e.map(|e| e.path()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            entries.extend(children);
        }
        index += 1;
    }
    Ok(entries)
}

pub fn run(app: &AppHandle, args: &CliArgs) -> Result<i32, String> {
    app.manage(Arc::new(
        HistoryManager::new(app).map_err(|e| e.to_string())?,
    ));
    app.manage(crate::long_audio::TranscriptionQueue::start(app.clone()));
    let import = history::import_local_audio_file(
        app.clone(),
        args.canonical_import
            .as_ref()
            .ok_or("canonical import required")?
            .to_string_lossy()
            .into_owned(),
    );
    let end = Instant::now() + Duration::from_secs(120);
    loop {
        let page = history::canonical_history_page(
            app.clone(),
            history::CanonicalHistoryFilter {
                from_ms: None,
                to_ms: None,
                origin: Some("manual".into()),
            },
            200,
            None,
        )?;
        if page.entries.len() == 1
            && matches!(
                page.entries[0].attempt_status.as_str(),
                "success" | "failed"
            )
        {
            let payload = serde_json::json!({"import": import, "history": page});
            std::fs::write(
                args.canonical_sandbox
                    .as_ref()
                    .ok_or("sandbox required")?
                    .join("result.json"),
                payload.to_string(),
            )
            .map_err(|e| e.to_string())?;
            println!("{payload}");
            return Ok(if page.entries[0].attempt_status == "success" {
                0
            } else {
                1
            });
        }
        if Instant::now() >= end {
            return Err("canonical import timeout".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn canonical_cli_requires_both_paths() {
        assert!(CliArgs::try_parse_from(["handy", "--canonical-import", "input.opus"]).is_err());
        assert!(CliArgs::try_parse_from(["handy", "--canonical-sandbox", "sandbox"]).is_err());
    }

    #[test]
    fn canonical_cli_rejects_remote_control_before_paths() {
        let args = CliArgs::try_parse_from([
            "handy",
            "--canonical-import",
            "input.opus",
            "--canonical-sandbox",
            "missing",
            "--cancel",
        ])
        .unwrap();
        assert_eq!(
            validate(&args).unwrap_err(),
            "canonical sandbox conflicts with other CLI modes"
        );
    }

    #[test]
    fn ordinary_cli_does_not_enter_sandbox() {
        assert!(validate(&CliArgs::default()).is_ok());
    }

    #[test]
    fn unmarked_directory_is_rejected_before_environment() {
        assert_eq!(
            validate_paths(
                Path::new("unmarked"),
                Path::new("unmarked/handy.exe"),
                Path::new("input.opus")
            )
            .unwrap_err(),
            "explicit private portable sandbox required"
        );
    }
}
