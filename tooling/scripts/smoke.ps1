# Workspace smoke: Rust tests + optional desktop build
$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
Set-Location $Root
& (Join-Path $Root "tooling\benchmarks\phase2-demo.ps1")
