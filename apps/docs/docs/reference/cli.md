---
title: CLI reference
---

# CLI reference

Binary: `pytxo` (v0.1.0)

## Default behavior

```bash
pytxo          # interactive TUI dashboard
pytxo --help   # subcommand list
```

Set `PYTXO_NO_TUI=1` to print help instead of launching the TUI.

## Commands

| Command | Description |
|---------|-------------|
| `pytxo init` | Create `.pytxo/` dirs and gitignore hint |
| `pytxo doctor` | Verify git, HEAD, worktrees, writable `.pytxo/`, PTY smoke |
| `pytxo run` | Schedule and execute agents in worktrees |
| `pytxo status` | List runs and agents from SQLite |
| `pytxo logs --agent <id>` | Tail stored stdout/stderr events |
| `pytxo stop` | Stop active run or all tracked processes |
| `pytxo domains` | List registered execution domains |
| `pytxo project` | Multi-root modular project manifest |
| `pytxo hitl` | Human-in-the-loop approval queue |

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
| `--cmd` | `echo pytxo` | Shell command in each worktree |
| `--dry-run` | off | Print JSON execution plan |
| `--keep-worktrees` | off | Do not remove worktrees after run |
| `--execution` | from config | Override `pty`, `subprocess`, or `cloud` |

## `pytxo project` subcommands

| Subcommand | Description |
|------------|-------------|
| `init <id>` | Create project manifest under `~/.pytxo/projects/` |
| `list` | List path roots for a project |
| `paths --add <path>` | Add a path root |
| `status` | Recent runs for the project primary domain |
| `run` | Coordinated run across project roots |

## `pytxo hitl` subcommands

| Subcommand | Description |
|------------|-------------|
| `list` | Pending approval requests |
| `approve <id>` | Approve a request |
| `deny <id>` | Deny a request |

## Agent IDs

Format: `{run_uuid}:agent-N` — use with `pytxo logs --agent`.

See also: [`pytxo.toml`](/docs/reference/pytxo-toml)
