param(
  [Parameter(Mandatory=$true)][string]$Repo,
  [Parameter(Mandatory=$true)][string]$WorktreePath
)
$ErrorActionPreference = "Stop"
Set-Location $Repo
if (-not (Test-Path $WorktreePath)) { throw "Missing worktree: $WorktreePath" }
$dirty = git -C $WorktreePath status --porcelain
if ($dirty) { throw "Worktree has uncommitted changes; refusing removal." }
git worktree remove $WorktreePath
git worktree prune
