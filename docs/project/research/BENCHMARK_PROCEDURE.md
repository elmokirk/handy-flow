---
id: "benchmark-procedure"
title: "Repeatable Performance Benchmark Procedure"
type: "research"
status: "accepted"
version: "1.0"
updated: "2026-08-24"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---

# Repeatable Performance Benchmark Procedure

[[research/HANDY_BASELINE|← Baseline]] · [[status/PERFORMANCE_BASELINE|Baseline Results]]

Purpose: make every performance comparison (G0 → G6 regression checks,
≤10 % end-to-paste budget) reproducible on the same machine, same build
type and same fixtures. Two tiers:

## Tier A — Automated (agent-runnable, no interaction)

Prerequisites: toolchain bootstrap per root `README.md`
(rust/cargo, cmake+ninja via pip user, Vulkan SDK, Bun).

```powershell
# 1. Build debug binary (deterministic dev profile)
cd src-tauri; $env:Path = "$env:USERPROFILE\.cargo\bin;$env:APPDATA\Python\Python314\Scripts;C:\VulkanSDK\*\Bin;$env:Path"; $env:VULKAN_SDK = (Get-ChildItem C:\VulkanSDK | Sort-Object Name -Desc | Select -First 1).FullName
cargo build            # record wall time + target\debug\handy.exe size

# 2. Startup latency + idle RAM (3 iterations, no mic use)
$exe = "src-tauri\target\debug\handy.exe"
1..3 | ForEach-Object {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $p = Start-Process $exe -PassThru
  while ($p.MainWindowHandle -eq [IntPtr]::Zero -and -not $p.HasExited -and $sw.ElapsedMilliseconds -lt 30000) { Start-Sleep -m 25; $p.Refresh() }
  $startup = $sw.ElapsedMilliseconds; Start-Sleep 3
  $p.Refresh(); "$startup ms | WS $([math]::Round($p.WorkingSet64/1MB,1)) MB | Priv $([math]::Round($p.PrivateMemorySize64/1MB,1)) MB"
  Stop-Process -Id $p.Id -Force; Start-Sleep 1
}

# 3. Full quality suite timing (also the CI gate)
powershell -File tests\harness\run_quality_suite.ps1
```

Report per run: machine OS/CPU/RAM/GPU, commit SHA, build profile, all
timings, binary size, startup median of iterations 2–3 (discard
iteration 1 = cold start), idle RAM after 3 s settle.

## Tier B — Interactive (owner-supported, required for STT budgets)

These need microphone, a loaded model and real focus targets. Execute
on the owner machine before G2 and re-run at G6.

1. **Model state**: install the reference model (same one across all
   future comparisons; record exact model id/quantization in results).
   Note `Task Manager → Details → handy.exe` WorkingSet and GPU VRAM
   after first dictation (model loaded).
2. **Fixture**: fixed test utterance, ≥10 s silence-padded WAV,
   16 kHz mono, identical file every run (`fixtures/baseline-utterance.wav`).
3. **End-to-paste latency** (the ≤10 % budget metric): focus Notepad;
   start external stopwatch at shortcut press, stop when text appears.
   10 repetitions, report median + p95. Repeat once each with a plain
   browser address bar and VS Code editor focused.
4. **Log corroboration**: run app with `--debug`; transcription lifecycle
   logs allow splitting capture/VAD/inference/paste phases if deeper
   analysis is needed.

## Comparison rules

- Only same-tier numbers are comparable (Tier A never compared to Tier B).
- Regression budget applies to Tier-B end-to-paste medians (QA gates v1.1).
- Machine/driver/OS updates invalidate the baseline → re-seed both tiers
  and note it in the results document.
