---
id: "architecture-freeze"
title: "Architecture Freeze"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Architecture Freeze

[[orchestration/README|← Orchestration Index]] · [[orchestration/IMPLEMENTATION_CONTRACTS|Implementation Contracts →]]

## Rule

**Coding agents write code; they do not redesign the project.**

Frozen:
- technology stack;
- canonical data semantics;
- schema concepts;
- migration ownership;
- QueryService boundary;
- REST/MCP permissions;
- connector transport;
- MVP scope;
- retention policy;
- Prompt Profile concept;
- Scratchpad storage model;
- Git/ownership model.

## Local decisions agents may make

- private helper names;
- private implementation structs;
- local component extraction;
- test organization;
- equivalent internal refactors;
- local performance improvements that preserve contracts.

## Must escalate

- public DTO change;
- new/changed DB table/column outside assigned migration;
- new dependency;
- new endpoint/tool;
- auth change;
- write permission;
- retention/deletion change;
- network binding change;
- canonical raw meaning;
- global shortcut semantics beyond ticket;
- telemetry/encryption/model bundling;
- MVP scope.

## Escalation levels

- L0: private implementation choice — agent decides.
- L1: contract-compatible change — document.
- L2: stable contract change — escalate.
- L3: data/security/migration — owner decision.
- L4: product/release scope — owner decision.

See [[orchestration/ESCALATION_PROTOCOL|Escalation Protocol]].


## Hardening v1.1 frozen decisions
- `custom/main` is the canonical custom integration base.
- planning/control-plane artifacts are Git-versioned under `docs/project/`.
- Delete for Captures/Notes means Trash; Purge is separate and explicit.
- Dictionary correction is inside deterministic `normalized_stt`; Snippets are derived.
- Snippets are single-pass, longest-match-first, non-recursive.
- `app_meta` owns schema/query compatibility.
- normal logging excludes private content/secrets.
- source window title is opt-in.
- connector data directory is explicit; connectors do not migrate.
- Feature Agents obey Agent Safety Policy and retry budget.
