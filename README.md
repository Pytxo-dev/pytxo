# Pytxo

**Agentic control plane and telemetry layer** for orchestrating parallel headless developer agents on local silicon or isolated cloud sandboxes.

- **Site:** [ptyxo.com](https://ptyxo.com)
- **GitHub:** [github.com/Pytxo-dev](https://github.com/Pytxo-dev) (canonical org for Pytxo repositories)
- **Stack:** Rust (tokio, portable-pty, tree-sitter) · Svelte 5 (Runes) · Tauri v2
- **Docs vault:** Open [`docs/`](docs/) in [Obsidian](https://obsidian.md) or start at [`docs/00-meta/MOC-home.md`](docs/00-meta/MOC-home.md)

## What Pytxo is

Pytxo is an invisible, bare-metal **agent operating system**: it does not replace your IDE. It plugs into Cursor, VS Code, NeoVim, or CLIs via a local-first **MCP hub**, with a passive **Reality Deck** UI for telemetry—not sixteen embedded terminal webviews.

## Repository layout

| Path | Purpose |
|------|---------|
| [`docs/`](docs/) | Obsidian-friendly knowledge vault (architecture, ADRs, guides) |
| [`AGENTS.md`](AGENTS.md) | Canonical instructions for AI coding agents |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to contribute code and documentation |
| `crates/` | Rust workspace (CLI, scheduler, runner, store, MCP, …) |
| `apps/desktop/` | Svelte 5 + Tauri v2 Reality Deck |

## For contributors and agents

- Humans: [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`docs/00-meta/style-guide.md`](docs/00-meta/style-guide.md)
- Agents: [`AGENTS.md`](AGENTS.md) and [`.cursor/rules/`](.cursor/rules/)

## Status

Phase 0–2: CLI, MCP stub, sanitize, DAG `depends_on`, and Tauri Reality Deck v1. Cloud sandboxes are not in this repo yet.

```bash
cargo build -p pytxo-cli
cargo run -p pytxo-cli -- init
cargo run -p pytxo-cli -- run --config pytxo.toml.example --dry-run
```

This monorepo: [Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).

See [`docs/07-guides/tutorials/first-three-agent-run.md`](docs/07-guides/tutorials/first-three-agent-run.md).
