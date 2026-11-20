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

Every start attempt advances the delivery generation before recorder startup,
including an attempt that fails. A previous pending result then remains in
history instead of pasting. The final delivery gate deliberately retains the
capture lock through delivery: releasing it after the check would let a new
start invalidate the result before the actual paste. Cancellation also retains
this lock through the synchronous recorder drain. These choices can delay new
starts and queued UI updates by the delivery/drain duration, including a main
thread cancel. They are latency tradeoffs, not a deadlock proof or a hard
realtime guarantee; preserve the ordering when maintaining these paths.

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

The 2026-11-20 acceptance follow-up exported the actual Specta command/event
registry in an isolated temporary test, without starting the user app. Generated
bindings replace stale accelerator/model/settings declarations and match that
export. Its canonical staging/preparation measurements used synthetic 10/60-minute
float WAVs and 359-second Opus/WebM files in separate processes. These measure
only preparation, excluding models, queue, database and WebView; they establish
neither a universal RAM ceiling nor model-backed end-to-end acceptance. Commands,
fixture hashes, baseline/peak counters and exclusions are in the workspace's
`status/runs/UP-700-acceptance-followup-*` evidence.

A fresh debug build also transcribed synthetic Opus/WebM speech with an already
cached Parakeet model through the existing headless CLI, using a private portable
copy and no user settings/audio. Its normal debug startup re-exported the same
Specta contract. This CLI shares preparation and inference but does not insert
canonical history; it does not validate the GUI import/queue/history flow.

The explicit `--canonical-sandbox <directory> --canonical-import <file>` route
extends that same executable with HistoryManager, the production queue/importer,
finish/promotion and `canonical_history_page` readback. It never registers live
metadata, so completion stays history-only. No queue or worker protocol changed.
Before portable/plugin/manager initialization it requires a private directory
named `handy-up700-canonical-*`, the executable directly inside it, the exact
portable marker and `UP700-SANDBOX` containing `synthetic-only`, input beneath
`inputs`, local TEMP/TMP/HF paths and offline mode. Reparse points and conflicting
CLI modes are rejected. This is an acceptance interface for self-created test
data, not a general unattended importer. Use a newly created sandbox per case;
never point it at an installed app directory. History must be empty before import;
successful imports are matched by their returned capture ID. Rejections before
capture creation immediately write the original error and empty history to
`result.json` and exit 1. Preparation failures retain their failed capture/attempt
readback. Polling is every 250 ms with a finite 120-second deadline.
The path walk assumes no concurrent hostile sandbox mutation; it is not a
continuous filesystem security boundary. Settings/model/native DLL copies
are prepared outside the measured process; no downloads are needed.

The external Windows runner creates the root suspended, assigns it to a
non-inherited kill-on-close Job Object before resume, preserves the root exit
code, explicitly terminates remaining job processes on completion/error/timeout,
and waits for zero active job processes. This is necessary because process::exit
does not run the detached queue worker's destructors. Do not replace this runner
with subprocess.run(timeout=...) or a PID snapshot. The runner smoke also checks
live child/grandchild job membership and waits on retained process handles.

The workspace `status/runs/UP-700-canonical-headless-*` evidence records Opus and
WebM successes, corrupt supported input, missing selected model, unsupported
extension and empty supported input failures.
Certain chunks are pruned during successful promotion, so their final count is
zero for these short fixtures; retained compressed source hashes are compared
with the caller input, not with the prepared WAV. Maintain these checks when
changing import, recovery, promotion or worker startup. Owner runtime checks
remain separate acceptance work. This page is not a release approval.
