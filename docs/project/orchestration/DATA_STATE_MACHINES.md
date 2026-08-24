---
id: "data-state-machines"
title: "Data State Machines"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Data State Machines

[[planning/04-DATA_PERSISTENCE|← Data & Persistence]] · [[orchestration/IMPLEMENTATION_CONTRACTS|Implementation Contracts]]

## Capture integrity
Allowed `integrity_state`: `pending_audio`, `audio_valid`, `audio_missing`, `audio_corrupt`, `recovered_orphan`.

## Transcription attempt
`pending → running → success | failed`. Terminal attempt payload is immutable; canonical selection is the only allowed later state change.

## Representation
`success | failed`. A failed optional processor never replaces the last successful text.

## Export job
`pending → running → succeeded | retry_wait | failed_permanent`, with `retry_wait → running`.

## Deletion
High-value content (`captures`, `notes`) uses `active → trashed → restored | purged`.

Rules: UI Delete sets `deleted_at_ms`; normal QueryService excludes Trash; restore clears it; purge requires explicit confirmation; no background job purges raw captures in MVP.

## Derived state rule
Do not persist cheaply derivable lifecycle state. Notes do not keep `current_version_id`; Captures do not duplicate transcription status when attempts can derive it. All persisted state strings use Rust enums plus DB `CHECK` constraints.
