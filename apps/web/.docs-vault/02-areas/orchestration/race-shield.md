---
title: Race Shield (swarm registry)
slug: race-shield
status: active
tags: [orchestration, concurrency, moat]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-04
related: [[dag-flow-engine]], [[sqlite-wal-logging]], [[execution-domains]], [[permission-profile-engine]], [[product-vision]]
---

# Race Shield (swarm registry)

**Race Shield** is Pytxo’s **thread-safe swarm registry** for parallel agents within one **execution domain** ([[execution-domains]])—typically one monorepo per domain.

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

## Execution domain scope

Each [[execution-domains|ExecutionDomain]] owns its own `SwarmRegistry` instance. Path claims, PTY handles, and stdin buffers **do not cross** `domain_id` boundaries. The hypervisor registry ([[execution-domains]]) holds one domain per canonical `repo_root`.

## Galaxy HITL extension

For **`Galaxy`** [[permission-profile-engine|permission profiles]], high-risk actions submit to a per-domain **HITL queue** (`HitlQueue` in `pytxo-runner`) and block until orchestration resolves them. The queue exposes `submit`, `pending`, `resolve`, and `wait_blocking`; orchestration surfaces `list_hitl_pending` / `hitl_respond`, the Deck shows an approval panel, and `pytxo hitl list|approve|deny` drives it from the CLI.

**Shipped:** the queue, orchestration API, Tauri IPC (`list_hitl`, `hitl_respond`), Deck panel, and CLI. **Remaining:** wiring runner enforcement call sites for each class of high-risk action (the flush boundary already documents this contract in `commit_workspace`).

## Integration point

All spawns from `pytxo-runner` register before `exec`. Stop/kill paths deregister and release claims.

## Status

| Capability | Status |
|------------|--------|
| PID registry and stop/run tracking | **Shipped** (`pytxo-runner`) |
| In-process path claims per run / wave | **Shipped** |
| Per-domain `SwarmRegistry` in hypervisor | **Shipped** |
| Stdin buffer + MCP `pytxo_stdin` | **Shipped** |
| Stdin pump into agent PTY | **Shipped** when `execution_backend = pty` (default) — continuous ~15ms drain loop |
| Subprocess stdin (`subprocess_stdin = true`) | **Shipped (opt-in)** — single drain at spawn; use PTY for long-lived interactive stdin |
| Galaxy HITL queue + `hitl_respond` IPC + Deck panel + CLI | **Shipped** (enforcement call sites land per risky-op) |
| Lock-free hot paths | **Deferred** until profiling |

Do not spawn agents that bypass the registry.
