---
id: "file-ownership"
title: "Repository File Ownership"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# File Ownership

[[orchestration/IMPLEMENTATION_CONTRACTS|← Contracts]] · [[orchestration/GIT_WORKTREE_PR_STRATEGY|Git Strategy →]]

Exact ticket allow/deny paths are canonical in `TICKET_CATALOG.json`.

## Shared/exclusive files
| Path | Owner |
|---|---|
| `src-tauri/src/lib.rs` | Integrator |
| `src-tauri/src/commands/mod.rs` | Integrator |
| `src-tauri/src/managers/mod.rs` | Integrator |
| cross-feature wiring in `actions.rs` | Integrator unless explicitly delegated |
| `src-tauri/src/storage/migrations.rs` | Storage Lead |
| `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` | Integrator |
| `src/App.tsx` | Integrator |
| `src/components/Sidebar.tsx` | Integrator |
| `src/bindings.ts` | Integrator/generated only |
| `src/i18n/**` | Integrator |
| `package.json`, `bun.lock` | Integrator |
| `.github/workflows/**` | QA Lead + Integrator |
| `docs/project/TICKET_CATALOG.json` | Orchestrator/Architecture control plane only |
| `docs/project/FEATURE_MATRIX.json` | Orchestrator/Architecture control plane only |

## Domain ownership
Storage Lead; Capture/STT Agent; Dictionary Agent; Snippet Agent; Prompt Agent; Scratchpad Agent; Knowledge Agent; Query Agent; REST Agent; MCP Agent; QA/Evaluator as defined in `TICKET_CATALOG.json`.

## Integration request
If a ticket needs a shared file outside its catalog allowlist, do not edit it. Add `INTEGRATION_REQUEST` to RUN_STATE with requested symbol/wiring. Integrator executes it in an explicit Integrator ticket/commit.

## Dependencies
Feature Agents never edit manifests. Core dependency wiring is `DEP-100`; connector dependency lock/wiring is `DEP-510`. Additional dependency needs are escalated rather than silently added.
