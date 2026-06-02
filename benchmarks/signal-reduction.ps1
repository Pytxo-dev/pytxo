# Signal Core token reduction benchmark
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

$fixture = "tests/fixtures/tiny-monorepo/src/a.ts"
if (-not (Test-Path $fixture)) {
    throw "fixture not found: $fixture"
}

Write-Host "== Signal Core scaffold report =="
$json = cargo run -q -p pytxo-signal --example scaffold_report -- $fixture low
Write-Host $json

$pct = ($json | ConvertFrom-Json).stats.token_reduction_pct
Write-Host "token_reduction_pct: $pct"

if ($pct -le 0) {
    throw "expected token_reduction_pct > 0 for $fixture"
}

Write-Host "Signal reduction benchmark OK"
