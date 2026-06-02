# AGENTS.md — Pytxo

Instructions for AI coding agents working in this repository.

## Project identity

**Pytxo** is an **agent hypervisor and telemetry plane** ([ptyxo.com](https://ptyxo.com)). Source: [github.com/Pytxo-dev](https://github.com/Pytxo-dev) (`Pytxo-dev/pytxo` + `pytxo-desktop`). It coordinates headless agents (Claude Code, Codex, Antigravity CLI, …) in background PTYs on local silicon or cloud sandboxes.

**Stack:** Rust (tokio, portable-pty, tree-sitter) · Svelte 5 (Runes) · Tauri v2

**Canonical vision:** [`docs/06-product/vision.md`](docs/06-product/vision.md)

**Not:** BridgeSpace-style multi-terminal web workspaces. **Is:** bare-metal control plane + optional **Reality Deck** (structural telemetry, 3D AST topology target) via **MCP hub**.

**Three moats** — route new orchestration code through these concepts:

| Moat | Doc | Responsibility |
|------|-----|----------------|
| Signal Core | [`docs/02-areas/orchestration/signal-core.md`](docs/02-areas/orchestration/signal-core.md) | `tree-sitter` read-path skeletons |
| Blast Shield | [`docs/02-areas/orchestration/blast-shield.md`](docs/02-areas/orchestration/blast-shield.md) | CoW sandbox; approve-to-flush |
| Race Shield | [`docs/02-areas/orchestration/race-shield.md`](docs/02-areas/orchestration/race-shield.md) | `SwarmRegistry`, stdin buffering |

## Repository map

| Path | Purpose |
|------|---------|
| `docs/` | Obsidian vault — canonical architecture and product knowledge |
| `docs/00-meta/MOC-home.md` | Start here for navigation |
| `docs/06-product/vision.md` | Product vision and moats |
| `docs/05-adr/` | Architecture Decision Records (immutable when Accepted) |
| `crates/pytxo-orchestrate` | Shared run/stop/status/dry-run |
| `crates/pytxo-sanitize` | Log/MCP redaction |
| `crates/pytxo-signal` | Signal Core (`tree-sitter` skeletons) |
| `crates/pytxo-mcp` | Stdio MCP server binary |
| `crates/` | Core, scheduler, runner, store, CLI |
| [pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop) | Svelte 5 + Tauri v2 Reality Deck (separate repo) |

## Build and test

```bash
cargo build -p pytxo-cli
cargo build -p pytxo-mcp
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p pytxo-cli -- status --json
```

Desktop ([pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop)):

```bash
git clone https://github.com/Pytxo-dev/pytxo-desktop.git
cd pytxo-desktop
npm ci && npm run check
cargo build -p pytxo-desktop   # requires Tauri system deps on Linux
```

## Documentation rules

1. Prefer **atomic notes** in `docs/` (one concept per file, ~300–700 words).
2. Use **YAML frontmatter** per [`docs/00-meta/style-guide.md`](docs/00-meta/style-guide.md).
3. Link with Obsidian wikilinks: `[[note-slug]]`.
4. When adding a concept, update [`docs/00-meta/MOC-home.md`](docs/00-meta/MOC-home.md) or the relevant MOC.
5. **Irreversible architecture choices** → new ADR in `docs/05-adr/` (do not edit Accepted ADRs; supersede with a new ID).
6. No `data:image` base64 in markdown; use `docs/_attachments/`.
7. Human tutorials → `docs/07-guides/`; reference material → `docs/08-reference/`.

## Security

- Never commit API keys, tokens, or `.env` secrets.
- Examples must reflect **Sovereign Shield** patterns: sanitize secrets in logs and MCP payloads ([`docs/02-areas/security/regex-sanitization.md`](docs/02-areas/security/regex-sanitization.md)).
- BYOK: cloud features use the developer's keys; Pytxo does not store provider credentials in plaintext.

## Product language (canonical)

| Term | Meaning |
|------|---------|
| **Signal Core** | Context arbitrage via AST skeletons on read |
| **Blast Shield** | CoW sandbox; flush on user approve |
| **Race Shield** | Global swarm registry; collision-free writes |
| **Execution Yard** | Headless CLI agent processes under orchestration |
| **Reality Deck** | Space-console UI; 3D AST topology (target), not terminal walls |
| **BYOK** | Bring your own API keys (including Pytxo Cloud) |
| **Sovereign Shield** | Sanitization + cryptographic remote actions |

Full glossary: [`docs/00-meta/glossary.md`](docs/00-meta/glossary.md).

## Competitive framing

IDE-embedded agent teams solve coordination in-product. Pytxo differentiates on **hypervisor moats** (Signal / Blast / Race), DAG scheduling, and structural telemetry — see [`docs/07-guides/compare/pytxo-vs-claude-agent-teams.md`](docs/07-guides/compare/pytxo-vs-claude-agent-teams.md).

## Cursor-specific

Scoped rules live in [`.cursor/rules/`](.cursor/rules/). Root [`.cursorrules`](.cursorrules) is a deprecated shim only.

## Claude Code

See [`CLAUDE.md`](CLAUDE.md) and [`docs/00-meta/CLAUDE.md`](docs/00-meta/CLAUDE.md).
