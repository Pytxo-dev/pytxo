# Phase 2 smoke: control plane + desktop compile
$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
Set-Location $Root

Write-Host "== pytxo sanitize tests =="
cargo test -p pytxo-sanitize

Write-Host "== DAG depends_on test =="
cargo test -p pytxo-scheduler dependency_orders

Write-Host "== workspace tests =="
cargo test --workspace

Write-Host "== context launch contract (PYTXO_CONTEXT_DIR + manifest.json) =="
cargo test -p pytxo-runner --test context_contract -- --nocapture

Write-Host "== status json =="
cargo run -p pytxo-cli -- status --json

if (Test-Path "apps/desktop/package.json") {
  Write-Host "== desktop compile =="
  cargo build -p pytxo-desktop
} else {
  Write-Host "== desktop skipped (apps/desktop missing) =="
}

Write-Host "Phase 2 demo OK"
