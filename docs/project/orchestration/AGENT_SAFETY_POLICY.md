---
id: "agent-safety"
title: "Agent Safety Policy"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Agent Safety Policy

[[orchestration/README|← Orchestration Index]] · [[orchestration/ORCHESTRATOR_RUNBOOK|Orchestrator Runbook]]

## Purpose
Autonomous agents may modify only bounded feature branches/worktrees and may never use destructive Git or data-repair shortcuts to escape a failing task.

## Hard prohibitions for Feature Agents
Never:
- `git push --force` / `--force-with-lease`;
- `git reset --hard`;
- `git clean -fd`, `-fdx` or equivalent destructive cleanup;
- delete `custom/main`, `phase/*`, tags, migration backups or recovery artifacts;
- merge directly to `custom/main`;
- bypass required PR/status checks;
- rewrite migration history;
- modify another agent's worktree;
- discard unknown/unowned changes;
- stage broad mixed trees with `git add -A`, `git add .` or `git add --all`;
- run destructive DB repair against the only user database;
- expose REST beyond loopback;
- add REST/MCP write capabilities.

## Safe Git behavior
Feature Agent works in one ticket worktree, stages only declared ticket-owned paths, commits focused changes, pushes only its ticket branch, and opens a draft PR into the phase branch. Integrator owns phase integration and only resolves mechanical conflicts; semantic conflicts are escalated.

## Automated retry budget
Default per ticket: 1 implementation attempt + 2 focused evaluator-driven fix attempts. A third failed cycle becomes `E2` escalation. The Orchestrator records attempt count, failing gate, last commit, repeated failure signature and approximate context/token use if available.

A repeated failure is not solved by widening scope.

## Data safety
Tests touching real user data use a disposable copy/fixture unless explicitly authorized. Migration tests require a verified backup and purge is never automatic in MVP.

See [[orchestration/SECURITY_THREAT_MODEL|Security Threat Model]].
