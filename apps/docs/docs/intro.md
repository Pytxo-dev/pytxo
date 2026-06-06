---
sidebar_position: 1
slug: /
title: Introduction
---

# Introduction

**Pytxo** is a local-first **agent hypervisor and telemetry plane**. It coordinates headless terminal agents — Claude Code, Codex, Antigravity CLI, or any shell command — inside managed pseudo-terminals (PTYs), schedules dependency-aware waves, and logs structured telemetry to SQLite.

Pytxo is **not** an IDE and **not** a cloud virtual workspace. Your editor stays your editor. Pytxo plugs in through a local MCP hub and optional **Reality Deck** desktop telemetry.

## What you get

- **Parallel agents** in isolated git worktrees without stomping the same files
- **DAG wave scheduling** so conflicting tasks run in separate waves
- **Signal Core** context scaffolding to cut noisy token spend (roadmap: tree-sitter AST skeletons)
- **Blast Shield** copy-on-write isolation and **Race Shield** stdin buffering (partial / rolling out per crate)
- **MCP integration** so Cursor or your IDE drives orchestration locally

## Who this is for

| Audience | Start here |
|----------|------------|
| New users | [Install](/docs/getting-started/install) → [First three-agent run](/docs/getting-started/first-three-agent-run) |
| IDE users | [MCP from Cursor](/docs/getting-started/mcp-from-cursor) |
| Operators | [CLI reference](/docs/reference/cli), [`pytxo.toml`](/docs/reference/pytxo-toml) |
| Contributors | [Architecture](/docs/developers/architecture), [Contributing](/docs/developers/contributing) |

## Terminal dashboard

Run `pytxo` with no subcommand to open the **TUI** — doctor status, recent runs, execution domains, and HITL approvals. Subcommands (`run`, `doctor`, `status`, …) are for scripts and automation.

## Quick mental model

```text
IDE / CLI  →  MCP hub  →  Orchestration (Rust)  →  PTY agents in worktrees
                              ↓
                    Reality Deck (optional telemetry UI)
```

Read [What is Pytxo?](/docs/concepts/what-is-pytxo) for positioning vs cloud ADE products, or jump straight to [Install](/docs/getting-started/install).
