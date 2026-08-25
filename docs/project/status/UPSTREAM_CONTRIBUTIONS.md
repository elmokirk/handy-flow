---
id: "upstream-contributions"
title: "Upstream Contribution Candidates — cjpais/Handy"
type: "status"
status: "generated"
version: "1.0"
updated: "2026-08-25"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Upstream Contribution Candidates

Branch `contribution/upstream-fixes` (Basis: `af48dd6` = upstream main v0.9.6)
enthält **9 isolierte, jeweils eigenständig PR-fähige Commits** mit Fixes für
Probleme im ursprünglichen Handy-Core, die während des Bootstraps/Phase 0/1
aufgefallen sind. Verifikation pro Stand: `cargo test` 206/206,
**clippy --all-targets = 0 warnings** (vorher 22), `cargo fmt --check` grün.

Für echte Upstream-PRs gilt das Mandat aus repo/AGENTS.md: Abschnitte
"Human Written Description"/AI-Disclosure vom **Nutzer** ausfüllen lassen;
Rezept siehe Root-README (Ordner "Upstream-Contribution").

## Commit-Liste (PR-Reihenfolge nach Nutzen sortiert)

| # | Commit | Problem im Original | Vorschlag PR-Titel |
|---|---|---|---|
| 1 | `117e4e8` build .gitattributes | Windows-Clones default CRLF → `format:check` schlägt auf 350+ unveränderten Dateien fehl; kein `.gitattributes` vorhanden | `build: enforce LF via .gitattributes so prettier passes on Windows clones` |
| 2 | `0b7e0ae→fc98995` style test modules | `items_after_test_module` in lib.rs/recorder.rs/transcription.rs; recorder hat Testmodul verschachtelt IN run_consumer | `style: keep cfg(test) modules last in file (3 files)` |
| 3 | `312f5b8` cfg-gate imports | `unused import Emitter` (secure_input) + glob (clamshell tests) auf Nicht-macOS-Builds | `fix(lint): gate macOS-only imports to silence warnings on other targets` |
| 4 | `238d621` dead assignment | `model_takes_initial_prompt` initialisiert, aber Wert nur im Probe-Block gelesen → Warnung | `fix(lint): scope initial-prompt capability to its only read site` |
| 5 | `3bde94a` needless borrows ×10 | `&model.filename` in format!-Args (model.rs) | `style(clippy): drop needless borrows in format! arguments` |
| 6 | `461f87b` duplicate arms | identische Denied-Zweige + unnötiges return (commands/audio.rs) | `refactor(clippy): merge duplicate denial branches` |
| 7 | `9d007ff` repeat().take() | modernere `repeat_n` (gguf_meta.rs Test-Fixture) | `style(clippy): use iter::repeat_n` |
| 8 | `3c4cecc` ptr casts | manueller Doppel-Cast statt `std::ptr::eq` (paste_tx/windows.rs) | `fix(clippy): use ptr::eq for pending-tx identity check` |
| 9 | `df9c8b6` writeln! | write! mit trailing `\n` (portable.rs Test) | `style(clippy): prefer writeln!` |

## Ergebnis nach Anwendung aller Commits

```text
cargo clippy --all-targets          → 0 warnings (vorher 22 distinct)
cargo test                          → 206 passed / 0 failed
cargo fmt --check                   → clean
bun run format:check                → clean auf frischem Windows-Clone (via #1)
```

Damit wird ein strenger `-D warnings`-CI-Gate für upstream machbar;
unsere Fork-Harness (custom-quality.yml) zeigt den Weg.

## Bewusst NICHT upstream-fähig (fork-spezifisch)

- `.prettierignore: docs/project` — existiert nur in unserem Fork.
- MSBuild-Race bei ggml-Vulkan-Shader-Generierung (`ssm_conv.comp.cpp`
  leer bei -parallel 32) — Ursprung liegt im Drittanbieter-Crate
  transcribe-cpp/ggml, nicht im Handy-Repo; ggf. dort melden.
- `test.yml` prüft cargo test nur unter Linux — Beobachtung/Diskussions-
  kandidat, kein Fix-Commit (Windows-x64 hat vulkan-Feature aktiv).

## Ablauf bei echtem Upstream-PR (später)

Root-README "Upstream-Contribution"-Rezept befolgen; je Commit ein eigener
PR gegen `cjpais:main`; Template-Pflichten (Human-Written-Description!)
vom Nutzer erledigen lassen.
