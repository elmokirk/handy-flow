---
id: "ticket-hist-107"
title: "HIST-107 — History queries and raw/derived UI model"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "HIST-107"
phase: "1"
owner: "History Agent"
gate: "G1"
catalog_ref: "TICKET_CATALOG.json#HIST-107"
---



# HIST-107 — History queries and raw/derived UI model

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-1|Phase 1]]

**Owner:** History Agent  
**Dependencies:** [[planning/tickets/DATA-105|DATA-105]], [[planning/tickets/DATA-106|DATA-106]], [[planning/tickets/DATA-107|DATA-107]]  
**Allowed ownership:** `history manager/feature UI; shared wiring via Integrator`

## Goal

Move History to canonical capture/attempt/representation model.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] user distinguishes raw vs derived
- [ ] audio playback preserved
- [ ] legacy entries visible

## Required tests/evidence

- [ ] repository integration
- [ ] `Playwright history flow`
- [ ] Windows playback smoke

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
