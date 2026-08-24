---
id: "ticket-boot-004"
title: "BOOT-004 — GitHub and worktree preparation"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "BOOT-004"
phase: "BOOT"
owner: "Integrator"
gate: "BG"
catalog_ref: "TICKET_CATALOG.json#BOOT-004"
---



# BOOT-004 — GitHub and worktree preparation

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-BOOT|Phase BOOT]]

**Owner:** Integrator  
**Dependencies:** [[planning/tickets/BOOT-003|BOOT-003]]  
**Allowed ownership:** `git/GitHub config only`

## Goal

Prepare phase/worktree conventions and branch protection capability.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] worktrees directory prepared
- [ ] branch protection/rules status recorded
- [ ] no feature branch started

## Required tests/evidence

- [ ] `git worktree list`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
