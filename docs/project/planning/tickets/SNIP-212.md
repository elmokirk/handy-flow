---
id: "ticket-snip-212"
title: "SNIP-212 — Snippet UI and processing integration"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "SNIP-212"
phase: "2"
owner: "Snippet Agent"
gate: "G2"
catalog_ref: "TICKET_CATALOG.json#SNIP-212"
---



# SNIP-212 — Snippet UI and processing integration

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-2|Phase 2]]

**Owner:** Snippet Agent  
**Dependencies:** [[planning/tickets/SNIP-211|SNIP-211]], [[planning/tickets/STT-103|STT-103]]  
**Allowed ownership:** `src/components/snippets/** + snippet pipeline adapter`

## Goal

Manage Snippets and create derived representation without mutating raw.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] trigger expansion works mid-sentence
- [ ] normalized_stt unchanged
- [ ] import/export works

## Required tests/evidence

- [ ] pipeline integration
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
