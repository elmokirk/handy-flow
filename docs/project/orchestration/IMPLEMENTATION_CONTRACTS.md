---
id: "implementation-contracts"
title: "Implementation Contracts"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Implementation Contracts

[[orchestration/ARCHITECTURE_FREEZE|← Architecture Freeze]] · [[orchestration/IMPLEMENTATION_MAP|Implementation Map →]]

## Backend target
Evolve Handy's Manager/Command pattern; do not replace Tauri/Rust/React foundations.

```text
src-tauri/src/
├── actions.rs
├── llm_client.rs
├── settings.rs
├── managers/
│   ├── history.rs
│   ├── transcription.rs
│   ├── dictionary.rs
│   ├── snippets.rs
│   ├── prompt_profiles.rs
│   ├── notes.rs
│   ├── knowledge.rs
│   └── query.rs
├── storage/
│   ├── mod.rs
│   ├── database.rs
│   ├── migrations.rs
│   ├── ids.rs
│   ├── models.rs
│   └── repositories/**
├── connectors/
│   ├── mod.rs
│   ├── rest.rs
│   └── mcp.rs
└── bin/
    ├── handy-rest.rs
    └── handy-mcp.rs
```

Shared registration files remain Integrator-owned.

## Canonical deterministic text pipeline

```text
engine output
→ engine_raw
→ Dictionary/custom-word correction
→ filler removal + deterministic normalization
→ normalized_stt   # Knowledge Raw
→ Snippets         # derived, single pass/non-recursive
→ Prompt Profile   # derived, snapshot provenance
→ delivery
```

No stage may overwrite `engine_raw` or an earlier successful representation.

## Repository boundary
Only `storage/repositories/**` owns SQL. Managers/services call repositories; connectors call QueryService; UI calls typed Tauri commands. `app_meta` stores `schema_version` and `query_contract_version`.

## QueryService
Stable conceptual read interface includes transcript search/detail/versions, note search/detail, and list operations for Dictionary/Snippets/Prompt Profiles. Exact Rust ownership/async signatures may be finalized once by QUERY-501, then freeze.

## Pagination
Default 50, max 200, deterministic `(created_at DESC, id DESC)`, opaque versioned cursor, no unbounded corpus list.

## IDs
UUIDv7 strings via one centrally wired dependency (`DEP-100`). No competing ID scheme.

## Errors
Domain taxonomy: `NotFound`, `InvalidInput`, `Conflict`, `StorageUnavailable`, `CorruptData`, `ExternalUnavailable`, `Unauthorized`, `Internal`. Transport adapters map errors without leaking stack traces or SQL details.

## State
Persisted state follows [[orchestration/DATA_STATE_MACHINES|Data State Machines]]. Avoid duplicated mutable state: note current version and capture transcription status are derived.

## Deletion
Captures/Notes use Trash/Restore/explicit Purge. Normal QueryService excludes soft-deleted records. REST/MCP expose no destructive surface.

## Prompt provenance
Representation stores effective prompt snapshot, provider/model IDs and processor version. Profile deletion/edit never destroys historical interpretation.

## Blocking I/O
SQLite remains synchronous/repository-owned. Async REST/MCP handlers cross a bounded blocking boundary rather than blocking Tokio reactor threads.

## Data directory / companions
Main GUI resolves the actual app data directory (including portable mode) and generates explicit connector launch configuration containing absolute `--data-dir`. Companion binaries do not guess or migrate databases.

## Settings vs content
JSON settings store: UI preferences, provider config, shortcuts, connector config/secrets as appropriate. SQLite: durable content/domain records. Never maintain two canonical copies.

## LLM
Reuse Handy `llm_client.rs`; no second LLM HTTP client.

## Machine contracts
- Tauri: [[orchestration/TAURI_COMMAND_CONTRACT|Tauri Command Contract]]
- Shortcuts: [[orchestration/SHORTCUT_CONTRACT|Shortcut Contract]]
- REST: `REST_OPENAPI_CONTRACT.yaml`
- MCP: `MCP_TOOL_CONTRACT.json`
- tickets: `TICKET_CATALOG.json`
- acceptance coverage: `FEATURE_MATRIX.json`
