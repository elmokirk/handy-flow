---
id: "ticket-audio-250"
title: "AUDIO-250 — Extended native audio import formats"
type: "ticket"
status: "in_review"
updated: "2026-10-26"
project: "custom-handy"
ticket_id: "AUDIO-250"
phase: "6"
owner: "Integrator"
gate: "G6"
catalog_ref: "TICKET_CATALOG.json#AUDIO-250"
branch: "feat/AUDIO-250-import-formats"
---

# AUDIO-250 — Extended native audio import formats

**Baseline:** branched off `phase/06-long-audio-remediation` at `6c74b11`
(owner request, 2026-10-26): import phone recordings without conversion.

## Outcome

The history import (picker + drag-and-drop) accepts every audio format the
already-linked rodio/Symphonia stack decodes natively: WAV, MP3, M4A/MP4
(AAC-LC), raw ADTS AAC, FLAC and OGG-Vorbis, plus audio-only MOV/MP4.
Phone recordings from iOS/Android recorder apps and WhatsApp audio shares
import without any external converter. Recordings needing absent decoders
(Opus voice notes, AMR) are rejected with a named-formats error.

## Scope and decisions

- Keep the existing `manual` origin: no new tag; imported phone recordings
  already surface as "Manually imported" with their own filter.
- Audio-track selection inside video containers (screen recordings with a
  H.264 track) is deferred: rodio picks the first track, so such files fail
  with a clear decode error. Owner accepted this for now.
- Opus support waits for Symphonia 0.6 (in development); a libopus C
  sidecar was rejected as too risky for this release.

## Implementation

- `long_audio.rs`: `SUPPORTED_IMPORT_EXTENSIONS` allowlist
  (wav, mp3, m4a, mp4, m4b, m4r, aac, flac, ogg, oga, mov) and shared
  `unsupported_import_error`; decode stays probe-based.
- `commands/history.rs`: import guard uses the same allowlist and message.
- `lib.rs` (`--transcribe-file`): headless CLI runs through the same
  `prepare_working_wav` pipeline; work WAV cached by source hash under
  `%TEMP%\handy-cli-transcribe`.
- `HistorySettings.tsx` dialog filter plus en/de texts; mechanical English
  fallback text update across the other locales.

## Verification

- `cargo test -j 4`: 231 lib tests plus all integration targets pass.
- New tests: unsupported-extension rejection, stereo/float WAV probe
  decode, env-gated real-recording decode.
- Real AAC sample (WhatsApp ADTS `.aac` from the owner's downloads folder)
  decodes via `HANDY_TEST_AAC=<file> cargo test --lib -- --ignored
  --exact long_audio::tests::decodes_real_recording_when_provided`.
- Clippy ratchet 18 < 22; bun lint, prettier/cargo-fmt, translation parity
  and the frontend build are green. Playwright skipped: no e2e coverage
  exists for the native file-dialog import flow.
- `cargo test` needs `-j 4` on the owner machine (os error 1455 pagefile
  pressure while other apps run) — documented workaround.

## Completion

Release via the public signed updater as 0.9.14, dispatched from this
branch. The owner validates with real phone recordings in the installed
app. The ticket flips to DONE after the updater release passes its checks
and the owner validation succeeds.