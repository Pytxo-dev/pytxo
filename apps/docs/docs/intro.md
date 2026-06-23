---
sidebar_position: 1
slug: /
title: Introduction
---

# Introduction

**Pytxo** is a local-first **agent hypervisor**. It runs headless terminal agents (Claude Code, Codex, Antigravity CLI, or any shell command) in managed pseudo-terminals, schedules dependency-aware waves, and logs structured telemetry to SQLite on your machine.

Pytxo is **not** an IDE and **not** a cloud virtual workspace. Your editor stays your editor. Pytxo plugs in through a local MCP hub and an optional **Reality Deck** desktop app.

## At a glance

- **Parallel agents** in isolated copies of your repo — no accidental overwrites
- **Wave scheduling** so conflicting tasks never run at the same time
- **Signal Core** — send structural code skeletons instead of whole files to save tokens
- **Human approvals** (Galaxy tier) for risky commands and merges
- **Multi-folder projects** — one swarm across API, web, and shared protos
- **Fleet runs** — coordinate tasks across multiple repos when you need barriers between them
- **MCP integration** so Cursor or your IDE drives orchestration locally

## Who this is for

| You want to… | Start here |
|--------------|------------|
| Install and run your first swarm | [Install](/docs/getting-started/install) → [First three-agent run](/docs/getting-started/first-three-agent-run) |
| Use Cursor or another IDE | [MCP from Cursor](/docs/getting-started/mcp-from-cursor) |
| Operate from the terminal | [Hypervisor Shell](/docs/getting-started/hypervisor-shell) |
| Look up commands and config | [CLI reference](/docs/reference/cli), [`pytxo.toml`](/docs/reference/pytxo-toml) |

## Hypervisor Shell

Run `pytxo` with no subcommand to open the **Hypervisor Shell** — trust the folder, preview waves, dispatch BYOK agents, and approve risky actions. See [Folder trust](/docs/getting-started/folder-trust) and [Hypervisor Shell](/docs/getting-started/hypervisor-shell).

## Quick mental model

```text
IDE / CLI  →  MCP hub  →  Orchestration  →  PTY agents in isolated copies
                              ↓
                    Reality Deck (optional telemetry UI)
```

Read [What is Pytxo?](/docs/concepts/what-is-pytxo) for how this differs from cloud agent workspaces, or jump to [Install](/docs/getting-started/install).

## Troubleshooting

Run `pytxo doctor` in any repo. It checks git, writable data dirs, PTY support, approvals persistence, and (when enabled) cloud or billing connectivity.
