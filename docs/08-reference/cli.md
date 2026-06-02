---
title: CLI reference (Phase 1)
slug: cli-reference
status: active
tags: [reference, cli]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[pytxo-toml]]
---

# CLI reference (Phase 1)

Build: `cargo build -p pytxo-cli`  
Binary: `cargo run -p pytxo-cli -- <cmd>`

## Commands

| Command | Description |
|---------|-------------|
| `pytxo init` | Create `.pytxo/` dirs and gitignore hint |
| `pytxo run` | Schedule and execute agents in worktrees |
| `pytxo status` | List runs and agents from SQLite |
| `pytxo logs --agent <id>` | Tail stored stdout/stderr events |
| `pytxo stop` | Clear active run state; optional worktree cleanup |

## `pytxo run` flags

| Flag | Default | Description |
|------|---------|-------------|
| `--agents` | `3` | Cap parallel agents per wave |
| `--cmd` | `echo pytxo` | Shell command run in each worktree |
| `--config` | `pytxo.toml` if present | Task definitions |
| `--dry-run` | off | Print JSON execution plan |
| `--keep-worktrees` | off | Do not remove worktrees after run |
| `--repo` | cwd | Git repository root |

## Agent IDs in logs

Format: `{run_uuid}:{agent-N}` (e.g. for `pytxo logs --agent`).
