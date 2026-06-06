---
title: CLI reference
slug: cli-reference
status: active
tags: [reference, cli]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [pytxo-toml](/docs/pytxo-toml), [repository-layout](/docs/repository-layout)
---

# CLI reference

Build: `cargo build -p pytxo-cli`  
Binary: `cargo run -p pytxo-cli -- <cmd>`

## Commands

| Command | Description |
|---------|-------------|
| `pytxo init` | Create `.pytxo/` dirs and gitignore hint |
| `pytxo doctor` | Verify git, HEAD, worktree support, writable `.pytxo/`, PTY smoke |
| `pytxo run` | Schedule and execute agents in worktrees |
| `pytxo status` | List runs and agents from SQLite |
| `pytxo logs --agent <id>` | Tail stored stdout/stderr events |
| `pytxo stop` | Stop active run or all tracked processes |

## Shared flags

| Flag | Commands | Description |
|------|----------|-------------|
| `--repo` | `init`, `doctor`, `run`, `status`, `logs`, `stop` | Git repository root (default: cwd) |
| `--config` | `run`, `status`, `logs`, `stop` | Path to `pytxo.toml` |
| `--json` | `doctor`, `status` | Machine-readable output |

## `pytxo doctor`

Fails fast before `run` when:

- `git` is on PATH
- cwd (or `--repo`) is inside a git work tree
- `HEAD` exists (at least one commit)
- `git worktree` works
- `.pytxo/` is writable

## `pytxo run` flags

| Flag | Default | Description |
|------|---------|-------------|
| `--agents` | `3` | Cap parallel agents per wave |
| `--cmd` | `echo pytxo` | Shell command run in each worktree |
| `--dry-run` | off | Print JSON execution plan |
| `--keep-worktrees` | off | Do not remove worktrees after run |
| `--execution` | from `pytxo.toml` | Override `execution_backend`: `pty` or `subprocess` |

`run` calls the same git preflight as `doctor` before scheduling. Default execution uses **PTY** ([ADR-0010-pty-default-execution-backend](/docs/adr-0010-pty-default-execution-backend)).

## Agent IDs in logs

Format: `{run_uuid}:{agent-N}` (e.g. for `pytxo logs --agent`).
