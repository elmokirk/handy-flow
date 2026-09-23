---
id: "run-audio-240"
title: "RUN_STATE — AUDIO-240"
type: "run-state"
status: "in_progress"
updated: "2026-09-23"
project: "custom-handy"
ticket_id: "AUDIO-240"
run_status: "IN_PROGRESS"
branch: "phase/06-long-audio-remediation"
last_commit: "b955c96"
---

# RUN_STATE — AUDIO-240

## Implemented

- Plan, ADR-025 and ticket documented in commit `8856961`.
- Orphan WAV recovery is invoked at startup and on history reads; recovered
  cards use an estimated recording start from file mtime minus WAV duration.
- Corrupt or empty orphan WAVs receive `audio_corrupt`; recovery is idempotent.
- The history UI distinguishes pending, recovered and corrupt audio; retry and
  playback are disabled for corrupt audio.
- Automatic canonical retention excludes captures without a successful
  canonical transcription, so failed and unfinished recordings remain.
- Live dictation now inserts the capture before microphone startup, refines
  its timestamp at the first real sample, writes one final-name WAV from the
  recorder consumer, and reuses that capture for success or failure.
- The same WAV is flushed and disk-synced every 30 seconds and at stop; no
  extra original or on-disk chunk file is generated. Cancellation records a
  failed attempt rather than leaving an unexplained pending card.
- A pending attempt is inserted after WAV verification and before inference,
  then completed in place without duplicating the capture. Startup converts
  interrupted attempts to explicit retryable failures; history cards display
  pending/failed status and the local error detail.
- Until isolated bounded inference exists, unstreamed batch calls over six
  minutes are rejected safely and retain the recording for later retry.
- The private 9:40 WAV was only read for SHA-256 confirmation and remains
  unchanged (`4224AC615AD3E1BB34C90BD44100AFBB6B1F2F465E853862ADDC8F543DD7C191`).

## Verification

- `cargo test --test orphan_recovery_test --no-default-features`: 3 passed.
- `cargo test --test canonical_history_test --no-default-features`: 2 passed.
- `cargo test --test live_pipeline_test --no-default-features`: 9 passed,
  including capture reuse, first-sample timestamp, and pending-attempt completion.
- Re-run `cargo test --test orphan_recovery_test --no-default-features`: 5 passed,
  including interruption repair and active-recording safety.
- `cargo test original_wav_flushes_same_file_and_finalizes_at_stop --lib
  --no-default-features`: 1 passed.
- `npm run build`: passed; Vite emitted an existing large-chunk warning.
- `git diff --check`: passed.

## Not yet implemented — release blocker

The live recorder still buffers all PCM and its long-audio path still runs
in-process without bounded sample windows, worker isolation, persisted chunk
checkpoints, deterministic seam review, or fair short-job scheduling. The
9:40 regression benchmark, 30/60-minute gates, complete E2E, version bump,
Tauri production build, installer and updater-button test are not done. Do
not release or run the private WAV through the unsafe current inference path.
