//! Optional user hook: `<app data dir>/hooks/transcription` receives the text
//! about to be pasted on stdin, and its stdout is pasted instead.

use anyhow::Context;
use log::error;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tauri::AppHandle;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Hung hooks are killed so they can't leave the pipeline stuck in "transcribing".
const HOOK_TIMEOUT: Duration = Duration::from_secs(30);

/// Returns the hook's output, or `text` unchanged if there's no hook or it
/// fails. Dropping the future kills the hook, so callers can cancel it.
pub async fn apply(app: &AppHandle, text: String) -> String {
    let Ok(dir) = crate::portable::app_data_dir(app) else {
        return text;
    };
    let hook = dir.join("hooks").join("transcription");
    apply_at_path(&hook, text).await
}

async fn apply_at_path(hook: &Path, text: String) -> String {
    if text.is_empty() || !hook.is_file() {
        return text;
    }

    let result = run(hook, &text).await;
    output_or_original(text, result)
}

fn output_or_original(text: String, result: anyhow::Result<String>) -> String {
    result.unwrap_or_else(|e| {
        error!("Transcription hook failed, using original text: {:#}", e);
        text
    })
}

async fn run(hook: &Path, text: &str) -> anyhow::Result<String> {
    run_command(Command::new(hook), text, HOOK_TIMEOUT).await
}

async fn run_command(
    mut command: Command,
    text: &str,
    timeout: Duration,
) -> anyhow::Result<String> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    // Write input while reading output so neither side blocks on a full pipe.
    // Write errors are ignored: a hook may exit without reading its input.
    let mut stdin = child.stdin.take().context("stdin not piped")?;
    let write = async move {
        let _ = stdin.write_all(text.as_bytes()).await;
    };
    let (_, output) = tokio::time::timeout(timeout, async {
        tokio::join!(write, child.wait_with_output())
    })
    .await
    .context("timed out")?;
    let output = output?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::ensure!(
        output.status.success(),
        "exited with {}: {}",
        output.status,
        stderr.trim()
    );

    // Drop one trailing newline so `echo` output doesn't submit a terminal line.
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.strip_suffix('\n').unwrap_or(&stdout).to_string())
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    fn powershell(script: &str) -> Command {
        let mut command = Command::new("powershell.exe");
        command.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ]);
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        command
    }

    #[tokio::test]
    async fn absent_empty_and_unlaunchable_hooks_keep_the_original_text() {
        let dir = tempfile::tempdir().unwrap();
        let hook = dir.path().join("transcription");
        assert_eq!(apply_at_path(&hook, "original".into()).await, "original");
        std::fs::write(&hook, "not an executable").unwrap();
        assert_eq!(apply_at_path(&hook, String::new()).await, "");
        assert_eq!(apply_at_path(&hook, "original".into()).await, "original");
    }

    #[tokio::test]
    async fn hook_success_transforms_stdin_without_pipe_deadlock() {
        let text = "original text ".repeat(10_000);
        let result = run_command(powershell("$text = [Console]::In.ReadToEnd(); [Console]::Out.Write($text.ToUpperInvariant() + \"`n\")"), &text, Duration::from_secs(10)).await;
        assert_eq!(
            output_or_original(text.clone(), result),
            text.to_uppercase()
        );
    }

    #[tokio::test]
    async fn hook_removes_exactly_one_trailing_newline_and_keeps_empty_output() {
        let result = run_command(
            powershell("[Console]::Out.Write(\"changed`r`n`n\")"),
            "input",
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(result, "changed\r\n");
        let result = run_command(powershell("exit 0"), "input", Duration::from_secs(10)).await;
        assert_eq!(output_or_original("input".into(), result), "");
    }

    #[tokio::test]
    async fn hook_failure_preserves_detail_and_falls_back_to_original() {
        let result = run_command(
            powershell("[Console]::Error.Write('hook failure detail'); exit 9"),
            "original",
            Duration::from_secs(10),
        )
        .await;
        let detail = format!("{:#}", result.as_ref().unwrap_err());
        assert!(
            detail.contains("9") && detail.contains("hook failure detail"),
            "{detail}"
        );
        assert_eq!(output_or_original("original".into(), result), "original");
    }

    #[tokio::test]
    async fn hook_timeout_falls_back_to_original() {
        let result = run_command(
            powershell("Start-Sleep -Seconds 60"),
            "original",
            Duration::from_millis(50),
        )
        .await;
        assert!(format!("{:#}", result.as_ref().unwrap_err()).contains("timed out"));
        assert_eq!(output_or_original("original".into(), result), "original");
    }

    #[tokio::test]
    async fn dropping_a_running_hook_future_kills_its_process() {
        use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
        use windows::Win32::System::Threading::{
            OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
        };
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("hook.pid");
        let mut command = powershell("[IO.File]::WriteAllText($env:HANDY_HOOK_TEST_PID_FILE, [string]$PID); Start-Sleep -Seconds 60");
        command.env("HANDY_HOOK_TEST_PID_FILE", &marker);
        let mut hook = Box::pin(run_command(command, "original", Duration::from_secs(20)));
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        let pid = loop {
            tokio::select! {
                result = &mut hook => panic!("hook ended before cancellation: {result:?}"),
                _ = tokio::time::sleep(Duration::from_millis(10)) => {}
            }
            if let Some(pid) = std::fs::read_to_string(&marker)
                .ok()
                .and_then(|text| text.parse::<u32>().ok())
            {
                break pid;
            }
            assert!(tokio::time::Instant::now() < deadline, "hook did not start");
        };
        let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }.unwrap();
        drop(hook);
        let ended = unsafe { WaitForSingleObject(process, 5_000) };
        unsafe { CloseHandle(process) }.unwrap();
        assert_eq!(ended, WAIT_OBJECT_0, "kill_on_drop must terminate the hook");
    }
}
