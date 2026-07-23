---
title: Race Shield (swarm registry)
slug: race-shield
status: active
tags: [orchestration, concurrency, moat]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-07-10
related: [[dag-flow-engine]], [[sqlite-wal-logging]], [[execution-domains]], [[permission-profile-engine]], [[product-vision]], [[pytxo-improvement-research]]
---

# Race Shield (swarm registry)

**Race Shield** is Pytxo’s **thread-safe swarm registry** for parallel agents within one **execution domain** ([[execution-domains]])—typically one monorepo per domain.

## Goal

Prevent **concurrent file-write collisions** and stdin/stdout cross-talk when multiple headless agents run in overlapping paths.

## Design

```text
SwarmRegistry
  ├── paths: RwLock<path claims + live agents>
  └── stdin: Mutex<StdinBuffer>   # separate lock (Phase 70)
```

Principles:

- **Separate locks for path claims vs stdin** so PTY stdin pumps do not serialize against disjoint claim waves
- **Lock-free / path-prefix shards** only after `registry_contention_*` profiling proves need
- Scheduler ([[dag-flow-engine]]) assigns waves; Race Shield enforces **runtime claims**
- WAL ([[sqlite-wal-logging]]) remains the audit trail; registry is the live guard

## Execution domain scope

Each [[execution-domains|ExecutionDomain]] owns its own `SwarmRegistry` instance. Path claims, PTY handles, and stdin buffers **do not cross** `domain_id` boundaries. The hypervisor registry ([[execution-domains]]) holds one domain per canonical `repo_root`.

## Galaxy HITL extension

For **`Galaxy`** [[permission-profile-engine|permission profiles]], high-risk actions submit to a per-domain **HITL queue** (`HitlQueue` in `pytxo-runner`) and block until orchestration resolves them. The queue exposes `submit`, `pending`, `resolve`, and `wait_blocking`; orchestration surfaces `list_hitl_pending` / `hitl_respond`, Pytxo Desktop shows an approval panel, and `pytxo hitl list|approve|deny` drives it from the CLI.

**Shipped:** queue, orchestration API, Tauri IPC, Deck panel, CLI; runner gates for spawn (`classify_risky_command`), MCP proxy, flush / out-of-root writes; **stdin HITL** via `enqueue_agent_stdin` → `gate_spawn_command` on Galaxy (orchestrate).

**Remaining:** broader runtime syscall hooks beyond command-string classifiers; path-prefix claim shards if contention profiles demand them.

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
| Galaxy HITL queue + stdin/spawn/MCP/flush gates | **Shipped** (Phase 64/70) |
| Separate stdin vs path-claim locks | **Shipped** (Phase 70) |
| Lock-free / sharded path claims | **Deferred** until profiling |

Do not spawn agents that bypass the registry.
