# Creates the task board fixture as a fresh Git repository for the fleet demo.
param([string]$Path = (Join-Path $env:USERPROFILE "pytxo-demo\taskboard"))
$ErrorActionPreference = "Stop"
if (Test-Path $Path) { throw "$Path already exists. Run reset.ps1 first." }

New-Item -ItemType Directory -Force (Split-Path $Path) | Out-Null
Copy-Item -Recurse (Join-Path $PSScriptRoot "template") $Path
git -C $Path init -q
# reset.ps1 deletes only folders that carry this marker.
New-Item -ItemType File (Join-Path $Path ".git\pytxo-demo") | Out-Null
git -C $Path add .
git -C $Path -c user.name="Pytxo Demo" -c user.email=demo@pytxo.local -c commit.gpgsign=false commit -qm "Task board fixture"

Write-Host "Task board ready at $Path"
Write-Host "Mission: $(Join-Path $PSScriptRoot 'mission.txt')"
