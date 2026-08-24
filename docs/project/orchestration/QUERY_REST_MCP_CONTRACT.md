---
id: "query-rest-mcp"
title: "QueryService / REST / MCP Contract"
type: "architecture"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# QueryService / REST / MCP Contract

[[orchestration/QA_EVALUATOR_PROTOCOL|← QA]] · [[orchestration/MASTER_EXECUTION_CHECKLIST|Master Checklist →]] · [[orchestration/SECURITY_THREAT_MODEL|Security Threat Model]]

## Dependency direction
`Repositories → QueryService → stable DTOs → REST | MCP`. No adapter SQL. No REST↔MCP dependency.

## Data-directory resolution
Connector processes never guess an arbitrary database. Canonical launch contract:
1. GUI resolves the actual app data directory using the same shared resolver as storage/portable mode.
2. GUI produces connector launch config with an explicit absolute `--data-dir`.
3. MCP client config and REST launcher use that explicit path.
4. Manual CLI use must supply `--data-dir` unless a future shared resolver is proven equivalent.

## Compatibility
On open, read `app_meta.schema_version` and `app_meta.query_contract_version`, compare to supported range, open SQLite read-only, never run migrations, and fail closed with a clear compatibility error.

## Shared DTO contract
`QUERY_DTO_CONTRACT.json` is the canonical field/type/privacy contract for QueryService, REST and MCP. Transport adapters may not invent alternate response fields.

## Query contract
Default limit 50, maximum 200. Order `(created_at DESC, id DESC)`. Cursor is opaque, versioned base64url; clients must not parse it. Normal QueryService calls exclude soft-deleted captures/notes.

## REST
Separate `handy-rest.exe`: loopback `127.0.0.1` only, read-only, bearer token, no wildcard CORS, no token in URL, bounded request/response/concurrency, `/api/v1`.

Canonical wire contract: `REST_OPENAPI_CONTRACT.yaml`.

Unauthenticated `/health` may reveal only service/version/compatibility status, never content paths, token state or user data.

Token lifecycle: generate through GUI/core command; high entropy; persist in current-user app config with restrictive ACL where supported; rotate explicitly; never log token/Authorization header.

## MCP
Separate `handy-mcp.exe`: official Rust SDK pinned through [[orchestration/DEPENDENCY_LOCK_POLICY|Dependency Lock Policy]], stdio only, read-only, explicit `--data-dir`, no migrations and bounded results.

Canonical tool contract: `MCP_TOOL_CONTRACT.json`.

## Privacy defaults
`source_window` is omitted from external DTOs by default. `engine_raw` exposure through external adapters defaults off and requires an explicit local privacy setting.

## Adapter parity test
One seeded fixture DB must produce equivalent normalized domain results through QueryService, REST handlers and MCP handlers.

## Outbound HTTP/RAG
A future push connector belongs to `ExportService → HttpKnowledgeConnector`, not this read API.
