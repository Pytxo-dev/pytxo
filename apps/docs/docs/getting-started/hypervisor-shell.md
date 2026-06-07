---
sidebar_position: 3
title: Hypervisor Shell
---

# Hypervisor Shell

**Mission control for agent fleets** — not another coding chatbot.

Running `pytxo` with no subcommand opens the **Hypervisor Shell**: a three-zone TUI with a **board** (runs, waves, HITL), **scrollback** (command output and plan previews), and an operator **prompt** (slash commands).

Your ADE (Antigravity, Claude Code, Codex, …) still does the thinking. Pytxo schedules waves, spawns processes in worktrees, and records telemetry.

## Workflow

```text
1. pytxo                         → trust tier picker (first visit) → shell
2. /models search deepseek       → browse BYOK models (key in env)
3. /dry-run --agents 3           → preview wave plan (conflicts, task graph)
4. /run --cmd "agy --help"       → dispatch your CLI in worktrees
5. /status                       → poll run + agent rows on the board
6. /logs agent-0                 → tail WAL events
7. /stop                         → halt the active run
```

Untrusted folders block `/run` but allow `/doctor` and `/dry-run`. See [Folder trust](/docs/getting-started/folder-trust).

## Slash commands (v0.3.0)

| Command | Purpose |
|---------|---------|
| `/help` | List commands |
| `/doctor` | Preflight checks |
| `/dry-run` | JSON wave plan (`--agents N`) |
| `/run` | Dispatch fleet (`--agents N --cmd "…"`) |
| `/status` | Recent runs (`--limit N`) |
| `/logs <agent>` | Event tail (`--tail N`) |
| `/stop` | Stop active run (`--all` for every tracked process) |
| `/trust [tier]` | Trust folder or change permission tier |
| `/models list` | List cached models (`--provider …`) |
| `/models search Q` | Fuzzy model search |

## Scripts and CI

Set `PYTXO_NO_TUI=1` (or pass an explicit subcommand) to keep the classic CLI for automation:

```bash
pytxo doctor
pytxo run --dry-run --agents 2
pytxo run --agents 1 --cmd "echo pytxo"
```

The router lives in the `pytxo-shell` crate so TUI, CLI, and MCP can share the same control plane over time.

## Optional NL planner (v0.2.x)

Natural-language mission lines are **off by default**. Enable with:

```bash
export PYTXO_PLANNER=1
```

or in `pytxo.toml`:

```toml
[planner]
enabled = true
```

When enabled, a mission line is decomposed into `Vec<Task>`, validated through `build_plan()`, previewed in scrollback, then dispatched with `/run`.

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| Enter | Submit prompt |
| ↑ / ↓ (empty prompt) | Command history |
| ↑ / ↓ (while typing) | Scroll scrollback |
| Tab | Cycle HITL selection |
| a / x | Approve / deny HITL (empty prompt) |
| q / Ctrl+C | Quit (Ctrl+C also sends `/stop`) |

See also: [ADR-0012 Hypervisor Shell](/docs/developers/architecture) (in-repo `docs/05-adr/`), [CLI reference](/docs/reference/cli).
