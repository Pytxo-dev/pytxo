$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "..\lib\Resolve-PytxoBin.ps1")

if (-not $env:PYTXO_TEST_REPO) {
    throw "Run scaffold.ps1 -Profile context first"
}
Set-Location $env:PYTXO_TEST_REPO

$Pytxo = Resolve-PytxoBin
$env:PYTXO_BIN = $Pytxo
$TestEnvRoot = Split-Path $PSScriptRoot -Parent
$Stub = Join-Path $TestEnvRoot "fixtures\stub-agent\stub-agent.ps1"

Write-Host "=== dry-run ==="
& $Pytxo run --config pytxo.toml --dry-run

Write-Host "=== execute (stub-agent) ==="
& $Pytxo run --config pytxo.toml --cmd "powershell -NoProfile -ExecutionPolicy Bypass -File `"$Stub`""

Write-Host "=== status ==="
& $Pytxo status

Write-Host "OK: smoke-context complete (PYTXO_BIN=$Pytxo)"
