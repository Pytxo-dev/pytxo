---
title: Race Shield
---

# Race Shield

**Race Shield** prevents cross-agent write collisions and stdin races in monorepo swarms through a lock-free swarm registry and buffered stdin handling.

## Scheduling

- Tasks declare `paths` and optional `depends_on` edges
- Pytxo builds a **DAG** and executes **waves** of non-conflicting agents
- `max_agents` caps parallelism per wave

## Stdin

- PTY backend: continuous stdin drain loop
- Subprocess backend: `subprocess_stdin = true` pumps the Race Shield queue at spawn time

## Configuration

```toml
max_agents = 3
dag_explicit_deps = false   # set true to force DAG mode
```

Use `pytxo run --dry-run` to inspect waves before execution.

Back: [Three moats](/docs/concepts/three-moats)
