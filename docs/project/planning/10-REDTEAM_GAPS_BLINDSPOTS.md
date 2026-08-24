---
id: "risk-review"
title: "Red Team, Gap & Blindspot Analysis"
type: "risk"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Red Team, Gaps & Blindspots

[[planning/09-ROADMAP|← Roadmap]] · [[planning/11-DECISIONS_OPEN_QUESTIONS|Decisions →]]

## Hardened findings

| Area | Failure | Severity | Mitigation | Status |
|---|---|---:|---|---|
| Audio | crash mid-write | S0 | temp→validate→atomic rename→reconcile | planned |
| Raw semantics | existing cleanup destroys provenance | S1 | `engine_raw` + `normalized_stt` | accepted |
| Retry | overwrite old transcript | S1 | append-only attempts | accepted |
| DB migration | app.db rename adds risk | S1 | keep physical `history.db` for MVP | accepted hardening |
| LLM | corrupts valid source | S1 | representations + fail-open | accepted |
| Snippets | false-positive speech expansion | S1 | deterministic boundary match | accepted |
| REST | localhost data exfiltration | S1 | bearer auth + no wildcard CORS | accepted |
| MCP | destructive agent action | S0 | read-only stdio | accepted |
| Connectors | schema drift breaks tools | S1 | QueryService contract/version | accepted |
| Concurrency | readers collide with writes | S2 | WAL + busy timeout + read-only connectors | planned |
| Swarm | migration conflicts | S1 | Storage Lead exclusive ownership | accepted |
| Swarm | shared-file merge chaos | S2 | Integrator ownership | accepted |
| QA | Playwright misses native bugs | S1 | Windows native smoke | accepted |
| Performance | features slow base dictation | S2 | baseline + regression threshold | planned |
| KB | duplicate retries | S2 | idempotency keys | planned |
| Storage | preserve-all grows forever | S2 | disk usage + future opt-in retention | accepted |
| Logging | private transcript leaks | S1 | no normal-level transcript logging | planned |
| REST | server accessible on LAN | S1 | bind `127.0.0.1` only | accepted |
| REST | arbitrary browser pages query API | S1 | token + restrictive CORS | accepted |

## Gap analysis

| Gap | Resolution |
|---|---|
| exact future fork name | defer to Phase 6; not architecture-blocking |
| branch protection availability depends on GitHub plan/permissions | bootstrap records actual capability |
| app-level encryption not in MVP | rely on OS account/full-disk encryption; future security ADR |
| Wispr import unspecified | not MVP |
| automatic App Styles | P2 |
| selected-text transform | P2 |
| outbound HTTP RAG push | P2 unless promoted |

## Blindspots to revalidate at implementation start

- upstream Handy may advance beyond pinned commit;
- Rust/Tauri dependencies may have changed;
- official MCP SDK API may evolve;
- exact Windows native behavior can vary across apps;
- new upstream changes may already solve/alter planned code paths.

These are handled through Bootstrap freshness/research, not by silent architectural improvisation.

## Stop-ship

Never release with:
- known raw audio loss;
- known canonical transcript loss;
- migration corruption;
- connector write surface;
- unbounded external query;
- REST non-loopback binding;
- source-overwriting transform;
- unresolved S0/S1.


## Hardening v1.1 resolutions
Resolved: `custom/main` baseline isolation; Git-versioned planning control plane; JSON ticket/feature catalogs; exact file allowlists; Soft Delete/Trash/Purge; verified online SQLite backup; deterministic Dictionary/Snippet pipeline; `app_meta` compatibility; Tauri/shortcut/UI/state-machine contracts; interactive Windows native QA; Agent Safety/Retry policy; REST OpenAPI + MCP machine contracts; Prompt provenance snapshots; Phase-0 privacy logging policy; hard STT performance budget.

Remaining risks are implementation/environmental and must be surfaced by gates rather than redesigned by sub-agents.
