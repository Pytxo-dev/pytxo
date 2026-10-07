# Deletes a task board fixture created by setup.ps1, and nothing else.
param([string]$Path = (Join-Path $env:USERPROFILE "pytxo-demo\taskboard"))
$ErrorActionPreference = "Stop"
if (-not (Test-Path $Path)) { Write-Host "Nothing to reset at $Path"; return }
if (-not (Test-Path (Join-Path $Path ".git\pytxo-demo"))) {
  throw "$Path was not created by setup.ps1; refusing to delete it."
}
Remove-Item -Recurse -Force $Path
Write-Host "Removed $Path"
