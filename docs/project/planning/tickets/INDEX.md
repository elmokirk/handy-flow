---
id: "ticket-index"
title: "Ticket Index"
type: "index"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Ticket Index

[[planning/phases/INDEX|← Phases]] · [[orchestration/MASTER_EXECUTION_CHECKLIST|Master Checklist →]]

## Phase BOOT
- [[planning/tickets/BOOT-001|BOOT-001 — Workspace and fork initialization]]
- [[planning/tickets/BOOT-002|BOOT-002 — Pin baseline and verify source version]]
- [[planning/tickets/BOOT-003|BOOT-003 — Baseline build, tests and native smoke]]
- [[planning/tickets/BOOT-004|BOOT-004 — GitHub and worktree preparation]]

- [[planning/tickets/BOOT-005|BOOT-005 — Source Map Verification]]

## Phase 0
- [[planning/tickets/BASE-001|BASE-001 — Performance and behavior baseline]]
- [[planning/tickets/QA-001|QA-001 — CI and quality harness hardening]]
- [[planning/tickets/ARCH-001|ARCH-001 — Validate module skeleton and contracts]]

## Phase 1
- [[planning/tickets/DATA-101|DATA-101 — Storage foundation and database policy]]
- [[planning/tickets/DATA-102|DATA-102 — Canonical schema and legacy migration]]
- [[planning/tickets/STT-103|STT-103 — Persist engine_raw and normalized_stt]]
- [[planning/tickets/AUDIO-104|AUDIO-104 — Atomic WAV lifecycle]]
- [[planning/tickets/DATA-105|DATA-105 — Attempts, canonical selection and representations]]
- [[planning/tickets/DATA-106|DATA-106 — Recovery reconciler]]
- [[planning/tickets/HIST-107|HIST-107 — History queries and raw/derived UI model]]
- [[planning/tickets/DATA-108|DATA-108 — Preserve-all and disk usage]]

- [[planning/tickets/HIST-109|HIST-109 — Trash, Restore & Explicit Purge]]

- [[planning/tickets/DEP-100|DEP-100 — Core Rust dependency/feature wiring]]

- [[planning/tickets/DATA-107|DATA-107 — Delivery Event Audit]]

## Phase 2
- [[planning/tickets/DICT-201|DICT-201 — Dictionary persistence and matcher adaptation]]
- [[planning/tickets/DICT-202|DICT-202 — Dictionary UI and import/export]]
- [[planning/tickets/SNIP-211|SNIP-211 — Snippet persistence and deterministic matcher]]
- [[planning/tickets/SNIP-212|SNIP-212 — Snippet UI and processing integration]]
- [[planning/tickets/PROMPT-221|PROMPT-221 — PromptProfile domain and persistence]]
- [[planning/tickets/PROMPT-222|PROMPT-222 — Manual Styles and Dictation Transform shortcuts]]
- [[planning/tickets/HIST-231|HIST-231 — FTS search and version UI]]

## Phase 3
- [[planning/tickets/NOTE-301|NOTE-301 — Notes and note versions]]
- [[planning/tickets/PAD-302|PAD-302 — Floating Markdown Scratchpad and autosave]]
- [[planning/tickets/PAD-303|PAD-303 — Scratchpad dictation and transforms]]
- [[planning/tickets/NOTE-304|NOTE-304 — Notes search and restore]]
- [[planning/tickets/PAD-305|PAD-305 — Transcript delivery routing and sinks]]

## Phase 4
- [[planning/tickets/KB-401|KB-401 — Knowledge DTO and export service]]
- [[planning/tickets/KB-402|KB-402 — Export outbox and idempotency]]
- [[planning/tickets/KB-403|KB-403 — Markdown/Second-Brain connector]]
- [[planning/tickets/KB-404|KB-404 — Knowledge settings and audio modes]]

## Phase 5
- [[planning/tickets/QUERY-501|QUERY-501 — Stable QueryService and DTOs]]
- [[planning/tickets/REST-511|REST-511 — REST companion server, auth and config]]
- [[planning/tickets/REST-512|REST-512 — REST v1 query endpoints]]
- [[planning/tickets/MCP-521|MCP-521 — MCP stdio server and query tools]]
- [[planning/tickets/MCP-522|MCP-522 — MCP client/schema/concurrency integration]]
- [[planning/tickets/INT-530|INT-530 — REST/MCP parity and security gate]]

- [[planning/tickets/DEP-510|DEP-510 — Connector dependency compatibility lock]]
- [[planning/tickets/INT-590|INT-590 — Local Integrations settings and launch configuration]]

## Phase 6
- [[planning/tickets/REL-601|REL-601 — Distribution identity and updater hardening]]
- [[planning/tickets/REL-602|REL-602 — Backup, upgrade and recovery validation]]
- [[planning/tickets/REL-603|REL-603 — Security, privacy and license review]]
- [[planning/tickets/REL-604|REL-604 — Full regression and performance report]]
- [[planning/tickets/UAT-605|UAT-605 — Seven-day daily-driver acceptance]]
