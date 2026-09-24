---
id: "long-audio-remediation"
title: "Long audio remediation — durable WAV and visible transcription jobs"
type: "implementation-plan"
status: "accepted"
updated: "2026-09-24"
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
  normalized transcript and compact seam diagnostics are committed. An
  interrupted pending attempt resumes its saved chunks after restart. An
  explicitly retried failed attempt starts afresh, because language, model
  prompts or translation settings may have changed; its old chunks remain
  available for diagnosis. Pending and failed captures are exempt from
  automatic retention.
- The release gate is one successful real 15-minute dictation on the owner's
  Windows model/device. Test 30- and 60-minute files as best-effort cases;
  failure must retain the original, history card and actionable error.
- Local WAV/MP3 picker and drag-and-drop import are in AUDIO-240. Preserve
  source bytes, label the capture `Manually imported`, and use import time
  unless a trustworthy recording timestamp is available. No mobile release,
  upstream merge, MCP, or general backend update is included.
- Owner confirmation on 2026-09-24: temporarily suspend in-process live text
  preview for streaming-capable models. Recording remains live and durable;
  transcription after stop runs only in the isolated worker. Restore live
  preview only through a separately reviewed worker protocol later.
- Owner hotfix correction on 2026-09-24, reviewed against installed commit
  `a16698b`: successful ordinary dictations must deliver one final text to
  the focused field. The 0.9.11 implementation accidentally capped delivery
  at 60 seconds of audio. Keep a 30-second post-stop grace period and refuse
  delivery after a newer recording begins or the foreground window changes
  on Windows; do not let a late job paste into another task. The app cannot
  identify a different field within the same window, so this is not a full
  focus-identity guarantee. Use one worker inference step for audio up to six minutes
  when the model reports a compatible limit, falling back to bounded chunks
  if the isolated worker fails. Six minutes is a UX/optimistic one-shot
  ceiling, not a maximum recording duration or guaranteed memory allocation.
- The history card shows a calm spinner while processing and reveals only the
  final text, not intermediate chunk text. Progress events refresh cards
  without replacing the entire history view. Chunk diagnostics remain visible
  for genuinely long work, and uncertain joins still need human review.

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
3. Route all supported native model inference through one persistent headless
   child worker per device, extending the existing executable rather than
   adding another runtime. The parent alone mutates SQLite. Start with no
   more than 30 seconds or the smaller model limit for long jobs. Ordinary
   recordings up to six minutes may first try one step in the child when the
   reported model limit permits it; Moonshine's known 64-second engine limit
   is respected, while ONNX engines without a reported bound rely on the
   isolated-worker fallback. A failed attempt reverts to bounded
   windows and shrinks at most twice. Neither limit caps recording duration.
   Device memory readings are diagnostic hints, not a hard 80% allocation
   guarantee.
4. Persist chunk boundaries, outputs, and attempt progress. Prefer silence
   cuts and start with about two seconds of overlap, increasing to five when
   needed. Use word timestamps where reliable; otherwise perform Unicode-safe
   suffix/prefix matching requiring an unambiguous chain of at least three
   words. Never splice within a word or silently discard unmatched text.
   Store uncertain seams and both short raw alternatives so the UI can
   underline and inspect them. Normalize/post-process only after one merged
   engine-raw transcript exists.
   The owner may inspect both seam fragments and confirm or correct the full
   transcript. Each confirmation appends a provenance-linked `manual_edit`
   representation; history, copy actions and knowledge export prefer the
   latest confirmed version while canonical raw text and audio stay intact.
5. Separate recording and inference states. Queue short dictations before
   remaining long windows once the current window finishes; after three
   short jobs, give a waiting long job one window. Keep one heavy inference
   call per device. Late results remain in history, while ordinary short
   dictations retain the existing immediate flow.
6. A picker and drag-and-drop call the same import command. Validate WAV/MP3
   by decoding, not extension alone; copy each source unchanged into the
   recordings directory and retain its original filename in the capture
   title. Reuse the existing MP3-capable decoder. A derived work format is
   permitted only if bounded decoding requires it; it is not an original or
   per-chunk audio file and is removed after terminal processing. Recording,
   imported-file and retry attempts enter the same queue.

## Verification and release gate

- Red-to-green regression: a finalized WAV with no capture is adopted and
  shown once; before the fix the real 9:40 WAV exists with zero capture rows.
- Unit/integration: 30-second flush, crash between flushes, every DB/file
  kill point, corrupt/empty WAV, idempotent recovery, retry, retention,
  Unicode seam matching, repeated German words, and uncertain joins.
- End-to-end: capture card at start; processing badges, chunk number and
  percentage progress (no unstable predicted total when windows shrink);
  child crash leaves app open; restart resumes committed work; short dictation
  overtakes the remainder of a 20-minute job; original audio remains playable.
- Local-only benchmark: hash the private 9:40 file before/after three runs on
  the affected Parakeet Q8/Vulkan setup. Prove one real 15-minute dictation
  succeeds on the owner's Windows device. Measure wall time, peak RAM/VRAM,
  flush latency, overrun count, priority latency, worker exits, and seam
  quality. Exercise 30- and 60-minute WAV/MP3 fixtures; success is best effort
  but failure must be visible and preserve source audio. Do not invent a WER
  score without ground truth.
- Review the pinned implementation diff on two separate axes: spec/data
  safety and repository standards/simplicity. Block release for unbounded
  audio loads, in-process native inference, missing capture/error paths,
  unnecessary one-use abstractions, duplicated shared behavior, or unreadable
  error flow. Re-review fixes; line count never trumps clarity or safety.
- Verified SQLite backup before schema migration; Rust/frontend tests,
  formatting/lint, Tauri production build, version bump above the 0.9.10
  source baseline, and a real in-app updater test from the installed version.
- No signed public test release may bypass the real 15-minute microphone and
  human transcript-quality gates. The updater only offers a newer signed
  public release; CI success alone does not make this candidate installable
  through the update button.

## Scope and traceability

Implementation ticket: [AUDIO-240](tickets/AUDIO-240.md). Decision:
[ADR-025](adr/ADR-025.md). Historical source contract:
`planning/04-DATA_PERSISTENCE.md` and `AUDIO-104` (superseded for live
dictation only). Local audio drag and drop was pulled forward from the
roadmap by the owner on 2026-09-24. Baseline before implementation:
`297ee323e53575a76348f2c72baf99f2e49f1066`. Agent completion requires catalog status,
RUN_STATE, status rendering, and a three-sentence user TL;DR.
