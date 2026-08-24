---
id: "research-summary"
title: "Research Summary"
type: "research"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Research Summary

[[planning/01-PRODUCT_VISION|← Product Vision]] · [[planning/03-ARCHITECTURE|Architecture →]]

## Handy baseline

The frozen repository baseline is documented in [[research/HANDY_BASELINE|Handy Baseline]].

Key verified facts at planning time:

| Fact | Planning implication |
|---|---|
| `main` source declares version `0.9.5` | Version baseline is 0.9.5 |
| pinned `main` SHA is `0e503672…` | Implementation is reproducible |
| current repo uses Tauri 2 + Rust + React/TS | no framework rewrite |
| history uses SQLite/rusqlite + WAV files | evolve persistence, do not replace capture stack |
| post-processing/LLM client already exists | reuse provider transport |
| generated Specta TypeScript bindings exist | Integrator owns regeneration |
| Rust tests, Playwright, ESLint/Prettier CI exist | extend existing QA harness |
| upstream has an AI-oriented `AGENTS.md` | fork is agent-friendly |
| upstream contribution policy is feature-freeze oriented | upstream PRs are optional bonuses |

## Long-horizon agents

Research summarized in [[research/LONG_HORIZON_AGENT_RESEARCH|Long-Horizon Agent Research]] supports:
- small bounded implementation chunks;
- persistent progress artifacts;
- role separation between planning, implementation and evaluation;
- stable interfaces instead of conversational memory.

## Git/worktrees

Research summarized in [[research/GIT_WORKTREE_RESEARCH|Git Worktree Research]] supports one checkout/worktree per active branch and protected integration flow.

## Connectors

Research summarized in [[research/CONNECTOR_RESEARCH|Connector Research]] supports:
- a shared Rust QueryService;
- `axum` for a thin localhost REST layer;
- official Rust MCP SDK (`rmcp`) for stdio MCP;
- connector dependencies isolated from domain business logic.
