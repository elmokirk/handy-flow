---
id: "ticket-boot-005"
title: "BOOT-005 — Source Map Verification"
type: "ticket"
status: "ready"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "BOOT-005"
phase: "BOOT"
owner: "Bootstrap Agent"
gate: "BG"
catalog_ref: "TICKET_CATALOG.json#BOOT-005"
---

# BOOT-005 — Source Map Verification

[[planning/tickets/INDEX|← Ticket Index]] · [[planning/phases/PHASE-BOOT|Bootstrap]]

**Owner:** Bootstrap Agent  
**Dependencies:** [[planning/tickets/BOOT-004|BOOT-004]]

## Goal
Verify that the pinned Handy checkout matches `REPO_PATH_MAP.json` before any Phase-0 architecture skeleton or feature work starts.

## Acceptance
- [ ] every `existing_required` path exists;
- [ ] every `planned_new` path is absent or explicitly reconciled as a known baseline collision;
- [ ] checkout resolves to the pinned baseline;
- [ ] `REPO_PATH_VERIFICATION.json` is written under `docs/project/status/`;
- [ ] any mismatch becomes E2 architecture/planning escalation rather than local invention.

## Tests
Run `scripts/validate_repo_baseline.py --repo <repo>`.
