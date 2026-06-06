---
title: Architecture
---

# Architecture

Pytxo follows a **three-tier model**:

```text
Presentation  →  Orchestration  →  Execution
(Reality Deck)    (pytxo-orchestrate)   (pytxo-runner + PTY)
       ↑                  ↑
   passive telemetry   MCP / CLI
```

## Crates

| Crate | Role |
|-------|------|
| `pytxo-cli` | User-facing commands |
| `pytxo-orchestrate` | Scheduling, hypervisor, doctor |
| `pytxo-runner` | Agent execution, worktrees, PTY |
| `pytxo-core` | Config, tasks, billing types, moat interfaces |
| `pytxo-store` | SQLite schema and WAL |
| `pytxo-mcp` | MCP server for IDE integration |

## Data flow

1. CLI or MCP submits a run plan from `pytxo.toml`
2. Orchestrator computes waves (path conflicts + `depends_on`)
3. Runner spawns agents in worktrees, streams events to SQLite
4. Optional Reality Deck reads store for topology UI

## Moat routing

All new orchestration features should route through [Signal Core](/docs/concepts/signal-core), [Blast Shield](/docs/concepts/blast-shield), and [Race Shield](/docs/concepts/race-shield) — not bypass them.

See [repo layout](/docs/developers/repo-layout) for directory map.
