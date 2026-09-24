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
- **2026-09-23 / AUDIO-240 / ADR-025:** One final WAV grows in the recordings directory from capture start; checkpoint/sync every 30 seconds and on stop. Capture precedes inference, failures and orphans remain visible, in-memory windows run in an isolated worker, CPU fallback requires user choice, short jobs outrank remaining long windows, and late long results are history-only.
- **2026-09-24 / AUDIO-240:** The release gate is a successful real 15-minute Windows dictation, not guaranteed 60-minute completion. WAV/MP3 picker and drag-and-drop import are included in the same release and use the same bounded queue; imported captures use import time and a manual-import label. Separate spec/safety and simplicity/standards reviews block release on open findings.
- **2026-09-24 / AUDIO-240 live preview:** Kirk prioritized crash isolation over in-process live text preview for the first release. Temporarily disable live preview on streaming-capable models; keep live recording and post-stop transcription. A future worker-based preview needs separate review.
- **2026-09-24 / AUDIO-240 hotfix:** Installed `a16698b` regressed direct paste by limiting background delivery to 60 seconds of audio. Kirk requires one final focused-field paste for timely ordinary dictations, a calm history spinner, and one-step worker inference checked now for short recordings. Keep the isolated worker and history-only behavior for late/stale results; six minutes is the optimistic one-shot/foreground UX ceiling, not a recording or hardware limit. A changed foreground window also suppresses Windows auto-paste; same-window field changes cannot be reliably identified. The owner's Parakeet Q8/Vulkan worker completed a 5:48 local WAV in one step, but transcript quality still needs human review.

## Deferred decisions
None block Bootstrap through Phase 5.

| Decision | Needed by | Default |
|---|---|---|
| final product/fork name | Phase 6 | temporary working name |
| final bundle identifier | Phase 6 | new identifier |
| Markdown default export root | Phase 4 | user-configured |
| MCP audio-path tool | future | omitted/disabled |
| app-level encryption | future | OS/BitLocker threat model |
| Wispr history import | Phase 3 | ACCEPTED 2026-08-31: IMP-001 (copy semantics, read-only source, idempotent; owner deletes Wispr dir after uninstall) |
| auto-app Styles | P2 | deferred |
| Selected-Text Transform | P2 | deferred |
| REL-601 update source (private repo) | Phase 6 / on demand | gh-authenticated in-app update (no signing infra); signed latest.json as later hardening |
| Rich Text | P2 | deferred |

All new irreversible product/security/data decisions use [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].
