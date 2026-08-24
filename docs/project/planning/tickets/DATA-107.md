---
id: "ticket-data-107"
title: "DATA-107 — Delivery Event Audit"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "DATA-107"
phase: "1"
owner: "Storage Lead + Capture/STT Agent"
gate: "G1"
catalog_ref: "TICKET_CATALOG.json#DATA-107"
---

# DATA-107 — Delivery Event Audit

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-1|Phase 1]]

**Owner:** Storage Lead + Capture/STT Agent  
**Dependencies:** [[planning/tickets/DATA-105|DATA-105]], [[planning/tickets/STT-103|STT-103]]

## Goal
Persist an append-only audit event identifying the exact immutable text source used by a successful/failed delivery action.

## Scope
- `delivery_events` repository/table;
- source is canonical attempt `normalized_stt` or a Representation;
- store source reference + text SHA-256, destination kind, success/error, timestamp;
- Integrator wires event creation into final delivery/paste routing;
- no transcript-body duplication required.

## Acceptance
- [ ] every successful normal paste records one event;
- [ ] failed delivery can be diagnosed without mutating source;
- [ ] hash matches referenced immutable source text;
- [ ] retries create new events rather than overwrite;
- [ ] History can query delivery events by capture.

## Tests
- [ ] normalized-STT delivery;
- [ ] representation delivery;
- [ ] failed paste/delivery;
- [ ] duplicate retry remains append-only.
