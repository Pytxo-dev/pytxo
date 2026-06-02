# Local dev bootstrap for Pytxo monorepo
$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
Set-Location $Root

Write-Host "== rustup (if missing) =="
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
  Write-Host "Install Rust from https://rustup.rs"
} else {
  rustup show active-toolchain
}

Write-Host "== cargo fetch =="
cargo fetch

if (Test-Path "apps/desktop/package.json") {
  Write-Host "== desktop npm ci (monorepo layout) =="
  Push-Location apps/desktop
  npm ci
  Pop-Location
} else {
  Write-Host "Reality Deck: clone https://github.com/Pytxo-dev/pytxo-desktop"
}

Write-Host ""
Write-Host "Reminder: pytxo run needs a git repo with at least one commit."
Write-Host "Run: cargo run -p pytxo-cli -- doctor"
Write-Host "Docs: docs/08-reference/repository-layout.md"
