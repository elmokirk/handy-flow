---
id: "implementation-map"
title: "Implementation Map"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Implementation Map

[[orchestration/IMPLEMENTATION_CONTRACTS|← Implementation Contracts]] · [[orchestration/FILE_OWNERSHIP|File Ownership]]

Exact per-ticket paths are canonical in `TICKET_CATALOG.json`.

| Domain | Create/Own | Shared integration requests | Forbidden direct edits |
|---|---|---|---|
| Storage | `src-tauri/src/storage/**` | `lib.rs`, `Cargo.toml` | UI, connector adapters |
| STT | delegated transcription helpers/tests | `actions.rs`, `lib.rs` | migrations |
| Dictionary | manager, repository, `src/components/dictionary/**`, hook | Sidebar/App/i18n/bindings | manifest, migration registry |
| Snippets | manager, repository, `src/components/snippets/**`, hook | pipeline/App/i18n | manifest |
| Prompt | manager/repo/profile UI/tests | shortcuts/lib/App/i18n | second LLM client |
| Scratchpad | notes manager/repo, scratchpad UI/hooks | window registration/App/i18n | connector code |
| Knowledge | knowledge/export manager/repos/UI | App/i18n | QueryService internals |
| Query | `managers/query.rs`, DTOs/tests | manager registration | connector transport |
| REST | `connectors/rest.rs`, `bin/handy-rest.rs`, tests | manifest feature wiring | SQL/repositories |
| MCP | `connectors/mcp.rs`, `bin/handy-mcp.rs`, tests | manifest feature wiring | SQL/repositories |
| QA | fixtures/tests/reports | workflows via Integrator | feature redesign |

## Dispatch invariant
The Orchestrator refuses tickets with empty exact allowlists, overlapping active exclusive paths, unresolved shared-file needs, unmet dependencies or missing architecture references.
