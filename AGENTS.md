# AGENTS.md — Pytxo

Instructions for AI coding agents working in this repository.

## Project identity

**Pytxo** runs and coordinates the coding agents you already use ([ptyxo.com](https://ptyxo.com)). As a category it is a local **agent hypervisor and telemetry plane**. Source: [github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo) (monorepo: `crates/*` + `apps/desktop`). It schedules headless agents (Claude Code, Codex, Antigravity CLI, …) in background PTYs on local silicon or cloud sandboxes.

**Stack:** Rust (tokio, portable-pty, tree-sitter) · Svelte 5 (Runes) · Tauri v2

**Execution yard:** `portable-pty` default ([[ADR-0010-pty-default-execution-backend]]); `execution_backend = "subprocess"` for CI fallback.

**Canonical vision:** [`docs/06-product/vision.md`](docs/06-product/vision.md)

**Not:** BridgeSpace-style multi-terminal web workspaces (ADEs). **Is:** bare-metal control plane + optional **Pytxo Desktop** (structural telemetry, 3D AST topology) via **MCP hub**.

**Three moats** — route new orchestration code through these concepts:

| Plain language | Moat | Doc | Responsibility |
|----------------|------|-----|----------------|
| Smarter context | Signal Core | [`docs/02-areas/orchestration/signal-core.md`](docs/02-areas/orchestration/signal-core.md) | `tree-sitter` read-path skeletons |
| Safe sandbox until approve | Blast Shield | [`docs/02-areas/orchestration/blast-shield.md`](docs/02-areas/orchestration/blast-shield.md) | CoW sandbox; approve-to-flush |
| No write collisions | Race Shield | [`docs/02-areas/orchestration/race-shield.md`](docs/02-areas/orchestration/race-shield.md) | `SwarmRegistry`, stdin buffering |

**Pytxo Ultra billing** (when `billing.mode = ultra` in `pytxo.toml`): `TokenWallet`, `UsageMeter`, `ArbitrageProfiler`, `ManagedTransport` — see [`docs/05-adr/ADR-0009-ultra-managed-metering.md`](docs/05-adr/ADR-0009-ultra-managed-metering.md).

**Policy and hypervisor** (orchestration-only; never in UI):

| Concept | Doc | Responsibility |
|---------|-----|----------------|
| Permission Profile Engine | [`docs/02-areas/security/permission-profile-engine.md`](docs/02-areas/security/permission-profile-engine.md) | Four-tier local capability ladder (`PermissionProfile`) |
| Execution domains | [`docs/02-areas/orchestration/execution-domains.md`](docs/02-areas/orchestration/execution-domains.md) | `HypervisorRegistry`, per-repo WAL separation |

When changing `pytxo-runner` or `pytxo-orchestrate`, declare which **permission profile** and **execution domain** scope applies. ADR: [`docs/05-adr/ADR-0008-local-permission-profile-four-tiers.md`](docs/05-adr/ADR-0008-local-permission-profile-four-tiers.md).

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
| `apps/desktop/` | Svelte 5 + Tauri v2 Pytxo Desktop (`pytxo-desktop` crate) |
| `apps/web/content/docs/` | Public user docs (Fumadocs MDX) — served at pytxo.com/docs from the Next app |
| `tooling/scripts/`, `tooling/benchmarks/` | Smoke and competitive repro scripts |
| `apps/desktop-export/` | Export / release staging (not canonical source) |

## Build and test

```bash
cargo build -p pytxo-cli
cargo build -p pytxo-mcp
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p pytxo-cli -- status --json
```

Pytxo Desktop (`apps/desktop`):

```bash
cd apps/desktop && npm ci && npm run check
cargo build -p pytxo-desktop   # from repo root; Tauri system deps on Linux
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
| **Signal Core** | Smarter context via AST skeletons on read |
| **Blast Shield** | Safe sandbox; flush on user approve |
| **Race Shield** | Global swarm registry; collision-free writes |
| **Execution Yard** | Headless CLI agent processes under orchestration |
| **Pytxo Desktop** | Optional control UI; 3D AST topology, not terminal walls (formerly Reality Deck) |
| **Workspace** | One or more project folders under one coordinated run |
| **BYOK** | Bring your own API keys (including Pytxo Cloud) |
| **Sovereign Shield** | Sanitization + cryptographic remote actions |
| **Permission profile** | Local trust tier: DeepSpace, Orbit (default), Galaxy, Supernova |
| **Execution domain** | One repo root’s isolated run/registry/store slice |
| **Hypervisor registry** | Multi-project map of active execution domains |

Full glossary: [`docs/00-meta/glossary.md`](docs/00-meta/glossary.md).

## Competitive framing

IDE-embedded agent teams solve coordination in-product. Pytxo differentiates on **hypervisor moats** (Signal / Blast / Race), DAG scheduling, and structural telemetry — see [`docs/07-guides/compare/pytxo-vs-claude-agent-teams.md`](docs/07-guides/compare/pytxo-vs-claude-agent-teams.md).

## Cursor-specific

Scoped rules live in [`.cursor/rules/`](.cursor/rules/). Root [`.cursorrules`](.cursorrules) is a deprecated shim only.

## Claude Code

See [`CLAUDE.md`](CLAUDE.md) and [`docs/00-meta/CLAUDE.md`](docs/00-meta/CLAUDE.md).

# AGENTS — PlugDev project

This is a Minecraft Paper plugin developed with PlugDev. Prefer `plug run` over manually starting Paper.

## Loop

1. `npm install -g @plugdev/cli` (once)
2. `plugdev init --setup --agents --mcp` (once per project)
3. `plug run` — server + watch + client
4. Players are auto-OP on join when `dev.op` is true (default). Type console commands in the same terminal after ready (`list`, `gamemode creative @a`, …)
5. `plug doctor` if boot or detection fails
6. Multi-module: `plugdev module list` / `plugdev module use <name>`
7. Deps: `plugdev deps add|remove|list` (or TUI Dependencies)
8. `plug clean` when you need a fresh world; `plug clean --all` for a cold `.plugdev/run`

## Facts

| Item | Value |
|------|--------|
| Bins | `plug` and `plugdev` (same CLI) |
| Config | `plugdev.yml` |
| Run dir | `.plugdev/run/` |
| Cache | `~/.plugdev/` |
| Modules | `plugdev module list|use` (multi-module Maven/Gradle) |
| Deps | `plugdev deps add|remove|list` (+ TUI Dependencies) |
| Reload | Safe JAR reload (not `/reload`); optional `--hotswap` for method bodies |
| Folia | Prefer full restart over safe reload |
| Headless | `plugdev server start|stop|status|command|logs` + `--json` |
| MCP | `npx @plugdev/mcp` — structured tools for the same loop |
| Skill install | `npx skills add mattbaconz/plugdev --skill plugdev` |
| Docs | https://pluglabs.app/plugdev |

Optional MCP: `plugdev agent install --mcp` writes `.cursor/mcp.json` and `.mcp.json`. Prefer MCP tools for headless control when configured; otherwise use CLI `--json`.
