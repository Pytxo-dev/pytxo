---
title: Race Shield (swarm registry)
slug: race-shield
status: active
tags: [orchestration, concurrency, moat]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[dag-flow-engine]], [[sqlite-wal-logging]], [[product-vision]]
---

# Race Shield (swarm registry)

**Race Shield** is Pytxo’s **thread-safe global process registry** for parallel agents on one monorepo.

## Goal

Prevent **concurrent file-write collisions** and stdin/stdout cross-talk when multiple headless agents run in overlapping paths.

## Design

```text
Arc<RwLock<SwarmRegistry>>
  ├── agent_id → PTY handle, cwd, worktree/overlay binding
  ├── path claims (coarse or semantic, per scheduler wave)
  └── stdin stream buffers (serialize writes per target)
```

Principles:

- **Lock-free where profiling proves safe**; `RwLock` on the registry is the documented baseline
- Scheduler ([[dag-flow-engine]]) assigns waves; Race Shield enforces **runtime claims**
- WAL ([[sqlite-wal-logging]]) remains the audit trail; registry is the live guard

## Integration point

All spawns from `pytxo-runner` register before `exec`. Stop/kill paths deregister and release claims.

## Status

- **Shipping:** PID registry and stop/run tracking in `pytxo-runner`
- **Next:** path claims, stdin buffering, lock-free hot paths as measured

Do not spawn agents that bypass the registry.
