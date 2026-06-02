---
title: Agent OS vs virtual workspace
slug: agent-os-vs-virtual-workspace
status: active
tags: [positioning, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[beyond-the-ade]], [[mcp-hub-integration]], [[three-tier-model]]
---

# Agent OS vs virtual workspace

Pytxo operates as an invisible, bare-metal **agent operating system**—not a replacement IDE.

## Principles

- **IDE-agnostic:** Plugs into Cursor, VS Code, NeoVim, or plain CLIs via a stateless, local-first [[mcp-hub-integration]].
- **Lean footprint:** Orchestration and telemetry stay out of the hot path of your editor’s renderer.
- **Throughput-first:** Maximize hardware execution for headless CLIs rather than painting N terminal UIs at once.

## What users still get

A optional **Reality Deck** ([[presentation-passive-telemetry]]) provides high-fidelity monitoring—passive telemetry, not the sole control surface.

## Related

- [[beyond-the-ade]]
- [[three-tier-model]]
