# AGENTS.md — Pytxo

Instructions for AI coding agents working in this repository.

## Project identity

**Pytxo** is a low-overhead, systems-level **agentic control plane and telemetry layer** ([ptyxo.com](https://ptyxo.com)). Official source: [github.com/Pytxo-dev](https://github.com/Pytxo-dev) (main monorepo: `Pytxo-dev/pytxo`). It orchestrates parallel headless developer agents (Claude Code, Aider, Codex CLI, etc.) on local silicon or isolated cloud sandboxes.

**Stack:** Rust (tokio, portable-pty, tree-sitter) · Svelte 5 (Runes) · Tauri v2

Pytxo is **not** a multi-pane terminal IDE. It is an invisible agent OS with a passive **Reality Deck** UI, connected to existing tools via a local-first **MCP hub**.

## Repository map

| Path | Purpose |
|------|---------|
| `docs/` | Obsidian vault — canonical architecture and product knowledge |
| `docs/00-meta/MOC-home.md` | Start here for navigation |
| `docs/05-adr/` | Architecture Decision Records (immutable when Accepted) |
| `crates/pytxo-orchestrate` | Shared run/stop/status/dry-run |
| `crates/pytxo-sanitize` | Log/MCP redaction |
| `crates/pytxo-mcp` | Stdio MCP server binary |
| `crates/` | Core, scheduler, runner, store, CLI |
| `apps/desktop/` | Svelte 5 + Tauri v2 Reality Deck |

## Build and test

```bash
cargo build -p pytxo-cli
cargo build -p pytxo-mcp
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p pytxo-cli -- status --json
```

Desktop (from repo root):

```bash
cd apps/desktop && npm ci && npm run check
cargo build -p pytxo-desktop
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
| **Execution Yard** | Headless CLI agent processes under orchestration |
| **Orchestration layer** | Rust core: PTY, scheduling, scaffolding, WAL |
| **Presentation layer** | Svelte 5 + Tauri; telemetry only, no direct FS access |
| **Reality Deck** | Dashboard for live execution visualization |
| **Pytxo Link** | P2P remote control with signed approve-and-write |
| **BYOK** | Bring your own API keys |
| **Adaptive Semantic Scaffolding** | tree-sitter fidelity tiers for context |
| **Sovereign Shield** | Local sanitization + cryptographic remote actions |

## Competitive framing

Document honestly: native **Claude Code Agent Teams** and similar tools solve multi-agent coordination in-product. Pytxo's differentiation is **resource-bounded orchestration** (DAG scheduler, sparse overlay FS, WAL telemetry, MCP router)—see [`docs/07-guides/compare/pytxo-vs-claude-agent-teams.md`](docs/07-guides/compare/pytxo-vs-claude-agent-teams.md).

## Cursor-specific

Scoped rules live in [`.cursor/rules/`](.cursor/rules/). Root [`.cursorrules`](.cursorrules) is a deprecated shim only.

## Claude Code

See [`CLAUDE.md`](CLAUDE.md) and [`docs/00-meta/CLAUDE.md`](docs/00-meta/CLAUDE.md).
