---
id: "run-dict-201"
title: "RUN_STATE — DICT-201"
type: "run-state"
status: "generated"
version: "1.0"
updated: "2026-08-25"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "DICT-201"
run_status: "IN_REVIEW"
attempt_count: "2"
branch: "phase/02-dictionary-snippets"
worktree: ""
last_commit: ""
---

# RUN_STATE — DICT-201

## Evidence
| Check | Result |
|---|---|
| 16 Dictionary fixtures (Persistence + Matcher-Regression) | PASS |
| Full local suite | 254 tests green |
| Clippy ratchet / fmt | PASS (22<=22) |

## Key findings
- Upstream matcher semantics empirisch gepinnt (Probe-Läufe): Case-Pattern-Erhalt, ASCII-only-Fallback ("händy" wird übersprungen), Soundex-Boost erzeugt aggressive False Positives (anderes→Handy), Multi-Wort-Aliase wirken via N-Gram (second brain→SecondBrain). Fixtures dokumentieren IST-Verhalten als Regressionsschutz.
- Alias→Term-Ersetzung ist unter aktueller Upstream-API nicht ausdrückbar (flache Wortliste ersetzt sich selbst) → Integration-Request an das Phase-2-Wiring-Bundle; Kandidat für eigenen Upstream-Contribution-PR (find_best_match replacement mapping).
- Zweiter Reentrant-Lock-Deadlock (upsert_entry) durch Fixture-Hang aufgedeckt und gescopet; systematischer Muster-Scan über alle Repositories clean.

## Integration requests
- Pipeline-Anbindung: correction_terms + snapshot_sha256 in post_process_transcription_text (Integrator-Bundle mit STT-Wiring)
- Command-/Binding-Exposure für Settings-UI (DICT-202)

## Deviations
- Migration V7 + TARGET_VERSION=7 als Integrator-Aktion (migrations.rs ist Storage-Lead-Shared-File, nicht in Ticket-Pfaden) — konsistentes etabliertes Muster.
