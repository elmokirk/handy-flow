---
id: "dependency-lock"
title: "Dependency Lock Policy"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Dependency Lock Policy

[[research/CONNECTOR_RESEARCH|← Connector Research]] · [[planning/phases/PHASE-5|Phase 5]]

## Rule
Preferred libraries are named during planning, but exact versions are pinned in the introducing phase after compatibility/security/license review.

## Observed during hardening — 2026-08-21
- official MCP Rust SDK latest observed: `rmcp-v3.1.2`;
- Axum docs latest observed: `0.8.9`.

These are research anchors, not perpetual pins.

## Phase-5 lock gate
Confirm Rust toolchain; review current `rmcp`/`axum` changes; select compatible versions; Integrator updates manifest/lock; record `DEPENDENCY_LOCK.json`; then dispatch connector tickets. Feature agents may not independently upgrade dependencies.
