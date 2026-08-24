---
id: "run-base-001"
title: "RUN_STATE — BASE-001"
type: "run-state"
status: "generated"
version: "1.0"
updated: "2026-08-24"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "BASE-001"
run_status: "DONE"
attempt_count: "1"
branch: "phase/00-baseline-harness"
worktree: ""
last_commit: ""
---

# RUN_STATE — BASE-001

## Checklist
- [x] baseline/context read (QA Gates, TEST_METRIC)
- [x] Agent Safety Policy read
- [x] exact catalog paths verified (docs/project/status/** + docs/project/research/** only — no code touched)
- [x] repeatable benchmark procedure documented
- [x] baseline results stored
- [x] Notepad/browser/editor smoke DOCUMENTED (execution deferred to owner session per QA-gates interactive policy)
- [x] self-review complete

## Files changed
- `research/BENCHMARK_PROCEDURE.md` (new — Tier A automated + Tier B interactive procedure, fixture spec, comparison rules)
- `status/PERFORMANCE_BASELINE.md` (new — recorded G0 results incl. deferral rationale)

## Tests/Evidence
| Metric | Result |
|---|---|
| Startup → main window (debug) | median 383 ms (3 iterations, cold discarded) |
| Idle RAM | ~79.3 MB WS / ~35.0 MB private |
| cargo test | 206/206, suite 0.34 s |
| clippy ratchet ceiling seeded | 22 distinct warnings (Windows) |

## Escalations
- none

## Deviations
- Tier-B interactive metrics deferred with documented procedure (consistent with BOOT-003 native-smoke handling; QA gates require interactive evidence from owner machine anyway).
