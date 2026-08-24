---
id: "git-strategy"
title: "Git Worktree, Branch & PR Strategy"
type: "orchestration"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Git Worktree, Branch & PR Strategy

[[orchestration/FILE_OWNERSHIP|← File Ownership]] · [[orchestration/ORCHESTRATOR_RUNBOOK|Orchestrator Runbook →]]

## Workspace

```text
<workspace>/
├── custom-handy-final-starter-kit/
├── repo/
└── worktrees/
    ├── phase-01-data/
    ├── DATA-101/
    └── ...
```

## Branch hierarchy

```text
upstream/main      fork/main (informational)
      \              /
       custom/main   # pinned + custom green integration base
          └── phase/01-data-core
    ├── feat/DATA-101-storage
    ├── feat/STT-103-engine-raw
    └── fix/PH1-INT-001
```

Each ticket branch gets one worktree.

## Ticket flow

```text
phase branch
→ ticket branch/worktree
→ code + tests
→ focused commits
→ draft PR ticket → phase
→ independent evaluator
→ Integrator merge
```

## Phase flow

```text
phase branch
→ integrated gate
→ phase gate report
→ PR phase → custom/main
→ merge only when green
```

## Protection

Recommended for fork:
- PR required to `custom/main`;
- required status checks;
- conversation resolution;
- prevent force push;
- linear history where practical.

Protect `phase/*` as well if the GitHub plan/permissions support the desired rule without blocking Integrator workflow.

## Worktree safety

- no two agents share a worktree;
- no ticket agent works directly in the canonical integration checkout;
- do not force-remove unresolved worktrees;
- stage only ticket-owned paths;
- never use broad `git add -A` for mixed/shared work.

## Upstream updates

Upstream sync is a separate branch/task.
Do not mix upstream rebases into feature PRs.


## Canonical branch invariant
`custom/main` is the only branch from which new product phases are created. Fork `main` is never assumed to equal the planning baseline and may track upstream separately. Feature Agents cannot merge or force-update `custom/main`.
