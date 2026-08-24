---
id: "handy-repo-map"
title: "Handy Repository Map"
type: "research"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Handy Repository Map

[[research/HANDY_BASELINE|← Baseline]] · [[orchestration/FILE_OWNERSHIP|File Ownership]]

## Existing backend hotspots

- `src-tauri/src/lib.rs` — Tauri setup, command/event registration, generated binding export.
- `src-tauri/src/actions.rs` — recording/transcription/post-process lifecycle.
- `src-tauri/src/managers/audio.rs` — audio manager.
- `src-tauri/src/managers/history.rs` — SQLite history + recording retention.
- `src-tauri/src/managers/transcription.rs` — engine loading/streaming/transcription + current text normalization pipeline.
- `src-tauri/src/llm_client.rs` — existing LLM provider transport.
- `src-tauri/src/settings.rs` — app settings/store structures.
- `src-tauri/src/commands/history.rs` — history Tauri commands.
- `src-tauri/src/clipboard.rs`, `paste_tx/windows.rs` — native paste/clipboard behavior.

## Existing frontend hotspots

- `src/App.tsx`
- `src/components/Sidebar.tsx`
- `src/components/settings/post-processing/**`
- `src/components/settings/PostProcessingSettingsPrompts.tsx`
- `src/stores/settingsStore.ts`
- `src/bindings.ts`
- `src/i18n/**`

## Existing QA

- Rust `cargo test`;
- ESLint;
- Prettier;
- translation checks;
- Playwright;
- build workflows.

## Agent implication

These existing central files have high merge-conflict potential.
Feature agents should mostly add isolated modules; the Integrator handles registration and global wiring.

## Baseline refresh note (2026-08-24, v0.9.6)

At baseline `af48dd68` the tray icon handling was refactored upstream (`5ec2276 single writer tray icon`), making `src-tauri/src/tray.rs` an additional high-conflict shared file alongside `actions.rs` and `lib.rs`. No data/persistence or contract-relevant behavior changed versus the originally captured map.
