---
id: "ticket-hist-109"
title: "HIST-109 — Trash, Restore & Explicit Purge"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "HIST-109"
phase: "1"
owner: "History Agent + Storage Lead"
gate: "G1"
catalog_ref: "TICKET_CATALOG.json#HIST-109"
---


# HIST-109 — Trash, Restore & Explicit Purge

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-1|Phase 1]]

**Owner:** History Agent + Storage Lead  
**Dependencies:** [[planning/tickets/DATA-106|DATA-106]], [[planning/tickets/HIST-107|HIST-107]]

## Goal
Implement recoverable deletion for Captures before normal user workflows rely on the new History model.

## Scope
- soft-delete via `deleted_at_ms`;
- normal History/Search excludes Trash;
- Trash list + Restore;
- explicit Purge with confirmation;
- partial audio/DB purge failure is surfaced/reconciled;
- REST/MCP never expose Purge.

## Acceptance
- [ ] Delete never immediately destroys raw audio.
- [ ] Restore returns capture/attempts/representations.
- [ ] Purge is explicit and separately permissioned.
- [ ] Crash during purge is reconciled/surfaced.
- [ ] no background auto-purge.

## Tests
- [ ] trash/restore repository integration;
- [ ] purge crash/failure fixture;
- [ ] QueryService excludes Trash;
- [ ] Windows UI confirmation smoke.

Use canonical machine contract in `TICKET_CATALOG.json`.
