# Cold-start wall times for headless pytxo (Windows).
# Usage (from repo root after `cargo build -p pytxo-cli`):
#   powershell -File tooling/benchmarks/cli-cold-start.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$Pytxo = if ($env:PYTXO_BIN) { $env:PYTXO_BIN } else { Join-Path $Root "target\debug\pytxo.exe" }

if (-not (Test-Path $Pytxo)) {
    throw "pytxo binary not found at $Pytxo"
}

function Measure-Cmd([string]$Label, [string[]]$Args) {
    $sw = [Diagnostics.Stopwatch]::StartNew()
    & $Pytxo @Args | Out-Null
    $code = $LASTEXITCODE
    $sw.Stop()
    Write-Host ("{0}_ms={1} exit={2}" -f $Label, [int]$sw.ElapsedMilliseconds, $code)
}

Write-Host "=== cli cold-start ==="
Write-Host "bin=$Pytxo"
Measure-Cmd "help" @("--help")
Measure-Cmd "status" @("status", "--limit", "1")
Measure-Cmd "doctor_quick" @("doctor", "--quick")
Write-Host "OK: cli cold-start sample complete"
