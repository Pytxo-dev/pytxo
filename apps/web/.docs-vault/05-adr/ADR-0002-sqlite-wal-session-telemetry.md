---
title: ADR-0002 SQLite WAL for session telemetry
slug: adr-0002-sqlite-wal-session-telemetry
status: accepted
tags: [adr, telemetry]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
adr_id: ADR-0002
related: [[sqlite-wal-logging]]
---

# ADR-0002: SQLite WAL for session telemetry

## Status

Accepted

## Context

Hundreds of subagents and high-frequency terminal streams can exhaust RAM if all history is held in-process.

## Decision

Persist terminal output, token metrics, IPC events, and VFS diffs to a local **SQLite** database with **WAL** mode. Keep orchestration heap bounded.

## Consequences

**Positive**

- Long-running pipelines without OOM.
- Replay and analytics from disk.

**Negative**

- Disk growth; requires retention/compaction policy.
- WAL tuning per platform.

## Links

- [[sqlite-wal-logging]]
