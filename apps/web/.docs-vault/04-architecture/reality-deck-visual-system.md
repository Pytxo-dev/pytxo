---
title: Reality Deck visual system
slug: reality-deck-visual-system
status: active
tags: [presentation, ui, design]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-06-02
related: [[presentation-passive-telemetry]], [[product-vision]], [[signal-core]]
---

# Reality Deck visual system

The **Reality Deck** is Pytxo’s premium **space-themed console** — telemetry and structure, not a terminal wall.

## Aesthetic

| Token | Value | Use |
|-------|-------|-----|
| **Deep void** | `#020205` | App background, panels |
| **Teal** | accent | Active runs, healthy agents |
| **Violet** | accent | Waves, DAG edges |
| **Solar gold** | accent | Cost, approvals, warnings |

Typography: crisp system UI or geometric sans; **tabular numbers** for token/cost readouts.

## Primary surface (target)

**3D AST dependency topology** — interactive graph of modules/symbols derived from Signal Core’s parse tree. Developers see **live blast radius** as agents edit (which nodes glow, which edges are new).

Supporting panels (not primary):

- Collapsed log stream (xterm or stripped ANSI feed from WAL)
- Per-agent diff on approve path ([[blast-shield]])
- Run list and wave timeline (shipped in v0.1 desktop)

## Constraints

- Svelte 5 + Tauri v2 only; **no direct filesystem access** from UI ([[presentation-passive-telemetry]])
- IPC → `pytxo-orchestrate` / `pytxo-store` only
- No embedded multi-pane raw terminal grid (anti-pattern per [[product-vision]])

## Repo

Implementation: `apps/desktop`. Theme tokens should live as CSS variables for light/dark void variants later.

## Status

| Feature | Status |
|---------|--------|
| Run / wave / log / diff panels | v0.1 shipped |
| Void + neon design system (CSS tokens in `App.svelte`) | shipped; consumed by canvas via `getComputedStyle` |
| 2D AST topology (blast radius) | shipped — `TopologyPanel.svelte` sizes/colors nodes by per-agent edited paths + saved tokens (`agent_arbitrage` IPC, fed from `arbitrage_samples`) |
| Closed-loop retry telemetry | shipped — runner emits a `signal-retry` WAL event |
| 3D AST topology | planned (evaluate Three.js/WebGPU only if 2D force graph proves insufficient) |

Capture marketing stills under `docs/_attachments/` when the theme lands — not inline base64.
