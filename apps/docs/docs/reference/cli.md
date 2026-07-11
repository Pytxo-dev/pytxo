---
title: CLI reference
---

# CLI reference

Binary: `pytxo` (v0.3.4)

## Default behavior

```bash
pytxo          # interactive Hypervisor Shell
pytxo --help   # subcommand list
```

Set `PYTXO_NO_TUI=1` to print help instead of launching the shell.

## Commands

<div className="table-scroll">

| Command | Description |
|---------|-------------|
| `pytxo init` | Create `.pytxo/` dirs and gitignore hint |
| `pytxo doctor` | Verify git, PTY, approvals persistence, billing/cloud when enabled |
| `pytxo run` | Schedule and execute agents in isolated copies |
| `pytxo status` | List runs and agents from SQLite |
| `pytxo logs --agent <id>` | Tail stored stdout/stderr events |
| `pytxo stop` | Stop active run or all tracked processes |
| `pytxo domains` | List registered execution domains |
| `pytxo project` | Multi-root modular project manifest |
| `pytxo fleet` | Cross-repo fleet DAG |
| `pytxo hitl` | Human-in-the-loop approval queue |
| `pytxo trust [tier]` | Trust repo folder (`orbit`, `galaxy`, …) |
| `pytxo providers` | Provider registry + key configured (boolean) |
| `pytxo models` | List, search, or refresh model catalog |

</div>

## Shared flags

| Flag | Commands | Description |
|------|----------|-------------|
| `--repo` | most | Git repository root (default: cwd) |
| `--config` | `run`, `status`, `logs`, `stop` | Path to `pytxo.toml` |
| `--json` | `doctor`, `status`, `domains` | Machine-readable output |

## `pytxo run` flags

| Flag | Default | Description |
|------|---------|-------------|
| `--agents` | `3` | Max parallel agents per wave |
| `--cmd` | `echo pytxo` | Shell command in each isolated copy |
| `--dry-run` | off | Print JSON execution plan |
| `--keep-worktrees` | off | Do not remove isolated copies after run |
| `--execution` | from config | Override `pty`, `subprocess`, or `cloud` |

## `pytxo project` subcommands

| Subcommand | Description |
|------------|-------------|
| `init <id>` | Create project manifest under `~/.pytxo/projects/` |
| `list` | List path roots for a project |
| `paths --add <path>` | Add a path root |
| `status` | Recent runs for the project primary domain |
| `run` | Coordinated run across project roots |

## `pytxo fleet` subcommands

| Subcommand | Description |
|------------|-------------|
| `init <id>` | Create fleet manifest under `~/.pytxo/fleets/` |
| `dry-run` | Print fleet execution plan JSON |
| `run` | Run fleet DAG with barrier sync |
| `status` | List fleet runs from hypervisor catalog |

`pytxo fleet run --continue-on-error` continues later waves after a node failure.

## `pytxo hitl` subcommands

| Subcommand | Description |
|------------|-------------|
| `list` | Pending approval requests |
| `approve <id>` | Approve a request |
| `deny <id>` | Deny a request |

## Agent IDs

Format: `{run_uuid}:agent-N`. Use with `pytxo logs --agent`.

See also: [`pytxo.toml`](/docs/reference/pytxo-toml) · [Fleet runs](/docs/concepts/fleet-runs) · [Galaxy approvals](/docs/concepts/galaxy-approvals)
