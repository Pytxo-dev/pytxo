---
title: ADR-0015 Hypervisor fleet DAG (cross-repo orchestration)
slug: adr-0015-hypervisor-fleet-dag
status: accepted
tags: [adr, orchestration, hypervisor]
audience: [human, agent]
layer: orchestration
created: 2026-06-19
updated: 2026-06-19
adr_id: ADR-0015
related: [[execution-domains]], [[hypervisor-fleet-dag]], [[ADR-0007-explicit-task-dependencies]], [[ADR-0011-modular-project-manifest]]
---

# ADR-0015: Hypervisor fleet DAG (cross-repo orchestration)

## Status

Accepted

## Context

[[ADR-0007-explicit-task-dependencies]] and `pytxo-scheduler` support `depends_on` **within a single domain run** (one `pytxo.toml`, one `pytxo.db`). [[ADR-0011-modular-project-manifest]] extends that to **cross-root** tasks inside one coordinated `project run` on a primary domain.

Productivity max also requires **cross-repo** barriers: e.g. finish agents on `/project1` before dispatching `/project2`, without merging WAL streams or sharing a `SwarmRegistry` across unrelated repos ([[execution-domains]]).

## Decision

### Fleet manifest

1. A **fleet manifest** is TOML with `[fleet]` and one or more `[[node]]` entries:

```toml
[fleet]
id = "api-then-web"
name = "Fix API then deploy web"

[[node]]
id = "fix-api"
repo = "/abs/path/to/api"
cmd = "echo fix"
agents = 2
depends_on = []

[[node]]
id = "deploy-web"
repo = "/abs/path/to/web"
cmd = "echo deploy"
depends_on = ["fix-api"]
```

2. Discovery: explicit `--manifest <file>`, else `~/.pytxo/fleets/<id>.toml`.

3. Types live in `pytxo-core::fleet` (`FleetManifest`, `FleetNode`, `FleetPlan`).

### Orchestration

1. `pytxo-orchestrate::fleet` builds topological **waves** from `depends_on` (acyclic; cycles error).
2. Each wave dispatches nodes in parallel via `HypervisorRegistry::dispatch` — one domain run per node.
3. The coordinator **barrier-waits** until every node run in a wave reaches a terminal status before starting the next wave.
4. Fleet metadata is recorded in `~/.pytxo/hypervisor.db` (`fleet_runs`, `fleet_nodes` tables). Per-domain `pytxo.db` streams are **never merged**.

### CLI

- `pytxo fleet init | dry-run | run | status`

### Non-goals (this ADR)

- Cross-domain Race Shield path claims
- Fleet-level MCP tools (defer until IDE integration needs them)
- Replacing modular `project run` for same-primary multi-root work

## Consequences

- Cross-repo DAG is explicit opt-in via fleet manifests — single-repo users unchanged.
- Dashboard can show fleet run status from `hypervisor.db` alongside per-domain catalog rows.
- Docs: [[hypervisor-fleet-dag]], [[mvp-bootstrap]] Phase 23.
