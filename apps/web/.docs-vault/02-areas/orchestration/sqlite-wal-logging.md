---
title: SQLite WAL session logging
slug: sqlite-wal-logging
status: active
tags: [orchestration, telemetry, sqlite]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[ADR-0002-sqlite-wal-session-telemetry]], [[execution-domains]]
---

# SQLite WAL session logging

High-frequency terminal output, MCP traffic, and diff history must not live unbounded in RAM—especially with large swarms or long-running pipelines.

## Design

Persist to a local **SQLite** database with **Write-Ahead Logging (WAL)**:

- CLI stdout/stderr streams
- Token usage metrics
- IPC transactions
- Virtual filesystem diff history

## Result

- Orchestration RAM stays at a **constant bound** (target on the order of ~500 MB for core process limits—tune per platform).
- **Reality Deck** can replay or stream from WAL without retaining full scrollback in memory.
- Multi-day pipelines remain viable without OOM.

## Multi-project / execution domains

When multiple repo roots run concurrently ([[execution-domains]]):

- Each domain uses its own `{data_dir}/pytxo.db` with WAL enabled—no shared `events` table across projects.
- Append paths resolve `domain_id` (or equivalent DB handle) **before** insert so PTY lines from project A and B never interleave in storage.
- UI polls `tail_events` / `poll_log_lines` with an explicit `domain_id` ([[presentation-passive-telemetry]]); orchestration routes reads to the correct file.

Optional future: a global `~/.pytxo/hypervisor.db` catalog pointing at per-domain DB paths (v2 in [[execution-domains]]).

ADR: [[ADR-0002-sqlite-wal-session-telemetry]].

See also: [[cost-and-swarm-limits]].
