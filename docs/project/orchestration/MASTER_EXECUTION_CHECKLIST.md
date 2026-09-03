---
id: "master-checklist"
title: "Master Execution Checklist"
type: "status"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Master Execution Checklist

[[orchestration/QUERY_REST_MCP_CONTRACT|← Connector Contract]] · [[status/README|Status →]]

**Owner: Orchestrator only.**
Sub-agents update per-ticket RUN_STATE, not this shared file.

## Bootstrap
- [x] CONTROL-001 starter kit imported into `repo/docs/project/`
- [x] CONTROL-002 machine-readable catalog/feature matrix validated
- [x] CONTROL-003 Agent Safety Policy active
- [x] BOOT-001 workspace/fork
- [x] BOOT-002 remotes/pinned baseline
- [x] BOOT-003 baseline build/tests/native smoke
- [x] BOOT-004 GitHub/worktree/branch preparation
- [x] BOOT-005 source map verification
- [x] BG PASS

## Phase 0
- [x] BASE-001 performance baseline
- [x] QA-001 CI/quality hardening
- [x] ARCH-001 module/contract skeleton verification
- [x] G0 PASS

## Phase 1
- [x] DEP-100 core dependency/feature wiring
- [x] DATA-101 storage foundation
- [x] DATA-102 canonical schema + legacy migration
- [x] STT-103 raw/normalized capture
- [x] AUDIO-104 atomic WAV lifecycle
- [x] DATA-105 retries + representations
- [x] DATA-106 recovery reconciler
- [x] DATA-107 delivery event audit
- [x] HIST-107 history/query migration
- [x] DATA-108 retention/disk usage
- [x] HIST-109 Trash/Restore/Explicit Purge
- [x] G1 PASS

## Phase 2
- [x] DICT-201 dictionary persistence/engine
- [x] DICT-202 dictionary UI/import-export
- [x] SNIP-211 snippet persistence/matcher
- [x] SNIP-212 snippet UI/pipeline
- [x] PROMPT-221 PromptProfile domain
- [x] PROMPT-222 styles/transforms/hotkeys
- [x] HIST-231 FTS/search/version UI
- [x] G2 PASS

## Phase 3
- [x] NOTE-301 note/version storage
- [ ] PAD-302 Scratchpad window/autosave
- [ ] PAD-303 dictation/transforms
- [ ] NOTE-304 search/restore
- [ ] G3 PASS

## Phase 4
- [ ] KB-401 Knowledge DTO/export service
- [ ] KB-402 outbox/idempotency
- [ ] KB-403 Markdown connector
- [ ] KB-404 settings/audio modes
- [ ] G4 PASS

## Phase 5
- [ ] DEP-510 connector dependency lock
- [ ] QUERY-501 stable QueryService
- [ ] REST-511 server/auth/config
- [ ] REST-512 endpoints/tests
- [ ] MCP-521 stdio server/tools
- [ ] MCP-522 integration/client tests
- [ ] INT-590 Local Integrations settings/launch config
- [ ] INT-530 adapter parity/security
- [ ] G5 PASS

## Phase 6
- [ ] REL-601 distribution identity
- [ ] REL-602 backup/upgrade/recovery
- [ ] REL-603 security/license/privacy
- [ ] REL-604 full regression/performance
- [ ] UAT-605 seven-day daily driver
- [ ] G6 PASS
- [ ] MVP RELEASE
