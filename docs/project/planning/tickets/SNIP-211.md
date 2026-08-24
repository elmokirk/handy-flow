---
id: "ticket-snip-211"
title: "SNIP-211 — Snippet persistence and deterministic matcher"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "SNIP-211"
phase: "2"
owner: "Snippet Agent"
gate: "G2"
catalog_ref: "TICKET_CATALOG.json#SNIP-211"
---



# SNIP-211 — Snippet persistence and deterministic matcher

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-2|Phase 2]]

**Owner:** Snippet Agent  
**Dependencies:** [[planning/tickets/DATA-102|DATA-102]]  
**Allowed ownership:** `snippets manager/repository/tests`

## Goal

Implement exact phrase-boundary spoken Snippets.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] case semantics explicit
- [ ] punctuation preserved
- [ ] no fuzzy/regex/AI matching

## Required tests/evidence

- [ ] `15+ snippet fixtures`
- [ ] `conflict fixtures`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.


## Frozen matcher semantics
- Unicode-aware phrase boundaries.
- Single pass over `normalized_stt`.
- At each position choose the longest enabled matching trigger.
- Ties: `(priority DESC, trigger length DESC, id ASC)`.
- Replacement text is not rescanned in the same pass.
- No recursion, fuzzy matching, regex or AI in MVP.
