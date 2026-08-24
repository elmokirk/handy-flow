---
id: "git-research"
title: "Git Worktree & Branch Research"
type: "research"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Git Worktree & Branch Research

[[planning/02-RESEARCH|← Research Summary]] · [[orchestration/GIT_WORKTREE_PR_STRATEGY|Git Strategy]]

## Primary sources

- Git worktree documentation: https://git-scm.com/docs/git-worktree
- GitHub protected branches: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches

## Applied principles

- one branch per active worktree;
- no shared working directory between coding agents;
- ticket PRs integrate into phase branch;
- phase branch runs full integration gate before `custom/main`;
- `custom/main` is protected from direct autonomous feature pushes;
- upstream refresh work is isolated from feature development.
