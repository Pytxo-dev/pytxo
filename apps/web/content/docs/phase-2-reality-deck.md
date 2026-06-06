---
title: Phase 2 — Reality Deck
slug: phase-2-reality-deck
status: active
tags: [project]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-04
related: [mvp-bootstrap](/docs/mvp-bootstrap), [presentation-passive-telemetry](/docs/presentation-passive-telemetry), [execution-domains](/docs/execution-domains), [permission-profile-engine](/docs/permission-profile-engine)
---

# Phase 2 — Reality Deck

## Delivered

- **Control plane:** sanitize, cost telemetry, MCP stub, DAG `depends_on`, orchestrate lib, stop/PID hardening.
- **Presentation:** `apps/desktop` — run list, wave timeline, xterm panel, diff (via Rust `git diff`), start/stop/dry-run.

## IPC v1 (Tauri)

| Command | Purpose |
|---------|---------|
| `list_runs` | WAL runs |
| `list_agents` | Agents for `run_id` |
| `tail_events` | Log history |
| `poll_log_lines` | Incremental tail for live UI |
| `dry_run` | Plan JSON |
| `dispatch_run_cmd` | Start orchestrated run (non-blocking; domain-scoped) |
| `stop_run` | Kill tracked PIDs |
| `git_diff` | Read-only diff via Rust |

UI reads SQLite through Tauri only (ADR-0001).

## IPC v2 (hypervisor + permissions) — shipped

| Command | Purpose | Min `PermissionProfile` | Status |
|---------|---------|------------------------|--------|
| `list_domains_cmd` | Active execution domains + repo roots | Any (read telemetry) | Shipped |
| `select_domain` | Deck UI selected domain | Any | Shipped |
| `dispatch_run_cmd` | Start run on `repo_root` (creates/reuses domain) | Orbit (default) | Shipped |
| `commit_workspace` | Approve Blast Shield flush to physical disk | Orbit | Shipped |
| `list_hitl` / `hitl_respond` | List + approve/deny blocked Galaxy actions | Galaxy | **Shipped** (enforcement hooks land per risky-op) |

`tail_events`, `poll_log_lines`, `list_runs`, and `list_agents` accept optional **`domain_id`** (defaults to selected domain or CWD). Store reads use the domain’s `repo_root`-anchored SQLite path (`.pytxo/data/pytxo.db`). `poll_log_lines` keeps a per-(domain, agent) cursor so switching agents does not skip or duplicate lines.

## Phase 3 (deferred)

- Global hypervisor catalog DB (`~/.pytxo/hypervisor.db`) for cross-project dashboard
- Cross-repo DAG edges at hypervisor level
