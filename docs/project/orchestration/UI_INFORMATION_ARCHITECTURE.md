---
id: "ui-ia"
title: "UI Information Architecture"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# UI Information Architecture

[[planning/01-PRODUCT_VISION|← Product Vision]] · [[orchestration/TAURI_COMMAND_CONTRACT|Tauri Command Contract]]

## Primary navigation
1. **History** — search, Raw/Derived versions and Trash.
2. **Dictionary** — durable vocabulary/aliases.
3. **Snippets** — spoken triggers/replacements.
4. **Profiles** — Styles and Transforms in one screen with kind filter.
5. **Scratchpad** — primarily floating; optional Notes history entry.
6. **Settings** — provider/model/local integrations/export configuration.

## History detail
Show audio/playback, Knowledge Raw (`normalized_stt`), optional advanced `engine_raw`, attempt history, representations, and Trash/Restore/Purge with explicit destructive confirmation.

## Connector settings
A single **Local Integrations** section exposes REST status/port/token rotation/client config and MCP launch config. These are local integrations, not remote/cloud sharing.

## Scratchpad
Floating Markdown/Plain Text editor with autosave, dictate, transform and version history; no Rich Text toolbar in MVP.
