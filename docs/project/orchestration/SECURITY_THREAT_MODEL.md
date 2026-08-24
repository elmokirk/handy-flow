---
id: "security-threat-model"
title: "Security & Privacy Threat Model"
type: "security"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Security & Privacy Threat Model

[[planning/10-REDTEAM_GAPS_BLINDSPOTS|← Risk Review]] · [[orchestration/QUERY_REST_MCP_CONTRACT|Connector Contract]]

## Protected assets
Raw audio, transcript/note bodies, prompts, provider keys, REST token and source metadata.

## Trust boundary
Trusted: current Windows user and intentionally launched local project processes. Not automatically trusted: arbitrary browser pages, arbitrary localhost processes, network peers, unconfigured MCP clients, logs/diagnostic uploads.

Protection against malware already running as the same Windows user is outside MVP; full-disk encryption/BitLocker is recommended.

## REST controls
Loopback only; high-entropy bearer token; no token in URL; no wildcard CORS; read-only; bounded requests/responses/concurrency; token rotation; restrictive token-file ACL where supported.

## MCP controls
Stdio only; explicit executable/data-dir configuration; read-only bounded tools; no path disclosure by default; no transform/write execution.

## Logging policy from Phase 0
Normal Info/Warn/Error logs must not contain transcript/note/prompt bodies, provider secrets, REST token or Authorization headers. Debug content logging is opt-in and disabled by default.

## Metadata minimization
`source_app` may be stored. `source_window` is disabled by default, opt-in only and omitted from REST/MCP unless explicitly enabled.
