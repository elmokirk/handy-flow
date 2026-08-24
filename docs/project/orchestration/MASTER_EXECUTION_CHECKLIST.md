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
- [ ] CONTROL-001 starter kit imported into `repo/docs/project/`
- [ ] CONTROL-002 machine-readable catalog/feature matrix validated
- [ ] CONTROL-003 Agent Safety Policy active
- [ ] BOOT-001 workspace/fork
- [ ] BOOT-002 remotes/pinned baseline
- [ ] BOOT-003 baseline build/tests/native smoke
- [ ] BOOT-004 GitHub/worktree/branch preparation
- [ ] BOOT-005 source map verification
- [ ] BG PASS

## Phase 0
- [ ] BASE-001 performance baseline
- [ ] QA-001 CI/quality hardening
- [ ] ARCH-001 module/contract skeleton verification
- [ ] G0 PASS

## Phase 1
- [ ] DEP-100 core dependency/feature wiring
- [ ] DATA-101 storage foundation
- [ ] DATA-102 canonical schema + legacy migration
- [ ] STT-103 raw/normalized capture
- [ ] AUDIO-104 atomic WAV lifecycle
- [ ] DATA-105 retries + representations
- [ ] DATA-106 recovery reconciler
- [ ] DATA-107 delivery event audit
- [ ] HIST-107 history/query migration
- [ ] DATA-108 retention/disk usage
- [ ] HIST-109 Trash/Restore/Explicit Purge
- [ ] G1 PASS

## Phase 2
- [ ] DICT-201 dictionary persistence/engine
- [ ] DICT-202 dictionary UI/import-export
- [ ] SNIP-211 snippet persistence/matcher
- [ ] SNIP-212 snippet UI/pipeline
- [ ] PROMPT-221 PromptProfile domain
- [ ] PROMPT-222 styles/transforms/hotkeys
- [ ] HIST-231 FTS/search/version UI
- [ ] G2 PASS

## Phase 3
- [ ] NOTE-301 note/version storage
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
