param(
  [Parameter(Mandatory=$true)][string]$Repo,
  [Parameter(Mandatory=$true)][string]$WorktreeRoot,
  [Parameter(Mandatory=$true)][string]$Ticket,
  [Parameter(Mandatory=$true)][string]$BaseBranch,
  [string]$BranchPrefix = "feat"
)
$ErrorActionPreference = "Stop"
Set-Location $Repo
git fetch origin 2>$null
$Branch = "$BranchPrefix/$Ticket"
$Path = Join-Path $WorktreeRoot $Ticket
if (Test-Path $Path) { throw "Worktree path exists: $Path" }
if (-not (git show-ref --verify --quiet "refs/heads/$Branch")) {
  git branch $Branch $BaseBranch
}
git worktree add $Path $Branch
Write-Host "$Ticket -> $Path ($Branch)"
