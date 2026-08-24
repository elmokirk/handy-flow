---
id: "tpl-task"
title: "Task Packet Template"
type: "template"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Task Packet — `<TICKET>`

**Catalog entry:** `TICKET_CATALOG.json#<TICKET>`  
**Owner role:**  
**Attempt:** 1/3  
**Branch:**  
**Worktree:**  
**Base phase branch:**  
**Gate:**  
**QA profile:**  

## Goal
One bounded behavior outcome copied from the accepted ticket.

## Dependencies
Only `DONE` dependencies from the catalog may appear here.

## Exact allowed paths
```text
...
```

## Forbidden/shared paths
```text
...
```

## Frozen contracts
List only the contract/ADR files needed by this ticket.

## Relevant existing source files
List exact files to inspect; do not instruct a whole-repo reread.

## Scope
- [ ] ...

## Out of scope
- architecture redesign;
- manifest/shared-file changes not explicitly owned;
- unrelated cleanup;
- roadmap features.

## Tasks
- [ ] read Agent Safety Policy;
- [ ] verify branch/worktree + allowlist;
- [ ] write failing test/fixture where behavior is testable;
- [ ] implement minimum compliant change;
- [ ] refactor without widening scope;
- [ ] run focused QA profile;
- [ ] self-review diff against contracts;
- [ ] update RUN_STATE frontmatter + checklist;
- [ ] open/update draft PR.

## Acceptance
- [ ] ...

## Mandatory escalation triggers
- public DTO/schema/permission change;
- new dependency;
- required path outside allowlist;
- destructive operation not already approved;
- architecture contradicts pinned source;
- third failed implementation/fix cycle.

## Completion
Return commit, PR, changed files, test evidence, integration requests, deviations and escalation links. Do not start another ticket.
