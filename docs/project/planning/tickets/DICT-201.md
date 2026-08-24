---
id: "ticket-dict-201"
title: "DICT-201 — Dictionary persistence and matcher adaptation"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "DICT-201"
phase: "2"
owner: "Dictionary Agent"
gate: "G2"
catalog_ref: "TICKET_CATALOG.json#DICT-201"
---



# DICT-201 — Dictionary persistence and matcher adaptation

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-2|Phase 2]]

**Owner:** Dictionary Agent  
**Dependencies:** [[planning/tickets/DATA-102|DATA-102]]  
**Allowed ownership:** `dictionary manager/repository/tests`

## Goal

Persist rich Dictionary entries while reusing Handy's existing fuzzy/custom-word correction logic.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] legacy Custom Words migratable
- [ ] aliases supported
- [ ] existing matching behavior regression-tested

## Required tests/evidence

- [ ] `15+ dictionary fixtures`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.


## Frozen semantic placement
Dictionary/custom-word correction is part of deterministic STT normalization and contributes to `normalized_stt`. Each successful attempt stores Dictionary snapshot hash + normalizer version; later Dictionary edits never rewrite historical attempts.
