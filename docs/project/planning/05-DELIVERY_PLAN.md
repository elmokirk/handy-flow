---
id: "delivery-plan"
title: "Delivery Plan"
type: "planning"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Delivery Plan

[[planning/04-DATA_PERSISTENCE|← Data & Persistence]] · [[planning/06-QA_GATES|QA Gates →]]

## Two-stage execution

### Stage 1 — Workspace Bootstrap
Prepare source/fork/remotes/build/test/worktree environment.
**No feature implementation.**

### Stage 2 — Product implementation
Run phases sequentially; parallelize safe tickets within a phase.

## Phase summary

| Phase | Outcome | Gate |
|---|---|---|
| Bootstrap | reproducible fork/workspace | BG |
| 0 | baseline + harness + CI | G0 |
| 1 | durable data/provenance/recovery | G1 |
| 2 | Dictionary/Snippets/Profiles/Search | G2 |
| 3 | Scratchpad | G3 |
| 4 | Markdown Knowledge export | G4 |
| 5 | QueryService + REST + MCP | G5 |
| 6 | release hardening + UAT | G6 |

See [[planning/phases/INDEX|Phase Index]] and [[planning/tickets/INDEX|Ticket Index]].

## Parallelization

Recommended active coding batch:
- 4–6 implementers;
- 1 Orchestrator;
- 1 Integrator;
- 1 QA/Evaluator;
- optional Red-Team reviewer.

Do not parallelize unstable contracts.

## Release scope

Public MVP includes:
- Core through Phase 4;
- Phase 5 REST + MCP;
- Phase 6 release hardening.

No MVP tag before G6.
