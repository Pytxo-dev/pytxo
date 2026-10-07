# AGENTS.md — Pytxo

Instructions for AI coding agents working in this repository.

## Project identity

**Pytxo is an agent hypervisor** ([pytxo.com](https://pytxo.com)): a vendor-neutral
control plane for preparing, running, observing, verifying, and integrating
delegated work. Its reviewed repository Apply boundary is a shipping primitive;
generalized effect contracts remain a proposed expansion. It schedules headless agents
(Claude Code, Codex, Antigravity CLI, …) in background PTYs, isolates their
work, and prepares exact bytes for reviewed Apply. Source:
[github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo) (monorepo:
`crates/*` + `apps/desktop`).

**Stack:** Rust (tokio, portable-pty, tree-sitter) · Svelte 5 (Runes) · Tauri v2

**Execution yard:** `portable-pty` default ([[ADR-0010-pty-default-execution-backend]]); `execution_backend = "subprocess"` for CI fallback.

**Canonical vision:** [`docs/06-product/vision.md`](docs/06-product/vision.md)

**Not:** BridgeSpace-style multi-terminal web workspaces (ADEs), a proprietary
model router, generic workflow engine, or universal undo layer. **Is today:** a
bare-metal execution yard plus reviewed repository commit boundary and optional
**Pytxo Desktop** for Work, History, and Setup, carrying the run ledger,
enforcement receipts, approvals, and review. **North star:** typed effect
contracts at the boundary to production
systems ([commit layer](docs/02-areas/orchestration/commit-layer.md)). The MCP
hub remains available to agents and editors.

**Current control primitives** — route new orchestration code through these
concepts; future differentiation comes from effect semantics, evidence, and
recovery depth:

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
| Commit layer | [`docs/02-areas/orchestration/commit-layer.md`](docs/02-areas/orchestration/commit-layer.md) | Proposed effect contract, prepare/commit, verification, evidence, and recovery boundary |

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
| `apps/web/` | Next.js marketing site + public Fumadocs documentation |
| `apps/web/content/docs/` | Public user docs (Fumadocs MDX) — served at pytxo.com/docs from the Next app |
| `apps/demo-video/` | Remotion product demo source, real Desktop captures, and ElevenLabs script |
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
cd apps/desktop
npm ci
npm run check
npm run build:native  # builds the frontend and release executable, not an installer
npm run build:msi     # Windows MSI with voice support and the distribution CRT policy
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
| **Commit layer** | Boundary that authorizes, commits, verifies, and records consequential agent effects |
| **Effect contract** | Typed commitment covering identity, state, authority, evidence, and recovery |
| **Signal Core** | Smarter context via AST skeletons on read |
| **Blast Shield** | Safe sandbox; flush on user approve |
| **Race Shield** | Global swarm registry; collision-free writes |
| **Execution Yard** | Headless CLI agent processes under orchestration |
| **Pytxo Desktop** | Optional control UI with three destinations — Work, History, Setup — plus a title-bar workspace switcher and an approvals overlay; 3D topology is legacy/secondary (formerly Reality Deck) |
| **Epistemic state** | The four states every rendered state resolves to: verified, claimed, unknown, refuted ([`ADR-0038`](docs/05-adr/ADR-0038-epistemic-state-contract.md)) |
| **Enforcement receipt** | `PermissionEnforcementReceipt`: four isolation surfaces, each with a status and mechanism, per run and per agent |
| **Workspace** | One or more project folders under one coordinated run |
| **BYOK** | Bring your own API keys (including Pytxo Cloud) |
| **Sovereign Shield** | Sanitization + cryptographic remote actions |
| **Permission profile** | Local trust tier: DeepSpace, Orbit (default), Galaxy, Supernova |
| **Execution domain** | One repo root’s isolated run/registry/store slice |
| **Hypervisor registry** | Multi-project map of active execution domains |

Full glossary: [`docs/00-meta/glossary.md`](docs/00-meta/glossary.md).

## Competitive framing

IDE-embedded agent teams solve coordination in-product. Pytxo's current proof
is a vendor-neutral, reviewed repository Apply over its Signal / Blast / Race
controls. Its proposed wider differentiation is effect-bound authority,
independent post-state verification, causal evidence, and honest recovery — see
[`docs/06-product/vision.md`](docs/06-product/vision.md) and
[`docs/01-projects/pytxo-commit-layer-alignment.md`](docs/01-projects/pytxo-commit-layer-alignment.md).

## Cursor-specific

Scoped rules live in [`.cursor/rules/`](.cursor/rules/). Root [`.cursorrules`](.cursorrules) is a deprecated shim only.

## Claude Code

See [`CLAUDE.md`](CLAUDE.md) and [`docs/00-meta/CLAUDE.md`](docs/00-meta/CLAUDE.md).
