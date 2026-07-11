---
title: Multi-agent orchestration landscape
slug: multi-agent-orchestration-landscape
status: active
tags: [ecosystem, research]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-09
related: [[pytxo-vs-claude-agent-teams]], [[beyond-the-ade]], [[pytxo-vs-ade-virtual-workspace]]
---

# Multi-agent orchestration landscape (2026)

Informal map of approaches—not a competitive hit piece. Revisit quarterly.

## Categories

| Category | Examples | Typical model |
|----------|----------|----------------|
| **IDE-embedded ADE** | BridgeSpace (BridgeMind), Emdash, Cursor Agents Window | Heavy UI, parallel CLI panes / worktrees |
| **Cloud control plane** | Warp Oz | Multi-harness fleet ops, often cloud-first |
| **CLI-native swarms** | Claude Agent Teams, community coordinators | File mailboxes, multiple CLI processes |
| **Agent runtimes** | Hermes Agent, OpenHands, Devin-class workers | Single evolving / autonomous agent — not multi-harness hypervisors |
| **Governance SDKs** | Microsoft Agent Governance Toolkit (“agent hypervisor”) | Enterprise policy rings — lexical overlap only |
| **Local agent hypervisor** | **Pytxo** | Rust scheduler + MCP + optional Desktop; Signal / Blast / Race moats |

## Named peers (adjacency)

| Product | Overlap with Pytxo | Differentiation |
|---------|--------------------|-----------------|
| **BridgeSpace** | Multi-agent + CLI harnesses + MCP | ADE workroom UI vs Pytxo headless hypervisor |
| **Emdash** | Parallel headless CLIs, worktrees | Open-source ADE; thinner structural moats |
| **Warp Oz** | Orchestrate heterogeneous coding agents | Cloud/enterprise control plane vs local-first Desktop |
| **Hermes Agent** | Local/self-hosted agent that runs tools | Single agent runtime — closer to execution yard than competitor |
| **Claude Agent Teams** | Parallel agents on one repo | Single-vendor; no cross-tool scheduler |

## Trends

- **ADE** is the default market label for multi-agent coding UIs — Pytxo uses it to *contrast*, not to join.
- **Control plane** language is crowded (Warp Oz); pair **hypervisor** with *local coding agents* to avoid Microsoft governance SDK confusion.
- **Token cost** and **write collisions** dominate “how big can my swarm be?” more than CPU for many users.
- Developers increasingly **compose stacks** (Cursor + Claude Code ± cloud agents) — favor “coordinates the agents you already run.”

## Pytxo positioning

See [[agent-os-vs-virtual-workspace]], [[beyond-the-ade]], and [[MOC-home]].

Deep compare: [[pytxo-vs-claude-agent-teams]], [[pytxo-vs-ade-virtual-workspace]].
