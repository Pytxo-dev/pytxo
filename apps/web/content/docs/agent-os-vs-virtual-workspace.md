---
title: Agent OS vs virtual workspace
slug: agent-os-vs-virtual-workspace
status: active
tags: [positioning, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [beyond-the-ade](/docs/beyond-the-ade), [mcp-hub-integration](/docs/mcp-hub-integration), [three-tier-model](/docs/three-tier-model), [execution-domains](/docs/execution-domains), [product-vision](/docs/product-vision)
---

# Agent OS vs virtual workspace

Pytxo is an **agent hypervisor** — a bare-metal control plane, not a replacement IDE or a BridgeSpace-style virtual workspace. See [product-vision](/docs/product-vision).

## Principles

- **IDE-agnostic:** Plugs into Cursor, VS Code, NeoVim, or plain CLIs via a stateless, local-first [mcp-hub-integration](/docs/mcp-hub-integration).
- **Lean footprint:** Orchestration and telemetry stay out of the hot path of your editor’s renderer.
- **Throughput-first:** Maximize hardware execution for headless CLIs rather than painting N terminal UIs at once.
- **Multi-repo hypervisor:** One control plane, many [execution-domains](/docs/execution-domains)—concurrent runs on different `repo_root` paths without cross-project state leakage ([product-vision](/docs/product-vision)).

## What users still get

An optional **Reality Deck** ([reality-deck-visual-system](/docs/reality-deck-visual-system)) shows structural blast radius and telemetry — not walls of raw terminal panes.

## Related

- [beyond-the-ade](/docs/beyond-the-ade)
- [three-tier-model](/docs/three-tier-model)
