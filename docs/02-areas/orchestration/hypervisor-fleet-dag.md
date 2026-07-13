---
title: Hypervisor fleet DAG
slug: hypervisor-fleet-dag
status: active
tags: [orchestration, hypervisor, scheduling]
audience: [human, agent]
layer: orchestration
created: 2026-06-19
updated: 2026-06-19
related: [[execution-domains]], [[dag-flow-engine]], [[ADR-0015-hypervisor-fleet-dag]], [[modular-projects]]
---

# Hypervisor fleet DAG

**Fleet DAG** is hypervisor-level orchestration: a directed acyclic graph of **nodes**, each dispatching a separate [[execution-domains|execution domain]] run, with **barrier sync** between waves.

Distinct from:

| Mechanism | Scope |
|-----------|--------|
| `depends_on` in `pytxo.toml` | Single domain / single `pytxo.db` |
| [[modular-projects]] `project run` | Cross-**root** within one primary domain |
| **Fleet DAG** | Cross-**repo** domains with explicit barriers |

ADR: [[ADR-0015-hypervisor-fleet-dag]].

## Fleet manifest

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

Discovery: `--manifest <file>` or `~/.pytxo/fleets/<id>.toml`.

## Execution model

```text
Wave 0: dispatch node A → domain A → wait terminal
Wave 1: dispatch nodes B,C in parallel → wait all
```

- Each node calls `HypervisorRegistry::dispatch` (non-blocking PTY run).
- Coordinator polls each domain's `pytxo.db` until `completed` / `failed`.
- Fleet metadata lands in `~/.pytxo/hypervisor.db` (`fleet_runs`, `fleet_nodes`).
- Per-domain WAL streams are **never merged**.

## CLI

```bash
pytxo fleet init my-pipeline --add /path/api --add /path/web --cmd "echo wave"
pytxo fleet dry-run --id my-pipeline
pytxo fleet run --id my-pipeline
pytxo fleet status --id my-pipeline --json
```

Edit `depends_on` in the manifest after `fleet init` for ordering.

## Dashboard

- `pytxo domains` — enriched per-domain `active_runs` and `latest_run_status` (use `--paths-only` for legacy output).
- Pytxo Desktop `list_domains_status` IPC — domain badges when viewing **All** projects.
- TUI board aggregates recent runs when multiple catalog domains exist.

Back: [[execution-domains]] · [[MOC-home]]
