# Pytxo

**Agent hypervisor and telemetry plane** for coordinating headless developer agents on local silicon or hosted sandboxes — without cloud-heavy multi-terminal workspaces.

- **Site:** [ptyxo.com](https://ptyxo.com)
- **GitHub:** [github.com/Pytxo-dev](https://github.com/Pytxo-dev)
- **Stack:** Rust (`portable-pty`, `tree-sitter`) · Svelte 5 (Runes) · Tauri v2
- **Vision:** [`docs/06-product/vision.md`](docs/06-product/vision.md)

## What Pytxo is

Pytxo runs **heterogeneous headless agents** (Claude Code, Codex, Antigravity CLI, …) in managed background PTYs. It is not your IDE and not sixteen embedded terminal webviews.

Three engineering moats (documented in the vault):

| Moat | Role |
|------|------|
| **Signal Core** | `tree-sitter` skeletons on read — context arbitrage, lower token burn |
| **Blast Shield** | Memory sandbox; disk flush on approve; fast rollback |
| **Race Shield** | Swarm registry + stdin buffering — no monorepo write races |

The **Reality Deck** ([pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop)) is a space-console UI (`#020205` void, teal / violet / solar gold) aimed at **3D AST dependency topology** and blast-radius telemetry — not raw terminal walls.

**Pytxo Cloud** (roadmap): hosted sandboxes + server-side context cache, strictly **BYOK**.

## Repository layout

[`docs/08-reference/repository-layout.md`](docs/08-reference/repository-layout.md)

| Path | Purpose |
|------|---------|
| [`docs/`](docs/) | Obsidian vault — start at [`docs/00-meta/MOC-home.md`](docs/00-meta/MOC-home.md) |
| [`crates/`](crates/) | Rust control plane (CLI, MCP, scheduler, runner, store) |
| [`scripts/`](scripts/) | Smoke and dev setup |

## Quick start

```bash
cargo build -p pytxo-cli
cargo run -p pytxo-cli -- doctor
cargo run -p pytxo-cli -- run --config pytxo.toml.example --dry-run
./scripts/smoke.ps1   # or scripts/smoke.sh
```

## Status

**v0.1.0** — CLI, MCP, DAG `depends_on`, sanitize, git worktrees, WAL telemetry. Moats and 3D Deck are phased; see [`docs/01-projects/mvp-bootstrap.md`](docs/01-projects/mvp-bootstrap.md).
