---
id: "orch-index"
title: "Orchestration Index"
type: "index"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Orchestration Index

[[START_HERE|← Start Here]] · [[planning/README|Planning]] · [[orchestration/ORCHESTRATOR_RUNBOOK|Runbook →]]

| Document | Purpose |
|---|---|
| [[orchestration/ARCHITECTURE_FREEZE|Architecture Freeze]] | what agents may decide |
| [[orchestration/IMPLEMENTATION_CONTRACTS|Implementation Contracts]] | fixed code-level boundaries |
| [[orchestration/FILE_OWNERSHIP|File Ownership]] | conflict prevention |
| [[orchestration/GIT_WORKTREE_PR_STRATEGY|Git / Worktree / PR]] | branch execution model |
| [[orchestration/ORCHESTRATOR_RUNBOOK|Orchestrator Runbook]] | phase loop |
| [[orchestration/TASK_HANDOFF_PROTOCOL|Task & Handoff]] | long-horizon context |
| [[orchestration/ESCALATION_PROTOCOL|Escalation]] | owner decision queue |
| [[orchestration/QA_EVALUATOR_PROTOCOL|QA/Evaluator]] | independent verification |
| [[orchestration/QUERY_REST_MCP_CONTRACT|Query/REST/MCP]] | connector contracts |
| [[orchestration/MASTER_EXECUTION_CHECKLIST|Master Checklist]] | project observability |

| [[orchestration/AGENT_SAFETY_POLICY|Agent Safety Policy]] | destructive Git/data and retry limits |
| [[orchestration/IMPLEMENTATION_MAP|Implementation Map]] | exact domain/path map |
| [[orchestration/DATA_STATE_MACHINES|Data State Machines]] | lifecycle invariants |
| [[orchestration/TAURI_COMMAND_CONTRACT|Tauri Commands]] | frontend/backend command contract |
| [[orchestration/SHORTCUT_CONTRACT|Shortcut Contract]] | shortcut IDs/conflicts/routing |
| [[orchestration/UI_INFORMATION_ARCHITECTURE|UI Architecture]] | screen/navigation placement |
| [[orchestration/SECURITY_THREAT_MODEL|Security Threat Model]] | privacy/auth/logging boundaries |
| [[orchestration/DEPENDENCY_LOCK_POLICY|Dependency Lock]] | phase-local dependency pinning |
| [[orchestration/TEST_METRIC|Planning Test Metric]] | starter-kit quality metric |

## Machine contracts

- `QUERY_DTO_CONTRACT.json` — canonical shared Query DTO schemas for REST/MCP.
- `REST_OPENAPI_CONTRACT.yaml` — canonical REST v1 wire contract.
- `MCP_TOOL_CONTRACT.json` — canonical MCP tool/input/output contract.
- `DEPENDENCY_LOCK.json` — exact dependency pins once Phase-5 compatibility gate runs.
