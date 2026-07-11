---
title: Agent OS vs virtual workspace
slug: agent-os-vs-virtual-workspace
status: active
tags: [positioning, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-09
related: [[beyond-the-ade]], [[mcp-hub-integration]], [[three-tier-model]], [[execution-domains]], [[product-vision]]
---

# Agent OS vs virtual workspace

Pytxo is an **agent hypervisor** — a bare-metal control plane, not a replacement IDE or a BridgeSpace-style virtual workspace. See [[product-vision]].

## Principles

- **IDE-agnostic:** Plugs into Cursor, VS Code, NeoVim, or plain CLIs via a stateless, local-first [[mcp-hub-integration]].
- **Lean footprint:** Orchestration and telemetry stay out of the hot path of your editor’s renderer.
- **Throughput-first:** Maximize hardware execution for headless CLIs rather than painting N terminal UIs at once.
- **Multi-repo hypervisor:** One control plane, many [[execution-domains]]—concurrent runs on different `repo_root` paths without cross-project state leakage ([[product-vision]]).

## What users still get

Optional **Pytxo Desktop** ([[desktop-visual-system]]) shows structural blast radius and telemetry — not walls of raw terminal panes.

## Related

- [[beyond-the-ade]]
- [[three-tier-model]]
