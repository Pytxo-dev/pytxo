# Reality Deck desktop smoke — verifies Tauri crate tests and Svelte check.
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..\..")

Write-Host "==> cargo test -p pytxo-desktop"
cargo test -p pytxo-desktop

Write-Host "==> npm run check (desktop)"
Push-Location apps/desktop
npm ci
npm run check
Pop-Location

Write-Host "Desktop smoke OK"
