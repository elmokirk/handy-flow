---
id: "task-handoff"
title: "Task & Long-Horizon Handoff Protocol"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Task & Long-Horizon Handoff Protocol

[[orchestration/ORCHESTRATOR_RUNBOOK|← Orchestrator]] · [[orchestration/ESCALATION_PROTOCOL|Escalation →]]

## Minimal Task Packet
Every sub-agent receives only: ticket goal; branch/worktree; exact allow/deny paths from `TICKET_CATALOG.json`; frozen contracts; merged dependencies; relevant upstream files; acceptance criteria; QA profile; escalation triggers; attempt number; expected completion format.

Avoid sending the whole planning corpus.

## Startup checklist
- verify branch/worktree;
- verify current phase base derives from green `custom/main`;
- `git status` understood;
- read upstream `AGENTS.md`;
- read Task Packet + Agent Safety Policy;
- read only linked contracts/ADRs;
- inspect only relevant repo files;
- run smallest relevant baseline test;
- begin TDD.

## Persistent handoff
Each ticket gets `docs/project/status/runs/<TICKET>.md` copied from [[orchestration/templates/RUN_STATE|RUN_STATE Template]]. Its YAML frontmatter is machine-readable and records ticket/status/attempt/branch/worktree/last commit. The body is the human checklist/evidence.

Before context/session ends: update RUN_STATE, exact last passing/failing command, current blocker and next smallest action; commit safe work or explicitly record uncommitted state. No architectural assumption may exist only in chat.

## Completion package
`STATUS`, `TICKET`, `ATTEMPT`, `BRANCH`, `COMMIT`, `FILES_CHANGED`, `TESTS_ADDED`, `TESTS_PASSED`, `ACCEPTANCE`, `INTEGRATION_REQUESTS`, `DEVIATIONS`, `ESCALATIONS`.


## Status rendering
After ticket state changes and before each phase report, run `scripts/render_execution_status.py`. `status/EXECUTION_STATUS.md` is generated and must not be manually edited.
