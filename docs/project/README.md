---
id: "readme"
title: "handy-flow — Workspace & Project Guide"
type: "context"
status: "accepted"
version: "1.0"
updated: "2026-08-24"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# handy-flow — Custom Handy Fork

Windows-first, local-first Fork von [cjpais/Handy](https://github.com/cjpais/Handy) (v0.9.6 @ Pin `af48dd68`), erweitert um: dauerhafte Audio-/Transcript-Provenance, Dictionary, Snippets, Styles/Dictation Transforms, lokale LLM-Verarbeitung, durchsuchbare History, Markdown-Scratchpad, Second-Brain-Export sowie read-only REST + MCP für lokale Agenten.

**Einstieg für Agenten:** [`START_HERE.md`](START_HERE.md) · Projekt-Kontext: [`00-PROJECT_CONTEXT.md`](00-PROJECT_CONTEXT.md) · Status: [`status/EXECUTION_STATUS.md`](status/EXECUTION_STATUS.md)

---

## Ordnerstruktur

| Ordner/Datei | Zweck |
|---|---|
| `repo/` | **Git-Clone des Projektrepos** (`origin` = privates Repo `elmokirk/handy-flow`, `upstream` = `cjpais/Handy`). Hier entsteht der Produktcode. |
| `worktrees/` | Git-Worktrees pro Phase/Ticket (`phase/…`, `feat/TICKET-…`). Wird ab Phase 0 befüllt. |
| `planning/` | Produktplanung: Vision, Architektur, Datenmodell, Delivery Plan, QA Gates, ADRs, Phasen, 47 Tickets. |
| `orchestration/` | Ausführungs-Contracts: Architecture Freeze, File Ownership, Task Packets, Escalations, REST/MCP/Tauri-Contracts, Agent-Prompts. |
| `research/` | Baseline-Lock, Repository-Map, Connector-/Worktree-Research. |
| `scripts/` | Bootstrap-Skript, Validatoren (`validate_*.py`), Status-Renderer, Worktree-Helfer. |
| `status/` | Maschinell generierter Ausführungsstatus (`EXECUTION_STATUS.md`, `BOOTSTRAP_STATUS.md`, RUN_STATEs unter `runs/`). |
| `escalations/` | Eskalationsprotokolle nach `orchestration/ESCALATION_PROTOCOL.md`. |
| `conventions/` | Planungs-Handoff-Konventionen. |
| Root `*.json` | Machine-readable Control Plane: `TICKET_CATALOG.json` (kanonisch), `FEATURE_MATRIX.json`, `BASELINE_LOCK.json`, `DEPENDENCY_LOCK.json`, Checksums, Validation-Reports. |
| `CHECKSUMS.sha256` | SHA256 über das komplette Starterkit (Snapshot nach jedem Control-Plane-Update). |

**Regel:** Alles außer `repo/` und `worktrees/` ist die versionskontrollierte Control Plane und wird nach `repo/docs/project/` importiert (siehe Bootstrap).

## Branch- & Remote-Modell

```text
upstream/main (cjpais/Handy, nur lesend)
        \
         custom/main          ← kanonische Integrations-Basis am gepinnten Baseline-Commit
            ├── phase/*       ← Phasen-Branches
            │     └── feat/*  ← Ticket-Branches (eigene Worktrees)
            └── bootstrap/control-plane
```

- `origin` = **privates Repo** `elmokirk/handy-flow` — kein öffentlicher GitHub-Fork, nichts ist extern sichtbar.
- `upstream` = `cjpais/Handy`.
- Baseline-Strategie: **PINNED** an `af48dd68a64d58aad128fdbb920492a03da53c79` (v0.9.6, Refresh 2026-08-24 owner-approved; Historie siehe [`research/HANDY_BASELINE.md`](research/HANDY_BASELINE.md)). Upstream-Updates nur als eigener `BASELINE-REFRESH`-Task.

### Upstream-Contribution (falls später gewünscht)

Das private Repo blockiert nichts — ein echter Fork wird erst bei Bedarf angelegt:

```powershell
gh repo fork cjpais/Handy --clone=false   # einmalig, öffentlicher Fork entsteht
cd repo
git remote add public-fork https://github.com/<login>/Handy.git
git push public-fork <feature-branch>     # Branch in den Fork pushen
# PR auf cjpais/Handy von <login>:<feature-branch> aus öffnen
git remote remove public-fork             # optional danach wieder entfernen
```

## Toolchain (Bootstrap-Umgebung, 2026-08-24)

| Werkzeug | Version |
|---|---|
| Windows | 11, x64, MSVC Build Tools 2022 (C++-Workload), WebView2 Runtime |
| Rust | rustc/cargo 1.98.0 (stable-x86_64-pc-windows-msvc via rustup 1.29.0) |
| Bun | 1.3.8 |
| Node | 24.11.1 |
| Python | 3.14.0 |
| Git / gh CLI | Systeminstalltion, authentifiziert als `elmokirk` |

## Ausführungsmodell

Phasenreihenfolge: **BOOT → G0 → G1 → … → G6 → MVP** (siehe [`orchestration/MASTER_EXECUTION_CHECKLIST.md`](orchestration/MASTER_EXECUTION_CHECKLIST.md)).

- `TICKET_CATALOG.json` ist kanonisch für Dependencies, erlaubte Pfade, Gates und Retry-Budgets; Markdown-Tickets erklären Intent/Acceptance.
- Ein Agent übernimmt hier Orchestrator-/Integrator-/Implementer-Rollen sequentiell entlang des DAG; Ticket-Arbeit bleibt in eigenen Branches/Worktrees, Integrator-Dateien (`lib.rs`, `Cargo.toml`, `App.tsx`, i18n, Bindings) werden gebündelt gemergt.
- Jeder Gate-Fortschritt nur bei grünen QA-Gates ([`planning/06-QA_GATES.md`](planning/06-QA_GATES.md)); Blocker → Eskalation in `escalations/`.
- Status immer generiert via `python scripts/render_execution_status.py` — nie manuell editieren.

## Layout-Adaption gegenüber dem Original-Starterkit

Das Kit war ursprünglich für ein Geschwister-Layout (`starter-kit/ + repo/ + worktrees/`) ausgelegt. In diesem Workspace liegen `repo/` und `worktrees/` **innerhalb** des Kit-Ordners. Konsequenzen:

- `scripts/bootstrap.ps1` nicht unverändert verwenden (Import würde `repo/` rekursiv in sich selbst kopieren); die Bootstrap-Schritte wurden adaptiert manuell ausgeführt.
- `scripts/validate_starter_kit.py` schließt `repo/`, `worktrees/`, `node_modules`, `.git` explizit aus.
