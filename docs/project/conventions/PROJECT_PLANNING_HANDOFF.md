---
id: "planning-handoff"
title: "Reusable Project Planning Handoff"
type: "convention"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
scope: "generic-reusable"
---


# Project Planning Handoff

## 0. How to use

Attach **this single file** to a new chat or coding agent. Then give only the project-specific context.

Examples:

```text
Create a planning pack for this project using the attached Planning Handoff.
Research the current ecosystem first. Optimize for an MVP.
```

```text
Update the existing project plan using the attached Planning Handoff.
Focus on Phase 2 and preserve all accepted decisions.
```

```text
Implement Phase 1 based on the attached Planning Handoff and the existing project plan.
Do not continue to the next phase until its QA gate passes.
```

```text
Red-team the current architecture and update the plan, tickets, ADRs and open questions accordingly.
```

The attached file is the **default planning convention** unless the user explicitly overrides it.

---

# 1. Core operating contract

The agent MUST:

1. understand the project goal before designing the solution;
2. research current facts when external/current information matters;
3. separate facts, assumptions, recommendations and decisions;
4. prioritize foundations before optional features;
5. define architecture before broad implementation;
6. create explicit phase gates;
7. make every important decision traceable;
8. use cross-linked Markdown as the default planning format;
9. keep tickets small enough for independent agent execution;
10. never move to the next phase while a blocking QA gate is red;
11. record unresolved user decisions instead of silently inventing them;
12. update affected documents when architecture or scope changes;
13. prefer simple, stable contracts over clever abstractions;
14. treat data integrity, migration safety, security and recoverability as first-class design concerns;
15. preserve a clean separation between MVP, P1, P2 and future ideas.

The agent SHOULD optimize for:

- clarity;
- low coupling;
- deterministic behavior;
- testability;
- reversible decisions;
- agent-friendly file ownership;
- upstream/open-source contribution potential where relevant;
- minimal duplicated documentation;
- low token/context overhead.

---

# 2. Work modes

Determine the mode from the user request.

## MODE A — PLAN

Use when creating a new project plan.

Output the complete planning pack defined in Section 5.

## MODE B — UPDATE

Use when a planning pack already exists.

Rules:

- preserve accepted ADRs unless explicitly reopened;
- update only documents affected by the change;
- add or revise tickets;
- update roadmap and risk analysis;
- update open questions;
- never silently invalidate previous decisions.

## MODE C — IMPLEMENT

Use when asked to build a phase/ticket.

Before code:

1. read `README.md`;
2. read Architecture;
3. read Data/Persistence if relevant;
4. read QA gates;
5. read Agent Ownership;
6. read the target ticket;
7. read relevant ADRs;
8. inspect current code/research required dependencies.

During implementation:

- follow ticket file ownership;
- use TDD where behavior is testable;
- keep scope inside the ticket;
- update docs when contracts change;
- stop at the phase gate.

After implementation:

- run required tests;
- report changed files;
- report QA evidence;
- report deviations;
- do not start the next phase automatically unless instructed.

## MODE D — REVIEW / RED TEAM

Use when asked to challenge the plan or implementation.

Review:

- architecture;
- sequencing;
- dependencies;
- data model;
- migration/recovery;
- concurrency;
- performance;
- security/privacy;
- testing;
- operational failure modes;
- agent parallelization;
- external integrations;
- future extensibility;
- open-source/upstream compatibility.

Then update the relevant planning artifacts.

---

# 3. Research protocol

Research is required when accuracy depends on current or external facts.

Examples:

- current library/framework behavior;
- repository structure;
- licenses;
- pricing;
- APIs;
- platform constraints;
- hardware support;
- model availability;
- security advisories;
- competing product behavior;
- standards;
- deployment requirements.

## Research rules

1. Prefer primary sources:
   - official docs;
   - source repositories;
   - specifications;
   - vendor documentation;
   - release notes.
2. Use community sources only for:
   - practical experience;
   - known edge cases;
   - sentiment;
   - undocumented behavior.
3. Record:
   - fact;
   - source;
   - date/version if relevant;
   - implication for architecture/scope.
4. Distinguish:
   - verified fact;
   - inference;
   - recommendation.
5. Re-check facts that materially affect:
   - licensing;
   - security;
   - supported platforms;
   - APIs;
   - model/runtime compatibility;
   - release strategy.
6. Research must feed decisions/tickets. Do not collect sources without architectural consequence.

## Research output

For substantial research, create:

```text
research/
├── README.md
├── <topic>.md
└── ...
```

Each research note uses:

```markdown
# Topic

## Question
What are we validating?

## Findings
| Finding | Evidence | Confidence | Impact |
|---|---|---:|---|

## Recommendation
What should the project do?

## Sources
- ...
```

If research is small, summarize it in the relevant architecture/ADR instead.

---

# 4. Planning principles

## 4.1 Foundation first

Prioritize in this order unless the project requires otherwise:

```text
Baseline
→ Architecture
→ Data/State contracts
→ Migration/Recovery
→ Core workflow
→ Product features
→ Integrations
→ Automation/Agents
→ Polish
→ Expansion
```

## 4.2 MVP discipline

Classify every feature:

| Priority | Meaning |
|---|---|
| P0 | required for first usable core |
| P1 | required before intended MVP release |
| P2 | valuable expansion |
| P3 | long-term / separate project |

Do not call everything MVP.

## 4.3 Hard phase gates

Every phase ends with an explicit gate.

A phase can advance only when:

- blocking tickets are done;
- tests pass;
- no blocking defects remain;
- migration/data integrity checks pass if relevant;
- user UAT passes where required;
- performance/security thresholds pass if defined.

## 4.4 Append-only thinking

Prefer immutable history/versioning when information may need:

- auditability;
- recovery;
- comparison;
- retries;
- AI derivations;
- sync;
- external indexing.

Do not overwrite source data when a derived version can be stored.

## 4.5 Stable contracts

External integrations consume:

```text
Domain Service / Query Service / Export DTO
```

They SHOULD NOT depend directly on:

```text
internal tables
UI state
filesystem assumptions
private implementation details
```

---

# 5. Standard planning pack

Default project structure:

```text
planning/
├── README.md
├── 01-product-vision.md
├── 02-research.md
├── 03-architecture.md
├── 04-data-and-persistence.md
├── 05-delivery-plan.md
├── 06-qa-and-quality-gates.md
├── 07-agent-swarm-and-ownership.md
├── 08-integrations.md
├── 09-roadmap.md
├── 10-red-team-gap-blindspots.md
├── 11-decisions-and-open-questions.md
├── adr/
│   ├── README.md
│   └── ADR-###-<slug>.md
├── tickets/
│   ├── README.md
│   ├── phase-0-<slug>.md
│   ├── phase-1-<slug>.md
│   └── ...
└── research/
    ├── README.md
    └── ...
```

Not every project needs every file. Remove irrelevant files rather than filling them with noise.

---

# 6. Document responsibilities

## `README.md`

Purpose: project entry point.

Must contain:

- current status;
- project goal;
- fixed decisions summary;
- current phase;
- release boundary;
- document map with cross-links;
- short source/research alignment note;
- definition of success.

Do not duplicate full architecture here.

## `01-product-vision.md`

Contains:

- problem;
- target user;
- why the project exists;
- core journeys;
- non-goals;
- product principles;
- success metrics;
- MVP/P1 boundary.

## `02-research.md`

Contains only research that materially shapes the plan.

Use:

| Topic | Finding | Impact | Source |
|---|---|---|---|

Detailed research may live under `/research`.

## `03-architecture.md`

Contains:

- target architecture;
- system boundaries;
- domains/modules;
- dependencies;
- data flow;
- processing pipeline;
- public/stable contracts;
- platform-specific adapters;
- external integration boundaries;
- architectural invariants.

Prefer diagrams in Markdown code blocks.

## `04-data-and-persistence.md`

Use when the project stores durable state.

Contains:

- canonical entities;
- source vs derived data;
- versioning;
- migrations;
- transactions;
- concurrency;
- indexes/search;
- retention;
- backup;
- recovery;
- corruption/orphan handling;
- deletion semantics.

If data integrity matters, this document is mandatory.

## `05-delivery-plan.md`

Contains:

- phases;
- objective per phase;
- dependencies;
- parallel workstreams;
- hard gate per phase;
- internal RC/release boundary.

Use a summary table first.

## `06-qa-and-quality-gates.md`

Contains:

- test pyramid;
- static checks;
- TDD rules;
- unit tests;
- integration tests;
- E2E/native tests;
- performance gates;
- security tests;
- chaos/failure tests;
- UAT;
- defect severity policy;
- phase gates.

Quality gates must be measurable.

## `07-agent-swarm-and-ownership.md`

Contains:

- agent roles;
- exclusive shared files;
- migration ownership;
- generated file rules;
- branch strategy;
- ticket execution protocol;
- review flow;
- definition of done;
- anti-patterns.

This file is mandatory for parallel agent work.

## `08-integrations.md`

Use for:

- APIs;
- MCP;
- plugins;
- external services;
- knowledge bases;
- sync;
- webhooks;
- imports/exports.

For each integration specify:

- boundary;
- contract;
- auth/security;
- retries/idempotency;
- versioning;
- failure handling;
- privacy implications.

## `09-roadmap.md`

Separate:

```text
P0 Core
P1 Release-required
P2 Expansion
P3 Long-term
```

Future ideas belong here instead of polluting MVP tickets.

## `10-red-team-gap-blindspots.md`

Must contain three sections.

### Red Team
“How can this fail catastrophically?”

### Gap Analysis
“What capability or operational requirement is missing?”

### Blindspot Analysis
“What are we assuming without enough evidence?”

Use:

| Area | Finding | Severity | Impact | Mitigation | Status |
|---|---|---:|---|---|---|

Any issue that can be resolved without user input SHOULD be converted into:

- an architecture rule;
- a ticket;
- a QA gate;
- an ADR.

Do not leave obvious mitigations as mere observations.

## `11-decisions-and-open-questions.md`

Two sections:

### Accepted decisions

| ID | Decision | Reason | Status |
|---|---|---|---|

### Open decisions requiring user input

| Decision | Needed by | Options | Recommendation |
|---|---|---|---|

Only ask the user about decisions that materially affect:

- irreversible architecture;
- scope/release;
- data semantics;
- permissions/security;
- external dependencies;
- licensing/business model;
- destructive behavior.

Do not ask for trivial implementation choices.

---

# 7. Cross-linking convention

The planning pack must be navigable without search.

## Rules

Every major document starts with:

```markdown
[← Previous](./previous.md) · [Index](./README.md) · [Next →](./next.md)
```

`README.md` contains the canonical document map.

Tickets link to:

- parent phase;
- architecture section;
- QA gate;
- relevant ADRs;
- dependencies.

ADRs link back to:

- decisions file;
- affected architecture/tickets where helpful.

Research notes link to:

- decisions/tickets influenced by the research.

## Link quality rule

Never create a link without a reason.

Cross-link concepts and dependencies, not every repeated noun.

---

# 8. ADR convention

Create an ADR for decisions that are:

- architecture-shaping;
- expensive to reverse;
- likely to be questioned later;
- relevant to multiple tickets;
- important to external integrations/security/data semantics.

Format:

```markdown
# ADR-### — Title

**Status:** Proposed | Accepted | Superseded

## Context
Why is a decision required?

## Decision
What was chosen?

## Alternatives
What was rejected and why?

## Consequences
Positive and negative effects.

## Follow-up
Tickets/docs affected.
```

Never use ADRs for minor code-style choices.

---

# 9. Ticket convention

Tickets should be small enough for one primary agent.

Default format:

```markdown
## ABC-123 — Ticket title

**Owner:** <role>
**Priority:** P0 | P1 | P2 | P3
**Dependencies:** <ticket IDs>
**Files/Ownership:** <paths or domain>

### Goal
One-sentence outcome.

### Scope
- ...

### Out of scope
- ...

### Tasks
- [ ] ...

### Acceptance criteria
- [ ] ...

### Tests
- [ ] unit
- [ ] integration
- [ ] E2E/native where relevant
- [ ] regression fixture where relevant

### Risks
- ...

### Upstream/Open-source potential
High | Medium | Low | N/A
```

## Ticket sizing

Split when a ticket:

- touches multiple domains;
- requires multiple agents;
- changes schema and UI and external integration together;
- has more than one independently testable outcome;
- risks large shared-file conflicts.

---

# 10. Agent-swarm convention

## Ownership

There MUST be:

- one Integrator;
- one migration/schema owner if persistent storage exists;
- exclusive ownership for shared/generated files.

Feature agents own isolated domains.

## Shared files

Examples:

```text
app entrypoints
root routing
global navigation
dependency manifests
generated bindings
global i18n indexes
CI workflows
migration registry
```

Only the designated owner edits them.

## Parallelization rule

Parallelize **within a stable contract**, not while the contract itself is changing.

Good:

```text
Dictionary UI
Snippets engine
Prompt UI
```

after persistence/interfaces are fixed.

Bad:

```text
3 agents simultaneously redesign database schema.
```

## Review chain

```text
Feature Agent
→ Self Review
→ QA/Test Review
→ Peer Review
→ Integrator
→ Phase Gate
```

---

# 11. QA convention

## Minimum gate

Every code phase SHOULD include:

```text
format
lint
type/compile
unit tests
integration tests
build
relevant E2E/native smoke
```

## Additional gates by risk

### Persistence
- migration fixtures;
- backup/restore;
- crash recovery;
- integrity checks;
- concurrent access.

### External integrations
- timeout;
- 4xx/5xx;
- invalid payload;
- retry;
- duplicate delivery;
- version mismatch;
- offline mode.

### AI
- deterministic mock provider;
- timeout;
- malformed output;
- empty output;
- model metadata;
- source preservation.

### Security
- permissions;
- secret leakage;
- injection;
- unsafe file access;
- unbounded results;
- destructive operations.

### Performance
Establish baseline before optimization-sensitive work.

Track:

- startup;
- memory;
- CPU/GPU;
- latency;
- throughput;
- storage growth.

---

# 12. Red Team protocol

Before finalizing a major plan or phase, explicitly attack it.

Ask:

1. Where can data be lost?
2. Where can state become inconsistent?
3. What happens if the process crashes at each transition?
4. What if a dependency is unavailable?
5. What if input is malicious or malformed?
6. What if two processes/agents act concurrently?
7. What assumptions depend on one platform?
8. What happens during upgrades/migrations?
9. How does rollback/recovery work?
10. What silently grows without bounds?
11. What external contract could change?
12. What could leak private data?
13. What part cannot be tested realistically?
14. What will break first at 10× scale?
15. What feature creates disproportionate maintenance cost?

Convert mitigations into plan changes immediately where possible.

---

# 13. Gap analysis protocol

Compare the current plan against these capability classes:

| Class | Questions |
|---|---|
| Product | Does the core user journey actually close? |
| Architecture | Are boundaries/contracts explicit? |
| Data | Are source, derived, migration and recovery semantics defined? |
| Security | Are permissions, secrets, destructive actions controlled? |
| Performance | Is there a baseline and regression threshold? |
| Operations | Install, upgrade, backup, logs, recovery? |
| QA | Can critical behavior be proven? |
| Integration | Are retries/versioning/idempotency defined? |
| Agent execution | Can work be parallelized safely? |
| Roadmap | Are deferred features preserved without polluting MVP? |
| Licensing | Can dependencies/assets/models be used/distributed? |

Every important gap becomes:

```text
ticket
or ADR
or QA gate
or explicit accepted risk
```

---

# 14. Blindspot protocol

Blindspots are assumptions not yet validated.

Classify each:

```text
Verified
Likely
Unknown
Risky
```

For `Unknown` or `Risky`:

- research;
- create spike ticket;
- add test;
- ask user only if it changes product/architecture direction.

Examples:

- “This API supports our required mode.”
- “Windows clipboard works across all target apps.”
- “SQLite concurrency is sufficient.”
- “The model license allows redistribution.”
- “Users do not need import/migration.”
- “This data can safely be deleted.”

---

# 15. Decision policy

The agent SHOULD make technical defaults itself when they are:

- reversible;
- industry-standard;
- low-risk;
- non-product-specific.

The agent MUST ask the user when the choice affects:

- product behavior;
- public release scope;
- canonical data semantics;
- irreversible migrations;
- security/permissions;
- retention/deletion;
- network/cloud dependence;
- licensing/commercial distribution;
- telemetry;
- major cost.

Collect user decisions and ask them together at the end unless one blocks all progress.

---

# 16. Context/token optimization

Use these rules to keep planning useful in long-running projects.

## Do

- one source of truth per concept;
- link instead of duplicate;
- summarize tables before long prose;
- keep tickets executable;
- separate accepted decisions from discussion history;
- store future ideas only in Roadmap;
- keep research evidence close to the decision it affects;
- update documents incrementally.

## Do not

- copy the same architecture into multiple files;
- repeat every ticket in the roadmap;
- paste full external documentation;
- create one huge “master plan” file;
- embed implementation details that belong in code;
- generate dozens of empty template files.

---

# 17. Implementation handoff checklist

When the plan is ready for a Coding Agent, the minimum context is:

```text
README.md
Architecture
Data/Persistence (if relevant)
Delivery Plan
QA Gates
Agent Ownership
target Phase/Ticket
relevant ADRs
```

The implementation agent should be able to answer:

1. What am I building?
2. Why?
3. What files may I touch?
4. What must I not change?
5. What are my dependencies?
6. What proves completion?
7. What blocks the next phase?
8. Which decisions are already fixed?

If any answer is unclear, the planning pack is incomplete.

---

# 18. Final planning quality checklist

Before declaring the planning pack ready:

```text
[ ] Project goal and non-goals are explicit
[ ] Current research is sufficient
[ ] Architecture boundaries are clear
[ ] Canonical data semantics are defined
[ ] Migrations/recovery are addressed if needed
[ ] MVP/P1/P2/P3 are separated
[ ] Every phase has a hard QA gate
[ ] Tickets have owners and file ownership
[ ] Shared/generated files have exclusive owners
[ ] ADRs capture expensive decisions
[ ] Red Team analysis completed
[ ] Gap analysis completed
[ ] Blindspot analysis completed
[ ] Resolvable risks converted into actions
[ ] Remaining user decisions collected
[ ] Cross-links validated
[ ] No duplicated source of truth
[ ] Implementation handoff is executable
```

---

# 19. Default output behavior for the agent

When this file is attached and the user asks for planning:

1. briefly restate the interpreted objective;
2. research relevant current facts;
3. create/update the planning pack;
4. run Red Team, Gap and Blindspot analysis;
5. directly harden the plan using best practices;
6. validate cross-links;
7. report:
   - files created/updated;
   - key architecture decisions;
   - major risks resolved;
   - remaining user decisions;
   - recommended next action.

When the user asks for implementation:

1. do not redesign the project unless a blocker proves the plan invalid;
2. implement only the requested phase/tickets;
3. run all required tests;
4. stop at the gate;
5. report evidence and blockers.

---

# 20. Default philosophy

The planning system optimizes for:

> **A small number of well-linked, high-signal documents that make architecture, decisions, tickets, QA and risk explicit enough that a human or agent can continue the project without reconstructing intent from chat history.**

That is the success criterion for this handoff.
