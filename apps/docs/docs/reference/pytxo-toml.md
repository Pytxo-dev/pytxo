---
title: pytxo.toml
---

# pytxo.toml reference

## Top-level keys

| Key | Default | Description |
|-----|---------|-------------|
| `max_agents` | `3` | Max parallel agents per wave |
| `worktree_dir` | `.pytxo/worktrees` | Worktree root |
| `data_dir` | `.pytxo/data` | SQLite + state |
| `fail_fast` | `true` | Fail run if any agent exits non-zero |
| `permission_profile` | `orbit` | See [permission tiers](/docs/reference/permission-tiers) |
| `sanitize` | `true` | Redact secrets before WAL/MCP |
| `signal_core` | `true` | Scaffold context for task paths |
| `signal_fidelity` | `low` | `low` \| `medium` \| `high` |
| `isolation` | `worktree` | `worktree` \| `overlay` |
| `dag_explicit_deps` | `false` | Force DAG scheduling mode |

## `[[agent]]`

| Key | Description |
|-----|-------------|
| `name` | Scheduling profile name (referenced by `[[task]].agent`) |
| `paths` | Optional owned globs |
| `permission_profile` | Optional per-agent override |

## `[[task]]`

| Key | Description |
|-----|-------------|
| `id` | Task identifier |
| `agent` | Agent profile name |
| `paths` | Paths for conflict preflight and Signal Core |
| `depends_on` | Task ids that must finish first |
| `root` | Optional modular project root label |
| `signal_fidelity` | Per-task fidelity override |

Tasks with overlapping `paths` are scheduled in different **waves**.

## Example

Copy `pytxo.toml.example` from the repository root for a working three-agent configuration.
