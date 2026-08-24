---
id: "ticket-note-304"
title: "NOTE-304 — Notes search and restore"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "NOTE-304"
phase: "3"
owner: "Scratchpad Agent"
gate: "G3"
catalog_ref: "TICKET_CATALOG.json#NOTE-304"
---



# NOTE-304 — Notes search and restore

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-3|Phase 3]]

**Owner:** Scratchpad Agent  
**Dependencies:** [[planning/tickets/NOTE-301|NOTE-301]], [[planning/tickets/HIST-231|HIST-231]]  
**Allowed ownership:** `notes search/UI`

## Goal

Search notes and restore prior versions.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] search bounded
- [ ] restore append-only
- [ ] pinned state preserved

## Required tests/evidence

- [ ] FTS tests
- [ ] `Playwright`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
