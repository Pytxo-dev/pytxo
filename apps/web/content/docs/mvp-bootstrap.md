---
title: MVP bootstrap
slug: mvp-bootstrap
status: active
tags: [project]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-04
related: [MOC-home](/docs/moc-home), [ADR-0005-worktree-isolation-for-mvp](/docs/adr-0005-worktree-isolation-for-mvp), [phase-2-reality-deck](/docs/phase-2-reality-deck), [modular-projects](/docs/modular-projects)
---

# MVP bootstrap

## Phase 0 — complete

- [x] Cargo workspace (`pytxo-core`, `pytxo-scheduler`, `pytxo-runner`, `pytxo-store`, `pytxo-cli`)
- [x] Cross-platform CI (Linux, Windows, macOS)
- [x] ADR-0005 worktree isolation for MVP
- [x] `pytxo.toml.example`, fixture `tests/fixtures/tiny-monorepo`

## Phase 1 — complete (CLI)

- [x] **1a** Git worktree + `pytxo run --cmd`
- [x] **1b** Scheduler preflight + `--dry-run`
- [x] **1c** Multi-agent waves + `max_agents` + `pytxo stop`
- [x] **1d** SQLite WAL + `pytxo status` / `pytxo logs`
- [x] **1e** Config file, tutorial, benchmark scripts

## Phase 3 — complete (PTY + Race stdin)

- [x] ADR-0010 PTY default execution backend
- [x] `pytxo-runner` `pty.rs` + `ChildLaunchEnv`
- [x] Race Shield stdin pump + `pytxo_stdin` MCP tool
- [x] `pytxo doctor` PTY smoke check

## Phase 4–8 — complete (hypervisor, Blast merge, tier cap, Link stub)

- [x] **Phase 4** Multi-domain Deck: `list_domains_cmd`, `select_domain`, `dispatch_run_cmd`, domain-scoped `tail_events` / `poll_log_lines` / `list_runs` / `list_agents` (each takes optional `domain_id`)
- [x] **Phase 5** Blast approve: worktree merge flush + `commit_workspace` IPC + Deck "Approve merge"
- [x] **Phase 6** Signal closed-loop: high-fidelity retry on agent failure; MCP reads return `ScaffoldResult` stats
- [x] **Phase 7** Deck design tokens (void/teal/violet/gold) + 2D topology canvas (`TopologyPanel.svelte`)
- [x] **Phase 8** Tier cap (`tier_max_agents`) + `billing.link_reconcile` → `HttpBillingReconciler` stub; cloud sandbox documented as separate repo

Note: IPC command is `dispatch_run_cmd` (non-blocking), which replaced the blocking `start_run`.

## Audit (2026-06-04)

- [x] `PytxoConfig::db_path_at` / `state_path_at` (repo-root-anchored telemetry)
- [x] Deck poll cursors keyed per `(domain_id, agent_id)`
- [x] `billing.link_reconcile` wired; benchmark scripts resolve monorepo root
- [x] [github-organization](/docs/github-organization) public/private matrix

## Phases 9–16 — shipped

Modular project manifest + CLI, Galaxy HITL queue/Deck/CLI, hypervisor catalog, Signal retry WAL, MCP `project_id` routing, topology arbitrage, Link request envelopes.

## Phases 17–22 — shipped (control plane)

- [x] **17** — `task.root`, root-scoped Race claims, `agents.root_id`, catalog `project_id` ([ADR-0011-modular-project-manifest](/docs/adr-0011-modular-project-manifest))
- [x] **18** — HITL blocks flush; [context-launch-contract](/docs/context-launch-contract); smoke test for `PYTXO_CONTEXT_DIR`
- [x] **19** — Targeted closed-loop retry, per-agent/task fidelity, richer `manifest.json`, Deck signal-retry panel
- [x] **20** — Unified `project run` (single `run_id`), cross-root read-only context, Deck project picker + root filters, `pytxo project status`
- [x] **21** — `HttpBillingReconciler` HTTP transport (`link-http` default on CLI); reference [`services/pytxo-link`](../../services/pytxo-link/); [pytxo-link-service](/docs/pytxo-link-service)
- [x] **22** — `subprocess_stdin` spawn-time pump + tests; `overlay-fuse` copy-layer POC; cloud sandbox remains external ([cloud-sandbox-service](/docs/cloud-sandbox-service))

## Phases 17–22 completion (2026-06-05)

- Multi-root live integration tests (`project_run`, `multi_root`, `root_scoped_claim`)
- Unknown `task.root` / read-only execution enforcement in runner
- `link-http` on `pytxo-orchestrate` / `pytxo-cli`; `PYTXO_ULTRA_SESSION` auth header
- MCP hub v1: `pytxo_list_live_agents`, `pytxo_route_stdin`, WAL `mcp-tool` audit
- `pytxo doctor` link reconcile check; `[billing]` in [pytxo-toml](/docs/pytxo-toml)

## Cloud + MCP v2 + kernel overlays (2026-06-05)

- [x] **`pytxo-cloud-sandbox`** sibling repo (Axum API, worker POC, docker-compose)
- [x] Monorepo `ExecutionBackend::Cloud`, `[cloud]`, `HttpCloudDispatcher`, context cache hook
- [x] MCP v2: `pytxo_mcp_proxy`, `pytxo_mcp_tools_list`, domain `McpHub` registry
- [x] Linux `overlay-fuse-kernel` (overlay mount POC); Windows `overlay-projfs` stub + fallback chain
- [x] CI: `cloud_sandbox`, `mcp_proxy`, overlay feature matrix

## Phase 2 — complete

- [x] Phase 1.5: PID registry, streamed WAL events, `pytxo stop` / `stop --all`
- [x] `pytxo-orchestrate` shared library (CLI, MCP, desktop)
- [x] `pytxo-sanitize` + ADR-0006
- [x] WAL migration 002 + cost parsers + `pytxo status --json`
- [x] `pytxo-mcp` stdio server + [mcp-cursor-setup](/docs/mcp-cursor-setup)
- [x] `depends_on` DAG scheduling + ADR-0007
- [x] Reality Deck v1 in `apps/desktop` (monorepo)
- [x] IPC v1: `list_runs`, `list_agents`, `tail_events`, `dry_run`, `stop_run`, `git_diff` (dispatch is now `dispatch_run_cmd`)

## Build

```bash
cargo build -p pytxo-cli
cargo build -p pytxo-mcp
cargo test --workspace
cargo run -p pytxo-cli -- --version
```

Reality Deck (`apps/desktop`):

```bash
cd apps/desktop && npm ci && npm run check
cargo build -p pytxo-desktop   # from repo root
```

## References

- [first-three-agent-run](/docs/first-three-agent-run)
- [cli-reference](/docs/cli-reference)
- [cursor-mcp-pytxo](/docs/cursor-mcp-pytxo)
- [phase-2-reality-deck](/docs/phase-2-reality-deck)
- [ADR-0001-three-tier-rust-svelte-tauri](/docs/adr-0001-three-tier-rust-svelte-tauri)
