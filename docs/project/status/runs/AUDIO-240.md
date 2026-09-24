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
last_commit: "bf339bc"
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
- The installed 0.9.11 candidate exposed a foreground-UX regression. The
  0.9.12 hotfix restores one final paste for timely dictations up to six
  minutes, tries one isolated inference step when the engine permits it,
  falls back to bounded work, and updates history progress in-place. A
  Windows foreground-window check prevents pasting into another app; it
  cannot distinguish fields within the same window. Source/package versions
  were 0.9.12; CI signing secrets remain reserved for a later public release.
- The 0.9.13 source candidate adds seam-fragment review and append-only user
  confirmation. History, tray copy and the version-2 knowledge export prefer
  the latest confirmed text; raw attempt text and source audio remain intact.
  Search indexes each confirmation in the same SQLite transaction.

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
- Hotfix baseline `a16698b`: a red/green regression proved the 60-second
  delivery guard; the installed SQLite audit showed delivery for 27.72s but
  none for successful 83.22s, 115.71s, 174.75s and 347.64s dictations.
  The 0.9.12 Rust suite passed 224 tests; frontend build/lint and the
  23-language translation check passed. Two separate final reviews found no
  remaining concrete code finding in the hotfix diff.
- Read-only one-shot probes on the owner's Parakeet Q8/Vulkan model completed
  83s, 116s and 5:48 WAVs in the child worker. The 5:48 warm inference took
  5.266 seconds; source SHA was checked before/after. Its output differs
  materially from the previous chunked text, so audio/text quality must be
  judged by a human; speed and exit status alone do not prove accuracy.
- The 0.9.12 local NSIS build passed with the public updater frontend flag
  and updater artifacts disabled only for this unsigned candidate. The
  temporary override was removed after the build. The installer is
  `src-tauri/target/release/bundle/nsis/Handy Flow_0.9.12_x64-setup.exe`
  (21,699,897 bytes, SHA-256
  `997272525EA6AD8C01519E800D0D9E6DA53C812339FE5872DDF05617B69499C7`);
  the binary reports file/product version 0.9.12. Format check and Windows
  clippy ratchet passed (18 distinct warnings, ceiling 22).

## Release blockers still open

- A **real 15-minute Windows microphone dictation** must succeed, with one
  original WAV, one history card and a usable transcript.
- The owner has not yet reviewed the one-shot transcript quality. The seam
  editor and preferred Knowledge-Base text are coded but need installed UI
  verification and a human seam-quality review.
- A candidate must be installed and its focused-field paste,
  spinner/no-flicker behavior, and >200-card progress/filter case exercised.
- The owner superseded the prior public-release hold: publish a signed 0.9.13
  Windows x64 NSIS update via the in-app updater, then perform real microphone,
  listening and installed-UI validation. These results remain unproven and
  must not be represented as passed.
- Installed-app E2E must exercise WAV/MP3 import, worker kill/restart,
  short-job preemption, playback, retry, and progress in the actual UI.
- The public signed release and in-app updater test from installed 0.9.11 are
  pending. Release workflow must verify the 0.9.13 NSIS signature and manifest
  before publishing; the owner will exercise the actual update button.
- The 0.9.13 GitHub quality run `36023511312` passed (Rust, Clippy, frontend,
  Playwright). The Windows release build is pending.
  macOS/Linux release-path testing has not yet been established.
