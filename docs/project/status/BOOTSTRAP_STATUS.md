---
id: "bootstrap-status"
title: "Bootstrap Status"
type: "status"
status: "generated"
version: "1.1"
updated: "2026-08-24"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Bootstrap Status

**State:** COMPLETE — BG PASS (pending owner review)

## Checklist
- [x] Git available (system git, Windows)
- [x] Bun available (1.3.8)
- [x] Rust stable available (rustc/cargo 1.98.0, stable-msvc via rustup 1.29.0; installed during bootstrap)
- [x] GitHub auth/fork path resolved (`elmokirk`, private mirror `handy-flow` instead of public fork — owner decision for privacy; upstream-PR recipe in root README)
- [x] repo cloned (`repo/` inside project workspace)
- [x] `origin` is private repo `elmokirk/handy-flow`
- [x] `upstream` is `cjpais/Handy`
- [x] pinned commit available
- [x] baseline checkout verified (validate_repo_baseline green)
- [x] baseline tests run (see evidence below)
- [x] Windows build/native checks run or environment limitation recorded (native interactive smoke deferred to owner-supported QA per gate policy)
- [x] workspace/worktree root prepared (`worktrees/`)
- [x] branch protection capability recorded (rulesets possible on private repo; configuration = owner decision)
- [x] Bootstrap Gate report produced ([[status/GATE_BG_REPORT|GATE_BG_REPORT]])

## Actual baseline
- Commit: `af48dd68a64d58aad128fdbb920492a03da53c79` (v0.9.6)
- Version: `0.9.6` (package.json + tauri.conf.json)
- Current upstream head at bootstrap time: `af48dd68a64d58aad128fdbb920492a03da53c79`
- Decision: **REFRESH_APPROVED** (owner-approved pre-bootstrap refresh from originally captured `0e503672`; drift 5 commits, low risk; see [[research/HANDY_BASELINE|Baseline Lock]])

## Evidence summary (BOOT-003)

| Command | Result |
|---|---|
| `bun run lint` | PASS |
| `bun run format:check` | PASS |
| `bun run check:translations` | PASS (23/23) |
| `bun run build` | PASS |
| `bun run test:playwright` | PASS (2/2) |
| `cargo test` | PASS (206/206) |
| `cargo clippy --all-targets` | PASS (22 pre-existing upstream warnings = recorded baseline state) |

Environment provisioning during bootstrap: rustup/rustc 1.98, cmake+ninja (pip user), Vulkan SDK 1.4.357.0. Details in [[status/runs/BOOT-003|RUN_STATE BOOT-003]].

## Layout adaptation note
Workspace layout deviates from original kit assumption (kit root == workspace root): `repo/` and `worktrees/` live inside the kit folder; `scripts/bootstrap.ps1` was NOT used as-is (its kit-import step would recurse into itself). Steps were executed adapted/manually; `scripts/validate_starter_kit.py` excludes `repo/`, `worktrees/`, `node_modules`, `.git`. Documented in root `README.md`.

## Result
BG **PASS** with documented deviations (all owner-decided or mechanical adaptations, none architectural).
