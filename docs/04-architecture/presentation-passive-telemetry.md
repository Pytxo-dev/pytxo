---
title: Presentation layer — passive telemetry
slug: presentation-passive-telemetry
status: active
tags: [architecture, svelte, tauri]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-06-02
related: [[three-tier-model]]
---

# Presentation layer — passive telemetry

The Pytxo desktop shell is a **passive telemetry skin** built with **Svelte 5** and **Tauri v2**.

## Constraints

- **No direct filesystem access** from the UI.
- All interaction with workspace state flows through Tauri v2’s serialized **IPC event bus**.

## Implementation notes

- **Svelte 5 Runes** (`$state`, `$derived`, `$effect`) for high-frequency terminal streams.
- ANSI-stripped text painted toward **~60 FPS** without unbounded memory growth.
- **Monaco** for diffs; **xterm.js** for terminal rendering.

## Reality Deck

The primary dashboard for live execution visualization during local or cloud runs. See glossary: [[glossary#Reality Deck]].

This layer must never embed orchestration policy—that belongs in the Rust core ([[three-tier-model]]).
