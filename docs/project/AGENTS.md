---
id: "agents"
title: "AGENTS — Verbindliche Arbeitsregeln für dieses Projekt"
type: "policy"
status: "accepted"
version: "1.0"
updated: "2026-08-25"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# AGENTS.md — Verbindliche Arbeitsregeln

Gilt für jeden Agenten (Orchestrator-, Feature-, QA-Rollen) in diesem Workspace.
Zusätzlich maßgeblich: die Upstream-Konventionen in `repo/AGENTS.md`
(Code-Stil, Conventional Commits, PR-Templates); diese Datei regelt den
**Arbeitsablauf**.

## Regel 1 — Ticket-Abschluss: Katalog zuerst

Nachdem ein Ticket fertig gestellt ist, MUSS bevor das nächste Ticket startet
der Status in `TICKET_CATALOG.json` (Projekt-Root) angepasst werden:

1. `TICKET_CATALOG.json` → `tickets.<ID>.status` setzen (`DONE`, `IN_REVIEW`, …)
2. RUN_STATE unter `status/runs/<ID>.md` schreiben
3. `python scripts/render_execution_status.py` ausführen
4. Erst danach darf das nächste Ticket beginnen

Ein Ticket ohne aktualisierten Katalog-Status gilt als nicht abgeschlossen.

## Regel 2 — TL;DR nach jedem fertigen Ticket

Nach jedem fertigen Ticket gibt der Agent dem Nutzer eine
**TL;DR-Zusammenfassung in genau 3 Sätzen**: was gemacht wurde,
wie es verifiziert wurde, was als Nächstes kommt.

## Regel 3 — Sauberer Git-Baum & aussagekräftige Commits

- Ein Branch pro Phase (`phase/NN-*`), ein Fokus pro Commit, keine Misch-Commits.
- Commit-Messages müssen aussagekräftig und nachvollziehbar sein:
  Conventional-Commit-Präfix (`feat|fix|chore|docs|refactor|style|test`),
  Betreff ≤ 72 Zeichen, Body erklärt das **Warum** und die Verifikation.
- Integrator-Wiring (geteilte Dateien wie `lib.rs`, `Cargo.toml`) wird im
  Commit transparent als solche gekennzeichnet.
- Generierte Dateien (`clippy-signatures-*.txt`, `EXECUTION_STATUS.md`)
  gehören in den jeweils passenden Status-/Gate-Commit, nicht verstreut.
- Force-Push nur auf eigenen, noch nicht geteilten Vorbereitungs-Branches
  (z. B. Contribution-Squashes) und niemals auf `custom/main`, `phase/*`
  oder `bootstrap/*`.

## Regel 4 — Eskalation bei Problemen & kritischen Entscheidungen

Bei Problemen oder kritischen Architektur-/Produktentscheidungen MUSS der
Nutzer informiert werden und entscheiden können. Format:

1. **Kontext:** Was ist das Problem, welche Evidenz gibt es?
2. **Lösungsvarianten:** maximal 3, jeweils mit Konsequenz/Kosten.
3. **Empfehlung** des Agenten + klare Frage an den Nutzer.

Der Nutzer entscheidet; die Entscheidung wird in
[[planning/11-DECISIONS_OPEN_QUESTIONS]] bzw. der Gate-Report-Tabelle
protokolliert. Kein Agent trifft L2+-Entscheidungen alleine
(siehe [[orchestration/ARCHITECTURE_FREEZE]]).

## Schnellcheck pro Ticket (Kompakt)

```text
[ ] Ticket + Contracts gelesen        → Implementierung in allowed_paths
[ ] Tests/FMT/Ratchet grün            → fokussierter Commit (Regel 3)
[ ] TICKET_CATALOG-Status gesetzt     → RUN_STATE + Render (Regel 1)
[ ] TL;DR an Nutzer (3 Sätze)         → nächstes Ticket (Regel 2)
[ ] Blocker? → Regel-4-Eskalation     → niemals still entscheiden
```
