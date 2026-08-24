---
id: "ticket-boot-001"
title: "BOOT-001 — Workspace and fork initialization"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "BOOT-001"
phase: "BOOT"
owner: "Bootstrap Agent"
gate: "BG"
catalog_ref: "TICKET_CATALOG.json#BOOT-001"
---



# BOOT-001 — Workspace and fork initialization

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-BOOT|Phase BOOT]]

**Owner:** Bootstrap Agent  
**Dependencies:** None  
**Allowed ownership:** `workspace/git only`

## Goal

Create/connect fork and clone into sibling `repo/` without product changes.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] origin points to fork or remote limitation is explicitly escalated
- [ ] upstream points to cjpais/Handy
- [ ] main checkout remains clean

## Required tests/evidence

- [ ] `git remote -v`
- [ ] `git status`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
