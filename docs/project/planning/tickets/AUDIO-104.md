---
id: "ticket-audio-104"
title: "AUDIO-104 — Atomic WAV lifecycle"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "AUDIO-104"
phase: "1"
owner: "Capture/STT Agent"
gate: "G1"
catalog_ref: "TICKET_CATALOG.json#AUDIO-104"
---



# AUDIO-104 — Atomic WAV lifecycle

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-1|Phase 1]]

**Owner:** Capture/STT Agent  
**Dependencies:** [[planning/tickets/DATA-101|DATA-101]]  
**Allowed ownership:** `audio persistence helper/delegated actions integration`

## Goal

Make new audio capture recoverable across process crashes.

## Scope

Implement only the goal and acceptance criteria in this ticket using frozen architecture.

## Out of scope

- architecture redesign;
- unrelated cleanup;
- dependency additions by feature agent;
- shared-file edits unless Task Packet explicitly delegates;
- future-roadmap features.

## Acceptance criteria

- [ ] temp→validate→hash→atomic rename
- [ ] no silent orphan deletion
- [ ] DB integrity state explicit

## Required tests/evidence

- [ ] kill-point/recovery fixtures
- [ ] WAV validation

## Escalate instead of deciding if

- schema/public DTO contract needs to change beyond accepted ticket;
- a new dependency is required and not already approved;
- a shared/forbidden file must be edited;
- product/security/data semantics become ambiguous;
- upstream pinned code materially contradicts the frozen contract.

Use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].

## Agent completion

Update RUN_STATE from [[orchestration/templates/RUN_STATE|RUN_STATE Template]], then submit a focused PR to the phase branch.
