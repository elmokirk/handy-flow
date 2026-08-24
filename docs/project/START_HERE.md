---
id: "start-here"
title: "START HERE — Custom Handy Starter Kit"
type: "entrypoint"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# START HERE

> [!IMPORTANT]
> This starter kit is designed for a coding agent that has **zero context from the planning chat**.
> The agent must complete **Step 1 — Bootstrap** and stop before product implementation.
> Product implementation starts only after the Bootstrap Gate is explicitly green.

## What this kit is

This folder is the complete source of truth for creating a Windows-first, local-first fork of Handy that adds:

- durable audio/transcript provenance;
- Dictionary;
- deterministic Snippets;
- Styles + Dictation Transforms;
- local OpenAI-compatible LLM processing;
- searchable History;
- Markdown/Plain-Text Scratchpad;
- Markdown/Second-Brain export;
- read-only localhost REST API;
- read-only local MCP server;
- agent-ready orchestration, worktrees, QA gates and escalation logging.

The core principle is:

> **Agents write code. Agents do not invent architecture.**

## Frozen Handy baseline

| Item | Value |
|---|---|
| Upstream | `cjpais/Handy` |
| Planning date | `2026-08-21` |
| Baseline branch | `main` |
| Baseline commit | `af48dd68a64d58aad128fdbb920492a03da53c79` |
| Source version at baseline | `0.9.5` |
| Latest tagged release observed | `v0.9.5` |
| Reference platform | Windows |
| Frontend | React + TypeScript |
| Backend | Rust + Tauri 2 |
| Package manager | Bun |

The exact baseline rationale and freshness policy live in [[research/HANDY_BASELINE|Handy Baseline]].

## Folder model

Unzip this kit into a clean workspace, for example:

```text
D:\dev\custom-handy-workspace\
├── custom-handy-starter-kit\         ← this folder
├── repo\                             ← created by Bootstrap Agent
└── worktrees\                        ← created after bootstrap
```

Do **not** manually install the normal Handy application and try to customize the installed binary.
The project is built from source.

---

# Step 1 — Bootstrap only

Give your coding agent this instruction:

> **Read `START_HERE.md` and execute only Step 1 / Bootstrap using [[orchestration/prompts/BOOTSTRAP_AGENT|Bootstrap Agent Prompt]]. Do not implement product features. Stop after the Bootstrap Gate and report the result.**

The Bootstrap Agent must:

1. read [[00-PROJECT_CONTEXT|Project Context]];
2. read [[research/HANDY_BASELINE|Handy Baseline]];
3. verify Git/Bun/Rust/Python 3/Windows build prerequisites;
4. create or connect the GitHub fork;
5. clone the fork into sibling `repo/`;
6. configure `origin` and `upstream`;
7. fetch and verify the pinned baseline commit;
8. create `custom/main` at the pinned baseline, then work only on `bootstrap/control-plane`; never implement directly on fork `main` or `custom/main`;
9. run the baseline build, tests and native smoke checks;
10. record actual results in `status/BOOTSTRAP_STATUS.md`;
11. create the workspace/worktree layout;
12. prepare GitHub branch/PR rules where permissions allow;
13. run [[planning/tickets/BOOT-005|BOOT-005 Source Map Verification]];
14. produce [[orchestration/templates/PHASE_GATE_REPORT|Phase Gate Report]] for `BOOTSTRAP`;
15. **STOP**.

If the current upstream is newer than the pinned baseline, the agent must **not silently update the plan**.
It must follow the baseline freshness policy in [[research/HANDY_BASELINE|Handy Baseline]].

---

# Step 2 — Autonomous implementation

Only after Bootstrap Gate = `PASS`, give the Orchestrator:

> **Read [[orchestration/prompts/ORCHESTRATOR_AGENT|Orchestrator Agent Prompt]] and execute the accepted plan phase-by-phase. Spawn bounded sub-agents using ticket worktrees. Stop at every red gate or blocking escalation.**

Execution order:

```text
Bootstrap Gate
  ↓
Phase 0 — Baseline & Harness
  ↓ G0
Phase 1 — Durable Data Core
  ↓ G1
Phase 2 — Dictionary / Snippets / Profiles / Search
  ↓ G2
Phase 3 — Scratchpad
  ↓ G3
Phase 4 — Knowledge Export
  ↓ G4
Phase 5 — QueryService + REST + MCP
  ↓ G5
Phase 6 — Release Hardening
  ↓ G6
MVP Release
```

The master checklist is [[orchestration/MASTER_EXECUTION_CHECKLIST|Master Execution Checklist]].

---

# Required reading order for a fresh Orchestrator

1. [[00-PROJECT_CONTEXT|Project Context]]
2. [[planning/01-PRODUCT_VISION|Product Vision]]
3. [[research/HANDY_BASELINE|Handy Baseline]]
4. [[planning/03-ARCHITECTURE|Architecture]]
5. [[planning/04-DATA_PERSISTENCE|Data & Persistence]]
6. [[planning/05-DELIVERY_PLAN|Delivery Plan]]
7. [[planning/06-QA_GATES|QA Gates]]
8. [[orchestration/ARCHITECTURE_FREEZE|Architecture Freeze]]
9. [[orchestration/IMPLEMENTATION_CONTRACTS|Implementation Contracts]]
10. [[orchestration/FILE_OWNERSHIP|File Ownership]]
11. [[orchestration/GIT_WORKTREE_PR_STRATEGY|Git / Worktree / PR Strategy]]
12. [[orchestration/ORCHESTRATOR_RUNBOOK|Orchestrator Runbook]]
13. target phase + target tickets only.

## If an agent has a question

Agents do not make hidden architectural decisions.

They must use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]] and log:

- the problem;
- evidence;
- two viable variants;
- recommendation;
- impacted contracts;
- whether independent work can continue.

At phase end, unresolved decisions are presented to the owner in one table.

## Current owner decisions

The following are already accepted and must not be re-litigated by sub-agents:

- Knowledge `raw` = `normalized_stt`; `engine_raw` is also preserved.
- MCP is implemented only after Core contracts stabilize.
- Public MVP release occurs only after REST + MCP + release hardening.
- Scratchpad MVP is Markdown/Plain Text + autosave + versions + dictation.
- Rich Text is future roadmap only.
- Windows is the MVP reference platform.
- REST is a separate local companion process.
- REST MVP is read-only.
- REST binds loopback only and uses a random bearer token stored in app user data.
- MCP MVP is read-only stdio.
- REST and MCP are thin adapters over one QueryService.
- No bundled LLM or embedding model in V1.
- No multi-device sync in V1.

See [[planning/11-DECISIONS_OPEN_QUESTIONS|Decisions & Open Questions]] and [[planning/adr/INDEX|ADR Index]].


---

# Hardening v1.1 — Control Plane Bootstrap

The canonical custom integration base is **`custom/main`**, created exactly from the pinned baseline commit.

During Bootstrap the agent must:
1. create/verify `custom/main` at the baseline;
2. create `bootstrap/control-plane` from `custom/main`;
3. copy this complete starter kit into `repo/docs/project/`;
4. validate the imported control plane;
5. commit it on the bootstrap branch;
6. merge Bootstrap PR into `custom/main` only after BG passes.

All future `phase/*` branches originate from the latest green `custom/main`.

Machine-readable sources of truth: `TICKET_CATALOG.json`, `FEATURE_MATRIX.json`.

Required safety/control docs: [[orchestration/AGENT_SAFETY_POLICY|Agent Safety Policy]], [[orchestration/IMPLEMENTATION_MAP|Implementation Map]], [[orchestration/SECURITY_THREAT_MODEL|Security Threat Model]], [[orchestration/TEST_METRIC|Planning Test Metric]].
