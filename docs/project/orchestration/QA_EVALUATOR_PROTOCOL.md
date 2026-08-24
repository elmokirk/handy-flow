---
id: "qa-evaluator"
title: "Independent QA/Evaluator Protocol"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Independent QA/Evaluator Protocol

[[orchestration/ESCALATION_PROTOCOL|← Escalation]] · [[orchestration/QUERY_REST_MCP_CONTRACT|Connector Contract →]]

## Independence

Implementer completion is not proof.
Evaluator receives:
- ticket;
- contract;
- diff/PR;
- tests;
- gate requirements.

## Review axes

- correctness;
- scope discipline;
- architecture;
- data safety;
- failure behavior;
- test quality;
- security;
- performance;
- Clean Code;
- file ownership.

## Results

- `PASS`
- `FIX_REQUIRED`
- `ARCH_ESCALATION`

## Test proportionality

Use the cheapest test that proves the requirement.
Native/E2E tests are required only when behavior genuinely crosses native/system boundaries.

## Red Team

Before a phase gate, an independent reviewer challenges:
- crash points;
- invalid input;
- concurrency;
- migration;
- privacy;
- unbounded growth/query;
- future coupling.

Findings become:
- fix ticket;
- escalation;
- accepted debt if non-blocking.

See [[planning/06-QA_GATES|QA Gates]].


## Control-plane compliance
Evaluator verifies the PR only touches catalog-allowed paths plus explicitly approved Integrator wiring, dependencies match the catalog, no forbidden Git/data behavior occurred, and Task RUN_STATE records the actual attempt/test evidence.

For native Windows tickets, an interactive Windows evidence artifact is mandatory; a hosted non-interactive CI pass is insufficient for microphone/hotkey/clipboard/window claims.
