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
- **2026-09-24 / AUDIO-240 seam review and release:** Kirk chose traceable user-confirmed transcript versions as the preferred text in history, copy actions and knowledge export; canonical raw/audio remain unchanged. He rejected a public signed test release before the real 15-minute Windows microphone test and human transcript-quality review. CI success is necessary but not a waiver; the public in-app updater cannot deliver an unpublished candidate.
- **2026-09-24 / AUDIO-240 release decision (supersedes the prior release hold):** Kirk explicitly requests a public signed 0.9.13 update through the app and will perform the real 15-minute microphone, listening and installed-UI checks after updating. Publish only after code CI, Windows x64 NSIS production build, signature and updater-manifest checks pass. Label the remaining owner validation openly; do not claim that 15-minute microphone success is already proven. Other operating systems are outside this Windows validation release.
- **2026-09-24 / AUDIO-240 empty-tap correction:** Kirk reports that a quick trigger with no speech leaves a false capture and a misleading missing/corrupt-original badge. A zero-audio live tap and a live WAV of at most two seconds whose inference recognizes no word are discarded, including their generated audio/checkpoints; captures with potentially recoverable audio, model errors, imported audio and longer failed memos stay visible. This correction is part of 0.9.13 before publishing; old missing-audio cards are not automatically deleted because some represent genuine failures.
- **2026-10-26 / AUDIO-250 import formats:** Kirk requests native phone-recording import. Decided: reuse the existing `manual` origin (no new tag); ship the natively decodable set (WAV, MP3, M4A/MP4/M4B/M4R, ADTS `.aac`, FLAC, OGG-Vorbis, audio-only MOV/MP4) through the existing rodio/Symphonia stack without a converter; reject Opus and AMR with a named-formats error and wait for Symphonia 0.6 instead of a libopus C sidecar; defer audio-track selection inside video containers. Verified with the owner's real WhatsApp AAC sample; published as signed `v0.9.14` from the feature-branch release dispatch.
- **2026-10-27 / AUDIO-250 import hardening:** The 0.9.14 imports failed twice with bare os error 5 during the post-update lock window. Kirk approved retry-with-backoff plus import logging. Decided: absorb os error 5/32 on copy/rename only (3 attempts, ≤450 ms total); keep the bare io error text for the UI; log import phases, failures and retries into handy.log at file-log level; no auto-recreate of `.processing`. Verified with retry/exhaustion tests and the owner AAC sample; shipped as signed `v0.9.15`.
- **2026-10-27 / AUDIO-250 import root cause (supersedes the transient-lock read):** The 0.9.15 import logs plus a direct Win32 probe proved the os error 5 was deterministic, not transient: the staging flush opened the copied file read-only and called sync_all, and FlushFileBuffers requires write access on Windows (GENERIC_READ → ERROR_ACCESS_DENIED, GENERIC_WRITE → 0). Every GUI import failed since AUDIO-240 shipped; the headless CLI path never flushes a staging copy, which kept all earlier tests green. Fix: `long_audio::stage_import_copy` flushes through a write handle and carries a regression test over the exact GUI staging sequence. Shipped as signed `v0.9.16`; ticket stays IN_REVIEW until the owner validates a real in-app import.

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
