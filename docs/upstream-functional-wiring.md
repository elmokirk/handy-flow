# Canonical dictation wiring (UP-700)

This describes the functional upstream port. UP-700 acceptance remains open;
unit tests do not establish microphone, model, or installed-app acceptance.

## Component and data map

- `src-tauri/src/actions.rs` owns dictation lifecycle and foreground delivery.
  Each start freezes a delivery generation and cancellation generation. Stop
  carries that identity into the queue rather than reading a newer generation.
- `src-tauri/src/managers/audio.rs` drains the recorder into `StoppedRecording`.
  Samples, digital silence and the original error are frozen before Idle permits
  another recording. Stop claims the exact take synchronously before dispatching
  its drain, so a concurrent cancel cannot collect its sink first. Readiness
  invalidation is separate from delivery ownership.
- `src-tauri/src/audio_toolkit/audio/sink.rs` writes the canonical WAV before
  voice activity detection. A checkpoint failure retains its detail and partial
  WAV. Its delayed stop callback names the unique WAV, not just a reusable binding.
- `src-tauri/src/utils.rs` routes cancellation through the same capture lock and
  guards its delayed UI reset with the cancellation session. A write error remains
  the failed attempt's reason even when cancellation wins the drain.
- `src-tauri/src/long_audio.rs` schedules canonical attempts and isolated workers.
  Live metadata registration covers the durable attempt insert, so an already
  awake worker cannot complete without the corresponding dictation identity.
  Missing model selection fails the attempt before starting native inference.
- `src-tauri/src/managers/transcription.rs` shares `PreviewScript` conversion
  with production normalization; this does not enable live preview.
- `src-tauri/src/storage/repositories/` owns database access. The attempt keeps
  engine output and normalized text; optional post-processing is a representation.
- `src-tauri/src/transcription_hook.rs` transforms only eligible delivery text
  after history is committed. Imports and late/history-only results skip the hook.
- `src-tauri/src/delivery/` selects the existing sink and records the hash of
  the actual delivered text. Hook output never replaces the original history.

## Maintaining session boundaries

Use `update_dictation_ui` for asynchronous dictation notices and overlay changes.
It rechecks ownership on the main thread and serializes updates with starts;
stale work must neither clear nor replace a newer notice. Digital silence,
voice-activity rejection/no recognized speech, and recording failure have distinct
outcomes. Failed audio remains associated with its canonical capture/attempt;
existing confirmed-empty/no-speech cleanup rules still apply.

Keep all foreground delivery checks in `live_delivery_is_ready`: capture active,
same generation and cancellation state, no current recording, same destination
and (for focused-app delivery) foreground window, at most six minutes of audio,
and at most 30 seconds since stop. Check before the hook, while it waits, after
completion, and again on the main thread immediately before delivery.

Cancellation checks run every 25 ms while awaiting the hook. Dropping the future
reaches the existing `kill_on_drop`; hook timeout remains 30 seconds. Failed hooks
fall back to their input text, empty stdout suppresses delivery, and exactly one
trailing LF is removed. Expired foreground eligibility still makes the result
history-only, even when the hook itself could return fallback text.

## Verification and remaining acceptance

Run `cargo test --manifest-path src-tauri/Cargo.toml -j4`, strict all-target and
all-feature Clippy, and `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
Run the existing Bun lint, format, build, translation and Playwright scripts when
changing completion events or frontend behavior. Reuse the existing target tree;
run heavy Cargo jobs serially and check disk before each run.

Regression tests cover script conversion, silence/error classification, stale
capture claims, session ownership, queue registration, hook process I/O,
fallback/timeout/drop, delivery eligibility, immutable history, and delivery audit.
The hook process tests run on Windows and require Windows PowerShell.

Owner runtime checks, measured import RAM bounds, actual model-backed Opus/WebM
import, and a real Specta export remain separate acceptance work. This page is
not a release approval or a claim that those checks ran.
