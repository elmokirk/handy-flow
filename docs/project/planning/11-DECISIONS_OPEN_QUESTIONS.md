---
id: "decisions"
title: "Decisions & Open Questions"
type: "decision-log"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Decisions & Open Questions

[[planning/10-REDTEAM_GAPS_BLINDSPOTS|← Risk Review]] · [[planning/adr/INDEX|ADRs →]]

## Accepted architecture decisions
Existing ADR-001…015 remain accepted.

Hardening decisions:
- **ADR-016:** `custom/main` is the canonical custom base pinned to the source baseline.
- **ADR-017:** the starter-kit/control-plane is imported into `repo/docs/project/` and Git-versioned during Bootstrap.
- **ADR-018:** captures/notes use Soft Delete/Trash; Purge is explicit.
- **ADR-019:** Dictionary/custom-word correction is part of deterministic `normalized_stt`; Snippets remain derived.
- **ADR-020:** Snippets are single-pass, left-to-right, longest-match-first and non-recursive.
- **ADR-021:** `app_meta` defines schema/query compatibility; connectors fail closed.
- **ADR-022:** source-window metadata is opt-in and private body/secret logging is prohibited by default.
- **ADR-023:** feature agents have destructive-Git prohibitions and a three-cycle implementation/fix budget.
- **ADR-024:** Note current version and Capture transcription status are derived, not duplicated mutable state.

## Deferred decisions
None block Bootstrap through Phase 5.

| Decision | Needed by | Default |
|---|---|---|
| final product/fork name | Phase 6 | temporary working name |
| final bundle identifier | Phase 6 | new identifier |
| Markdown default export root | Phase 4 | user-configured |
| MCP audio-path tool | future | omitted/disabled |
| app-level encryption | future | OS/BitLocker threat model |
| Wispr history import | future | excluded |
| auto-app Styles | P2 | deferred |
| Selected-Text Transform | P2 | deferred |
| Rich Text | P2 | deferred |

All new irreversible product/security/data decisions use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].
