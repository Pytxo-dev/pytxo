---
title: Multi-agent orchestration landscape
slug: multi-agent-orchestration-landscape
status: active
tags: [ecosystem, research]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[pytxo-vs-claude-agent-teams]], [[beyond-the-ade]]
---

# Multi-agent orchestration landscape (2026)

Informal map of approaches—not a competitive hit piece. Revisit quarterly.

## Categories

| Category | Examples | Typical model |
|----------|----------|----------------|
| **IDE-embedded ADE** | BridgeSpace-style multi-terminal workspaces | Heavy UI, proprietary cloud credits |
| **CLI-native swarms** | Claude Agent Teams, community “Gas Town”, Conductor patterns | File mailboxes, multiple CLI processes |
| **Orchestration frameworks** | Custom tmux + scripts, Hive-like coordinators | Bring-your-own glue |
| **Control plane** | **Pytxo** | Rust scheduler + MCP + optional cloud |

## Trends (social / eng discourse)

- **Peer-to-peer agent messaging** (not only hub-and-spoke subagents) is becoming productized.
- **Token cost** dominates “how big can my swarm be?” more than CPU for many users.
- **IDE lock-in** is resisted by developers who want Cursor/Neovim unchanged.

## Pytxo positioning

See [[agent-os-vs-virtual-workspace]] and [[MOC-home]].

Deep compare: [[pytxo-vs-claude-agent-teams]].
