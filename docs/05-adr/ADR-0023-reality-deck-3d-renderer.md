---
title: ADR-0023 Reality Deck 3D topology renderer
slug: adr-0023-reality-deck-3d-renderer
status: accepted
tags: [adr, desktop, reality-deck, signal-core]
audience: [human, agent]
layer: presentation
created: 2026-06-22
updated: 2026-06-22
adr_id: ADR-0023
related: [[phase-2-reality-deck]], [[signal-core]], [[ADR-0001-three-tier-rust-svelte-tauri]], [[ADR-0012-hypervisor-shell-default-ux]]
---

# ADR-0023: Reality Deck 3D topology renderer

## Status

Accepted

## Context

Phase 27 shipped a **2D canvas** topology panel (`TopologyPanel.svelte`) fed by Signal Core structural graph IPC. The product vision ([[phase-2-reality-deck]]) targets a **space-console AST topology** — depth, orbit navigation, and blast-radius emphasis — without embedding terminal walls or IDE chrome.

The Deck stack is **Svelte 5 + Tauri v2** ([[ADR-0001-three-tier-rust-svelte-tauri]]). Presentation must stay read-only: graph data comes from orchestration IPC, not filesystem access in the UI.

## Decision

1. **Three.js** is the default 3D renderer for Reality Deck topology (Phase 47).
2. **`TopologyScene3D.svelte`** renders module nodes as spheres positioned from the structural graph; **OrbitControls** for camera; **click-to-select** emits node id to the center viewport.
3. The **center column** layout is: 3D topology (primary) · collapsible telemetry log · sidebar left · diff right — unchanged orchestration boundary.
4. Structural graph IPC schema gains an optional **`version`** field (`2` = 3D-aware consumer); Rust payload remains backward compatible.
5. **2D canvas** (`TopologyPanel.svelte`) stays in the sidebar as a compact fallback; 3D is the main viewport.

## Consequences

- `three` added to `apps/desktop` npm dependencies (~600 KB gzip; acceptable for desktop shell).
- Light/dark theme tokens apply to xterm and 3D background via CSS variables.
- Future Reality Deck work (instancing, edge particles, root_id color lanes) extends this scene graph — no second renderer fork.

### Honesty addendum (Phase 73)

Desktop **v0.5.0+** defaults to **Desktop 2** structural Focus (`FocusScreen.svelte`), not Three.js. `TopologyScene3D.svelte` remains on the **legacy shell** only (`desktop_shell_v1=true`). The Decision above still describes the 3D renderer when that shell is enabled; it is **not** the default primary surface. The historical `TopologyPanel.svelte` 2D fallback is **not** present in the tree — do not claim it as shipped. See [[desktop-visual-system]] and [[pytxo-improvement-research]].

**Superseded (primary viewport):** [[ADR-0032-desktop-2-focus-flow-primary]] supersedes the “3D is the primary center viewport” claim in this ADR.

## Alternatives rejected

| Alternative | Why rejected |
|-------------|--------------|
| Babylon.js | Heavier API surface; no existing Deck integration |
| raw WebGL | Reinvents controls, picking, and resize handling |
| Keep 2D only | Does not match space-console product narrative |
| Tauri 3D native | Cross-platform GPU story harder than WebGL in webview |

Back: [[index]]
