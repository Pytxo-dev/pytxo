---
title: Asynchronous DAG flow engine
slug: dag-flow-engine
status: active
tags: [orchestration, scheduling]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[ADR-0004-dag-scheduler-over-sequential-locks]]
---

# Asynchronous DAG flow engine

Parallel multi-agent edits risk **file-lock deadlocks** under a naive sequential scheduler. Pytxo uses an **asynchronous directed acyclic graph (DAG) flow engine**.

## Construction

Swarm proposals become a task DAG:

- **Vertices** — execution tasks
- **Edges** — file or semantic dependencies

Topological scheduling runs ready tasks concurrently when resources allow.

## Cycle detection

If agents deadlock (A waits on B, B waits on A):

1. Detect the cycle in the dependency graph.
2. Pause one agent (e.g. B).
3. Inject a **mocked intermediate state** into the other’s buffer so the dependency chain can complete.
4. Resume and reconcile real artifacts.

## Stall recovery (Phase 35)

When scheduling cannot advance (`ready` queue empty) or Race Shield path claims stall at runtime:

- Set `PYTXO_DAG_RECOVERY=1` so `pytxo-scheduler` force-schedules a deferred task and emits `warnings` in dry-run JSON.
- The runner injects a synthetic WAL `dag-recovery` event when path claims would block under the same flag.

Document target workloads in benchmarks ([[competitive-benchmarks]]).

ADR: [[ADR-0004-dag-scheduler-over-sequential-locks]].
