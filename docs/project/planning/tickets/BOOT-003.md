---
id: "ticket-boot-003"
title: "BOOT-003 — Baseline build, tests and native smoke"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "BOOT-003"
phase: "BOOT"
owner: "QA Agent"
gate: "BG"
catalog_ref: "TICKET_CATALOG.json#BOOT-003"
---



# BOOT-003 — Baseline build, tests and native smoke

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-BOOT|Phase BOOT]]

**Owner:** QA Agent  
**Dependencies:** [[planning/tickets/BOOT-002|BOOT-002]]  
**Allowed ownership:** `no product code`

## Goal

Prove upstream baseline is buildable/testable in the implementation environment.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] existing automated quality commands recorded
- [ ] Windows native limitations explicitly recorded
- [ ] failures classified as upstream/environment/custom

## Required tests/evidence

- [ ] `bun run lint`
- [ ] `bun run format:check`
- [ ] `bun run build`
- [ ] `bun run test:playwright`
- [ ] `cargo test`

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
