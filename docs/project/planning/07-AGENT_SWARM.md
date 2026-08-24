---
id: "agent-swarm"
title: "Agent Swarm Model"
type: "planning"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Agent Swarm

[[planning/06-QA_GATES|← QA Gates]] · [[planning/08-INTEGRATIONS|Integrations →]]

## Roles

| Role | Responsibility |
|---|---|
| Orchestrator | DAG, spawning, state, escalation aggregation |
| Integrator | shared files, phase integration, generated code |
| Storage Lead | schema/migrations |
| Feature Agent | one bounded domain/ticket |
| QA/Evaluator | independent verification |
| Red-Team Reviewer | adversarial phase review |

## Core rule

Parallelize implementation **inside frozen contracts**.

Do not allow multiple agents to:
- redesign the schema;
- edit shared registration files;
- independently add dependencies;
- change external contracts.

Detailed mechanics:
- [[orchestration/ORCHESTRATOR_RUNBOOK|Orchestrator Runbook]]
- [[orchestration/FILE_OWNERSHIP|File Ownership]]
- [[orchestration/TASK_HANDOFF_PROTOCOL|Task & Handoff Protocol]]
- [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]]
