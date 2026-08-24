---
id: "ticket-query-501"
title: "QUERY-501 — Stable QueryService and DTOs"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "QUERY-501"
phase: "5"
owner: "Query Agent"
gate: "G5"
catalog_ref: "TICKET_CATALOG.json#QUERY-501"
---



# QUERY-501 — Stable QueryService and DTOs

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-5|Phase 5]]

**Owner:** Query Agent  
**Dependencies:** [[planning/tickets/HIST-231|HIST-231]], [[planning/tickets/NOTE-304|NOTE-304]], [[planning/tickets/DICT-201|DICT-201]], [[planning/tickets/SNIP-211|SNIP-211]], [[planning/tickets/PROMPT-221|PROMPT-221]]  
**Allowed ownership:** `managers/query.rs + query DTOs/tests`

## Goal

Freeze the read contract for UI/REST/MCP.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] pagination default/max enforced
- [ ] deterministic cursor ordering
- [ ] no transport dependencies
- [ ] service contract versioned

## Required tests/evidence

- [ ] fixture DB QueryService tests

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.


## Frozen compatibility requirements
- Query contract version lives in `app_meta`.
- Normal queries exclude soft-deleted records.
- Cursor ordering is `(created_at DESC, id DESC)`.
- External DTOs omit `source_window` by default.
