$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "..\lib\Resolve-PytxoBin.ps1")

if (-not $env:PYTXO_TEST_REPO) {
    throw "Run scaffold.ps1 -Profile antigravity first"
}
Set-Location $env:PYTXO_TEST_REPO

$agy = Get-Command agy -ErrorAction SilentlyContinue
if (-not $agy) {
    throw "agy not found on PATH. Install Antigravity CLI first."
}

$Pytxo = Resolve-PytxoBin
$env:PYTXO_BIN = $Pytxo

Write-Host "=== pytxo ==="
Write-Host "PYTXO_BIN=$Pytxo"
& $Pytxo --version

Write-Host "=== agy version ==="
& agy --version 2>&1

Write-Host "=== dry-run ==="
& $Pytxo run --config pytxo.toml --dry-run

Write-Host "=== execute (agy --help smoke) ==="
& $Pytxo run --config pytxo.toml --cmd "agy --help"

Write-Host "=== status ==="
& $Pytxo status

Write-Host ""
Write-Host "OK: live-antigravity smoke complete"
Write-Host "Next: replace --cmd with your non-interactive agy invocation"
