---
id: "ticket-prompt-221"
title: "PROMPT-221 — PromptProfile domain and persistence"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "PROMPT-221"
phase: "2"
owner: "Prompt Agent"
gate: "G2"
catalog_ref: "TICKET_CATALOG.json#PROMPT-221"
---



# PROMPT-221 — PromptProfile domain and persistence

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-2|Phase 2]]

**Owner:** Prompt Agent  
**Dependencies:** [[planning/tickets/DATA-102|DATA-102]]  
**Allowed ownership:** `prompt_profiles manager/repository/UI domain`

## Goal

Unify Styles and Dictation Transforms over existing Handy LLM transport.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] kind style/transform
- [ ] existing provider config reused
- [ ] result stores prompt/model provenance

## Required tests/evidence

- [ ] `domain tests`
- [ ] mock provider

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.


## Provenance requirement
Every PromptProfile representation snapshots the effective prompt, provider/model identifiers and processor version. Historical output must remain interpretable after profile edits/deletion.
