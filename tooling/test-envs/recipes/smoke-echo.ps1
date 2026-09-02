$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "..\lib\Resolve-PytxoBin.ps1")

if (-not $env:PYTXO_TEST_REPO) {
    throw "Run scaffold.ps1 first (sets PYTXO_TEST_REPO)"
}
Set-Location $env:PYTXO_TEST_REPO

$Pytxo = Resolve-PytxoBin
$env:PYTXO_BIN = $Pytxo

Write-Host "=== dry-run ==="
Invoke-PytxoChecked $Pytxo trust orbit
Invoke-PytxoChecked $Pytxo run --config pytxo.toml --dry-run

Write-Host "=== execute (echo) ==="
Invoke-PytxoChecked $Pytxo run --config pytxo.toml --cmd "echo pytxo-agent"

Write-Host "=== status ==="
Invoke-PytxoChecked $Pytxo status

Write-Host "OK: smoke-echo complete (PYTXO_BIN=$Pytxo)"
