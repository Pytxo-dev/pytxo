# Pytxo

[![CI](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml/badge.svg)](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Pytxo-dev/pytxo-releases)](https://github.com/Pytxo-dev/pytxo-releases/releases)
[![npm](https://img.shields.io/npm/v/pytxo)](https://www.npmjs.com/package/pytxo)
[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](LICENSE)

**The landing layer for coding agents.** Split one request across the agent
CLIs you already use (Codex, Claude Code, Cursor Agent; OpenCode and
Antigravity experimental), or bring branches from any agent tool with
`pytxo land`. Pytxo checks the combined change with your commands, shows
exactly what will land, refuses if anything moved, and leaves a receipt.
Many agents, one verified change.

![pytxo land on two branches that each pass their tests alone: the combined change fails twice and the decision is BLOCKED](docs/_attachments/pytxo-land-blocked.gif)

<sub>Recorded with pytxo 1.3.5 from npm on the sample repository from Pybench's
rename scenario: one branch renames `price()`, the other adds a caller of
`price()`. The branches come from Pybench, not from an agent run. The GIF cuts
the 3 s wait before the output to 1 s; the
[asciinema recording](docs/_attachments/pytxo-land-blocked.cast) keeps the real
timing.</sub>

```bash
npm i -g pytxo
pytxo land codex:agent/a claude:agent/b --check "npm test" --per-input   # shadow mode: changes nothing
```

Pass your agents' branches as `label:branch` (the label is only a name) and
your own test command. Checks run in a fresh worktree where nothing is
installed, so a Node project installs in the check:
`--check "npm ci && npm test"`. `--per-input` also runs the check on each
branch alone, so a failure that only shows up in the combination is reported as
one, and when only the combination fails it runs once more to rule out a flaky
test. Add `--apply` to land a ready result. `pytxo land` calls no model and
sends nothing; receipts stay in `.git/pytxo/receipts`.

**Limits**

- It catches what your checks catch. A break your tests don't cover can still
  land, and merge conflicts are reported and blocked, not resolved.
- Pybench (`pytxo bench`) runs on synthetic repositories. It shows the
  mechanism, not how often this happens in real projects.
- The CLI ships for macOS, Linux and Windows. Pytxo Desktop is Windows x64 only.
- Receipts use the in-toto statement format but are not signed. This is a
  public beta.

- **Site:** [pytxo.com](https://pytxo.com)
- **Docs:** [pytxo.com/docs](https://pytxo.com/docs)
- **Discord:** [discord.gg/AUFRPFjSYv](https://discord.gg/AUFRPFjSYv)
- **Releases:** [github.com/Pytxo-dev/pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases)
- **Stack:** Rust (`portable-pty`, `tree-sitter`) · ratatui TUI · Svelte 5 · Tauri v2

## Install

```bash
npm i -g pytxo@1.3.5
pytxo doctor
```

The first `pytxo` command downloads the native binary for your platform and
checks it against the release's SHA-256 list. npm 12 blocks install scripts by
default and prints a warning about pytxo's `postinstall`; you can ignore it.

Installers and binaries are published through
[Pytxo releases](https://github.com/Pytxo-dev/pytxo-releases). Pytxo Desktop is
Windows-first for v1.3.5. The npm installer must fail if the binary for the
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

The wider product direction is a commit layer for autonomous work: effect-bound
authority, independent post-state verification, causal evidence, and honest
recovery across production systems. That production-effect gateway is a north
star, not a shipping claim.

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

**Current release: v1.3.5 (public beta).** The shipping proof is a local, vendor-neutral
mission that produces isolated work, explicit verification, an immutable
single-root review package, guarded Apply, and durable evidence/recovery state.
Optional cloud services and the broader production-effect commit layer are not
part of the default local release claim.
