---
id: "ticket-hist-231"
title: "HIST-231 — FTS search and version UI"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "HIST-231"
phase: "2"
owner: "History Agent"
gate: "G2"
catalog_ref: "TICKET_CATALOG.json#HIST-231"
---



# HIST-231 — FTS search and version UI

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-2|Phase 2]]

**Owner:** History Agent  
**Dependencies:** [[planning/tickets/HIST-107|HIST-107]], [[planning/tickets/DATA-105|DATA-105]]  
**Allowed ownership:** `query/search repository + history feature UI`

## Goal

Search canonical raw, representations and later notes; expose versions.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] FTS rebuildable
- [ ] filters bounded
- [ ] version navigation works

## Required tests/evidence

- [ ] FTS integration
- [ ] `Playwright search`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
