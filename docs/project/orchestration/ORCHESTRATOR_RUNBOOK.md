---
id: "orchestrator-runbook"
title: "Orchestrator Runbook"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Orchestrator Runbook

[[orchestration/GIT_WORKTREE_PR_STRATEGY|← Git Strategy]] · [[orchestration/TASK_HANDOFF_PROTOCOL|Task Protocol →]]

## Mandate

The Orchestrator schedules and tracks work.
It does not write features and does not redesign architecture.

## Per-phase loop

1. Read target phase and only relevant contracts/ADRs.
2. Build ticket dependency DAG.
3. Mark tickets:
   - READY
   - BLOCKED_DEPENDENCY
   - BLOCKED_ARCHITECTURE
   - IN_PROGRESS
   - IN_REVIEW
   - MERGED
   - FAILED_QA
   - DONE
4. Create phase branch/integration worktree.
5. Select non-conflicting ready batch (normally 4–6).
6. Generate one Task Packet per ticket.
7. Spawn one implementer per worktree.
8. Require persistent RUN_STATE updates.
9. On completion, spawn independent evaluator.
10. Integrator merges approved ticket PR into phase.
11. After all blockers merge, run full phase gate.
12. Run Red-Team review.
13. Aggregate open escalations into owner decision table.
14. If green, phase PR → `custom/main`; otherwise stop/repair.

## Decision table format

| Problem / Decision | Variant A | Variant B | Recommendation | Impact |
|---|---|---|---|---|

## Agent failure

If an implementer fails:
- keep worktree;
- preserve RUN_STATE/diff/logs;
- classify failure;
- respawn a fresh agent with original packet + failure evidence.

Do not make a new agent rediscover the project.

## Phase-end report

- ticket status;
- PRs/commits;
- QA evidence;
- performance delta;
- known accepted debt;
- unresolved owner decisions;
- `ADVANCE` or `DO NOT ADVANCE`.


## Machine control plane
Before computing a phase DAG, load and validate `TICKET_CATALOG.json` and `FEATURE_MATRIX.json`. The JSON catalog is canonical for dependencies, exact paths, QA profile, gate and attempt budget; Markdown tickets provide rationale and acceptance detail. No invalid catalog entry is dispatchable.

## Attempt budget
Track `attempt_count`. One implementation plus two evaluator-driven fix cycles are allowed by default. Third failure → E2 escalation.

## Branch rule
Every phase starts from the current green `custom/main`.


## Status rendering
After ticket state changes and before each phase report, run `scripts/render_execution_status.py`. `status/EXECUTION_STATUS.md` is generated and must not be manually edited.
