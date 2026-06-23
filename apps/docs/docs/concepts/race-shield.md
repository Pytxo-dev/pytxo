---
title: Race Shield
---

# Race Shield

**Race Shield** prevents cross-agent write collisions and stdin races through a lock-free swarm registry and buffered stdin handling.

## Scheduling

- Tasks declare `paths` and optional `depends_on` edges
- Pytxo builds a **DAG** and executes **waves** of non-conflicting agents
- `max_agents` caps parallelism per wave
- Cross-root path overlaps in [modular projects](/docs/concepts/modular-projects) surface as scheduler warnings

## Stdin

- PTY backend: continuous stdin drain loop
- Subprocess backend: `subprocess_stdin = true` pumps the Race Shield queue at spawn time

MCP tools `pytxo_stdin` and `pytxo_route_stdin` route bytes to live agents from your IDE.

## Configuration

```toml
max_agents = 3
dag_explicit_deps = false   # set true to force DAG mode
```

Use `pytxo run --dry-run` to inspect waves before execution.

## Galaxy approvals

Race Shield handles **when** agents run; [Galaxy approvals](/docs/concepts/galaxy-approvals) handle **what risky commands** may run once an agent is live.

Back: [Three moats](/docs/concepts/three-moats)
