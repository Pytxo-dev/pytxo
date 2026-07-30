---
title: Pytxo architecture research synthesis
slug: pytxo-architecture-research
status: active
tags: [project, research, architecture, synthesis]
audience: [human, agent]
layer: meta
created: 2026-07-27
updated: 2026-07-27
related: [[MOC-home]], [[product-vision]], [[architecture-index]], [[three-tier-model]], [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains]], [[adr-index]], [[repository-layout]], [[pytxo-improvement-research]], [[glossary]]
---

# Pytxo architecture research synthesis

Primary-source map of what Pytxo is, how it is layered, what each crate owns, and where documentation and code currently disagree. Sources are repo-local only: the `docs/` vault, [`AGENTS.md`](../../AGENTS.md), ADRs in `docs/05-adr/`, `Cargo.toml`, and crate/app source trees. Longer than an atomic note by design — see [[style-guide]] for the atomic-note rule this synthesis intentionally sits outside of, alongside [[pytxo-improvement-research]].

## 1. Executive summary

Pytxo is a **local agent hypervisor and telemetry plane**: a Rust control plane that schedules headless coding-agent CLIs (Claude Code, Codex, Antigravity CLI, …) inside managed background pseudo-terminals, keeps them from colliding on writes, compresses the code context they read, and holds their file writes in an approval-gated bubble until a human flushes them ([`AGENTS.md`](../../AGENTS.md), [[product-vision]]). It is explicitly **not** an ADE terminal wall and not a single-vendor agent desktop ([[beyond-the-ade]]). The shipping surfaces are a CLI (`crates/pytxo-cli`), a stdio MCP server (`crates/pytxo-mcp`), and an optional Svelte 5 + Tauri v2 control UI (`apps/desktop`), all of which stop at the `pytxo-orchestrate` boundary by design ([[repository-layout]], [[ADR-0001-three-tier-rust-svelte-tauri]]).

## 2. Product thesis and non-goals

**Thesis** ([[product-vision]], [[MOC-home]]): coordinate the agents a developer already runs, on local silicon, with structural telemetry rather than cloud-heavy multi-terminal workspaces. Category framing in [[agent-os-vs-virtual-workspace]] is "bare-metal control plane, not a replacement IDE": IDE-agnostic via a local-first MCP hub, lean footprint, throughput-first, multi-repo.

**Productivity model** has two layers ([[product-vision]], [[modular-projects]]):

1. Many projects at once — each repo root is an [[execution-domains|execution domain]] with its own scheduler, registry, WAL, and [[permission-profile-engine|permission profile]] (default Orbit).
2. Workspaces — one project may attach **multiple path roots** (API repo + web repo + shared protos) under a single coordinated run.

**Declared non-goals** ([[product-vision]]): multi-pane embedded terminal walls in the product UI; storing provider keys in plaintext; duplicating IDE editing surfaces; claiming unique multi-agent / unique sandbox / invented worktrees against 2026 vendor products. [[pytxo-improvement-research]] adds cycle-scoped non-goals: no Copilot Agent Merge or Cursor cloud artifact parity as shipping claims, no Seatbelt/Landlock/bwrap parity claims before code exists, no lock-free Race rewrite before contention profiling, no Firecracker/gVisor multi-tenant near term ([[ADR-0025-cloud-runtime-isolation]] defers it).

## 3. Layered architecture

[[three-tier-model]] (ADR: [[ADR-0001-three-tier-rust-svelte-tauri]]) defines a strictly event-driven three-tier layout:

| Tier | Contents | Boundary rule |
|------|----------|---------------|
| **Presentation** | Svelte 5 Runes + Tauri v2; Desktop 2 Focus / Ops / Flow / Approvals | No direct filesystem access; IPC intents only ([[presentation-passive-telemetry]]) |
| **Orchestration** | Rust microkernel: PTYs, Signal Core, Blast Shield, Race Shield, permission profiles, execution domains, DAG scheduling, SQLite WAL, sanitize | Sole location of policy enforcement ([[ADR-0008-local-permission-profile-four-tiers]] §4) |
| **Execution yard** | Headless CLI agent processes, reached through the MCP hub | Every spawn registers with Race Shield before `exec` ([[race-shield]]) |

Cloud is a fourth, optional path attached to orchestration over TLS/P2P ([[hybrid-execution]]). C4 views: [[context-diagram]] (L1) and [[c4-container]] (L2).

The presentation contract is concrete: the UI sends intents (`list_domains`, `tail_events(domain_id)`, `dispatch`, `commit_workspace`, `hitl_respond`) and orchestration validates them against the active profile before touching disk ([[presentation-passive-telemetry]], [[execution-domains]]).

## 4. The three moats

[`AGENTS.md`](../../AGENTS.md) instructs that new orchestration code route **through** these concepts, not around them.

| Moat | Plain language | Responsibility | Owning code | Doc |
|------|----------------|----------------|-------------|-----|
| **Signal Core** | Smarter context | `tree-sitter` AST skeletons on read: signatures, types, imports, module shape | `crates/pytxo-signal` (`language.rs`, `emit.rs`, `graph.rs`), `pytxo-core::moat::signal` | [[signal-core]] |
| **Blast Shield** | Safe sandbox until approve | Per-agent worktree or sparse copy-layer; flush to physical disk only on approval | `crates/pytxo-runner` (`blast.rs`, `overlay_projfs.rs`, `overlay_fuse_linux.rs`, `overlay_fuse_macos.rs`), `pytxo-core::moat::blast` | [[blast-shield]] |
| **Race Shield** | No write collisions | `SwarmRegistry` path claims + stdin buffering; Galaxy HITL queue | `crates/pytxo-runner` (`race.rs`, `hitl.rs`, `hitl_gate.rs`), `pytxo-core::moat::race` | [[race-shield]] |

**Signal Core.** Intercepts file reads and emits structural skeletons before context leaves the machine. The ~60% input-token reduction is documented as **aspirational**; the pinned measurement is **4.76%** on a tiny fixture (`tests/fixtures/tiny-monorepo/src/a.ts`, 21→20 bytes) via `tooling/benchmarks/signal-reduction.ps1` ([[signal-core]], [[competitive-benchmarks]]). Code confirms 9 grammar ids across 8+ languages: Rust, TypeScript/TSX, JavaScript, Python, Go, Java, C, C++, Ruby (`crates/pytxo-signal/src/language.rs`); unknown extensions fall back to raw bytes.

**Blast Shield.** Model is physical disk ← flush-on-approve ← per-agent worktree or copy-layer upper ← PTY agents. Shipping since Phase 69 is git worktrees plus a sparse **copy-layer** default via `prefer_kernel_overlay`, labeled `projfs-sparse-copy-v2` on Windows — explicitly *not* a kernel ProjFS provider and *not* an mmap CoW filesystem ([[blast-shield]], [[sparse-overlay-fs]], [[mvp-bootstrap]] Phase 69). Flush semantics are fixed by [[ADR-0017-overlay-flush-contract]]: copy-layer flush recursively copies the upper tree into `repo_root`; worktree backends use `merge_agent_branch`.

**Race Shield.** `SwarmRegistry` splits locks — `RwLock` over path claims, `Mutex` over the stdin buffer (Phase 70) — so PTY stdin pumps do not serialize against disjoint claim waves. Lock-free or path-prefix shards are deferred until `registry_contention_*` profiling justifies them ([[race-shield]]). Galaxy profiles route high-risk actions to a per-domain `HitlQueue` persisted to `{data_dir}/hitl.json` ([[ADR-0016-galaxy-hitl-enforcement]]).

## 5. Policy and hypervisor

**Permission Profile Engine** ([[permission-profile-engine]], [[ADR-0008-local-permission-profile-four-tiers]]) is the orchestration-layer facade gating read/write/exec/network before any PTY command or MCP tool runs. Four tiers, with `Orbit` as the serde default:

| Profile | Reads | Writes | Network | Primary moat binding |
|---------|-------|--------|---------|----------------------|
| **DeepSpace** (1) | Agent cwd only | Denied or ephemeral | Blocked | Signal read-only scaffold; fidelity capped to Low |
| **Orbit** (2, default) | Full repo | CoW/worktree; flush via IPC approve | Default deny (allowlist TBD) | Blast + Race + Signal |
| **Galaxy** (3) | Full repo | Physical writes; HITL for destructive/out-of-root | Local service ports | Race HITL + optional Blast |
| **Supernova** (4) | Full repo | Unrestricted | Full | Race advisory only |

The type lives in `crates/pytxo-core/src/moat/permission/mod.rs` with `filesystem.rs`, `network.rs`, and `environment.rs` sub-policies; `PermissionEngine` exposes `max_fidelity`, `flush_requires_approval`, `may_flush`, `may_read`, `use_worktree_isolation`, `spawn_egress_allowed`, and env sanitization. `ProcessPolicy` is documented as **deferred to Phase 75** and is correctly absent from the crates (verified: no matches in `crates/`).

Network enforcement is layered across three ADRs: spawn-time egress classification for Orbit/DeepSpace ([[ADR-0018-network-and-mcp-policy]]), per-platform DeepSpace hooks ([[ADR-0022-deepspace-network-namespace]]), and Linux `unshare -n` default-on with a doctor socket probe ([[ADR-0026-deepspace-netns-default]]). Windows WFP remains the weakest link ([[deepspace-network-v2]]).

**Execution domains and hypervisor registry** ([[execution-domains]]). `DomainId` is a hash of the canonical repo root; each `ExecutionDomain` owns its own `PytxoStore`, `SwarmRegistry`, and run handle, and never shares them. `HypervisorRegistry` (implemented in `crates/pytxo-orchestrate/src/hypervisor.rs`) provides `ensure_domain`, `run_blocking`, `dispatch`, `list_domains`. A global `~/.pytxo/hypervisor.db` catalog (`pytxo-store::Catalog`, `crates/pytxo-store/src/catalog.rs`) records where each per-domain database lives — **streams are never merged**. Cross-repo ordering uses an explicit fleet manifest rather than implicit multi-repo `depends_on` ([[hypervisor-fleet-dag]], [[ADR-0015-hypervisor-fleet-dag]]); `crates/pytxo-orchestrate/src/fleet.rs` and `pytxo-core/src/fleet.rs` back it.

Folder trust is a separate gate: trust is persisted per canonical root in `~/.pytxo/trusted-domains.json`, `run`/`dispatch` are blocked until trusted, and the trusted tier **overrides** `permission_profile` from config ([[ADR-0013-folder-trust-tier-picker]]; `crates/pytxo-core/src/trust.rs`).

## 6. Context and fidelity pipeline

Pytxo has no chat or LLM-memory layer in the control plane — only **materialized code context** ([[context-launch-contract]]).

1. **Scaffold.** `prepare_agent_context` (`crates/pytxo-runner/src/context.rs`) globs task paths, scaffolds each file through Signal Core at the effective fidelity, and writes copies under `.pytxo/data/context/{run_id}/{agent_id}/` with a `manifest.json` index.
2. **Hand off.** `ChildLaunchEnv::with_context_dir` (`crates/pytxo-core/src/child_env.rs`) sets `PYTXO_CONTEXT_DIR` and `PYTXO_SIGNAL_CORE=1` identically on PTY and subprocess backends. The manifest carries `source`, `scaffolded`, `token_reduction_pct`, `fallback_raw`, `fidelity`, `root_id`, `bytes_scaffolded`, and is append-only.
3. **Tier.** Effective fidelity is `min(config signal_fidelity, PermissionEngine::max_fidelity)`; Low/Medium emit AST skeletons, High copies full bytes; DeepSpace caps to Low ([[adaptive-semantic-scaffolding]]).
4. **Escalate.** On validation failure the runner detects implicated symbols, raises fidelity for just those paths, rewrites the manifest at `fidelity: "high"`, and re-spawns ([[closed-loop-fidelity]]; graph-neighbor escalation added in [[mvp-bootstrap]] Phase 31; `crates/pytxo-runner/src/failure.rs`).
5. **Schedule.** Tasks become a DAG with topological waves, cycle detection, and mock-state injection behind `PYTXO_DAG_MOCK` ([[dag-flow-engine]], [[ADR-0004-dag-scheduler-over-sequential-locks]], [[ADR-0007-explicit-task-dependencies]]; `crates/pytxo-scheduler/src/{dag,waves,overlap}.rs`).
6. **Persist.** Terminal output, token metrics, IPC events, and diff history land in per-domain SQLite WAL, keeping orchestration heap bounded ([[sqlite-wal-logging]], [[ADR-0002-sqlite-wal-session-telemetry]]).

Execution backend default is `portable-pty`, with `execution_backend = "subprocess"` as a CI fallback and a stdin pump draining `SwarmRegistry` queues into the PTY master writer ([[ADR-0010-pty-default-execution-backend]]; `crates/pytxo-runner/src/pty.rs`).

## 7. Surfaces

**CLI** (`crates/pytxo-cli`). `pytxo` with no subcommand opens the Hypervisor Shell — board, scrollback, operator prompt — with slash commands routed through `pytxo-shell`; Pytxo deliberately does not embed LLM chat, and the optional NL planner is feature-gated ([[ADR-0012-hypervisor-shell-default-ux]]). Actual subcommands in `crates/pytxo-cli/src/main.rs`: `init`, `doctor`, `run`, `agents`, `shell`, `status`, `logs`, `stop`, `project`, `hitl`, `domains`, `fleet`, `providers`, `trust`, `models`. Per [`crates/pytxo-cli/AGENTS.md`](../../crates/pytxo-cli/AGENTS.md) the crate is a thin wiring layer: no scheduling logic, no git/process code, no SQL.

**MCP hub** (`crates/pytxo-mcp`). Stdio MCP server exposing 13 tools (`crates/pytxo-mcp/src/main.rs`): `pytxo_dry_run`, `pytxo_run`, `pytxo_status`, `pytxo_logs`, `pytxo_read`, `pytxo_read_scaffolded`, `pytxo_stdin`, `pytxo_route_stdin`, `pytxo_list_live_agents`, `pytxo_mcp_proxy`, `pytxo_mcp_tools_list`, `pytxo_project_run`, `pytxo_fleet_run`, `pytxo_fleet_status`. Read tools scaffold on demand through the same Signal Core; `mcp_proxy_call` passes through a Galaxy `mcp.tool` HITL gate ([[ADR-0018-network-and-mcp-policy]]). Runtime registry and resource subscriptions live in `crates/pytxo-runner/src/mcp_hub.rs` ([[mvp-bootstrap]] Phase 67).

**Desktop** (`apps/desktop`, crate `pytxo-desktop`). Default surface is **Desktop 2**: a structural Focus graph (`FocusScreen.svelte`) plus Ops, Flow, Approvals, and Run Review; the interactive 3D AST topology (`TopologyScene3D.svelte`) is legacy-shell only behind `desktop_shell_v1=true` ([[desktop-visual-system]], [[presentation-passive-telemetry]], [[ADR-0028-desktop-product-name]]). Visual language: obsidian void `#020205` with teal / violet / solar gold accents, shared through `packages/chroma` ([[ADR-0029-chroma-shared-design-tokens]]). Phase 74 shipped live ~1s Ops polling, a selectable Approvals inbox, domain breadcrumbs, and a thin Fleet panel from `snapshot.fleets` ([[market-ready-polish-research]], [[mvp-bootstrap]]).

**Public docs** (`apps/web`). Fumadocs MDX inside the Next.js App Router at `/docs`, sourced from `apps/web/content/docs/**/*.mdx` (38 pages), with Docusaurus retired from the deploy path ([[ADR-0030-public-docs-fumadocs-next]]).

**Cloud** (`services/*`). Three Rust services are workspace members: `pytxo-link` (entitlements + run ledger), `pytxo-cloud-sandbox` (sandbox API), `pytxo-proxy` (Ultra managed inference). Sandboxes are hardened containers on an egress-deny-by-default internal Docker network with fail-closed `/exec` ([[ADR-0025-cloud-runtime-isolation]], [[ADR-0020-cloud-sandbox-runtime]]); all three expose a common `/health` contract ([[ADR-0027-service-observability-contract]]). Crucially, the default orchestration path uses `NoopCloudDispatcher`, so Cloud is **capability-gated, not GA** ([[hybrid-execution]]; `crates/pytxo-orchestrate/src/cloud.rs`). Flow when configured: [[sandbox-dispatch]] → [[delta-sync]] → Desktop.

**Billing.** Ultra defines `TokenWallet`, `UsageMeter`, `TokenEstimator`, `ModelRouter`, `ManagedTransport` in `pytxo-core` with migration-003 tables in `pytxo-store` ([[ADR-0009-ultra-managed-metering]]; `crates/pytxo-core/src/billing/*`, `crates/pytxo-store/src/billing.rs`). The dual ledger is `tokens_in_billed` (raw files) vs `tokens_in_sent` (scaffolded egress), with `saved_tokens` as arbitrage yield ([[token-arbitrage]]). Link Postgres is the authoritative run ledger keyed by `run_id` ([[ADR-0021-billing-source-of-truth]]); endpoints split between `billing.proxy_url` and `billing.inference_proxy_url` ([[ADR-0024-managed-transport-endpoint-split]], [[ADR-0019-ultra-managed-inference-proxy]]). The default reconciler is `NoopBillingReconciler`, so the shipping honest claim is a **local ledger** ([[tiers-hobbyist-pro-max]]).

## 8. Security — Sovereign Shield and BYOK

Sovereign Shield is sanitization plus cryptographic remote actions ([[glossary]]).

- **Sanitization.** `pytxo-sanitize` applies built-in regex rules (API keys, bearer tokens, home paths), gated by `sanitize = true` (default on), applied before `append_event` in orchestration and before MCP log payloads ([[ADR-0006-sovereign-shield-sanitize-pipeline]], [[regex-sanitization]]). The ADR is honest that this is best-effort regex, not semantic secret detection. Redaction is explicitly **not** authorization ([[permission-profile-engine]]).
- **Remote approvals.** Pytxo Link signs critical actions; the host verifies against a local trust store before the `portable-pty` stdin pipe is unlocked for the approved command stream ([[pytxo-link-signing]]).
- **BYOK.** Provider keys stay in OS environment variables; `pytxo providers` reports only set/missing, never values ([[ADR-0014-multi-provider-byok-catalog]]). `ProviderRegistry` lives in `pytxo-core`, with `crates/pytxo-catalog` handling model fetch/cache/search. On the Ultra path, provider keys live only on `pytxo-proxy` and clients authenticate with `PYTXO_ULTRA_SESSION` ([[ADR-0019-ultra-managed-inference-proxy]], [[pytxo-toml]]).
- **Repo policy.** Never commit keys; examples must reflect sanitization patterns ([`AGENTS.md`](../../AGENTS.md) Security).

## 9. ADR index summary

All 30 ADRs in `docs/05-adr/` are **accepted**; the catalog states they are immutable once accepted and must be superseded rather than edited ([[adr-index]]).

| ID | Decision in one line |
|----|----------------------|
| [[ADR-0001-three-tier-rust-svelte-tauri]] | Presentation (Svelte 5 + Tauri v2, IPC-only) / Orchestration (Rust) / Execution yard |
| [[ADR-0002-sqlite-wal-session-telemetry]] | Persist output, token metrics, IPC events, diffs to local SQLite in WAL mode |
| [[ADR-0003-sparse-overlay-not-ram-cow]] | Sparse overlay VFS (FUSE/ProjFS) as the target; reject copying dep trees into RAM |
| [[ADR-0004-dag-scheduler-over-sequential-locks]] | Async DAG engine with cycle detection instead of one sequential lock queue |
| [[ADR-0005-worktree-isolation-for-mvp]] | Git worktrees under `.pytxo/worktrees/{run_id}/{agent_id}` for Phase 1 isolation |
| [[ADR-0006-sovereign-shield-sanitize-pipeline]] | `pytxo-sanitize` regex redaction before WAL append and MCP payloads |
| [[ADR-0007-explicit-task-dependencies]] | `depends_on` on tasks; topological waves; cycles error unless `PYTXO_DAG_MOCK=1` |
| [[ADR-0008-local-permission-profile-four-tiers]] | `PermissionProfile` ladder (DeepSpace/Orbit/Galaxy/Supernova); domains under a hypervisor registry |
| [[ADR-0009-ultra-managed-metering]] | Ultra wallet/meter/transport traits, migration-003 tables, Noop reconciler until Link exists |
| [[ADR-0010-pty-default-execution-backend]] | `portable-pty` default; subprocess fallback; stdin pump; shared `ChildLaunchEnv` |
| [[ADR-0011-modular-project-manifest]] | TOML project manifest with `[project]` + `[[roots]]` for multi-path workspaces |
| [[ADR-0012-hypervisor-shell-default-ux]] | Bare `pytxo` opens the Hypervisor Shell; slash commands; no embedded LLM chat |
| [[ADR-0013-folder-trust-tier-picker]] | Per-root trust in `~/.pytxo/trusted-domains.json`; run/dispatch blocked until trusted |
| [[ADR-0014-multi-provider-byok-catalog]] | `ProviderRegistry`, expanded `ProviderId`, `pytxo-catalog`, keys stay in OS env |
| [[ADR-0015-hypervisor-fleet-dag]] | Fleet manifest of cross-repo nodes with barrier sync between waves |
| [[ADR-0016-galaxy-hitl-enforcement]] | Spawn-time risk classification; per-domain `HitlQueue` persisted to `hitl.json` |
| [[ADR-0017-overlay-flush-contract]] | Copy-layer flush copies upper into `repo_root`; worktrees still merge branches |
| [[ADR-0018-network-and-mcp-policy]] | Orbit/DeepSpace spawn egress deny; Galaxy package-install and `mcp.tool` HITL gates |
| [[ADR-0019-ultra-managed-inference-proxy]] | Dedicated `services/pytxo-proxy` holding provider keys; per-provider routes |
| [[ADR-0020-cloud-sandbox-runtime]] | Fail-closed `/exec` (503 without a live container); tar+`docker cp` delta sync; teardown |
| [[ADR-0021-billing-source-of-truth]] | Link Postgres `runs` is authoritative; local wallet is cache; `run_id` idempotency |
| [[ADR-0022-deepspace-network-namespace]] | Per-platform `isolate_deepspace_network` hook (Linux netns, macOS `sandbox-exec`) |
| [[ADR-0023-reality-deck-3d-renderer]] | Three.js `TopologyScene3D` as the topology renderer with the 3D center viewport |
| [[ADR-0024-managed-transport-endpoint-split]] | Split `billing.proxy_url` (Link) from `billing.inference_proxy_url` (Ultra proxy) |
| [[ADR-0025-cloud-runtime-isolation]] | Hardened containers, `--internal` egress deny by default, optional Redis scaffold cache |
| [[ADR-0026-deepspace-netns-default]] | `unshare -n` default-on for DeepSpace on Linux; doctor socket probe; Windows WFP stub |
| [[ADR-0027-service-observability-contract]] | Common `/health` JSON contract across the three services |
| [[ADR-0028-desktop-product-name]] | Product name is Pytxo Desktop (not Reality Deck); ADR-0023 technical choice unchanged |
| [[ADR-0029-chroma-shared-design-tokens]] | Extract `packages/chroma` shared tokens (renumbered from a duplicate ADR-0014) |
| [[ADR-0030-public-docs-fumadocs-next]] | Public docs move to Fumadocs MDX in `apps/web`; retire Docusaurus from deploy |

## 10. Repository and crate map (docs vs code)

`Cargo.toml` declares 18 workspace members. Crate ownership, cross-checked against source trees:

| Crate | Owns | Evidence |
|-------|------|----------|
| `pytxo-core` | Config, plan/task types, ids, path utils, `moat::{permission,blast,race,signal}`, `billing/*`, `cloud/*`, `child_env`, `trust`, `ade_registry`, `service_health` | `crates/pytxo-core/src/**` |
| `pytxo-scheduler` | DAG construction, waves, path-overlap preflight | `dag.rs`, `waves.rs`, `overlap.rs` |
| `pytxo-runner` | PTY/subprocess spawn, Blast overlays, Race registry, HITL gates, MCP hub, context materialization, arbitrage, network isolation, kill/process registry | `pty.rs`, `blast.rs`, `race.rs`, `hitl*.rs`, `mcp_hub.rs`, `context.rs`, `arbitrage.rs`, `network_isolation.rs` |
| `pytxo-store` | SQLite schema/migrations, per-domain store, hypervisor catalog, project store, billing tables | `schema.rs`, `migrate.rs`, `catalog.rs`, `project_store.rs`, `billing.rs` |
| `pytxo-sanitize` | Regex redaction rules | `lib.rs` |
| `pytxo-signal` | tree-sitter language detection, skeleton emit, structural graph | `language.rs`, `emit.rs`, `graph.rs` |
| `pytxo-orchestrate` | Shared run/stop/status/dry-run, hypervisor registry, fleet, projects, billing, cloud dispatch, doctor, dashboard, entitlements, structural snapshots | `hypervisor.rs`, `fleet.rs`, `project.rs`, `billing.rs`, `cloud.rs`, `doctor.rs`, `dashboard.rs` |
| `pytxo-cli` | Argument parsing and wiring only | `main.rs`, `commands.rs`, `models.rs` |
| `pytxo-tui` | Hypervisor Shell TUI, theme, input | `shell_app.rs`, `theme.rs`, `input.rs` |
| `pytxo-shell` | Slash-command grammar/session shared by TUI and other surfaces | `command.rs`, `eval.rs`, `session.rs` |
| `pytxo-planner` | Feature-gated NL → `Vec<Task>` planner | `lib.rs` |
| `pytxo-voice` | Voice capture (Desktop `native-capture` feature) | `lib.rs` |
| `pytxo-catalog` | Provider model fetch / cache / search | `fetch.rs`, `cache.rs`, `search.rs` |
| `pytxo-mcp` | Stdio MCP server binary | `main.rs` |
| `apps/desktop/src-tauri` | `pytxo-desktop` Tauri app | workspace member |
| `services/pytxo-link`, `services/pytxo-cloud-sandbox`, `services/pytxo-proxy` | Commercial/cloud services | workspace members |

Also present but outside the Cargo workspace: `packages/pytxo` (npm installer), `packages/chroma` (design tokens), `apps/web` (Next.js site + public docs), `apps/docs` (legacy Docusaurus tree), `apps/desktop-export`, `tooling/scripts`, `tooling/benchmarks`, `tests/fixtures`.

The documented dependency spine is `pytxo-core → {scheduler, store, sanitize, signal, runner} → orchestrate → {cli, mcp}`, with a stated **Desktop rule**: Tauri may depend on `pytxo-core`, `pytxo-store`, and `pytxo-orchestrate` only ([[repository-layout]]). See §13 — the code does not currently satisfy that rule.

## 11. Canonical glossary

Condensed from [[glossary]] and the product-language table in [`AGENTS.md`](../../AGENTS.md):

| Term | Meaning |
|------|---------|
| **Agent hypervisor** | Pytxo's role: run, schedule, isolate, and meter headless coding agents on PTYs |
| **Signal Core** | Smarter context — tree-sitter skeletons on read |
| **Blast Shield** | Safe sandbox until approve; disk flush on approve |
| **Race Shield** | No write collisions — swarm registry plus stdin buffering |
| **Execution yard** | Headless CLI agent processes under orchestration |
| **Execution domain** | One canonical repo root's isolated scheduler, registry, runner, and WAL |
| **Hypervisor registry** | Orchestration map of active execution domains |
| **Permission profile** | Local trust ladder: DeepSpace, Orbit (default), Galaxy, Supernova |
| **Workspace / Pytxo project** | Modular project: one or more path roots under one coordinated run |
| **Path root** | One directory on a project's allowlist; tasks and claims resolve relative to it |
| **Pytxo Desktop** | Optional control UI — Desktop 2 structural Focus; 3D topology is legacy-shell only |
| **Sovereign Shield** | Sanitization plus cryptographic remote approvals |
| **Sparse overlay FS** | Shipping sparse copy-layer (plus worktrees); kernel FUSE/ProjFS is north star |
| **Adaptive Semantic Scaffolding** | Fidelity tiers (Low/Medium/High) inside Signal Core |
| **Blast radius** | Structural footprint of an agent's edits on the AST graph |
| **Token arbitrage** | Difference between tokens billed on raw files and tokens actually sent |
| **MCP hub** | Local-first Model Context Protocol router |
| **BYOK** | Bring your own LLM API keys |
| **ADE** | Agentic development environment — UI-heavy multi-agent IDEs |

Note the deliberate disambiguation enforced by [[ADR-0008-local-permission-profile-four-tiers]]: *tier* means three different things — architecture layer, Signal `FidelityTier`, and subscription tier — so `PermissionProfile` is the only name for local trust.

## 12. Competitive framing

Primary-source position, from [[beyond-the-ade]], [[agent-os-vs-virtual-workspace]], [[pytxo-vs-claude-agent-teams]], and the landscape table in [[pytxo-improvement-research]]:

- Against **ADE terminal walls** (BridgeSpace-style): those treat multi-agent work as a layout problem, burning RAM/GPU and optimizing for demos over throughput. Pytxo rejects the virtual team room as the primary abstraction.
- Against **vendor agent desktops** (GitHub Copilot app): those are control centers with worktrees and per-session sandboxes inside one ecosystem. Pytxo's job is cross-CLI local orchestration for agents the developer already runs.
- Against **Claude Code Agent Teams**: Claude's own docs state teammates get no worktree isolation (same-file edits can overwrite) and that teams use significantly more tokens. Race Shield claims plus Blast worktree/copy-layer is the direct answer, and Pytxo can run Claude Code as an execution-yard process.
- Against **cloud VM agents** (Cursor Cloud Agents, Jules): explicitly *not* raced as near-term shipping claims.

[[pytxo-improvement-research]] fixes the claim boundary precisely. **Safe:** cross-CLI hypervisor; Race vs Claude's documented overwrite gap; Signal as cost control; local-first and BYOK on local surfaces. **Unsafe:** unique multi-agent; unique sandbox; invented worktrees; BYOK everywhere; Landlock as Pytxo's Linux story. [[competitive-benchmarks]] adds the measurement rule: cite the Phase 73 pin (Ryzen 7 7735HS / Windows 11 / rustc 1.95.0, pinned 2026-07-18) and never cite blocked cells.

## 13. Open gaps and doc↔code drift

Evidence-based, found while cross-checking this synthesis. These are observations, not edits.

1. **Desktop violates the documented dependency rule.** [[repository-layout]] states Tauri may depend on `pytxo-core`, `pytxo-store`, and `pytxo-orchestrate` only — not `pytxo-runner` or `pytxo-scheduler`. `apps/desktop/src-tauri/Cargo.toml` lines 33–38 declare path dependencies on `pytxo-runner`, `pytxo-signal`, and `pytxo-voice` in addition to the three allowed crates. Either the rule or the manifest needs to change.
2. **CLI reference is materially incomplete.** [[cli-reference]] documents `init`, `doctor`, `run`, `status`, `logs`, `stop`, `trust`, `providers`, `models`. `crates/pytxo-cli/src/main.rs` also implements `agents`, `shell`, `project`, `hitl`, `domains`, and `fleet` — six shipped commands with no entry in the reference table.
3. **Repository layout omits the services and packages that exist.** [[repository-layout]] lists no `services/*`, even though `Cargo.toml` makes `pytxo-link`, `pytxo-cloud-sandbox`, and `pytxo-proxy` workspace members; it also omits `packages/chroma` (required by [[ADR-0029-chroma-shared-design-tokens]]) and the still-present legacy `apps/docs` tree that [[ADR-0030-public-docs-fumadocs-next]] retired. The repo map in [`AGENTS.md`](../../AGENTS.md) has the same omissions and lists only 5 of 14 crates by name.
4. **Fixed:** [[ADR-0032-desktop-2-focus-flow-primary]] now supersedes the
   primary-viewport portion of [[ADR-0023-reality-deck-3d-renderer]]. Desktop 2
   structural Focus is the recorded default; 3D remains legacy-shell-only.
5. **Two ADRs are missing required frontmatter.** [[ADR-0006-sovereign-shield-sanitize-pipeline]] and [[ADR-0007-explicit-task-dependencies]] begin directly at the `#` heading with no YAML block, against [[style-guide]]. Separately, ADR-0009 and ADR-0019 through ADR-0027 carry `title`/`status` but no `adr_id` field.
6. **Fixed 2026-07-29:** the mismatch table in
   [[pytxo-improvement-research]] now marks the corrected 3D, copy-layer,
   Ultra/Cloud tier, and hybrid-execution claims as fixed. `ProcessPolicy`
   remains accurately open and absent from `crates/`.
7. **MCP hub described as stateless.** [[mcp-hub-integration]] calls the hub "stateless, local-first", but `crates/pytxo-runner/src/mcp_hub.rs` maintains a per-domain session registry with resource subscriptions (`subscribe_resource`, `notify_resource_updated`, [[mvp-bootstrap]] Phase 67). The note also still refers to [[mcp-router]] as a future implementation reference.
8. **Fixed 2026-07-29:** [[c4-container]] and [[context-diagram]] now draw a
   dashed, configured-only Cloud edge and identify the default
   `NoopCloudDispatcher` path.
9. **Primary surfaces remain untested.** [[pytxo-improvement-research]] P3 and [[mvp-bootstrap]] Phase 76 both record zero dedicated integration tests for `pytxo-cli` and `pytxo-mcp`; nothing in the crate trees contradicts that.

## 14. Source index

Read for this synthesis (paths relative to repo root):

1. `AGENTS.md`
2. `crates/pytxo-cli/AGENTS.md`
3. `Cargo.toml`
4. `docs/00-meta/MOC-home.md`
5. `docs/00-meta/glossary.md`
6. `docs/00-meta/style-guide.md`
7. `docs/00-meta/changelog-docs.md`
8. `docs/06-product/vision.md`
9. `docs/06-product/tiers-hobbyist-pro-max.md`
10. `docs/06-product/token-arbitrage.md`
11. `docs/06-product/modular-projects.md`
12. `docs/06-product/competitive-benchmarks.md`
13. `docs/04-architecture/index.md`, `three-tier-model.md`, `c4-container.md`, `context-diagram.md`, `presentation-passive-telemetry.md`, `desktop-visual-system.md`, `mcp-hub-integration.md`
14. `docs/02-areas/orchestration/signal-core.md`, `blast-shield.md`, `race-shield.md`, `execution-domains.md`, `hypervisor-fleet-dag.md`, `dag-flow-engine.md`, `sqlite-wal-logging.md`, `sparse-overlay-fs.md`, `adaptive-semantic-scaffolding.md`, `closed-loop-fidelity.md`, `context-launch-contract.md`
15. `docs/02-areas/security/permission-profile-engine.md`, `regex-sanitization.md`, `pytxo-link-signing.md`, `deepspace-network-v2.md`
16. `docs/02-areas/cloud/hybrid-execution.md`, `sandbox-dispatch.md`, `delta-sync.md`
17. `docs/02-areas/positioning/beyond-the-ade.md`, `agent-os-vs-virtual-workspace.md`
18. `docs/05-adr/index.md` and all 30 ADR files (full reads: 0001, 0006, 0008, 0009, 0010; decision sections for the rest)
19. `docs/08-reference/repository-layout.md`, `cli.md`, `pytxo-toml.md`
20. `docs/01-projects/pytxo-improvement-research.md`, `mvp-bootstrap.md`
21. `docs/07-guides/compare/pytxo-vs-claude-agent-teams.md`
22. `crates/pytxo-signal/src/language.rs`, `crates/pytxo-core/src/moat/permission/mod.rs`, `crates/pytxo-cli/src/main.rs`, `crates/pytxo-mcp/src/main.rs`, `apps/desktop/src-tauri/Cargo.toml`
23. Source-tree inventories: `crates/*/src/**`, `apps/desktop/src/**`, `apps/web/content/docs/**`, `services/*`, `packages/*`

Back: [[MOC-home]] · Program: [[pytxo-improvement-research]] · Decisions: [[adr-index]]
