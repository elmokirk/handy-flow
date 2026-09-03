---
id: "run-note-304"
title: "RUN_STATE — NOTE-304"
type: "run-state"
status: "generated"
version: "1.0"
updated: "2026-09-03"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "NOTE-304"
run_status: "DONE"
attempt_count: "1"
branch: "phase/03-scratchpad"
worktree: ""
last_commit: ""
---

# RUN_STATE — NOTE-304

## Evidence
| Check | Result |
|---|---|
| `note_search_test` (8 Fixtures) | 8 passed / 0 failed |
| Rust-Suite gesamt | 305 grün / 0 rot |
| Clippy-Ratchet (windows) | 18 <= 22, unverändert |
| `cargo fmt --check` | PASS |
| Playwright (`note-search.spec.ts`, 7 Tests) | 7 passed; Suite gesamt 10 passed |
| `tsc --noEmit` / `eslint src` / `check-translations` / `vite build` | PASS |

## Notes
- Notizen sind die dritte in `04-DATA_PERSISTENCE.md:82` eingefrorene
  FTS-Quelle ("normalized_stt, representations and active notes").
  HIST-231 hatte nur die ersten beiden geliefert; dieses Ticket schließt
  die dritte.
- Der Index bleibt DERIVED und repository-managed. Kanonische
  Note-Writes (create/append/set_title/trash/restore) aktualisieren das
  FTS-Dokument in DERSELBEN Transaktion, damit ein committeter Note-Write
  nie unsichtbar für die Suche sein kann. `set_title`, `trash_note` und
  `restore_note` wurden dafür von `conn.execute` auf Transaktionen
  umgestellt.
- Das FTS-Dokument trägt nur den AKTUELLEN Stand (Titel + höchste
  Version). Überholter Text verlässt den Index, sonst würde die Suche
  alte Versionen zurückliefern.
- Nur AKTIVE Notizen sind indiziert: Trash entfernt das Dokument,
  Restore legt es wieder an.
- `rebuild_search_index` bleibt der eine Recovery-Pfad für alle drei
  Quellen (Fixture 08 prüft Wiederherstellung nach Indexverlust).
- Suche ist gebunden: `limit` wird im Repository auf 1..=200 geklemmt,
  und die FTS-Syntax wird über das gemeinsame `to_match_expr` entschärft.
  Eine Query ohne suchbaren Term liefert leer statt alles.
- Pinned-Notizen ranken zuerst — gleiche Ordnung wie `list_active_notes`,
  damit Blättern und Suchen konsistent wirken.
- `openNote` flusht ausstehende Autosaves VOR dem Wechsel; der
  debounced Save zielt auf `noteRef.current`, ein Wechsel mit noch
  laufendem Timer hätte sonst den alten Text in die neue Notiz
  geschrieben.

## Gefundene und behobene Defekte
- `scratchpad.searchPlaceholder` wäre beinahe in allen 24 Locales
  gefehlt: der Schlüsselname existiert bereits in den Blöcken
  `languages` und `models`, sodass eine naive
  "Key kommt in der Datei vor"-Prüfung ihn übersprungen hat. Gefunden
  über eine explizite Prüfung aller `t()`-Keys der Komponente gegen die
  `en`-Quelle — genau die Lücke, die `check-translations.ts` nicht
  abdeckt (siehe Follow-up in [[escalations/PAD-303-01|PAD-303-01]]).

## Grenzüberschreitungen
Siehe [[escalations/NOTE-304-01|NOTE-304-01]]: die FTS-Naht liegt in
`storage/repositories/search.rs` und `storage/repositories/notes.rs`,
die Command-Freigabe zwingt zu `lib.rs` + `src/bindings.ts`. Variante A
(korrekte Owner, Eskalation protokolliert) wurde gewählt.
