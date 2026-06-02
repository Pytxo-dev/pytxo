# Phase 2 smoke: control plane + desktop compile
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

Write-Host "== pytxo sanitize tests =="
cargo test -p pytxo-sanitize

Write-Host "== DAG depends_on test =="
cargo test -p pytxo-scheduler dependency_orders

Write-Host "== workspace tests (no desktop link) =="
cargo test --workspace --exclude pytxo-desktop

Write-Host "== status json =="
cargo run -p pytxo-cli -- status --json

if (Test-Path "apps/desktop/package.json") {
  Write-Host "== desktop compile =="
  cargo build -p pytxo-desktop
} else {
  Write-Host "== desktop skipped (extracted to pytxo-desktop repo) =="
}

Write-Host "Phase 2 demo OK"
