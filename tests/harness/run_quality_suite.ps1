# Custom quality suite runner (owner machine / Windows)
#
# Runs every gate from planning/06-QA_GATES.md that is runnable locally,
# in the same order as .github/workflows/custom-quality.yml, and prints a
# PASS/FAIL matrix. Exits non-zero if any gate fails.
#
# Usage:  powershell -File tests\harness\run_quality_suite.ps1 [-SkipRust]

param(
    [switch]$SkipRust
)

$ErrorActionPreference = "Continue"
$RepoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))

# Toolchain bootstrap (cargo via rustup, cmake/ninja via pip --user, Vulkan SDK)
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:APPDATA\Python\Python314\Scripts;C:\VulkanSDK\*\Bin;$env:Path"
if (-not $env:VULKAN_SDK) {
    $sdk = Get-ChildItem "C:\VulkanSDK" -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1
    if ($sdk) { $env:VULKAN_SDK = $sdk.FullName }
}

$results = [System.Collections.Generic.List[object]]::new()
function Invoke-Gate([string]$Name, [scriptblock]$Action) {
    Write-Host "==> $Name" -ForegroundColor Cyan
    & $Action
    $ok = ($LASTEXITCODE -eq 0)
    $results.Add([pscustomobject]@{ Gate = $Name; Pass = $ok })
    Write-Host ""
}

Set-Location $RepoRoot

Invoke-Gate "translations"    { bun run check:translations }
Invoke-Gate "eslint"          { bun run lint }
Invoke-Gate "format"          { bun run format:check }

if (-not $SkipRust) {
    Push-Location "$RepoRoot\src-tauri"
    Invoke-Gate "clippy-ratchet" { python "$RepoRoot\tests\harness\clippy_ratchet.py" }
    Invoke-Gate "cargo-test"     { cargo test }
    Pop-Location
}

Invoke-Gate "frontend-build"  { bun run build }
Invoke-Gate "playwright"      { bun run test:playwright }

Write-Host "================ QUALITY SUITE ================" -ForegroundColor White
$failed = $false
foreach ($r in $results) {
    $mark = if ($r.Pass) { "PASS" } else { "FAIL"; $failed = $true }
    $color = if ($r.Pass) { "Green" } else { "Red" }
    Write-Host ("  [{0}] {1}" -f $mark.PadRight(4), $r.Gate) -ForegroundColor $color
}
Write-Host "==============================================="
if ($failed) { exit 1 } else { exit 0 }
