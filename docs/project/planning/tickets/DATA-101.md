---
id: "ticket-data-101"
title: "DATA-101 — Storage foundation and database policy"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "DATA-101"
phase: "1"
owner: "Storage Lead"
gate: "G1"
catalog_ref: "TICKET_CATALOG.json#DATA-101"
---



# DATA-101 — Storage foundation and database policy

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-1|Phase 1]]

**Owner:** Storage Lead  
**Dependencies:** [[planning/tickets/ARCH-001|ARCH-001]], [[planning/tickets/QA-001|QA-001]], [[planning/tickets/DEP-100|DEP-100]]  
**Allowed ownership:** `src-tauri/src/storage/**`

## Goal

Introduce AppDatabase abstraction over existing history.db, UUIDv7 IDs, WAL/FK/busy-timeout policy.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] no physical DB rename
- [ ] repositories own SQL
- [ ] connection policy tested

## Required tests/evidence

- [ ] storage unit tests
- [ ] concurrent connection integration test

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
