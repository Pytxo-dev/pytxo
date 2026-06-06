# Pytxo

[![CI](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml/badge.svg)](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Pytxo-dev/pytxo)](https://github.com/Pytxo-dev/pytxo/releases)
[![npm](https://img.shields.io/npm/v/pytxo)](https://www.npmjs.com/package/pytxo)
[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](LICENSE)

**Agent hypervisor and telemetry plane** for coordinating headless developer agents on local silicon or hosted sandboxes — without cloud-heavy multi-terminal workspaces.

- **Site:** [pytxo.com](https://pytxo.com)
- **Docs:** [pytxo.com/docs](https://pytxo.com/docs)
- **GitHub:** [github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo)
- **Stack:** Rust (`portable-pty`, `tree-sitter`) · ratatui TUI · Svelte 5 · Tauri v2

## Install (v0.1.0)

```bash
npm i -g pytxo
pytxo doctor
```

Or build from source: `cargo install --path crates/pytxo-cli`

Running `pytxo` with no subcommand opens the **terminal dashboard** (doctor, runs, domains, HITL).

## What Pytxo is

Pytxo runs **heterogeneous headless agents** (Claude Code, Codex, Antigravity CLI, …) in managed background PTYs. It is not your IDE and not sixteen embedded terminal webviews.

| Moat | Role |
|------|------|
| **Signal Core** | `tree-sitter` skeletons on read — context arbitrage, lower token burn |
| **Blast Shield** | Worktree isolation; explicit merge paths |
| **Race Shield** | Swarm registry + stdin buffering — no monorepo write races |

The **Reality Deck** ([`apps/desktop`](apps/desktop/)) is optional desktop telemetry — not raw terminal walls.

## Repository layout

| Path | Purpose |
|------|---------|
| [`crates/`](crates/) | Rust control plane (CLI, TUI, MCP, scheduler, runner, store) |
| [`packages/pytxo`](packages/pytxo/) | npm installer wrapper |
| [`apps/docs/`](apps/docs/) | Public docs (Docusaurus → pytxo.com/docs) |
| [`apps/web/`](apps/web/) | Marketing site ([pytxo.com](https://pytxo.com)) |
| [`apps/desktop/`](apps/desktop/) | Reality Deck (Svelte + Tauri) |
| [`docs/`](docs/) | Internal Obsidian vault |
| [`tooling/`](tooling/) | Install scripts, smoke tests, benchmarks |

## Quick start (developers)

```bash
cargo build -p pytxo-cli
cargo run -p pytxo-cli -- doctor
cargo run -p pytxo-cli -- run --config pytxo.toml.example --dry-run
./tooling/scripts/smoke.ps1   # or tooling/scripts/smoke.sh
```

## Status

**v0.1.0** — First public release: CLI + TUI dashboard, npm installer, GitHub Release binaries, MCP, git worktree orchestration, SQLite WAL telemetry.
