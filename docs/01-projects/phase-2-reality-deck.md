---
title: Phase 2 — Reality Deck
slug: phase-2-reality-deck
status: active
tags: [project]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[mvp-bootstrap]], [[presentation-passive-telemetry]]
---

# Phase 2 — Reality Deck

## Delivered

- **Control plane:** sanitize, cost telemetry, MCP stub, DAG `depends_on`, orchestrate lib, stop/PID hardening.
- **Presentation:** `apps/desktop` with run list, wave timeline, xterm panel, diff (via Rust `git diff`), start/stop/dry-run.

## IPC v1 (Tauri)

| Command | Purpose |
|---------|---------|
| `list_runs` | WAL runs |
| `list_agents` | Agents for `run_id` |
| `tail_events` | Log history |
| `poll_log_lines` | Incremental tail for live UI |
| `dry_run` | Plan JSON |
| `start_run` | Start orchestrated run |
| `stop_run` | Kill tracked PIDs |
| `git_diff` | Read-only diff via Rust |

UI reads SQLite through Tauri only (ADR-0001).
