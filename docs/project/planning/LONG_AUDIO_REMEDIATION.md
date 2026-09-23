---
id: "long-audio-remediation"
title: "Long audio remediation — durable WAV and visible transcription jobs"
type: "implementation-plan"
status: "accepted"
updated: "2026-09-23"
project: "custom-handy"
ticket: "AUDIO-240"
decision: "ADR-025"
---

# Long audio remediation

## Incident and why this customization exists

On 2026-09-23, a 580.38-second, 18,572,204-byte WAV was saved in the
Handy Flow recordings directory but had zero rows in `captures`. Inference of
the full 9,286,080-sample recording requested a roughly 10 GiB Vulkan buffer
on an approximately 8 GiB GPU and the native process crashed. The live path
saved WAV and ran inference concurrently, then inserted the capture only after
inference. Thus a native abort bypassed both the success and failure inserts;
the user saw no history entry despite the audio existing on disk. A second
14:55 orphan WAV exists, but its older crash has no conclusive model-memory
trace. The private recordings and transcript contents must never be committed
or uploaded as fixtures.

## Owner decisions

- Exactly one original WAV per dictation, written directly into the usual
  recordings directory during capture; no temporary original or on-disk
  audio chunks. Keep source audio by default.
- Flush the same WAV and request a disk sync every 30 seconds, and on normal
  stop. This updates its header; it does not create another recording.
- A capture exists from recording start. Every capture appears in canonical
  history, including pending, failed, interrupted, recovered, and corrupt
  cases. Preserve original capture time; label recovered estimates.
- Model inference runs in an isolated worker with bounded, overlapping
  in-memory windows. Persist chunk progress for restart. CPU fallback is a
  user choice, never automatic.
- Short new dictations outrank remaining chunks of a long job at the next
  chunk boundary. A late long result appears in history with notification and
  copy, never auto-pastes into a different focused app.
- Completed intermediate chunk payloads are removed after the merged raw and
  normalized transcript and compact seam diagnostics are committed. Failed
  or interrupted attempts retain progress for retry. Pending and failed
  captures are exempt from automatic retention.
- No upstream merge, general backend update, drag-and-drop import, MCP, or
  mobile release is included in AUDIO-240.

## Implementation contract

1. On recording start insert a `pending_audio` capture with the original UTC
   millisecond start time and unique final WAV name. The recorder consumer,
   not the real-time microphone callback, writes 16 kHz mono PCM to that one
   file. Checkpoint its header every 30 seconds. On stop, finalize, validate,
   hash, and attach duration/size; insert a pending attempt before inference.
2. Reconcile DB rows against final WAVs at startup and history open. Adopt
   orphans exactly once without modifying their bytes. Mark empty/corrupt
   files visibly and distinguish inferred timestamps from exact start times.
   Startup recovery must not loop forever on a repeatedly crashing chunk.
3. Route all native model inference through one headless child worker per
   device, using the existing executable/headless path rather than another
   model runtime. It reads bounded sample windows from the original WAV and
   reports structured results; the parent alone mutates SQLite. Admit windows
   against architecture/model limits and available device/system memory with
   a reserve. The 80% target is not a hard allocation guarantee. Shrink a
   failing window at most twice, then surface an actionable failure.
4. Persist chunk boundaries, outputs, and attempt progress. Prefer silence
   cuts and start with about two seconds of overlap, increasing to five when
   needed. Use word timestamps where reliable; otherwise perform Unicode-safe
   suffix/prefix matching requiring an unambiguous chain of at least three
   words. Never splice within a word or silently discard unmatched text.
   Store uncertain seams and both short raw alternatives so the UI can
   underline and inspect them. Normalize/post-process only after one merged
   engine-raw transcript exists.
5. Separate recording and inference states. Queue short dictations before
   remaining long windows once the current window finishes; after three
   short jobs, give a waiting long job one window. Keep one heavy inference
   call per device. Late results remain in history, while ordinary short
   dictations retain the existing immediate flow.

## Verification and release gate

- Red-to-green regression: a finalized WAV with no capture is adopted and
  shown once; before the fix the real 9:40 WAV exists with zero capture rows.
- Unit/integration: 30-second flush, crash between flushes, every DB/file
  kill point, corrupt/empty WAV, idempotent recovery, retry, retention,
  Unicode seam matching, repeated German words, and uncertain joins.
- End-to-end: capture card at start; processing badges and n/total progress;
  child crash leaves app open; restart resumes committed work; short dictation
  overtakes the remainder of a 20-minute job; original audio remains playable.
- Local-only benchmark: hash the private 9:40 file before/after three runs on
  the affected Parakeet Q8/Vulkan setup. Measure wall time, peak RAM/VRAM,
  flush latency, overrun count, priority latency, worker exits, and quality
  at seams. Also exercise 30- and 60-minute reproducible fixtures on CPU and
  available accelerators. Do not invent a WER score without ground truth.
- Verified SQLite backup before schema migration; Rust/frontend tests,
  formatting/lint, Tauri production build, version bump above published
  0.9.10, and a real in-app updater test from the installed version.

## Scope and traceability

Implementation ticket: [AUDIO-240](tickets/AUDIO-240.md). Decision:
[ADR-025](adr/ADR-025.md). Historical source contract:
`planning/04-DATA_PERSISTENCE.md` and `AUDIO-104` (superseded for live
dictation only). Future external-audio drag and drop is recorded in the
roadmap, not in this ticket. Agent completion requires catalog status,
RUN_STATE, status rendering, and a three-sentence user TL;DR.
