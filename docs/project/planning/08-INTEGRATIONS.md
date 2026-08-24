---
id: "integrations"
title: "Integration Architecture"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Integrations

[[planning/07-AGENT_SWARM|← Agent Swarm]] · [[planning/09-ROADMAP|Roadmap →]]

## Integration matrix

| Integration | MVP | Direction | Contract |
|---|---:|---|---|
| Markdown/Second Brain | Yes | push/export | Export DTO + outbox |
| REST Query API | Yes | local inbound reads | QueryService |
| MCP | Yes | local agent reads | QueryService |
| HTTP/RAG push connector | Future | outbound push | ExportService |
| Multi-device sync | Future | bidirectional | separate system |

## Markdown

Canonical Knowledge raw = `normalized_stt`.

Export is:
- stable-ID based;
- idempotent;
- version-aware;
- able to reference/copy/omit audio.

## REST

Separate `handy-rest` companion binary:
- Rust;
- loopback only;
- bearer token;
- read-only `/api/v1`;
- no wildcard CORS;
- no migrations.

## MCP

Separate `handy-mcp` companion binary:
- Rust official SDK;
- stdio;
- read-only;
- bounded query tools;
- no direct DB schema knowledge.

Exact connector contract:
[[orchestration/QUERY_REST_MCP_CONTRACT|Query / REST / MCP Contract]].
