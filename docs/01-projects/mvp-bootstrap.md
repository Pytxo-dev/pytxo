---
title: MVP bootstrap
slug: mvp-bootstrap
status: active
tags: [project]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[MOC-home]], [[ADR-0005-worktree-isolation-for-mvp]], [[phase-2-reality-deck]]
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

## Phase 2 — complete

- [x] Phase 1.5: PID registry, streamed WAL events, `pytxo stop` / `stop --all`
- [x] `pytxo-orchestrate` shared library (CLI, MCP, desktop)
- [x] `pytxo-sanitize` + ADR-0006
- [x] WAL migration 002 + cost parsers + `pytxo status --json`
- [x] `pytxo-mcp` stdio server + [[mcp-cursor-setup]]
- [x] `depends_on` DAG scheduling + ADR-0007
- [x] `apps/desktop` Tauri v2 + Svelte 5 Reality Deck v1
- [x] IPC v1: `list_runs`, `list_agents`, `tail_events`, `dry_run`, `start_run`, `stop_run`, `git_diff`

## Build

```bash
cargo build -p pytxo-cli
cargo build -p pytxo-mcp
cargo test --workspace
cargo run -p pytxo-cli -- --version
```

Desktop:

```bash
cd apps/desktop && npm ci && npm run check
cargo build -p pytxo-desktop
```

## References

- [[first-three-agent-run]]
- [[cli-reference]]
- [[cursor-mcp-pytxo]]
- [[phase-2-reality-deck]]
- [[ADR-0001-three-tier-rust-svelte-tauri]]
