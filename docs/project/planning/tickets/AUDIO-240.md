---
id: "ticket-audio-240"
title: "AUDIO-240 — Long transcription without lost captures"
type: "ticket"
status: "in_progress"
updated: "2026-09-24"
project: "custom-handy"
ticket_id: "AUDIO-240"
phase: "6"
owner: "Integrator"
gate: "G6"
catalog_ref: "TICKET_CATALOG.json#AUDIO-240"
---

# AUDIO-240 — Long transcription without lost captures

**Source plan:** [Long audio remediation](../LONG_AUDIO_REMEDIATION.md)
**Decision:** [ADR-025](../adr/ADR-025.md)

## Outcome

A real 15-minute recording succeeds on the owner's Windows device as one
original WAV and one visible capture. Longer recordings remain safe and
retryable, bounded work survives failure/restart, new short dictations can
overtake long jobs, and local WAV/MP3 files can be imported into the same
history and processing path.

## Allowed paths

`src-tauri/src/**`, `src-tauri/tests/**`, `src/**`, relevant manifests and
release configuration, `docs/project/**`, and the mirrored root control-plane
files. No private recordings or Wispr source files may be written.

## Work and gates

1. Update the plan and owner decision to include local WAV/MP3 import and the
   15-minute release gate. Pin baseline `297ee323e53575a76348f2c72baf99f2e49f1066`.
2. Add a failing capture/WAV/history regression at the shared live seam.
3. Implement direct final WAV with 30-second sync and durable capture-first
   lifecycle; prove crash/empty-audio recovery.
4. Implement bounded child-worker inference, checkpoints, joins, and fair
   priority queue; prove 15-minute real dictation and short-job overtaking.
5. Import WAV/MP3 by picker and drag-and-drop, using the same queue. Expose
   statuses, progress, uncertain-seam underlines, retries, source labels and
   original audio in canonical history; prove app end-to-end behavior.
6. Run private local 9:40 benchmark without modifying/uploading the file;
   run 30/60-minute safe-failure tests, independent spec/simplicity reviews,
   platform/build/update gates and bump the release version.

## Negative cases

Native abort, OOM, insufficient disk, unsupported/corrupt WAV, DB failure,
app restart, worker restart, ambiguous overlap, model switch, and a failed
retry following a successful earlier attempt must never silently remove
the original audio or the capture card.

## Completion

Document exact checks, exit codes, benchmark limits and remaining risks in
`status/runs/AUDIO-240.md`. Update `TICKET_CATALOG.json` before starting
another ticket, render execution status, and provide the project-mandated
three-sentence TL;DR. No release is complete without an updater test.
