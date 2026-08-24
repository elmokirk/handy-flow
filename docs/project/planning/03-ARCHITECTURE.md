---
id: "architecture"
title: "Target Architecture"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Target Architecture

[[planning/02-RESEARCH|← Research]] · [[planning/04-DATA_PERSISTENCE|Data & Persistence →]]

## System

```text
React / TypeScript UI
History · Dictionary · Snippets · Profiles · Scratchpad · Knowledge Settings
             │ typed Tauri commands/events
             ▼
Rust Application / Domain Layer
Capture · Transcription · Processing · Notes · Export · QueryService
        │                         │
        ▼                         ▼
Existing Handy STT Core       Persistence
audio/VAD/models             SQLite + WAV
                                  │
                    ┌─────────────┼──────────────┐
                    ▼             ▼              ▼
               Markdown       REST v1          MCP
               Export         loopback         stdio
```

## Domains

### Capture
Owns audio lifecycle and artifact integrity.
Does not know about Styles, MCP, REST or UI.

### Transcription
Owns attempts, model metadata, `engine_raw`, deterministic normalization and canonical attempt selection.

### Representation
Owns derived text versions:
- dictionary;
- snippet;
- style;
- transform;
- manual edit.

### Dictionary
Extends existing Handy correction logic with durable metadata/aliases/UI.

### Snippets
Deterministic phrase-boundary trigger → replacement.

### Prompt Profiles
One shared domain:
`kind = style | transform`.
Uses existing LLM provider/client transport.

### Notes
One domain with versions.
Scratchpad is a floating UI over Notes.

### Export
Markdown/Second-Brain export with durable outbox and idempotency.

### QueryService
Stable read contract shared by:
- app UI where practical;
- REST;
- MCP.

REST/MCP do not query SQLite directly.

## Processing pipeline

```text
audio
→ STT engine
→ engine_raw                immutable
→ Dictionary/custom-word correction
→ filler removal + deterministic normalize
→ normalized_stt            immutable; Knowledge "raw"
→ Snippets (optional, single-pass non-recursive)
→ Prompt Profile (optional)
→ Representation
→ paste / Scratchpad / export
```

Every optional processor is fail-open.

## Platform boundary

Platform-neutral:
- domain logic;
- storage;
- matching;
- QueryService;
- REST;
- MCP.

Windows-specific adapters:
- global shortcut behavior;
- active application/window metadata;
- clipboard/paste;
- installer/signing;
- native smoke tests.

## Stable integration rule

External interfaces depend on stable DTOs/services only.

Forbidden:
- `REST → raw SQL`;
- `MCP → raw SQL`;
- `REST → MCP`;
- `MCP → REST`;
- feature UI → direct SQLite.

Detailed implementation contracts live in [[orchestration/IMPLEMENTATION_CONTRACTS|Implementation Contracts]].
