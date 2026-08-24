---
id: "shortcut-contract"
title: "Shortcut Contract"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Shortcut Contract

[[planning/01-PRODUCT_VISION|← Product Vision]] · [[orchestration/TAURI_COMMAND_CONTRACT|Tauri Commands]]

## Stable IDs
- `transcribe` — existing Handy behavior.
- `transcribe_with_post_process` — legacy compatibility where retained.
- `custom_handy.transform_dictation` — active Transform Profile.
- `custom_handy.show_scratchpad` — show/focus Scratchpad.
- `custom_handy.dictate_to_scratchpad` — explicit Scratchpad dictation routing.

## Defaults
Preserve existing Handy defaults for existing IDs. New IDs have no collision-prone hard-coded global chord; first-run UI configures them and checks conflicts.

## Conflict behavior
A chord cannot map to two active actions. Failed registration leaves the prior valid binding unchanged and reports the conflict.

## Routing
Dictation destination is explicit: `FocusedApp | Scratchpad`. Automatic per-app routing remains P2.
