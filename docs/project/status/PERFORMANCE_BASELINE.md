---
id: "performance-baseline"
title: "G0 Performance & Behavior Baseline Results"
type: "status"
status: "generated"
version: "1.0"
updated: "2026-08-24"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# G0 Performance & Behavior Baseline Results

Procedure: [[research/BENCHMARK_PROCEDURE|Benchmark Procedure]]
Machine: Windows 11 (NT 10.0.26200), x64, MSVC Build Tools 2022, WebView2 151
Commit: `phase/00-baseline-harness` head at measurement time (skeleton + QA harness only)
Build profile: **debug/dev** (release build deferred; release timing recorded separately when first needed)

## Tier A — Automated results

| Metric | Result | Notes |
|---|---|---|
| Startup → main window | **median 383 ms** (383/358 warm, 1005 ms cold iter 1) | debug build, tray+window |
| Idle RAM (WorkingSet) | **~79.3 MB** steady-state | 3 samples after 3 s settle |
| Idle RAM (Private) | ~35.0 MB | same runs |
| Binary size | 84 MB | debug only — release/LTO not yet built |
| `bun install` | 36 s | 342 packages, cold cache |
| `bun run build` (tsc+vite) | 2.6–8.1 s | chunk-size warning pre-existing upstream |
| `cargo check` (warm) | ~9–16 s | 2 pre-existing upstream warnings |
| `cargo clippy --all-targets` | first run 3 m 14 s; warm cached | 22 distinct warnings = ratchet ceiling |
| `cargo test` | suite runtime **0.34 s** (206 tests) + build time | full matrix green |
| Playwright smoke | ~3 s (2 tests) | chromium |

## Tier B — Interactive results

| Metric | Result |
|---|---|
| Model-loaded RAM / VRAM | DEFERRED (owner-supported interactive step) |
| End-to-paste median (Notepad) | DEFERRED — procedure fixed in [[research/BENCHMARK_PROCEDURE]] |
| End-to-paste median (browser/VS Code) | DEFERRED ditto |
| Fixture transcription timing | DEFERRED ditto (fixture spec fixed there) |

Deferral rationale: interactive microphone/model/focus behavior requires the owner
present; QA gates v1.1 mandate interactive Windows evidence anyway. All Tier-B
numbers are re-measured before G2 and become the regression-reference set.

## Behavior smoke checklist (documented per BASE-001)

Interactive Notepad/browser/editor smoke steps are defined in
[[research/BENCHMARK_PROCEDURE]] §Tier B; executed results will be appended here
after the owner session.

## Regression budget hook

The ≤10 % end-to-paste budget (QA gates v1.1) binds against the FIRST measured
Tier-B medians, not these Tier-A numbers.
