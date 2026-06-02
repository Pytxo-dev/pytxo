---
title: Presentation layer — passive telemetry
slug: presentation-passive-telemetry
status: active
tags: [architecture, svelte, tauri]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-06-02
related: [[three-tier-model]], [[reality-deck-visual-system]], [[product-vision]]
---

# Presentation layer — passive telemetry

The Pytxo desktop shell is a **passive telemetry skin** built with **Svelte 5** and **Tauri v2** — see [[reality-deck-visual-system]] for the space-console aesthetic and 3D AST topology target.

## Constraints

- **No direct filesystem access** from the UI.
- All workspace state flows through Tauri v2 **IPC** into `pytxo-orchestrate` / `pytxo-store`.
- **Not** a multi-pane embedded terminal wall ([[product-vision]]).

## Implementation notes

- **Svelte 5 Runes** (`$state`, `$derived`, `$effect`) for high-frequency streams.
- Log panel: bounded scrollback, ~60 Hz poll budget (not unbounded raw text as the hero).
- **xterm.js** for supporting log view; **Monaco** or inline diff for approve-path review.
- **Primary (target):** 3D AST dependency graph driven by [[signal-core]] parse output.

## Reality Deck

Live execution visualization for local and cloud runs. Shipped panels: runs, waves, logs, diff. North-star UI: structural blast radius on the topology graph.

Repo: [pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop).

Orchestration policy never lives in this layer ([[three-tier-model]]).
