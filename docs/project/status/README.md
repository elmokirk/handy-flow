---
id: "status-index"
title: "Status & Observability"
type: "status"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Status & Observability

[[START_HERE|← Start Here]] · [[orchestration/MASTER_EXECUTION_CHECKLIST|Master Checklist]]

## Ownership

- Global phase/master status: Orchestrator only.
- Per-ticket status: ticket agent's RUN_STATE.
- Escalation aggregation: Orchestrator only.
- Gate report: QA/Evaluator + Integrator.

This prevents agents from conflicting on one shared checklist.

Recommended runtime additions:

```text
status/
├── BOOTSTRAP_STATUS.md
├── PHASE-0.md
├── PHASE-1.md
└── runs/
    ├── DATA-101.md
    └── ...
```


`EXECUTION_STATUS.md` is generated from the ticket catalog and per-ticket RUN_STATE frontmatter using `scripts/render_execution_status.py`; do not edit it manually.
