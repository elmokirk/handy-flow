---
id: "roadmap"
title: "Roadmap"
type: "roadmap"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Roadmap

[[planning/08-INTEGRATIONS|← Integrations]] · [[planning/10-REDTEAM_GAPS_BLINDSPOTS|Risk Review →]]

## P0/P1 — MVP release scope

- durable audio/history;
- raw provenance;
- Dictionary;
- deterministic Snippets;
- manual Styles;
- Dictation Transforms;
- local OpenAI-compatible LLM;
- History/search;
- Markdown Scratchpad;
- Markdown Knowledge export;
- QueryService;
- read-only REST;
- read-only MCP;
- release hardening.

## P2 — Productivity

- Selected-Text Transforms;
- automatic per-app Styles;
- richer Notes hub;
- tags/favorites;
- dynamic Snippet variables;
- wispr-like idle bubble (owner request, 2026-08-31): click-to-record floating button when idle; complements the recording overlay; requires tray/overlay coexistence and an E2 UI ADR before implementation.
- richer import/export.

## P2 — Rich Text

Explicitly deferred:
- rich-text Scratchpad;
- rich clipboard HTML;
- images/attachments;
- richer formatting toolbar.

Plain Markdown remains canonical unless a future ADR changes it.

## P2/P3 — Knowledge/RAG

- outbound HTTP Knowledge connector;
- semantic search;
- external embeddings;
- automatic tags/summaries as representations;
- multimodal/audio indexing.

## P3 — Sync

Separate distributed-system project:
- device identity;
- conflict resolution;
- encryption;
- tombstones;
- attachment/audio sync;
- auth.

## P3 — Write-enabled integrations

Potential:
- REST writes;
- MCP create/update tools.

Requires a new security/permission ADR.

## Commercialization track

If distributed beyond personal use:
- own branding/icons;
- new bundle identifier;
- signing;
- own updater keys/endpoints;
- dependency/model license audit;
- privacy/support/update policy.
