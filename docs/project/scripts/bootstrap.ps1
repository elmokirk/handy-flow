param(
  [Parameter(Mandatory=$true)][string]$Workspace,
  [string]$ForkRepo = "",
  [switch]$CreateFork
)

$ErrorActionPreference = "Stop"
$Baseline = "af48dd68a64d58aad128fdbb920492a03da53c79"
$Upstream = "https://github.com/cjpais/Handy.git"
$KitRoot = Split-Path -Parent $PSScriptRoot

New-Item -ItemType Directory -Force -Path $Workspace | Out-Null
$Repo = Join-Path $Workspace "repo"
$Worktrees = Join-Path $Workspace "worktrees"
New-Item -ItemType Directory -Force -Path $Worktrees | Out-Null
if (Test-Path $Repo) { throw "Repo path already exists: $Repo. Bootstrap refuses to overwrite it." }

if ($CreateFork) {
  if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { throw "GitHub CLI 'gh' is required for -CreateFork." }
  gh auth status
  gh repo fork cjpais/Handy --clone=false
  $Login = gh api user --jq .login
  $ForkRepo = "$Login/Handy"
}

if ($ForkRepo) {
  git clone "https://github.com/$ForkRepo.git" $Repo
  Set-Location $Repo
  if (-not (git remote | Select-String "^upstream$")) { git remote add upstream $Upstream }
} else {
  git clone $Upstream $Repo
  Set-Location $Repo
  git remote rename origin upstream
  Write-Warning "No fork origin configured. Add origin before PR work."
}

git fetch upstream --tags
if (git remote | Select-String "^origin$") { git fetch origin }
git cat-file -e "$Baseline^{commit}"

# Canonical custom branch; never silently overwrite divergence.
git show-ref --verify --quiet refs/heads/custom/main
if ($LASTEXITCODE -eq 0) {
  $existing = git rev-parse custom/main
  if ($existing -ne $Baseline) { throw "Existing local custom/main differs from baseline. Escalate; no force update." }
} else {
  git branch custom/main $Baseline
}

if (git remote | Select-String "^origin$") {
  git ls-remote --exit-code --heads origin custom/main *> $null
  if ($LASTEXITCODE -eq 0) {
    $remoteSha = (git ls-remote origin refs/heads/custom/main).Split("`t")[0]
    git fetch origin custom/main
    if ($remoteSha -ne $Baseline) {
      git merge-base --is-ancestor $Baseline origin/custom/main
      if ($LASTEXITCODE -eq 0) { throw "Existing initialized custom/main detected. Do not bootstrap again; use the resume/orchestrator workflow." }
      throw "origin/custom/main does not descend from the pinned baseline. Escalate; no force update."
    }
  } else {
    git push -u origin custom/main
  }
}

git switch -c bootstrap/control-plane custom/main
$DocsProject = Join-Path $Repo "docs/project"
New-Item -ItemType Directory -Force -Path $DocsProject | Out-Null
Copy-Item -Path (Join-Path $KitRoot "*") -Destination $DocsProject -Recurse -Force

if (Get-Command python -ErrorAction SilentlyContinue) { $Py = "python"; $PyArgs = @() }
elseif (Get-Command py -ErrorAction SilentlyContinue) { $Py = "py"; $PyArgs = @("-3") }
else { throw "Python 3 is required to run Starter Kit validators." }

& $Py @PyArgs (Join-Path $DocsProject "scripts/validate_starter_kit.py")
& $Py @PyArgs (Join-Path $DocsProject "scripts/validate_control_plane.py")
& $Py @PyArgs (Join-Path $DocsProject "scripts/validate_repo_baseline.py") --repo $Repo

Write-Host "Repo: $Repo"
Write-Host "Worktrees: $Worktrees"
Write-Host "Canonical custom base: custom/main @ $Baseline"
Write-Host "Control plane imported to: $DocsProject"
Write-Host "Next: run both validators, then targeted commit/Bootstrap PR. Do NOT implement features."
