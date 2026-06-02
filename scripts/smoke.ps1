# Workspace smoke: Rust tests + optional desktop build
$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
Set-Location $Root
& (Join-Path $Root "benchmarks\phase2-demo.ps1")
