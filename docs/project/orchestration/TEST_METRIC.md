---
id: "test-metric"
title: "Planning & Autonomous Execution Test Metric"
type: "quality"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Planning & Autonomous Execution Test Metric

[[START_HERE|← Start Here]] · [[planning/06-QA_GATES|QA Gates]]

This metric evaluates the starter kit itself, not product code.

## A — Software Planning Quality (10)
| Dimension | Weight |
|---|---:|
| product scope/non-goals | 1.0 |
| architecture boundaries | 1.5 |
| data/migration/recovery | 1.5 |
| security/privacy | 1.0 |
| QA/measurable gates | 1.5 |
| roadmap/decisions | 1.0 |
| integration contracts | 1.0 |
| operations/release | 0.75 |
| research/baseline reproducibility | 0.75 |

## B — Autonomous Execution Readiness (10)
| Dimension | Weight |
|---|---:|
| machine-readable ticket DAG | 1.5 |
| exact ownership/implementation map | 1.5 |
| architecture freeze/escalation | 1.25 |
| persistent state/observability | 1.0 |
| worktree/branch/PR safety | 1.25 |
| independent evaluator/gates | 1.25 |
| context/token efficiency | 0.75 |
| retry/stop policy | 0.75 |
| reproducible bootstrap/control plane | 0.75 |

## Hard checks for >9
No broken Wiki links or duplicate IDs; acyclic ticket DAG; every ticket has exact non-empty allowlist/gate/QA profile; all P0 hardening findings resolved; Bootstrap versions control plane in Git; canonical base is `custom/main`; MVP connector contracts are read-only.

## Interpretation
`<7` unsafe autonomous; `7–8` frequent human intervention; `8–9` strong supervised autonomy; `9–9.5` high-confidence autonomous phase execution; `>9.5` exceptional planning with residual risk mainly implementation/environmental.

## Current pre-Bootstrap assessment

| Metric | Before hardening | Hardened v1.1 |
|---|---:|---:|
| Software Planning Quality | ~8.0/10 | **9.6/10** |
| Autonomous Execution Readiness | ~6.5–7.0/10 | **9.4/10** |
| Combined | — | **9.5/10** |

The autonomous score is intentionally capped before Bootstrap. A green Bootstrap Gate must empirically prove the pinned repository/source map, GitHub permissions/rules, baseline build and interactive Windows QA environment before the score can be raised.
