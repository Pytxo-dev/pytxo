# ADR-0007: Explicit task dependencies (`depends_on`)

## Status

Accepted

## Context

Greedy path-overlap waves are insufficient when tasks must run in a defined order without sharing paths.

## Decision

- `task` may set `depends_on = ["task-a"]`.
- `pytxo-scheduler` builds waves with topological ordering plus path-conflict constraints within each wave.
- Cycles return an error unless `PYTXO_DAG_MOCK=1` (experimental; off by default).
- Enable explicit DAG mode globally with `dag_explicit_deps = true` in `pytxo.toml`.

## Consequences

- Mock-break file injection remains behind `PYTXO_DAG_MOCK` only (not production default).
- Overlap conflicts still split tasks across waves when paths collide.
