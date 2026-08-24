---
id: "project-context"
title: "Custom Handy — Project Context"
type: "context"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Project Context

[[START_HERE|← Start Here]] · [[planning/README|Planning Index]] · [[orchestration/README|Orchestration Index]]

## Mission

Build a personal, local-first fork of Handy that can replace the owner's daily Wispr Flow workflows while becoming a durable local capture layer for a Second Brain, RAG pipeline and coding agents.

This project starts from the **source repository**, not from an installed Handy binary.

## Why Handy

Handy already provides the difficult native foundations:

- Tauri desktop application;
- microphone capture;
- VAD;
- local STT model management/inference;
- global shortcuts;
- overlay;
- clipboard/paste behavior;
- history;
- WAV recordings;
- post-processing via LLM providers;
- React/TypeScript UI;
- Rust manager/command architecture.

The fork adds a product/knowledge layer without replacing those foundations.

## Product outcome

At MVP completion the user can:

1. dictate locally into Windows applications;
2. recover the original audio and transcript provenance;
3. maintain a Dictionary;
4. expand spoken Snippets;
5. choose Styles and Dictation Transforms;
6. use a local OpenAI-compatible LLM endpoint;
7. search History and derived transcript versions;
8. dictate into a Markdown/Plain-Text Scratchpad;
9. export raw transcriptions into Markdown/Second-Brain storage;
10. query local data through a read-only REST API;
11. query local data through a read-only MCP server from local agents.

## Non-goals for MVP

- pixel-for-pixel Wispr clone;
- Rich Text;
- Selected-Text Transforms;
- automatic per-app Style routing;
- bundled LLM;
- bundled embeddings/vector DB;
- remote REST exposure;
- REST writes;
- MCP writes;
- multi-device sync;
- team/collaboration features.

These are preserved in [[planning/09-ROADMAP|Roadmap]].

## Execution philosophy

This project uses an architecture-first agent harness:

```text
Architecture Session
→ frozen contracts
→ Orchestrator
→ bounded ticket agent
→ independent evaluator
→ Integrator
→ QA Gate
```

A sub-agent may implement local code choices but cannot modify product/data/security architecture without an escalation.

## Source of truth hierarchy

When documents conflict:

1. accepted ADR in `planning/adr/`;
2. machine execution metadata in `TICKET_CATALOG.json` / `FEATURE_MATRIX.json`;
3. [[orchestration/ARCHITECTURE_FREEZE|Architecture Freeze]];
4. [[orchestration/IMPLEMENTATION_CONTRACTS|Implementation Contracts]];
5. [[planning/03-ARCHITECTURE|Architecture]];
6. ticket rationale/acceptance narrative;
7. roadmap/research narrative.

A newer accepted ADR may supersede an older decision and must say so explicitly.

## Update policy

If implementation begins weeks later:

- first run the baseline freshness check;
- do not silently rebase to latest Handy;
- choose either pinned baseline implementation or an explicit baseline refresh;
- baseline refresh is its own planning/integration task with full regression.

See [[research/HANDY_BASELINE|Handy Baseline]].


## Machine execution source of truth
`TICKET_CATALOG.json` is canonical for ticket dependencies, exact allowed paths, gate, QA profile and retry budget. `FEATURE_MATRIX.json` maps MVP capabilities to tickets/gates. Human Markdown explains intent; it does not override the machine catalog for orchestration metadata.

Canonical custom branch: `custom/main`.
