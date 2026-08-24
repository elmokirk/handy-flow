---
id: "tauri-command-contract"
title: "Tauri Command Contract"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Tauri Command Contract

[[orchestration/IMPLEMENTATION_CONTRACTS|← Implementation Contracts]] · [[orchestration/UI_INFORMATION_ARCHITECTURE|UI Architecture]]

Command names are frozen before feature agents implement UI; Integrator owns registration and generated bindings.

## Dictionary
`list_dictionary_entries`, `create_dictionary_entry`, `update_dictionary_entry`, `set_dictionary_entry_enabled`, `delete_dictionary_entry`, `export_dictionary`, `import_dictionary`.

## Snippets
`list_snippets`, `create_snippet`, `update_snippet`, `set_snippet_enabled`, `delete_snippet`, `export_snippets`, `import_snippets`.

## Prompt Profiles
`list_prompt_profiles`, `create_prompt_profile`, `update_prompt_profile`, `set_prompt_profile_enabled`, `delete_prompt_profile`, `set_active_style_profile`, `run_dictation_transform`.

## Notes / Scratchpad
`list_notes`, `create_note`, `get_note`, `update_note_content`, `trash_note`, `restore_note`, `list_note_versions`, `restore_note_version`, `search_notes`, `show_scratchpad`, `hide_scratchpad`.

## History
`search_transcriptions`, `get_transcription_detail`, `list_transcript_versions`, `trash_capture`, `restore_capture`, `purge_capture`, `get_storage_usage`.

## Knowledge
`list_export_targets`, `upsert_export_target`, `test_export_target`, `retry_export_job`, `export_capture_now`.

## Connectors
`get_connector_status`, `ensure_rest_token`, `rotate_rest_token`, `get_rest_launch_config`, `get_mcp_launch_config`.

## Rules
DTOs are stable typed structs exported through Specta; feature handlers live in owned modules; Integrator registers them; commands return domain DTOs/errors rather than SQL records; destructive commands require explicit UI confirmation; REST/MCP map no destructive command in MVP.
