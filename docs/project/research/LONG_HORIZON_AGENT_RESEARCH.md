---
id: "long-horizon-research"
title: "Long-Horizon Agent Research"
type: "research"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Long-Horizon Agent Research

[[planning/02-RESEARCH|← Research Summary]]

## Sources

- Anthropic — Effective harnesses for long-running agents  
  https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
- Anthropic — Harness design for long-running application development  
  https://www.anthropic.com/engineering/harness-design-long-running-apps
- Anthropic — Scaling Managed Agents  
  https://www.anthropic.com/engineering/managed-agents
- Anthropic — Building a C compiler with a team of parallel agents  
  https://www.anthropic.com/engineering/building-c-compiler

## Applied principles

1. Break long work into tractable bounded chunks.
2. Persist state across context/session boundaries.
3. Separate planning, generation and evaluation.
4. Use stable interfaces instead of conversational assumptions.
5. Let multiple agents parallelize only where contracts permit.
6. Keep harness complexity load-bearing; do not orchestrate for its own sake.

## Project translation

- Task Packet = bounded work specification.
- RUN_STATE = persistent long-horizon memory.
- Orchestrator = scheduler/state manager.
- Implementer = code generator.
- Evaluator = independent verifier.
- Phase Gate = convergence barrier.
- Escalation = architecture uncertainty made explicit.
