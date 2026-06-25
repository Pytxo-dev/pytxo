---
title: MVP bootstrap
slug: mvp-bootstrap
status: active
tags: [project]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-21
related: [[MOC-home]], [[ADR-0005-worktree-isolation-for-mvp]], [[phase-2-reality-deck]], [[modular-projects]]
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
- [x] [[github-organization]] public/private matrix

## Phases 9–16 — shipped

Modular project manifest + CLI, Galaxy HITL queue/Deck/CLI, hypervisor catalog, Signal retry WAL, MCP `project_id` routing, topology arbitrage, Link request envelopes.

## Phases 17–22 — shipped (control plane)

- [x] **17** — `task.root`, root-scoped Race claims, `agents.root_id`, catalog `project_id` ([[ADR-0011-modular-project-manifest]])
- [x] **18** — HITL blocks flush; [[context-launch-contract]]; smoke test for `PYTXO_CONTEXT_DIR`
- [x] **19** — Targeted closed-loop retry, per-agent/task fidelity, richer `manifest.json`, Deck signal-retry panel
- [x] **20** — Unified `project run` (single `run_id`), cross-root read-only context, Deck project picker + root filters, `pytxo project status`
- [x] **21** — `HttpBillingReconciler` HTTP transport (`link-http` default on CLI); reference [`services/pytxo-link`](../../services/pytxo-link/); [[pytxo-link-service]]
- [x] **22** — `subprocess_stdin` spawn-time pump + tests; `overlay-fuse` copy-layer POC; cloud sandbox remains external ([[cloud-sandbox-service]])

## Phases 17–22 completion (2026-06-05)

- Multi-root live integration tests (`project_run`, `multi_root`, `root_scoped_claim`)
- Unknown `task.root` / read-only execution enforcement in runner
- `link-http` on `pytxo-orchestrate` / `pytxo-cli`; `PYTXO_ULTRA_SESSION` auth header
- MCP hub v1: `pytxo_list_live_agents`, `pytxo_route_stdin`, WAL `mcp-tool` audit
- `pytxo doctor` link reconcile check; `[billing]` in [[pytxo-toml]]

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
- [x] `pytxo-mcp` stdio server + [[mcp-cursor-setup]]
- [x] `depends_on` DAG scheduling + ADR-0007
- [x] Reality Deck v1 in `apps/desktop` (monorepo)
- [x] IPC v1: `list_runs`, `list_agents`, `tail_events`, `dry_run`, `stop_run`, `git_diff` (dispatch is now `dispatch_run_cmd`)

## Phases 23 — shipped (hypervisor fleet DAG)

- [x] **23** — `FleetManifest` + `pytxo fleet` CLI; `fleet_runs` / `fleet_nodes` in hypervisor catalog; enriched domain dashboard (CLI, TUI, Deck) ([[ADR-0015-hypervisor-fleet-dag]], [[hypervisor-fleet-dag]])

## Phases 24–29 — shipped (post–Hypervisor Phase 3)

- [x] **24** — Galaxy HITL spawn gate + `{data_dir}/hitl.json` persistence; `pytxo doctor` `hitl_persistence` ([[ADR-0016-galaxy-hitl-enforcement]])
- [x] **25** — Per-root `permission_profile`; project-scoped `~/.pytxo/projects/<id>/pytxo.db`; Deck path + fleet panels; cross-root scheduler hints
- [x] **26** — Overlay copy-layer flush to physical tree; `overlay_isolation` doctor probe ([[ADR-0017-overlay-flush-contract]])
- [x] **27** — Signal structural graph IPC + 2.5D Deck topology (import edges)
- [x] **28** — `pytxo fleet run --continue-on-error`; MCP `pytxo_fleet_run` / `pytxo_fleet_status`; Deck fleet panel
- [x] **29** — Ultra `billing-tiktoken` estimator feature; Link + cloud doctor checks when `[billing]` / `[cloud]` enabled

## Phase 30 — shipped (foundation + honesty)

- [x] Policy trait modules (`FilesystemPolicy`, `NetworkPolicy`, `EnvironmentPolicy`) in `pytxo-core::moat::permission`
- [x] Signal Core `signal-fallback` WAL events + `fallback_paths` in Deck arbitrage IPC
- [x] `permission_profile` + `isolation_mode` on CLI `status --json` and Deck run header
- [x] Docs sync: vault / Docusaurus / `crates/README.md` shipped-vs-planned alignment
- [x] `pytxo-link` entitlements status contract test

## Phase 32 — shipped (Galaxy HITL + Orbit network)

- [x] **ADR-0018** Network and MCP policy ([[ADR-0018-network-and-mcp-policy]])
- [x] `hitl_gate` classifiers: `mcp.tool`, `fs.write_outside_root`, `proc.package_install`
- [x] Orbit `spawn_egress_allowed` enforced before agent spawn
- [x] Galaxy `mcp_proxy_call` gated via domain `HitlQueue`
- [x] Galaxy outside-root flush HITL in `commit_workspace`
- [x] Deck HITL panel shows action, agent, and reason
- [x] `pytxo doctor` checks `network_policy` and `mcp_hitl`

## Phase 33 — shipped (sparse overlay + cloud sync)

- [x] `[blast].sparse_exclude` in `pytxo.toml` (defaults: `node_modules`, `.git`, `target`, `dist`, `build`)
- [x] Cloud `sync_delta` initial path list via `collect_sync_paths` on sandbox start
- [x] Deck shows isolation backend label (DiffPanel + run sidebar)
- [x] Linux kernel overlay multi-lowerdir sparse excludes; copy-layer skip
- [x] `overlay-fuse-macos` feature + `overlay_fuse_macos.rs`
- [x] Windows ProjFS copy-layer provider in `overlay_projfs.rs`
- [x] `isolation_backend_label` selection chain in `blast.rs`

## Phase 34 — shipped (project paths + dry-run warnings)

- [x] `find_cross_root_conflicts` → `warnings` in orchestrate `dry_run_json` / `ExecutionPlan`
- [x] IPC `project_add_root_cmd` / `project_remove_root_cmd` + Deck `ProjectPathPanel` add/remove UI
- [x] `pytxo project status --json` includes `agents_by_root` per run
- [x] CLI `pytxo project remove --label`

## Phase 35 — shipped (DAG recovery + fleet viz + cloud exec)

- [x] DAG deadlock recovery: `PYTXO_DAG_RECOVERY=1` in scheduler + runner `dag-recovery` WAL ([[dag-flow-engine]])
- [x] `fleet_run_status` IPC + `FleetPanel` wave/node DAG viz
- [x] `pytxo-cloud-sandbox` docker exec POC (alpine container per sandbox)
- [x] HITL audit WAL on `HitlQueue::resolve()` (`hitl-resolve` events)

## Phase 36 — shipped (signal modules + sanitize + topology)

- [x] Module-level aggregation in `pytxo-signal/src/graph.rs` (`build_module_graph`)
- [x] MCP sanitize middleware when `PYTXO_SANITIZE` enabled (default on)
- [x] Topology blast radius includes `fallback_paths` from Signal arbitrage
- [x] Light theme CSS variables + Deck toggle in `App.svelte`

## Phase 37 — shipped (MCP hub v3 + planner + audit export)

- [x] MCP multi-hop routing (`agent:a->agent:b`) in `mcp_hub.rs`
- [x] Production MCP child via `PYTXO_AGENT_MCP_ADDR` (test fixture fallback)
- [x] `pytxo-planner` heuristic decomposer v1 (`HeuristicPlanner`, `PYTXO_PLANNER=1`)
- [x] `export_mcp_audit` in `pytxo-orchestrate` (WAL `mcp-tool` rows per run)

## Phases 38–39 — shipped (cloud hybrid + enterprise beta)

- [x] **38** — Deterministic `delta_from_overlay_upper` in `pytxo-core`; runner `sync_delta` before cloud exec; Deck cloud / fallback badge from WAL events; `pytxo-cloud-sandbox` worker pool stub + auth hardening; Link production defaults documented; `link-http` on `pytxo-cli`; Ultra `link_reconcile` defaults on
- [x] **39** — Link org policy ceiling in orchestrate when `PYTXO_ULTRA_SESSION` set; Race Shield registry contention benchmark tests; [[deepspace-network-v2]] spike; Enterprise beta in [[tiers-hobbyist-pro-max]]; [[competitive-benchmarks]] overlay vs worktree note

## Phases 47–49 — shipped (Reality Deck 3D + planner v2 + Enterprise GA)

- [x] **47** — ADR-0023 Three.js 3D topology; `TopologyScene3D.svelte` center viewport; collapsible log panel; light xterm theme + localStorage; structural graph IPC v2 + symbol-level nodes (`enrich_symbol_nodes`)
- [x] **48** — `SignalBackedPlanner` (`PYTXO_PLANNER=signal` / `[planner] mode = "signal"`); canvas fleet wave graph in `FleetPanel.svelte`
- [x] **49** — Enterprise GA in [[tiers-hobbyist-pro-max]]; overlay vs worktree estimates in [[competitive-benchmarks]]; TUI org policy indicator in board header; Link org audit log (`/v1/orgs/{id}/audit`)

## Phase 40 — shipped (Link production foundation)

- [x] Postgres `runs` ledger + idempotency (`migrations/003_runs.sql`, ADR-0021)
- [x] Paddle webhook HMAC verification on Link
- [x] `doctor` Link `/health` HTTP ping when `link_reconcile` enabled
- [x] Account page shows Link tier via `/api/entitlements/status`
- [x] Linux overlay-fuse-kernel CI compile matrix (existing); copy-layer labeled explicitly in telemetry

## Phase 41 — shipped (Ultra managed-inference proxy)

- [x] **ADR-0019** Ultra managed-inference proxy ([[ADR-0019-ultra-managed-inference-proxy]])
- [x] `services/pytxo-proxy` Axum service with provider routes and SSE passthrough
- [x] Bearer `PYTXO_ULTRA_SESSION` / `LINK_API_KEY` auth; server-side provider keys
- [x] Deck Header Ultra wallet credits via `entitlement_status` IPC
- [x] Proxy reports provider token usage to Link `POST /v1/inference/usage` (`services/pytxo-proxy/src/metering.rs`)

## Phase 42 — shipped (Cloud sandbox real execution)

- [x] **ADR-0020** Cloud sandbox runtime ([[ADR-0020-cloud-sandbox-runtime]])
- [x] Fail-closed exec; `sync_sandbox` via tar + `docker cp`; teardown `docker rm -f`; TTL sweeper
- [x] Link `cloud_enabled` gate; SHA-256 `content_hash` in `pytxo-core::cloud::cache`
- [x] Deck cloud badge from WAL `cloud-delta` / `cloud-exec` events

## Phase 43 — shipped (Billing reconciliation + CI/CD)

- [x] Idempotent `settle_ultra_run`; `.github/workflows/deploy-services.yml` Railway deploy
- [x] Entitlements cache TTL (5 min); fail-loud when `link_reconcile` on
- [x] Account + Deck subscription portal links

## Phase 44 — shipped (Billing UX + Deck auth deep-link)

- [x] `pytxo-deck://` deep-link + `auth_store_session`; `UsagePanel` billing dashboard
- [x] Org policy ceiling in Header; tree-sitter Rust/TS import edges in `graph.rs`

## Phase 45 — shipped (DeepSpace network v2)

- [x] `network_isolation.rs` with `isolate_deepspace_network` (Linux netns behind `deepspace-netns` + `PYTXO_DEEPSPACE_NETNS=1`, macOS `sandbox-exec`, Windows stub)
- [x] DeepSpace hook in `run.rs` before spawn (subprocess + PTY wrap); WAL `network-isolation` event
- [x] `NetworkPolicyEngine::egress_allowed` runtime TCP gate on network-ish spawns
- [x] `pytxo doctor` `deepspace_network_isolation` probe ([[ADR-0022-deepspace-network-namespace]])
- [x] `pytxo-runner` `tests/network_policy.rs`

## Phase 46 — shipped (sparse overlay production)

- [x] `isolation_backend_label` telemetry strings (`overlay-kernel-fuse`, `overlay-copy-layer`, …)
- [x] Deck `DiffPanel` isolation backend badge (wired via `App.svelte` / IPC)
- [x] `overlay_upper_cloud_delta` hook in `pytxo-core::cloud::delta`
- [x] macOS `overlay-fuse-macos` CI compile-only step
- [x] `isolation_backend_label` integration test stub in `blast.rs`

## Phase 50 — shipped (deploy + config split)

- [x] ADR-0024: `billing.inference_proxy_url` separate from `billing.proxy_url` (Link)
- [x] `ManagedTransport` routes inference to proxy host; doctor `inference_proxy_health` + `cloud_health`
- [x] `services/pytxo-proxy/railway.toml` + `railway.json`; Railway README Part 2b for `proxy.pytxo.com`

## Phase 51 — shipped (E2E money loop validation)

- [x] `tooling/scripts/go-live-smoke.sh` + `.ps1` (health, entitlements, run ledger round-trip)
- [x] Run ledger idempotency integration test (`run_ledger_idempotency.rs`)

## Phase 52 — shipped (Ultra proxy production)

- [x] Streaming + header token metering in `pytxo-proxy`; per-bearer rate limiting
- [x] `GET /v1/wallet/balance` on Link; Deck UsagePanel live Ultra credits

## Phase 53 — shipped (cloud sandbox isolation)

- [x] ADR-0025: non-root Docker, resource limits, internal network, `CLOUD_EGRESS_ALLOWLIST`
- [x] Optional `REDIS_URL` durable scaffold cache; concurrent load tests

## Phase 54 — shipped (DeepSpace network v2 production)

- [x] ADR-0026: Linux netns default for DeepSpace; doctor socket probe to `1.1.1.1:443`
- [x] Windows WFP/AppContainer stub markers; macOS `sandbox-exec` verified path

## Phase 55 — shipped (sparse overlay hardening)

- [x] `projfs-sparse-copy-v2` / `macos-sparse-overlay-v2` labels; `node_modules` exclusion test
- [x] DiffPanel backend fallback from `isolation_mode`

## Phase 56 — shipped (observability + reliability)

- [x] ADR-0027: JSON health contract (`uptime_secs`), structured logging, Link rate/body limits
- [x] `docs/07-guides/link-db-backup-runbook.md`; deploy-services.yml health smoke

## Phase 57 — shipped (measured benchmarks)

- [x] `overlay-vs-worktree` + `multi-agent-ram` benchmark scripts; CI matrix entries
- [x] `competitive-benchmarks.md` Phase 57 methodology section

## Phase 58 — shipped (product surface)

- [x] Symbol-level 3D nodes in `TopologyScene3D.svelte`; structural graph IPC v3
- [x] `LlmPlanner` via inference proxy when `PYTXO_PLANNER_LLM=1`; `deck-3d-stills.md`

## Phase 59 — shipped (Enterprise GA launch)

- [x] `org_seats` migration + real seat management; `PUT /v1/orgs/{id}/policy` admin route
- [x] Audit for policy/seat changes; web account shows seats + org policy
- [x] `docs/07-guides/enterprise-ga-launch-checklist.md`

## Phase 31 — shipped (Signal depth + DeepSpace reads)

- [x] tree-sitter grammars: Java, C/C++, Ruby (`pytxo-signal`)
- [x] Closed-loop v3: graph-neighbor path escalation before full-task fallback
- [x] DeepSpace `may_read` on MCP reads + context materialization
- [x] Topology nodes colored by `root_id`
- [x] `billing-tiktoken` uses `tiktoken-rs` when feature enabled

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

- [[first-three-agent-run]]
- [[cli-reference]]
- [[cursor-mcp-pytxo]]
- [[phase-2-reality-deck]]
- [[ADR-0001-three-tier-rust-svelte-tauri]]
