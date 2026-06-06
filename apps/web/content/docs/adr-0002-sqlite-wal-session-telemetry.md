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

- [sqlite-wal-logging](/docs/sqlite-wal-logging)
