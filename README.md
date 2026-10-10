# Pytxo

[![CI](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml/badge.svg)](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Pytxo-dev/pytxo-releases)](https://github.com/Pytxo-dev/pytxo-releases/releases)
[![npm](https://img.shields.io/npm/v/pytxo)](https://www.npmjs.com/package/pytxo)
[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](LICENSE)

**Many coding agents, one verified change.** Pytxo splits one request across the
agent CLIs you already use (Codex, Claude Code, Cursor Agent, OpenCode,
Antigravity), runs each task in an isolated copy, keeps tasks that share a file
from running together, checks the combined result with your commands, and
applies exactly the files you reviewed. It is an agent hypervisor: a control
layer around the agents, not another agent or IDE.

The wider product direction is a commit layer for autonomous work: effect-bound
authority, independent post-state verification, causal evidence, and honest
recovery across production systems. That production-effect gateway is a north
star, not a shipping claim.

- **Site:** [pytxo.com](https://pytxo.com)
- **Docs:** [pytxo.com/docs](https://pytxo.com/docs)
- **Discord:** [discord.gg/AUFRPFjSYv](https://discord.gg/AUFRPFjSYv)
- **Releases:** [github.com/Pytxo-dev/pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases)
- **Stack:** Rust (`portable-pty`, `tree-sitter`) · ratatui TUI · Svelte 5 · Tauri v2

## Install

```bash
npm i -g pytxo@1.2.5
pytxo doctor
```

Installers and binaries are published through
[Pytxo releases](https://github.com/Pytxo-dev/pytxo-releases). Pytxo Desktop is
Windows-first for v1.2.5. The npm installer must fail if the binary for the
current platform is not present or cannot be verified; it must not imply that a
missing platform succeeded.

Running `pytxo` with no subcommand opens the terminal dashboard. Pytxo Desktop
is optional and organizes the same local control model around Work, History,
Setup, workspace switching, approvals, and reviewed Apply.

## What Pytxo is

Pytxo runs **heterogeneous headless agents** (Claude Code, Codex, Antigravity CLI, …) in managed background PTYs. It is not your IDE and not sixteen embedded terminal webviews.

Today, one successful Orbit or Galaxy mission can become an immutable review
package for one repository root. Apply uses the stored target bytes, validates
affected-path preimages, and journals recovery. This repository boundary is the
first concrete version of the broader commit-layer contract.

| Moat | Role |
|------|------|
| **Signal Core** | `tree-sitter` skeletons on read — context arbitrage, lower token burn |
| **Blast Shield** | Isolated workspaces; Orbit/Galaxy changes reach the repository only through reviewed Apply |
| **Race Shield** | Swarm registry + stdin buffering — no monorepo write races |

**Pytxo Desktop** ([`apps/desktop`](apps/desktop/)) is the optional Work,
History, and Setup control UI — structural evidence and decisions, not raw
terminal walls.

Permission profiles matter. DeepSpace is non-flushable, Orbit and Galaxy use
the reviewed repository boundary, and Supernova is host-direct. The effective
enforcement receipt is the authority; the UI does not claim every profile is
sandboxed.

## Repository layout

| Path | Purpose |
|------|---------|
| [`crates/`](crates/) | Rust control plane (CLI, TUI, MCP, scheduler, runner, store) |
| [`packages/pytxo`](packages/pytxo/) | npm installer wrapper |
| [`apps/web/content/docs/`](apps/web/content/docs/) | Public docs (Fumadocs → pytxo.com/docs) |
| [`apps/web/`](apps/web/) | Marketing site ([pytxo.com](https://pytxo.com)) |
| [`apps/desktop/`](apps/desktop/) | Pytxo Desktop (Svelte + Tauri) |
| [`docs/`](docs/) | Internal Obsidian vault |
| [`tooling/`](tooling/) | Install scripts, smoke tests, benchmarks |

## Quick start (developers)

Building from source requires Rust 1.88 or newer and the current Node.js/npm
toolchain used by the workspace lockfiles.

```bash
cargo build -p pytxo-cli
cargo run -p pytxo-cli -- doctor
cargo run -p pytxo-cli -- trust orbit
cargo run -p pytxo-cli -- run --config pytxo.toml.example --dry-run
```

Run the repository smoke separately with
`powershell -File tooling/scripts/smoke.ps1` on Windows or
`bash tooling/scripts/smoke.sh` on Unix-like hosts.

## Status

**Current release: v1.2.5 (public beta).** The shipping proof is a local, vendor-neutral
mission that produces isolated work, explicit verification, an immutable
single-root review package, guarded Apply, and durable evidence/recovery state.
Optional cloud services and the broader production-effect commit layer are not
part of the default local release claim.
