---
id: "ticket-data-102"
title: "DATA-102 — Canonical schema and legacy migration"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "DATA-102"
phase: "1"
owner: "Storage Lead"
gate: "G1"
catalog_ref: "TICKET_CATALOG.json#DATA-102"
---



# DATA-102 — Canonical schema and legacy migration

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-1|Phase 1]]

**Owner:** Storage Lead  
**Dependencies:** [[planning/tickets/DATA-101|DATA-101]]  
**Allowed ownership:** `storage/migrations + repository fixtures`

## Goal

Create new canonical tables and safely migrate legacy transcription_history.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] legacy table preserved
- [ ] legacy transcription text maps to normalized_stt
- [ ] legacy post-process preserved as representation
- [ ] counts/audio references validated

## Required tests/evidence

- [ ] `empty migration`
- [ ] `legacy fixture migration`
- [ ] `rollback backup procedure test`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
