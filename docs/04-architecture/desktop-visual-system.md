---
title: Pytxo Desktop visual system
slug: desktop-visual-system
status: active
tags: [presentation, ui, design]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-08-01
related: [[presentation-passive-telemetry]], [[product-vision]], [[signal-core]], [[pytxo-improvement-research]], [[ADR-0034-immutable-review-package-and-durable-apply]]
---

# Pytxo Desktop visual system

**Pytxo Desktop** is the optional control UI for composing missions, watching
execution, reviewing prepared changes, and deciding whether to Apply.

## Aesthetic

| Token | Value | Use |
|-------|-------|-----|
| **Deep void** | `#020205` | App background, panels |
| **Teal** | accent | Active runs, healthy agents |
| **Violet** | accent | Waves, DAG edges |
| **Solar gold** | accent | Cost, approvals, warnings |

Typography: crisp system UI or geometric sans; **tabular numbers** for token/cost readouts.

## Primary surface (shipping)

**Desktop 2** opens on Flow. Compose, Active, History, and Run Review stay in
one mission surface. Operations, Workspaces, and Settings are the other
primary destinations. Structural Focus remains available from mission context,
and per-run Signal scaffold savings appear when `agent_arbitrage` samples
exist.

**Legacy Deck:** optional 3D AST topology (`TopologyScene3D.svelte`) from
[[ADR-0023-reality-deck-3d-renderer]]. It is lazy-loaded only in development
when both Deck compatibility flags are enabled.

There is **no** shipped `TopologyPanel.svelte` 2D canvas fallback (historical Phase 7 claim; corrected Phase 73).

Supporting panels (not primary):

- Collapsed log stream (legacy shell)
- Exact prepared-file diff and Apply history inside Run Review
- Agent logs, ownership detail, and wave timing

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
| Flow mission surface | **default** (v1.1) |
| Structural Focus | contextual mission detail |
| Signal arbitrage bar on Focus / Review | shipped Phase 73 (`agent_arbitrage`) |
| Void + neon design system | shipped |
| 2D `TopologyPanel.svelte` | **removed / never present in current tree** — docs claim retired Phase 73 |
| 3D AST topology | development-only legacy Deck (`TopologyScene3D.svelte`) |
| Closed-loop retry telemetry | shipped — runner emits a `signal-retry` WAL event |
| Fleet panel in Desktop 2 | **shipped Phase 74** (thin Ops list from `snapshot.fleets`) |

Capture marketing stills under `docs/_attachments/` when needed — not inline base64.
