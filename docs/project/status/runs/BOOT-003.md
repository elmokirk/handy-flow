---
id: "run-boot-003"
title: "RUN_STATE — BOOT-003"
type: "run-state"
status: "generated"
version: "1.0"
updated: "2026-08-24"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
ticket_id: "BOOT-003"
run_status: "DONE"
attempt_count: "3"
branch: "bootstrap/control-plane"
worktree: ""
last_commit: ""
---

# RUN_STATE — BOOT-003

## Checklist
- [x] baseline/context read
- [x] Agent Safety Policy read
- [x] existing automated quality commands recorded
- [x] Windows native limitations explicitly recorded (see below)
- [x] failures classified as environment (CRLF, missing toolchain) — no upstream/custom defects found

## Evidence
| Command | Result |
|---|---|
| `bun install` | 342 packages OK |
| `bun run lint` | exit 0 |
| `bun run format:check` | exit 0 (after env normalization, see Deviations) |
| `bun run check:translations` | 23/23 languages complete |
| `bun run build` | tsc + vite OK |
| `bun run test:playwright` | 2 passed / 0 failed |
| `cargo test` | **206 passed / 0 failed** |
| `cargo clippy --all-targets` | exit 0; 22 pre-existing upstream warnings recorded as baseline state |
| `cargo fmt -- --check` | exit 0 (via format:check) |

## Environment provisioning performed
- rustup 1.29.0 → rustc/cargo 1.98.0 (stable-msvc)
- cmake 4.4.2 + ninja 1.13.0 via `pip install --user` (admin-free)
- Vulkan SDK 1.4.357.0 installed (upstream CI pins 1.4.309.0; newer verified working by full ggml-vulkan build + tests)

## Windows native limitations (recorded per gate BG)
- Interactive microphone/hotkey/paste/floating-window smoke NOT executed during automated bootstrap — requires interactive session with owner. Deferred to owner-supported native QA (mandated by QA gates v1.1 anyway).

## Escalations
- none blocking

## Deviations
- Repo-local `core.autocrlf=input` set and tracked files re-normalized to LF (Prettier `endOfLine: lf` vs Windows checkout default). No content changes.
- `.prettierignore` extended by `docs/project` on bootstrap branch: the imported control-plane corpus is not frontend code and must not be reformatted.
- One transient MSBuild race during ggml-vulkan shader generation (`ssm_conv.comp.cpp` emitted empty under `-parallel 32`); regenerated deterministically, verified, no source changes. Classified: environment/tooling flake.

## Next smallest action
— (done)
