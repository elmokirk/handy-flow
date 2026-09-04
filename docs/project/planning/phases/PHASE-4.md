---
id: "phase-4"
title: "Phase 4 — Knowledge Export"
type: "phase"
status: "accepted"
version: "1.2"
updated: "2026-09-04"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Phase 4 — Knowledge Export

[[planning/phases/INDEX|← Phase Index]] · [[planning/tickets/INDEX|Ticket Index]]

## Outcome

Markdown/Second Brain + outbox/idempotency.

## Entry condition

[[planning/tickets/PAD-306|PAD-306]] opens this phase and must be DONE before
any KB ticket dispatches. It pays the G2/G3 debt from
[[escalations/PAD-305-02|PAD-305-02]] (owner decision 2026-09-04, Variant B)
by making the canonical capture/attempt pipeline the live write path — the
same pipeline KB export and UAT-605 depend on. The DAG enforces this:
KB-401 depends on PAD-306.

## Gate

**G4** as defined in [[planning/06-QA_GATES|QA Gates]].

## Execution

The Orchestrator resolves the ticket dependency DAG and dispatches only ready, non-conflicting tickets.

See [[orchestration/ORCHESTRATOR_RUNBOOK|Orchestrator Runbook]] and [[orchestration/MASTER_EXECUTION_CHECKLIST|Master Checklist]].
