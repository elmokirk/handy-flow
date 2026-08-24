---
id: "handy-baseline"
title: "Handy Baseline Lock"
type: "research"
status: "accepted"
version: "1.2"
updated: "2026-08-24"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Handy Baseline Lock

[[START_HERE|← Start Here]] · [[planning/02-RESEARCH|Research Summary]]

## Frozen implementation baseline

| Field | Value |
|---|---|
| Baseline ID | `handy-main-2026-08-24-af48dd68` |
| Originally captured | 2026-08-21 |
| Refreshed (owner-approved) | 2026-08-24, before first `custom/main` commit |
| Upstream | `https://github.com/cjpais/Handy` |
| Source branch at capture/refresh | `main` |
| Pinned commit | `af48dd68a64d58aad128fdbb920492a03da53c79` |
| Commit date | 2026-08-24T15:01:27+08:00 |
| Source version | `0.9.6` |
| Latest tagged release observed | `v0.9.6` (`af48dd6`) |
| Canonical custom branch | `custom/main` |

Both `package.json` and `src-tauri/tauri.conf.json` at the pinned commit declare `0.9.6`.

## Important distinction
The implementation baseline is the **pinned source commit**, not the release installer or future upstream `main`. The tagged release equals the pinned source commit for this baseline.

## Refresh history

| From → To | Drift | Assessment |
|---|---|---|
| `0e503672` (v0.9.5, captured 2026-08-21) → `af48dd68` (v0.9.6, refreshed 2026-08-24) | 5 commits | Low risk |

Refresh diff contents:
1. `286e66c fix bindings` — `src/bindings.ts` (generated bindings only).
2. `5ec2276 single writer tray icon (#1952)` — refactor of tray icon handling; touches `src-tauri/src/actions.rs`, `lib.rs`, `secure_input.rs`, `shortcut/mod.rs`, `tray.rs`, `utils.rs`. These are shared integration files (Integrator-owned); no data/persistence or contract semantics changed.
3. `f6fac42 update to tauri-plugin-updater 2.10.1` — dependency bump.
4. `8fd6691 fix nix build` — packaging only.
5. `af48dd6 release v0.9.6` — version bumps.

Verification at refresh time (`scripts/validate_repo_baseline.py --repo repo`): all `REPO_PATH_MAP.existing_required` paths present, no `planned_new` collisions, old pin is ancestor of new pin.

## Freshness policy
Default: keep the pinned commit and create `custom/main` from it. A newer upstream is adopted only through a dedicated `BASELINE-REFRESH` task that diffs architecture hotspots, reruns affected risk assumptions/tests, updates the lock and reruns Bootstrap/G0. No Feature Agent may silently rebase to newer upstream.

## Sources
- https://github.com/cjpais/Handy
- https://github.com/cjpais/Handy/commit/af48dd68a64d58aad128fdbb920492a03da53c79
- https://github.com/cjpais/Handy/releases/tag/v0.9.6
