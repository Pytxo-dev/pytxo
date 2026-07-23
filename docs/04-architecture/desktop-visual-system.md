---
title: Pytxo Desktop visual system
slug: desktop-visual-system
status: active
tags: [presentation, ui, design]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-07-18
related: [[presentation-passive-telemetry]], [[product-vision]], [[signal-core]], [[pytxo-improvement-research]]
---

# Pytxo Desktop visual system

**Pytxo Desktop** is the optional control UI — telemetry and structure, not a terminal wall. (Formerly called Reality Deck.)

## Aesthetic

| Token | Value | Use |
|-------|-------|-----|
| **Deep void** | `#020205` | App background, panels |
| **Teal** | accent | Active runs, healthy agents |
| **Violet** | accent | Waves, DAG edges |
| **Solar gold** | accent | Cost, approvals, warnings |

Typography: crisp system UI or geometric sans; **tabular numbers** for token/cost readouts.

## Primary surface (shipping)

**Desktop 2** (default): **structural Focus graph** — Signal Core nodes/edges as an interactive list (`FocusScreen.svelte`), plus Ops / Flow / Approvals / Run Review. Per-run **Signal arbitrage** saved-token totals surface via `agent_arbitrage` IPC when samples exist.

**Legacy shell** (`desktop_shell_v1=true`): optional **3D AST topology** (`TopologyScene3D.svelte`) from [[ADR-0023-reality-deck-3d-renderer]]. Not the default product surface after v0.5.0.

There is **no** shipped `TopologyPanel.svelte` 2D canvas fallback (historical Phase 7 claim; corrected Phase 73).

Supporting panels (not primary):

- Collapsed log stream (legacy shell)
- Per-agent diff on approve path ([[blast-shield]])
- Run list and wave timeline

## Constraints

- Svelte 5 + Tauri v2 only; **no direct filesystem access** from UI ([[presentation-passive-telemetry]])
- IPC → `pytxo-orchestrate` / `pytxo-store` only
- No embedded multi-pane raw terminal grid (anti-pattern per [[product-vision]])

## Repo

Implementation: `apps/desktop`. Theme tokens should live as CSS variables for light/dark void variants later.

## Status

| Feature | Status |
|---------|--------|
| Run / wave / log / diff panels | shipped (legacy + Desktop 2) |
| Desktop 2 structural Focus | **default** (v0.5.0+) |
| Signal arbitrage bar on Focus / Review | shipped Phase 73 (`agent_arbitrage`) |
| Void + neon design system | shipped |
| 2D `TopologyPanel.svelte` | **removed / never present in current tree** — docs claim retired Phase 73 |
| 3D AST topology | legacy shell only (`TopologyScene3D.svelte`) |
| Closed-loop retry telemetry | shipped — runner emits a `signal-retry` WAL event |
| Fleet panel in Desktop 2 | **shipped Phase 74** (thin Ops list from `snapshot.fleets`) |

Capture marketing stills under `docs/_attachments/` when needed — not inline base64.
