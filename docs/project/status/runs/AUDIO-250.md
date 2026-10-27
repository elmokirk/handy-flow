---
id: "run-audio-250"
title: "RUN_STATE — AUDIO-250"
type: "run-state"
status: "in_review"
updated: "2026-10-26"
project: "custom-handy"
ticket_id: "AUDIO-250"
run_status: "IN_REVIEW"
branch: "feat/AUDIO-250-import-formats"
---

# RUN_STATE — AUDIO-250

Implemented on `feat/AUDIO-250-import-formats` (off `phase/06` at `6c74b11`)
as commit `02cae85` plus this documentation commit. The import pipeline now
accepts the natively supported set: WAV, MP3, M4A/MP4/M4B/M4R (AAC-LC),
ADTS `.aac`, FLAC, OGG/OGA (Vorbis) and audio-only MOV/MP4. Opus and AMR
recordings are rejected with a message naming every supported format. No
converter was added: the pinned rodio fork already compiles Symphonia 0.5.5
with the isomp4, AAC, OGG-Vorbis and FLAC codecs. The headless
`--transcribe-file` CLI now runs through the same bounded pipeline.

Verification (owner machine, Windows x64, 2026-10-26):

- `cargo test -j 4` in `src-tauri`: exit 0 — 231 lib tests and every
  integration target; the new long_audio tests pass.
- Real AAC: the owner's WhatsApp ADTS sample from the Downloads folder
  decodes through the env-gated ignored test
  (`HANDY_TEST_AAC=... cargo test -j 4 --lib -- --ignored --exact
  long_audio::tests::decodes_real_recording_when_provided`).
- Clippy ratchet PASS with 18 distinct warnings < ceiling 22.
- bun lint PASS; prettier and cargo fmt PASS; translation parity PASS
  (23 languages); frontend build (tsc + vite) PASS.
- Playwright skipped deliberately: the suite has no coverage for the native
  file-dialog import flow, and the UI change is a filter array plus hint
  strings.

Environment notes: plain `cargo test` on this machine can fail with
os error 1455 (pagefile mmap pressure while other apps run) after clippy
runs; rerun as `cargo test -j 4` from `src-tauri`. Poisoned check-mode
artifacts after failed clippy runs were cleared from `target/debug/deps`
before rerunning.

Release: the version was bumped to 0.9.14 and the signed public updater
workflow ran green (run 40389324028, dispatched from this branch). The
draft was published as public Latest `v0.9.14`; assets
`Handy.Flow_0.9.14_x64-setup.exe` (20.5 MB) + `.sig` + `latest.json` are
present and the public updater endpoint returns 0.9.14 with both platform
aliases pointing at the signed installer.

Hardening (owner request, 2026-10-27): the 0.9.14 device validation hit
"Zugriff verweigert (os error 5)" on two imports directly after the
updater restart. Post-mortem: the source stayed readable (bare io error
rules out hash_file) while staging copy/rename caught a transient Windows
lock window; afterwards every operation plus a full headless
`--transcribe-file` run of the installed binary transcribed the same
WhatsApp AAC sample end to end (213.7 s audio, 24 s, Parakeet Q8/Vulkan).
Commit `bc31315` absorbs os error 5/32 on copy/rename (3 attempts,
50/150/250 ms backoff; bare io error preserved for the UI), logs import
phases and retries to handy.log, and adds retry/exhaustion tests.
`cargo test -j 4` passes (234 lib tests + integration), the owner AAC
sample decodes via the env-gated test, clippy ratchet 18 < 22. Shipped as
signed Latest `v0.9.15` (run 40524746881 from this branch; updater
endpoint verified at 0.9.15). Still IN_REVIEW: the owner validates the
actual in-app import of the WhatsApp AAC recording.