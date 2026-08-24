---
id: "qa-gates"
title: "QA & Quality Gates"
type: "quality"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# QA & Quality Gates

[[planning/05-DELIVERY_PLAN|← Delivery Plan]] · [[planning/07-AGENT_SWARM|Agent Swarm →]]

## Static quality

Required where applicable:

```bash
bun run lint
bun run format:check
bun run check:translations
bun run build
bun run test:playwright

cd src-tauri
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Connector feature combinations must compile/test explicitly.

## Test selection

- Unit: matching, normalization, DTO mapping, validation, pagination.
- Integration: repositories, migrations, QueryService, connector adapters, outbox.
- Playwright: deterministic webview UI behavior.
- Windows smoke: microphone, shortcut, paste, floating window, packaging.
- E2E: only cross-layer user journeys where meaningful.
- Chaos/recovery: persistence and export crash points.

## Gate BG — Bootstrap

- pinned baseline verified;
- production/dev build usable on Windows;
- current upstream tests green or deviations documented;
- repo/remotes/worktree layout correct;
- no product code changed.

## G0

- performance baseline recorded;
- CI checks green;
- test harness prepared;
- architecture freeze complete.

## G1

- migration tests green;
- `PRAGMA integrity_check = ok`;
- crash/recovery suite green;
- zero raw audio/source loss in fixtures;
- retry does not overwrite attempts;
- 100 sequential captures pass.

## G2

Minimum:
- 15 Dictionary fixtures;
- 15 Snippet fixtures;
- 10 combined fixtures;
- 10 mocked Prompt Profile cases;
- owner Wispr comparison for representative workflows.

## G3

- autosave stress;
- forced kill/restart;
- dictation;
- transform creates version;
- restore;
- search;
- no text loss.

## G4

- idempotent export;
- stable IDs;
- retries create no duplicates;
- canonical raw never mutated;
- reference/copy/none audio modes tested.

## G5

REST:
- loopback only;
- bearer auth;
- no wildcard CORS;
- read-only endpoints;
- pagination/limits;
- no SQL bypass.

MCP:
- stdio;
- read-only;
- bounded results;
- schema compatibility;
- real client/inspector smoke.

Cross-adapter:
- same fixture query returns equivalent normalized data through QueryService, REST and MCP.

## G6

- full regression;
- performance comparison;
- packaging/update identity;
- backup/upgrade/recovery;
- security/privacy review;
- license/branding check;
- 7-day daily-driver UAT.

## Clean-code gate

Reject:
- duplicate SQL outside repositories;
- connector business logic;
- duplicate LLM client;
- production `unwrap()` without justification;
- broad TypeScript `any`;
- manual generated-binding edits;
- hidden global mutable state;
- unbounded query APIs;
- feature agent edits outside ownership.

## Severity

| Severity | Meaning | Gate |
|---|---|---|
| S0 | data/security catastrophic | always block |
| S1 | core flow corruption/unusable | block |
| S2 | important defect, workaround exists | block release unless explicitly accepted |
| S3 | polish/minor | may defer |


## Hardening gates added in v1.1

### Bootstrap control-plane gate
- `custom/main` exists at the pinned baseline.
- starter kit is imported under `repo/docs/project/` and committed on a bootstrap branch before merge.
- `TICKET_CATALOG.json` and `FEATURE_MATRIX.json` validate.
- ticket dependency DAG is acyclic and every ticket has non-empty exact `allowed_paths`.
- Agent Safety Policy is included in Task Packets.

### Migration backup gate
- online backup succeeds;
- backup `PRAGMA integrity_check = ok`;
- migration is refused if verified backup cannot be created.

### Performance budgets
For unprocessed core dictation, median end-to-paste latency regression must be ≤ **10%** versus G0 baseline on the same machine/model/fixture. Also report idle RAM, model-loaded RAM, VRAM and startup latency. LLM Transform latency is tracked separately.

### Windows native QA execution
GitHub-hosted CI is not accepted as proof for interactive microphone/hotkey/clipboard/focus/window behavior. Native gates must record evidence from an interactive Windows environment (owner machine or dedicated interactive runner).
