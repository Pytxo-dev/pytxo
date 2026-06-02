---
title: pytxo.toml reference
slug: pytxo-toml
status: active
tags: [reference, config]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[MOC-home]]
---

# pytxo.toml reference

## Top-level

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_agents` | usize | `3` | Max parallel agents per wave |
| `worktree_dir` | path | `.pytxo/worktrees` | Worktree root |
| `data_dir` | path | `.pytxo/data` | SQLite + state |
| `fail_fast` | bool | `true` | Fail run if any agent exits non-zero |
| `sanitize` | bool | `true` | Sovereign Shield redaction before WAL/MCP |
| `signal_core` | bool | `true` | Materialize scaffolded context for task paths |
| `signal_fidelity` | string | `low` | `low` \| `medium` \| `high` — Signal Core tier |
| `isolation` | string | `worktree` | `worktree` \| `overlay` (overlay delegates to worktrees until sparse FS) |
| `dag_explicit_deps` | bool | `false` | Force DAG scheduling mode |

## `[[agent]]`

| Key | Description |
|-----|-------------|
| `name` | Agent profile name |
| `paths` | Optional owned globs (documentation / future enforcement) |

## `[[task]]`

| Key | Description |
|-----|-------------|
| `id` | Task identifier |
| `agent` | Agent profile name |
| `paths` | Paths/globs used for **conflict preflight** |

Tasks with overlapping `paths` are scheduled in different **waves**.

## Example

See [`pytxo.toml.example`](../../pytxo.toml.example) at repo root.

## CLI

```bash
pytxo run --config pytxo.toml --dry-run
pytxo run --config pytxo.toml --cmd "claude"
```

If no config tasks are defined, `pytxo run --agents N` uses synthetic disjoint paths.
