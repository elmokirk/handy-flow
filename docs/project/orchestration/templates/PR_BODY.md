---
id: "tpl-pr"
title: "Pull Request Template"
type: "template"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# PR — `<TICKET>`

## Contract
- Ticket: `<TICKET>`
- Catalog: `TICKET_CATALOG.json#<TICKET>`
- Attempt: `<n>/3`
- Base: `phase/<phase>`

## Goal
...

## Scope implemented
- ...

## Out of scope preserved
- ...

## Changed files
- ...

## Tests / evidence
- [ ] required QA profile completed;
- [ ] lint/format/build as applicable;
- [ ] unit/integration tests;
- [ ] relevant native/E2E evidence or explicit N/A rationale.

## Architecture / safety compliance
- [ ] every changed path is catalog-allowed or an explicit Integrator change;
- [ ] no unapproved contract/DTO change;
- [ ] no unowned migration;
- [ ] no connector direct SQL/write capability;
- [ ] no destructive Git operation;
- [ ] source/canonical data semantics preserved;
- [ ] no private body/secret logging introduced.

## Integration requests
- none / ...

## Escalations
- none / links

## AI assistance
AI-assisted: Yes  
Agent/tool + extent: ...
