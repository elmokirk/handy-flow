---
id: "escalation-protocol"
title: "Escalation & Decision Protocol"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Escalation & Decision Protocol

[[orchestration/TASK_HANDOFF_PROTOCOL|← Task Protocol]] · [[orchestration/QA_EVALUATOR_PROTOCOL|QA/Evaluator →]]

## Categories

`ARCH`, `DATA`, `API`, `SEC`, `SCOPE`, `PERF`, `UX`, `LICENSE`, `OPS`.

## Severity

- E0 informational;
- E1 concern, ticket can continue;
- E2 ticket blocked;
- E3 phase blocked;
- E4 release/data/security stop.

## E2+ format

Every escalation must include:
- problem/decision;
- evidence;
- exact blocked task;
- Variant A;
- Variant B;
- recommendation;
- trade-offs;
- affected files/contracts;
- whether independent work can continue.

Use [[orchestration/templates/ESCALATION|Escalation Template]].

## No hidden architecture

An agent cannot ask A/B while silently implementing C.

## Phase aggregation

The Orchestrator owns the global decision queue.
At phase end it presents all unresolved owner decisions in one compact table.

## Resolution

Owner decision:
→ accepted/superseding ADR
→ contract update
→ ticket/DAG update
→ escalation resolved
→ blocked agent resumes.
