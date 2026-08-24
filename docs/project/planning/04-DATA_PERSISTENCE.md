---
id: "data-persistence"
title: "Data & Persistence"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Data & Persistence

[[planning/03-ARCHITECTURE|← Architecture]] · [[planning/05-DELIVERY_PLAN|Delivery Plan →]] · [[orchestration/DATA_STATE_MACHINES|State Machines]]

## Physical database
Keep Handy's physical `history.db` filename for MVP; expose it internally as `AppDatabase`.

## App metadata
```sql
CREATE TABLE app_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```
Required keys: `schema_version`, `query_contract_version`, `created_by_app_version`. Main app owns migrations. REST/MCP open read-only and fail closed on incompatible versions.

## Raw semantics and deterministic pipeline
```text
engine output
→ engine_raw                                  immutable
→ Dictionary/custom-word correction          deterministic
→ filler removal + normalization             deterministic
→ normalized_stt                             immutable Knowledge Raw
→ Snippets                                   derived representation
→ Prompt Profile                             derived representation
→ delivery/paste
```
Each successful attempt records `normalizer_version` and `dictionary_snapshot_sha256`; historical attempts are never rewritten after Dictionary changes.

## Canonical records

### captures
Stable UUIDv7 id, legacy history id, audio metadata/hash, saved/title/source app, opt-in source window, integrity state, timestamps and nullable `deleted_at_ms`. Do **not** persist duplicate transcription status; derive it from attempts.

### transcription_attempts
UUIDv7 id/capture id/attempt number, `engine_raw`, `normalized_stt`, model/language, `normalizer_version`, `dictionary_snapshot_sha256`, provenance/status/errors, canonical flag and timestamps. A partial unique index guarantees at most one canonical attempt.

### representations
Source attempt/optional parent, kind/text/content hash, processor/version, prompt profile id, effective prompt snapshot, provider/model snapshot, status/error/timestamp. Historical AI output remains interpretable after profile edits/deletion.

### dictionary_entries / snippets / prompt_profiles
Durable SQLite entities with explicit enabled state and deterministic behavior contracts.

### notes / note_versions
`notes`: id, title, pinned, deleted_at_ms, created/updated timestamps. `note_versions`: id, note id, monotonic `version_no`, content/hash/source/timestamp with `UNIQUE(note_id, version_no)`. There is no mutable `current_version_id`; current content is highest version number.

### delivery_events
Append-only audit of what text source was actually delivered:
- id; capture_id; source_attempt_id; optional representation_id;
- `source_kind = normalized_stt | representation`;
- destination kind (`focused_app | scratchpad | clipboard` where applicable);
- immutable `text_sha256`; success/error; created_at.

The event references immutable source text rather than duplicating transcript bodies.

### export_targets / export_jobs
Durable outbox with unique idempotency key.

## Soft Delete / Trash / Purge
High-value content (`captures`, `notes`) follows [[orchestration/DATA_STATE_MACHINES|Data State Machines]]: Delete sets `deleted_at_ms`; normal queries exclude Trash; Restore clears it; Purge is an explicit separately confirmed destructive operation. No scheduled purge exists in MVP.

Purge order is repository-controlled. Partial audio/DB failures are surfaced/reconciled, never silently treated as success.

## Backup before migration
Do **not** filesystem-copy an actively written WAL database. Use SQLite online backup through `rusqlite` backup support or an explicitly proven equivalent snapshot API, write to a new file, reopen it, run `PRAGMA integrity_check`, record source schema metadata, and only then migrate the primary DB. Failure to create/verify backup blocks migration.

## SQLite policy
WAL, foreign keys ON, bounded busy timeout, short transactions, repositories own SQL, connectors read-only, state text constrained by DB `CHECK` + Rust enums.

## FTS
FTS5 is derived and **repository-managed**, not trigger-designed by feature agents. Canonical repository write transactions update the corresponding FTS document in the same transaction; an explicit rebuild command truncates/repopulates FTS from canonical tables. Index `normalized_stt`, representations and active notes. A full rebuild is a recovery requirement.

## Audio lifecycle
`temp → close/flush → validate → hash/size → atomic rename → DB metadata → transcription`. Recovery never silently deletes unknown valid audio.

## Privacy metadata
`source_app` may be recorded. `source_window` is opt-in and disabled by default.

## Retention
Raw audio/source text are preserved by default; disk usage is visible; automatic destructive purge/retention is disabled in MVP.
