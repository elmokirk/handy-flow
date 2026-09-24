---
id: "run-audio-240"
title: "RUN_STATE — AUDIO-240"
type: "run-state"
status: "in_progress"
updated: "2026-09-24"
project: "custom-handy"
ticket_id: "AUDIO-240"
run_status: "IN_PROGRESS"
branch: "phase/06-long-audio-remediation"
last_commit: "b5da9a4"
---

# RUN_STATE — AUDIO-240

## Implemented

- One original WAV grows in `recordings`, with capture row from recording
  start and 30-second flush/sync; there is no fixed six-minute cap.
- One persistent child process performs bounded native inference. The parent
  owns SQLite checkpoints, short-job priority, restart recovery, failure
  cards, and the final canonical transcript. The in-process live preview is
  paused by owner decision for this release.
- WAV/MP3 import preserves original bytes and adds a manual-source badge.
  History shows pending/running/failed states, progress and uncertain seams.
- Explicit failed retries start a new attempt from zero because inference
  settings may have changed; interrupted pending attempts resume checkpoints.
- Source and package versions are 0.9.11. The branch and plan are pushed to
  GitHub. A local unsigned NSIS installer is built with the public updater
  frontend flag; CI signing secrets exist for a later published release.

## Verified locally

- `cargo test --no-default-features --lib -- --test-threads=1`: 223 passed.
- `cargo test --no-default-features --tests -- --test-threads=1`: passed.
  A parallel all-targets invocation first hit Windows linker artifact errors;
  serial execution passed. Focused attempt/recovery tests: 4 + 5 passed.
- `bun run build` with `VITE_UPDATE_SOURCE=public`, `bun run lint`,
  `bun run check:translations`, `bun run format:check`: passed.
- Private 9:40 WAV: three debug runs at 7.333/6.630/6.556 seconds and one
  release-binary run at 4.548 seconds on Vulkan1. Original SHA-256 remains
  `4224AC615AD3E1BB34C90BD44100AFBB6B1F2F465E853862ADDC8F543DD7C191`.
- Local repeated-audio fixtures of 15, 30 and 60 minutes completed in
  10.202, 22.506 and 45.456 seconds respectively. Their temporary WAVs were
  deleted; these are not a substitute for a real microphone dictation.
- Latest child-worker protocol loaded Parakeet Q8, returned raw text for a
  bounded 26-second request and normalized only after a separate request.
- Windows clippy ratchet: 17 distinct warnings against ceiling 22; passed.
- `bunx tauri build -b nsis` with updater artifacts disabled and no local
  signing key: passed. Final installer SHA-256:
  `9A4F30D2707A1609B3503E54540AAADEE6D4F18B0A555BD5B077AC65D02C70D1`.
- The 0.9.11 NSIS candidate installed successfully over 0.9.9 (installer
  exit 0; installed file and registry both report 0.9.11). Before installation,
  an online SQLite backup passed `quick_check`, v13, and 2,280 captures.
  On first 0.9.11 launch, v14 migration passed `quick_check`, retained all
  2,280 captures and recovered three orphan WAVs (two retryable, one corrupt).
  The migration also created its verified backup snapshot.
- The installed app remained running after first launch. Updater checks are
  enabled in local settings and this build uses the public updater source;
  the latest public release is 0.9.10, so a 0.9.11 install cannot yet exercise
  a newer-version download.
- Separate requirement/safety and simplicity reviews found no remaining
  concrete code blocker in their targeted paths.

## Release blockers still open

- A **real 15-minute Windows microphone dictation** must succeed, with one
  original WAV, one history card and a usable transcript.
- Installed-app E2E must exercise WAV/MP3 import, worker kill/restart,
  short-job preemption, playback, retry, and progress in the actual UI.
- The public signed release and in-app updater test from an older installed
  version are pending; do not publish until the real 15-minute gate passes.
- Linux GitHub quality run is in progress.
  macOS/Linux release-path testing has not yet been established.
