//! Bounded audio input and deterministic joins for background transcription.
//! Only working WAVs are generated; captures always point at the original.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rodio::source::UniformSourceIterator;
use rodio::Decoder;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::managers::history::HistoryManager;
use crate::managers::transcription::TranscriptionManager;
use crate::storage::repositories::transcriptions::{self, ChunkRecord, IncompleteJob};
use crate::TranscriptionOutput;

pub const SAMPLE_RATE: u32 = 16_000;
pub const INITIAL_WINDOW_SAMPLES: u64 = 30 * SAMPLE_RATE as u64;
pub const OVERLAP_SAMPLES: u64 = 2 * SAMPLE_RATE as u64;

pub fn inspect_wav(path: &Path) -> Result<u64, String> {
    let reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    if spec.sample_rate != SAMPLE_RATE
        || spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err("Expected 16 kHz mono 16-bit PCM WAV".into());
    }
    let samples = u64::from(reader.duration());
    if samples == 0 {
        return Err("Audio contains no samples".into());
    }
    Ok(samples)
}

/// Returns a bounded allocation even when `path` is hours long.
pub fn read_window(path: &Path, start: u64, end: u64) -> Result<Vec<f32>, String> {
    let total = inspect_wav(path)?;
    if start >= end
        || end > total
        || start > u32::MAX as u64
        || end - start > INITIAL_WINDOW_SAMPLES
    {
        return Err("Invalid or oversized audio window".into());
    }
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    reader.seek(start as u32).map_err(|e| e.to_string())?;
    reader
        .samples::<i16>()
        .take((end - start) as usize)
        .map(|sample| {
            sample
                .map(|s| s as f32 / i16::MAX as f32)
                .map_err(|e| e.to_string())
        })
        .collect()
}

/// Prefer a quiet 20 ms frame near the planned boundary. No pause means the
/// bounded boundary is kept and the overlap still protects nearby speech.
fn silence_cut(path: &Path, start: u64, end: u64, total: u64) -> Result<u64, String> {
    if end == total || end - start < 7 * SAMPLE_RATE as u64 {
        return Ok(end);
    }
    let probe_start = (end - 2 * SAMPLE_RATE as u64).max(start + 5 * SAMPLE_RATE as u64);
    let samples = read_window(path, probe_start, end)?;
    let frame = (SAMPLE_RATE / 50) as usize;
    let quiet = samples
        .chunks_exact(frame)
        .enumerate()
        .rfind(|(_, chunk)| chunk.iter().map(|s| s.abs()).sum::<f32>() / (frame as f32) < 0.01);
    Ok(quiet.map_or(end, |(index, _)| probe_start + ((index + 1) * frame) as u64))
}

/// Decode WAV/MP3 into one seekable work WAV using the already-linked rodio
/// decoder. Source bytes are never changed; no full-audio Vec is created.
pub fn prepare_working_wav(
    source: &Path,
    work_dir: &Path,
    capture_id: &str,
) -> Result<PathBuf, String> {
    let extension = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "wav" && extension != "mp3" {
        return Err("Only WAV and MP3 files are supported".into());
    }
    if extension == "wav" && inspect_wav(source).is_ok() {
        return Ok(source.to_path_buf());
    }

    std::fs::create_dir_all(work_dir).map_err(|e| e.to_string())?;
    let work_path = work_dir.join(format!("{capture_id}.wav"));
    if inspect_wav(&work_path).is_ok() {
        return Ok(work_path);
    }
    let file = File::open(source).map_err(|e| e.to_string())?;
    let decoder = Decoder::try_from(file).map_err(|e| format!("Cannot decode audio: {e}"))?;
    let staging = work_dir.join(format!("{capture_id}.part"));
    let mut writer = hound::WavWriter::create(
        &staging,
        hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .map_err(|e| e.to_string())?;
    let converted = UniformSourceIterator::new(decoder, 1, SAMPLE_RATE);
    let mut count = 0_u64;
    for sample in converted {
        let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(scaled).map_err(|e| e.to_string())?;
        count += 1;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    if count == 0 {
        return Err("Audio contains no samples".into());
    }
    if work_path.exists() {
        std::fs::remove_file(&work_path).map_err(|e| e.to_string())?;
    }
    std::fs::rename(staging, &work_path).map_err(|e| e.to_string())?;
    Ok(work_path)
}

fn words(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

fn comparable(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Returns joined text and whether the seam needs human inspection.
/// No uncertain word is silently removed.
pub fn join_chunks(left: &str, right: &str) -> (String, bool) {
    join_at_boundary(left, left, right)
}

fn join_at_boundary(merged: &str, previous_chunk: &str, right: &str) -> (String, bool) {
    let a = words(previous_chunk);
    let b = words(right);
    let limit = a.len().min(b.len()).min(16);
    for n in (3..=limit).rev() {
        let chain: Vec<_> = a[a.len() - n..]
            .iter()
            .map(|word| comparable(word))
            .collect();
        if chain.iter().any(String::is_empty) {
            continue;
        }
        let matches = |window: &[&str]| {
            window
                .iter()
                .map(|word| comparable(word))
                .eq(chain.iter().cloned())
        };
        if matches(&b[..n])
            && a.windows(n).filter(|window| matches(window)).count() == 1
            && b.windows(n).filter(|window| matches(window)).count() == 1
        {
            let tail = b[n..].join(" ");
            return (
                format!("{} {}", merged.trim(), tail).trim().to_string(),
                false,
            );
        }
    }
    (
        format!("{} {}", merged.trim(), right.trim())
            .trim()
            .to_string(),
        !previous_chunk.is_empty() && !right.is_empty(),
    )
}

pub fn transcribe_headless_file(
    manager: &TranscriptionManager,
    model_id: &str,
    device_index: Option<usize>,
    path: &Path,
    total: u64,
) -> Result<String, String> {
    let mut start = 0_u64;
    let mut text = String::new();
    let mut previous_raw = String::new();
    let mut language = None;
    while start < total {
        if !manager.is_model_loaded() {
            manager
                .load_model_with_device(model_id, device_index)
                .map_err(|e| e.to_string())?;
        }
        let model_limit = manager
            .model_max_audio_samples()
            .unwrap_or(INITIAL_WINDOW_SAMPLES);
        let mut window =
            INITIAL_WINDOW_SAMPLES.min(model_limit.saturating_sub(2 * OVERLAP_SAMPLES));
        if window < 5 * SAMPLE_RATE as u64 {
            return Err("Model audio window is too small for safe overlap".into());
        }
        let mut piece = None;
        for attempt in 0..=2 {
            let end = (start + window).min(total);
            let samples = read_window(
                path,
                start.saturating_sub(OVERLAP_SAMPLES),
                (end + OVERLAP_SAMPLES).min(total),
            )?;
            match manager.transcribe_engine_raw(samples) {
                Ok(output) => {
                    piece = Some((end, output));
                    break;
                }
                Err(error) if attempt < 2 => {
                    log::warn!("Headless window failed; retrying smaller section: {error}");
                    window /= 2;
                    if !manager.is_model_loaded() {
                        manager
                            .load_model_with_device(model_id, device_index)
                            .map_err(|e| e.to_string())?;
                    }
                }
                Err(error) => return Err(error.to_string()),
            }
        }
        let (end, output) = piece.ok_or("Headless window retries exhausted")?;
        text = join_at_boundary(&text, &previous_raw, &output.engine_raw).0;
        previous_raw = output.engine_raw;
        language = language.or(output.language);
        start = end;
    }
    Ok(manager.normalize_merged_raw(text, language).delivered_text)
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum WorkerRequest {
    Load {
        model_id: String,
    },
    Transcribe {
        path: PathBuf,
        start: u64,
        end: u64,
    },
    Normalize {
        raw: String,
        language: Option<String>,
    },
}

#[derive(Serialize, Deserialize)]
struct WorkerResponse {
    output: Option<TranscriptionOutput>,
    error: Option<String>,
    max_audio_samples: Option<u64>,
}

/// Runs in the existing executable's headless mode. Only one bounded request
/// is held at a time; the parent owns persistence and scheduling.
pub fn run_worker(app: &AppHandle) -> i32 {
    let tm = app.state::<Arc<TranscriptionManager>>();
    let mut selected_model: Option<String> = None;
    let stdin = std::io::stdin();
    let mut stdout = BufWriter::new(std::io::stdout().lock());
    for line in stdin.lock().lines() {
        let response = match line {
            Ok(line) => match serde_json::from_str::<WorkerRequest>(&line) {
                Ok(WorkerRequest::Load { model_id }) => {
                    let result = tm.load_model_with_device(&model_id, None);
                    if result.is_ok() {
                        selected_model = Some(model_id);
                    }
                    WorkerResponse {
                        output: None,
                        error: result.err().map(|e| e.to_string()),
                        max_audio_samples: tm.model_max_audio_samples(),
                    }
                }
                Ok(WorkerRequest::Transcribe { path, start, end }) => {
                    let result = selected_model
                        .as_ref()
                        .ok_or("Worker model not loaded".to_string())
                        .and_then(|model_id| {
                            if tm.is_model_loaded() {
                                Ok(())
                            } else {
                                tm.load_model_with_device(model_id, None)
                                    .map_err(|e| e.to_string())
                            }
                        })
                        .and_then(|_| read_window(&path, start, end))
                        .and_then(|samples| {
                            tm.transcribe_engine_raw(samples).map_err(|e| e.to_string())
                        });
                    match result {
                        Ok(output) => WorkerResponse {
                            output: Some(output),
                            error: None,
                            max_audio_samples: None,
                        },
                        Err(error) => WorkerResponse {
                            output: None,
                            error: Some(error),
                            max_audio_samples: None,
                        },
                    }
                }
                Ok(WorkerRequest::Normalize { raw, language }) => {
                    let result = selected_model
                        .as_ref()
                        .ok_or("Worker model not loaded".to_string())
                        .and_then(|model_id| {
                            if tm.is_model_loaded() {
                                Ok(())
                            } else {
                                tm.load_model_with_device(model_id, None)
                                    .map_err(|e| e.to_string())
                            }
                        })
                        .map(|_| tm.normalize_merged_raw(raw, language));
                    match result {
                        Ok(output) => WorkerResponse {
                            output: Some(output),
                            error: None,
                            max_audio_samples: None,
                        },
                        Err(error) => WorkerResponse {
                            output: None,
                            error: Some(error),
                            max_audio_samples: None,
                        },
                    }
                }
                Err(error) => WorkerResponse {
                    output: None,
                    error: Some(format!("Invalid worker request: {error}")),
                    max_audio_samples: None,
                },
            },
            Err(error) => WorkerResponse {
                output: None,
                error: Some(error.to_string()),
                max_audio_samples: None,
            },
        };
        if serde_json::to_writer(&mut stdout, &response).is_err()
            || stdout.write_all(b"\n").is_err()
            || stdout.flush().is_err()
        {
            return 1;
        }
    }
    0
}

pub struct WorkerClient {
    child: Child,
    stdin: ChildStdin,
    replies: Receiver<Result<String, String>>,
    max_audio_samples: Option<u64>,
}

impl WorkerClient {
    pub fn start() -> Result<Self, String> {
        let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .arg("--transcription-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Cannot start transcription worker: {e}"))?;
        let stdin = child.stdin.take().ok_or("Worker stdin unavailable")?;
        let stdout = child.stdout.take().ok_or("Worker stdout unavailable")?;
        let (reply_tx, replies) = mpsc::channel();
        std::thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                let reply = match stdout.read_line(&mut line) {
                    Ok(0) => Err("Transcription worker exited unexpectedly".to_string()),
                    Ok(_) => Ok(line),
                    Err(error) => Err(error.to_string()),
                };
                let done = reply.is_err();
                if reply_tx.send(reply).is_err() || done {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            replies,
            max_audio_samples: None,
        })
    }

    fn request(
        &mut self,
        request: WorkerRequest,
        timeout: Duration,
    ) -> Result<WorkerResponse, String> {
        serde_json::to_writer(&mut self.stdin, &request).map_err(|e| e.to_string())?;
        self.stdin
            .write_all(b"\n")
            .and_then(|_| self.stdin.flush())
            .map_err(|e| e.to_string())?;
        let line = self
            .replies
            .recv_timeout(timeout)
            .map_err(|_| "Transcription worker timed out or exited".to_string())??;
        serde_json::from_str(&line).map_err(|e| format!("Invalid worker response: {e}"))
    }

    pub fn load(&mut self, model_id: &str) -> Result<(), String> {
        let reply = self.request(
            WorkerRequest::Load {
                model_id: model_id.into(),
            },
            Duration::from_secs(180),
        )?;
        self.max_audio_samples = reply.max_audio_samples;
        reply.error.map_or(Ok(()), Err)
    }

    pub fn transcribe(
        &mut self,
        path: &Path,
        start: u64,
        end: u64,
    ) -> Result<TranscriptionOutput, String> {
        let audio_secs = (end - start).div_ceil(SAMPLE_RATE as u64);
        let timeout = Duration::from_secs((audio_secs * 10).clamp(180, 900));
        let reply = self.request(
            WorkerRequest::Transcribe {
                path: path.into(),
                start,
                end,
            },
            timeout,
        )?;
        reply.output.ok_or_else(|| {
            reply
                .error
                .unwrap_or_else(|| "Worker returned no text".into())
        })
    }

    pub fn normalize(
        &mut self,
        raw: String,
        language: Option<String>,
    ) -> Result<TranscriptionOutput, String> {
        let reply = self.request(
            WorkerRequest::Normalize { raw, language },
            Duration::from_secs(180),
        )?;
        reply.output.ok_or_else(|| {
            reply
                .error
                .unwrap_or_else(|| "Worker returned no normalized text".into())
        })
    }
}

impl Drop for WorkerClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn ensure_worker<'a>(
    worker: &'a mut Option<WorkerClient>,
    loaded_model: &mut Option<String>,
    model_id: &str,
) -> Result<&'a mut WorkerClient, String> {
    if worker.is_none() || loaded_model.as_deref() != Some(model_id) {
        *worker = Some(WorkerClient::start()?);
        worker.as_mut().expect("worker started").load(model_id)?;
        *loaded_model = Some(model_id.to_string());
    }
    Ok(worker.as_mut().expect("worker loaded"))
}

/// One scheduler thread and at most one native child process. The SQLite
/// attempts are the queue's durable source of truth, not another status store.
pub struct TranscriptionQueue {
    wake_tx: Sender<()>,
    live: Arc<Mutex<HashMap<String, LiveJob>>>,
}

pub struct LiveJob {
    pub post_process: bool,
    pub generation: u64,
    pub stopped_at: Instant,
    pub duration_samples: u64,
}

fn choose_job<'a>(jobs: &'a [IncompleteJob], short_streak: &mut u8) -> &'a IncompleteJob {
    let is_short = |job: &&IncompleteJob| job.audio_duration_ms.is_some_and(|ms| ms <= 60_000);
    let short = jobs.iter().find(is_short);
    let long = jobs.iter().rev().find(|job| !is_short(job));
    match (short, long) {
        (Some(short), Some(_)) if *short_streak < 3 => {
            *short_streak += 1;
            short
        }
        (_, Some(long)) => {
            *short_streak = 0;
            long
        }
        (Some(short), None) => {
            *short_streak = 0;
            short
        }
        (None, None) => unreachable!("queue called only with jobs"),
    }
}

fn next_window(previous: Option<&ChunkRecord>) -> u64 {
    previous
        .map(|chunk| chunk.window_samples.max(5 * SAMPLE_RATE as i64) as u64)
        .unwrap_or(INITIAL_WINDOW_SAMPLES)
        .min(INITIAL_WINDOW_SAMPLES)
}

impl TranscriptionQueue {
    pub fn start(app: AppHandle) -> Self {
        if let Some(hm) = app.try_state::<Arc<HistoryManager>>() {
            let dir = hm.recordings_dir().join(".processing");
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let id = name
                        .strip_suffix(".import.part")
                        .or_else(|| name.strip_suffix(".part"));
                    if id.is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
                        && entry.path().is_file()
                    {
                        if let Err(error) = std::fs::remove_file(entry.path()) {
                            log::warn!("Could not remove abandoned work file {name}: {error}");
                        }
                    }
                }
            }
        }
        let (wake_tx, wake_rx) = mpsc::channel();
        let live: Arc<Mutex<HashMap<String, LiveJob>>> = Arc::new(Mutex::new(HashMap::new()));
        let live_for_worker = Arc::clone(&live);
        std::thread::spawn(move || {
            let mut worker: Option<WorkerClient> = None;
            let mut loaded_model: Option<String> = None;
            let mut short_streak = 0_u8;
            loop {
                let hm = app.state::<Arc<HistoryManager>>();
                let db = match hm.canonical_db() {
                    Ok(db) => db,
                    Err(error) => {
                        log::error!("Transcription queue cannot open history: {error}");
                        if wake_rx.recv_timeout(Duration::from_secs(5)).is_err() {
                            continue;
                        }
                        continue;
                    }
                };
                let jobs = match transcriptions::incomplete_jobs(&db) {
                    Ok(jobs) => jobs,
                    Err(error) => {
                        log::error!("Transcription queue cannot read jobs: {error}");
                        let _ = wake_rx.recv_timeout(Duration::from_secs(5));
                        continue;
                    }
                };
                if jobs.is_empty() {
                    if wake_rx.recv().is_err() {
                        break;
                    }
                    continue;
                }
                let job = choose_job(&jobs, &mut short_streak);
                if let Err(error) = process_one_chunk(
                    &app,
                    &db,
                    job,
                    &mut worker,
                    &mut loaded_model,
                    &live_for_worker,
                ) {
                    log::error!("Transcription job {} failed: {error}", job.attempt_id);
                    worker = None;
                    loaded_model = None;
                    live_for_worker.lock().unwrap().remove(&job.attempt_id);
                    let failed = transcriptions::complete_attempt_with_raw(
                        &db,
                        &job.attempt_id,
                        None,
                        None,
                        Some(&error),
                        None,
                        None,
                    );
                    let work = hm
                        .recordings_dir()
                        .join(".processing")
                        .join(format!("{}.wav", job.capture_id));
                    let _ = std::fs::remove_file(work);
                    let part = hm
                        .recordings_dir()
                        .join(".processing")
                        .join(format!("{}.part", job.capture_id));
                    let _ = std::fs::remove_file(part);
                    let _ = app.emit("canonical-history-changed", ());
                    let _ = app.emit("transcription-error", error);
                    if let Err(db_error) = failed {
                        log::error!(
                            "Could not persist failed attempt {}: {db_error}",
                            job.attempt_id
                        );
                        let _ = wake_rx.recv_timeout(Duration::from_secs(5));
                    }
                }
            }
        });
        let queue = Self { wake_tx, live };
        queue.wake();
        queue
    }

    pub fn wake(&self) {
        let _ = self.wake_tx.send(());
    }

    pub fn register_live(&self, attempt_id: String, job: LiveJob) {
        self.live.lock().unwrap().insert(attempt_id, job);
        self.wake();
    }
}

fn process_one_chunk(
    app: &AppHandle,
    db: &crate::storage::database::AppDatabase,
    job: &IncompleteJob,
    worker: &mut Option<WorkerClient>,
    loaded_model: &mut Option<String>,
    live: &Arc<Mutex<HashMap<String, LiveJob>>>,
) -> Result<(), String> {
    let hm = app.state::<Arc<HistoryManager>>();
    if Path::new(&job.audio_file_name)
        .file_name()
        .and_then(|n| n.to_str())
        != Some(job.audio_file_name.as_str())
    {
        return Err("Invalid recording filename in history".into());
    }
    let source = hm.recordings_dir().join(&job.audio_file_name);
    let work_dir = hm.recordings_dir().join(".processing");
    let wav = prepare_working_wav(&source, &work_dir, &job.capture_id)?;
    let total = inspect_wav(&wav)?;
    let chunks =
        transcriptions::chunks_for_attempt(db, &job.attempt_id).map_err(|e| e.to_string())?;
    let start = chunks.last().map_or(0_u64, |chunk| chunk.end_sample as u64);
    let model_id = job
        .model_id
        .clone()
        .unwrap_or_else(|| crate::settings::get_settings(app).selected_model);
    if start >= total {
        finish_job(
            app,
            db,
            job,
            &chunks,
            &wav,
            &source,
            &model_id,
            worker,
            loaded_model,
            live,
        )?;
        return Ok(());
    }
    transcriptions::mark_running(db, &job.attempt_id).map_err(|e| e.to_string())?;
    let mut window = next_window(chunks.last());
    let overlap = if chunks.last().is_some_and(|chunk| chunk.seam_uncertain) {
        5 * SAMPLE_RATE as u64
    } else {
        OVERLAP_SAMPLES
    };
    let output = (0..=2)
        .find_map(|attempt| {
            let result = (|| {
                let client = ensure_worker(worker, loaded_model, &model_id)?;
                let max_input = client
                    .max_audio_samples
                    .unwrap_or(INITIAL_WINDOW_SAMPLES)
                    .min(INITIAL_WINDOW_SAMPLES);
                window = window.min(max_input.saturating_sub(2 * overlap));
                if window < 5 * SAMPLE_RATE as u64 {
                    return Err("Model audio window is too small for safe overlap".into());
                }
                let planned_end = (start + window).min(total);
                let end = silence_cut(&wav, start, planned_end, total)?;
                let read_start = start.saturating_sub(overlap);
                let read_end = (end + overlap).min(total);
                client
                    .transcribe(&wav, read_start, read_end)
                    .map(|output| (end, output))
            })();
            match result {
                Ok(output) => Some(Ok(output)),
                Err(error) if attempt < 2 => {
                    log::warn!("Worker window failed; retrying smaller section: {error}");
                    *worker = None;
                    *loaded_model = None;
                    window = (window / 2).max(SAMPLE_RATE as u64 * 5);
                    None
                }
                Err(error) => Some(Err(error)),
            }
        })
        .ok_or("Worker retries exhausted")??;
    let (end, output) = output;
    let previous = chunks
        .last()
        .and_then(|chunk| serde_json::from_str::<TranscriptionOutput>(&chunk.payload_json).ok());
    let seam_uncertain = previous
        .as_ref()
        .is_some_and(|prior| join_chunks(&prior.engine_raw, &output.engine_raw).1);
    let seam_left = seam_uncertain.then(|| {
        previous
            .as_ref()
            .unwrap()
            .engine_raw
            .chars()
            .rev()
            .take(100)
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    });
    let seam_right = seam_uncertain.then(|| output.engine_raw.chars().take(100).collect());
    transcriptions::insert_chunk(
        db,
        &job.attempt_id,
        &ChunkRecord {
            chunk_index: chunks.len() as i64,
            start_sample: start as i64,
            end_sample: end as i64,
            window_samples: window as i64,
            payload_json: serde_json::to_string(&output).map_err(|e| e.to_string())?,
            seam_uncertain,
            seam_left,
            seam_right,
        },
    )
    .map_err(|e| e.to_string())?;
    let _ = app.emit("canonical-history-changed", ());
    if end == total {
        let chunks =
            transcriptions::chunks_for_attempt(db, &job.attempt_id).map_err(|e| e.to_string())?;
        finish_job(
            app,
            db,
            job,
            &chunks,
            &wav,
            &source,
            &model_id,
            worker,
            loaded_model,
            live,
        )?;
    }
    Ok(())
}

// The two call sites share one explicit completion path; grouping borrowed
// app, database and worker state into a one-use type would obscure ownership.
#[allow(clippy::too_many_arguments)]
fn finish_job(
    app: &AppHandle,
    db: &crate::storage::database::AppDatabase,
    job: &IncompleteJob,
    chunks: &[ChunkRecord],
    wav: &Path,
    source: &Path,
    model_id: &str,
    worker: &mut Option<WorkerClient>,
    loaded_model: &mut Option<String>,
    live: &Arc<Mutex<HashMap<String, LiveJob>>>,
) -> Result<(), String> {
    let mut raw = String::new();
    let mut previous_raw = String::new();
    let mut language = None;
    for chunk in chunks {
        let output: TranscriptionOutput =
            serde_json::from_str(&chunk.payload_json).map_err(|e| e.to_string())?;
        raw = join_at_boundary(&raw, &previous_raw, &output.engine_raw).0;
        previous_raw = output.engine_raw;
        language = language.or(output.language);
    }
    if raw.trim().is_empty() {
        return Err("Audio contains no recognized speech".into());
    }
    let output = ensure_worker(worker, loaded_model, model_id)?.normalize(raw, language)?;
    if output.normalized_stt.trim().is_empty() {
        return Err("Audio contains no recognized speech".into());
    }
    transcriptions::complete_and_promote(
        db,
        &job.capture_id,
        &job.attempt_id,
        &output.engine_raw,
        &output.normalized_stt,
        job.model_id.as_deref(),
        output.language.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    if wav != source {
        if let Err(error) = std::fs::remove_file(wav) {
            log::warn!("Could not remove own work WAV {}: {error}", wav.display());
        }
    }
    let _ = app.emit("canonical-history-changed", ());
    log::info!(
        "Completed transcription {} ({} chars)",
        job.capture_id,
        output.delivered_text.len()
    );
    if let Some(meta) = live.lock().unwrap().remove(&job.attempt_id) {
        let app = app.clone();
        let capture_id = job.capture_id.clone();
        let attempt_id = job.attempt_id.clone();
        tauri::async_runtime::spawn(async move {
            crate::actions::deliver_background_live(&app, &capture_id, &attempt_id, output, meta)
                .await;
        });
    } else {
        let _ = app.emit("background-transcription-ready", &job.capture_id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_only_unambiguous_three_word_chain() {
        assert_eq!(
            join_chunks("Das ist ein guter Tag", "ein guter Tag für uns"),
            ("Das ist ein guter Tag für uns".into(), false)
        );
        assert_eq!(
            join_chunks("heute ist", "ist blau"),
            ("heute ist ist blau".into(), true)
        );
        assert!(join_chunks("und und und und", "und und und weiter").1);
        assert_eq!(
            join_chunks("Die Sonnenblume ist gelb,", "Sonnenblume ist gelb und groß"),
            ("Die Sonnenblume ist gelb, und groß".into(), false)
        );
    }

    #[test]
    fn earlier_repeated_phrase_does_not_change_a_safe_local_seam() {
        let first = "alpha beta gamma first one two";
        let second = "first one two alpha beta gamma";
        let third = "alpha beta gamma last";
        let merged = join_chunks(first, second);
        assert!(!merged.1);
        assert!(!join_chunks(second, third).1);
        assert_eq!(
            join_at_boundary(&merged.0, second, third),
            (
                "alpha beta gamma first one two alpha beta gamma last".into(),
                false
            )
        );
    }

    #[test]
    fn rejects_oversized_window_before_allocating() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bounded.wav");
        let mut writer = hound::WavWriter::create(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: SAMPLE_RATE,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();
        for _ in 0..(INITIAL_WINDOW_SAMPLES + 2 * OVERLAP_SAMPLES + 2) {
            writer.write_sample(123_i16).unwrap();
        }
        writer.finalize().unwrap();
        assert!(read_window(&path, 0, INITIAL_WINDOW_SAMPLES + 2 * OVERLAP_SAMPLES + 1).is_err());
        assert_eq!(read_window(&path, 16_000, 32_000).unwrap().len(), 16_000);
    }

    #[test]
    fn converts_working_audio_without_changing_the_original() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.wav");
        let mut writer = hound::WavWriter::create(
            &source,
            hound::WavSpec {
                channels: 1,
                sample_rate: 44_100,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();
        for sample in 0..44_100 {
            writer.write_sample((sample % 100) as i16).unwrap();
        }
        writer.finalize().unwrap();
        let before = std::fs::read(&source).unwrap();
        let working = prepare_working_wav(&source, &dir.path().join("work"), "capture").unwrap();
        assert_ne!(source, working);
        assert!((15_900..=16_100).contains(&inspect_wav(&working).unwrap()));
        assert_eq!(std::fs::read(&source).unwrap(), before);
    }

    #[test]
    fn short_dictations_preempt_long_work_without_starving_it() {
        let make_job = |id: &str, duration| IncompleteJob {
            attempt_id: id.into(),
            capture_id: id.into(),
            audio_file_name: format!("{id}.wav"),
            model_id: None,
            audio_duration_ms: Some(duration),
        };
        // Query order is newest first; a newly arriving short job must win
        // even if the long job has already processed many chunks.
        let jobs = [make_job("short", 10_000), make_job("long", 1_200_000)];
        let mut streak = 0;
        assert_eq!(choose_job(&jobs[1..], &mut streak).attempt_id, "long");
        for _ in 0..3 {
            assert_eq!(choose_job(&jobs, &mut streak).attempt_id, "short");
        }
        assert_eq!(choose_job(&jobs, &mut streak).attempt_id, "long");
        assert_eq!(choose_job(&jobs, &mut streak).attempt_id, "short");
    }

    #[test]
    fn successful_smaller_window_is_reused_after_restart() {
        let prior = ChunkRecord {
            chunk_index: 0,
            start_sample: 0,
            end_sample: (INITIAL_WINDOW_SAMPLES / 4) as i64,
            window_samples: (INITIAL_WINDOW_SAMPLES / 4) as i64,
            payload_json: String::new(),
            seam_uncertain: false,
            seam_left: None,
            seam_right: None,
        };
        assert_eq!(next_window(Some(&prior)), INITIAL_WINDOW_SAMPLES / 4);
    }

    #[test]
    fn quiet_boundary_is_selected_without_touching_original() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("speech.wav");
        let mut writer = hound::WavWriter::create(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: SAMPLE_RATE,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();
        for sample in 0..11 * SAMPLE_RATE {
            writer
                .write_sample(if sample / SAMPLE_RATE == 9 {
                    0_i16
                } else {
                    8_000_i16
                })
                .unwrap();
        }
        writer.finalize().unwrap();
        let end = silence_cut(&path, 0, 10 * SAMPLE_RATE as u64, 11 * SAMPLE_RATE as u64).unwrap();
        assert!(end >= 9 * SAMPLE_RATE as u64 && end <= 10 * SAMPLE_RATE as u64);
    }
}
