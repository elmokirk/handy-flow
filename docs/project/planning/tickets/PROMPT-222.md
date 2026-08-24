---
id: "ticket-prompt-222"
title: "PROMPT-222 — Manual Styles and Dictation Transform shortcuts"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "PROMPT-222"
phase: "2"
owner: "Prompt Agent"
gate: "G2"
catalog_ref: "TICKET_CATALOG.json#PROMPT-222"
---



# PROMPT-222 — Manual Styles and Dictation Transform shortcuts

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-2|Phase 2]]

**Owner:** Prompt Agent  
**Dependencies:** [[planning/tickets/PROMPT-221|PROMPT-221]], [[planning/tickets/STT-103|STT-103]]  
**Allowed ownership:** `prompt feature files; shortcut wiring request to Integrator`

## Goal

Apply selected profile after deterministic processing via explicit/manual actions.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] manual Styles work
- [ ] shortcut transform works
- [ ] failure returns previous successful text
- [ ] no auto-app routing

## Required tests/evidence

- [ ] mock success/timeout/500/malformed
- [ ] native shortcut smoke

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
