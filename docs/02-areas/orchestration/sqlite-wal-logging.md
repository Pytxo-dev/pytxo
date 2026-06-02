---
title: SQLite WAL session logging
slug: sqlite-wal-logging
status: active
tags: [orchestration, telemetry, sqlite]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[ADR-0002-sqlite-wal-session-telemetry]]
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

ADR: [[ADR-0002-sqlite-wal-session-telemetry]].

See also: [[cost-and-swarm-limits]].
