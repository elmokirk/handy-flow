---
id: "ticket-base-001"
title: "BASE-001 — Performance and behavior baseline"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "BASE-001"
phase: "0"
owner: "QA/Performance Agent"
gate: "G0"
catalog_ref: "TICKET_CATALOG.json#BASE-001"
---



# BASE-001 — Performance and behavior baseline

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-0|Phase 0]]

**Owner:** QA/Performance Agent  
**Dependencies:** [[planning/tickets/BOOT-005|BOOT-005]]  
**Allowed ownership:** `status/research artifacts only`

## Goal

Record latency/RAM/VRAM/native behavior before custom features.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] repeatable benchmark procedure
- [ ] baseline results stored
- [ ] Notepad/browser/editor smoke documented

## Required tests/evidence

- [ ] manual/native benchmark
- [ ] `recorded fixture timing`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
