---
title: Multi-agent orchestration landscape
slug: multi-agent-orchestration-landscape
status: active
tags: [ecosystem, research]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-08-27
related: [[product-vision]], [[commit-layer]], [[pytxo-commit-layer-alignment]], [[pytxo-vs-claude-agent-teams]], [[pytxo-vs-github-copilot-app]], [[beyond-the-ade]], [[pytxo-vs-ade-virtual-workspace]], [[pytxo-improvement-research]], [[competitive-landscape-2026-07]]
---

# Multi-agent orchestration landscape (2026)

> [!NOTE]
> Category snapshot for the execution-yard market. Pytxo now treats these
> products mainly as substrates or adjacent controls; the current thesis is the
> [[commit-layer|commit boundary for autonomous work]].

Informal map of approaches—not a competitive hit piece. Revisit quarterly. Deep synthesis: [[pytxo-improvement-research]]. Full competitor dossiers + GTM compare priorities: [[competitive-landscape-2026-07]].

## Categories

| Category | Examples | Typical model |
|----------|----------|----------------|
| **IDE-embedded ADE** | BridgeSpace (BridgeMind), Emdash, Cursor Agents Window | Heavy UI, parallel CLI panes / worktrees |
| **Vendor agent desktop** | GitHub Copilot app | Control center, worktree-per-session, local/cloud sandboxes, canvases |
| **Cloud VM agents** | Cursor Cloud Agents, Google Jules, Copilot cloud sandbox | Isolated VMs → PR / artifacts |
| **Cloud control plane** | Warp Oz | Multi-harness fleet ops, often cloud-first |
| **CLI-native swarms** | Claude Agent Teams, community coordinators | File mailboxes, multiple CLI processes |
| **OS-sandboxed coding agents** | OpenAI Codex (CLI/IDE) | Seatbelt / bwrap+seccomp / Windows token + approvals |
| **Agent runtimes** | OpenHands, Devin-class workers, Factory Missions (preview) | Single evolving / platform agent — not multi-harness hypervisors |
| **Governance SDKs** | Microsoft Agent Governance Toolkit (“agent hypervisor”) | Enterprise policy rings — lexical overlap only |
| **Local agent hypervisor** | **Pytxo** | Rust scheduler + MCP + optional Desktop; Signal / Blast / Race moats |

## Named peers (adjacency)

| Product | Overlap with Pytxo | Differentiation |
|---------|--------------------|-----------------|
| **GitHub Copilot app** | Parallel agents, worktrees, supervision UI | GitHub-native seat; not cross-CLI BYOK hypervisor — [[pytxo-vs-github-copilot-app]] |
| **OpenAI Codex** | Local sandbox + approval mental model | Single-vendor agent; OS Seatbelt/bwrap — not mixed-CLI Race registry |
| **Claude Agent Teams** | Parallel agents on one repo | Single-vendor; docs warn **no worktree isolation** / same-file overwrites — [[pytxo-vs-claude-agent-teams]] |
| **Cursor Cloud Agents** | Parallel agents, multi-repo, MCP | Cloud VMs + API billing — not local-first silicon |
| **Google Jules** | Plan approve → async code → PR | Cloud-only Gemini worker — not local orchestration |
| **BridgeSpace / Emdash** | Multi-agent + CLI harnesses | ADE workroom UI vs Pytxo headless hypervisor |
| **Warp Oz** | Heterogeneous coding agents | Cloud/enterprise control plane vs local-first Desktop |
| **Aider / OpenHands / Factory** | Local/BYOK or sandbox platforms | Adjacent runtimes — not category peers |

## Trends

- Market converged on **parallel agents + sandbox + control-center UX** — those alone are not moats.
- **ADE** remains the label Pytxo contrasts; **vendor agent desktops** (Copilot app) are the closer category peer for Desktop UX.
- **Token cost** and **write collisions** dominate swarm sizing; Claude Agent Teams docs explicitly warn about both.
- Developers **compose stacks** (Cursor + Claude Code + Codex ± cloud) — favor “coordinates the agents you already run.”

## Pytxo positioning

Wedge: **local multi-CLI hypervisor** with Race collision policy, Signal structural context, and honest Blast approve-to-flush — not unique multi-agent or unique sandbox.

See [[agent-os-vs-virtual-workspace]], [[beyond-the-ade]], [[pytxo-improvement-research]], [[MOC-home]].

Deep compare: [[pytxo-vs-claude-agent-teams]], [[pytxo-vs-github-copilot-app]], [[pytxo-vs-ade-virtual-workspace]].
