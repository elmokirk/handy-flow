---
id: "scripts-readme"
title: "Starter Kit Scripts"
type: "guide"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Starter Kit Scripts

[[START_HERE|← Start Here]]

## `bootstrap.ps1`

Creates the sibling workspace layout, clones the fork/upstream, adds remotes and checks out the frozen baseline on a bootstrap branch.

It deliberately does **not** implement product code.

Examples:

```powershell
.\scripts\bootstrap.ps1 -Workspace "D:\dev\custom-handy-workspace" -ForkRepo "YOUR_GITHUB_USER/Handy"
```

Or, with authenticated GitHub CLI:

```powershell
.\scripts\bootstrap.ps1 -Workspace "D:\dev\custom-handy-workspace" -CreateFork
```

If no fork is provided, the script clones upstream and renames the remote to `upstream`; a pushable `origin` must then be added before PR work.

## `New-TicketWorktree.ps1`

Creates an isolated ticket branch/worktree from a phase branch.

## `Remove-TicketWorktree.ps1`

Refuses to remove dirty worktrees.

## `validate_starter_kit.py`

Validates:
- frontmatter;
- required metadata keys;
- Obsidian wiki links.

Run:

```powershell
python .\scripts\validate_starter_kit.py
```


## `validate_control_plane.py`
Validates ticket dependencies/DAG, Markdown↔catalog parity, exact allow/deny configuration, architecture references, feature coverage and read-only REST/MCP invariants.


## `score_planning.py`
Runs both structural validators and prints `PLANNING_SCORECARD.json`. A structural failure invalidates the score.


## `validate_repo_baseline.py`
Run after the control plane is imported into the pinned repo. It verifies required existing source paths, planned-new path collisions and baseline ancestry; mismatches block Bootstrap.

## `render_execution_status.py`
Generates `status/EXECUTION_STATUS.md` from `TICKET_CATALOG.json` plus per-ticket RUN_STATE frontmatter. The generated file is observability output, not a second source of truth.
