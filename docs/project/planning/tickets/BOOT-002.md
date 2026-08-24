---
id: "ticket-boot-002"
title: "BOOT-002 — Pin baseline and verify source version"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "BOOT-002"
phase: "BOOT"
owner: "Bootstrap Agent"
gate: "BG"
catalog_ref: "TICKET_CATALOG.json#BOOT-002"
---



# BOOT-002 — Pin baseline and verify source version

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-BOOT|Phase BOOT]]

**Owner:** Bootstrap Agent  
**Dependencies:** [[planning/tickets/BOOT-001|BOOT-001]]  
**Allowed ownership:** `git metadata + status docs`

## Goal

Fetch and verify the frozen baseline commit/version.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] HEAD/baseline resolves to af48dd68a64d58aad128fdbb920492a03da53c79
- [ ] version 0.9.5 verified
- [ ] newer upstream is not silently adopted

## Required tests/evidence

- [ ] `git show --no-patch`
- [ ] inspect package.json and tauri.conf.json

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
