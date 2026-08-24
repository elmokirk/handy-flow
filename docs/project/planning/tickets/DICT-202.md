---
id: "ticket-dict-202"
title: "DICT-202 — Dictionary UI and import/export"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "DICT-202"
phase: "2"
owner: "Dictionary Agent"
gate: "G2"
catalog_ref: "TICKET_CATALOG.json#DICT-202"
---



# DICT-202 — Dictionary UI and import/export

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-2|Phase 2]]

**Owner:** Dictionary Agent  
**Dependencies:** [[planning/tickets/DICT-201|DICT-201]]  
**Allowed ownership:** `src/components/dictionary/**, hook`

## Goal

Provide CRUD/search/alias/import-export UX.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] CRUD works
- [ ] enable/disable works
- [ ] import/export roundtrip

## Required tests/evidence

- [ ] frontend tests/Playwright

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
