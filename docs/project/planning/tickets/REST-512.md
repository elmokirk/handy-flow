---
id: "ticket-rest-512"
title: "REST-512 — REST v1 query endpoints"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "REST-512"
phase: "5"
owner: "REST Agent"
gate: "G5"
catalog_ref: "TICKET_CATALOG.json#REST-512"
---



# REST-512 — REST v1 query endpoints

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-5|Phase 5]]

**Owner:** REST Agent  
**Dependencies:** [[planning/tickets/REST-511|REST-511]]  
**Allowed ownership:** `REST adapter/tests`

## Goal

Expose versioned QueryService endpoints with bounded pagination.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] all documented GET endpoints
- [ ] stable error envelope
- [ ] limits enforced
- [ ] no write route

## Required tests/evidence

- [ ] router integration
- [ ] `invalid query`
- [ ] concurrent DB read

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
