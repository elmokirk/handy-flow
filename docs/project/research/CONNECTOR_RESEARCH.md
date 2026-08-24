---
id: "connector-research"
title: "REST & MCP Research"
type: "research"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# REST & MCP Research

[[planning/02-RESEARCH|← Research Summary]] · [[orchestration/QUERY_REST_MCP_CONTRACT|Connector Contract]]

## MCP

Official Rust SDK:
https://github.com/modelcontextprotocol/rust-sdk

Use:
- shared Rust runtime;
- stdio server;
- typed tool schemas;
- no second Python/Node backend.

## REST

Axum:
https://docs.rs/axum/

Use:
- thin router/handler layer;
- existing Tokio ecosystem;
- loopback-only local server;
- read-only QueryService adapter.

## Security posture

Localhost is not treated as automatically trusted.

REST requires:
- bearer token;
- loopback bind;
- restrictive CORS;
- bounded queries.

MCP uses stdio and exposes read-only tools only.

## Architecture implication

REST and MCP are transports, not application layers.
All filtering/search semantics live in QueryService.


## Observed versions during hardening — 2026-08-21
- official Rust MCP SDK latest observed: `rmcp-v3.1.2`;
- Axum docs latest observed: `0.8.9`.

These are research anchors only. Exact pins are selected through [[orchestration/DEPENDENCY_LOCK_POLICY|Dependency Lock Policy]] immediately before Phase 5.
