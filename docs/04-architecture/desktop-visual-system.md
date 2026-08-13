---
title: Pytxo Desktop visual system
slug: desktop-visual-system
status: active
tags: [presentation, ui, design]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-08-13
related: [[presentation-passive-telemetry]], [[product-vision]], [[signal-core]], [[pytxo-improvement-research]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0035-desktop-2-quiet-instrument-ia]]
---

# Pytxo Desktop visual system

**Pytxo Desktop** is the optional control UI — a quiet operator instrument for composing missions, watching execution, reviewing prepared changes, and deciding whether to Apply. (Formerly called Reality Deck.)

## Aesthetic

| Token | Value | Use |
|-------|-------|-----|
| **Deep void** | `#020205` | App background, panels |
| **Teal** | only action/live accent | Active runs, primary buttons, healthy live state |
| **Solar gold** | needs-you / cost | Approvals, spend |
| **Light** | the other skin | Bright workspace; match-system uses Void or Light |

Typography: Geist + Geist Mono; body/rows **13px**, meta **12px**, kbd **11px**; **tabular numbers** for money and counts. One radius: 4px controls, 6px panels. Motion only for selection, open/close, and status change (140–180ms). Reduced motion already disables animation.

Do **not** ship spectrum hairlines, Terminal/Nebula skins, sparkle CTAs, fake waveforms, orbit rings, or decorative CSS graphs. Desktop overrides live in `apps/desktop` CSS — do not restyle `packages/chroma` in a way that silently restyles the marketing site ([[ADR-0029-chroma-shared-design-tokens]]).

## Primary surface (shipping)

**Desktop 2** (default): **Ops** (today), **Missions** (composer + Plan/Live/Review), **Approvals**, **Workspaces**, **Agents**, **Settings**. Per [[ADR-0035-desktop-2-quiet-instrument-ia]]. Structural files-touched lists live inside Mission Review, not a destination named Topology Focus. Per-run Signal scaffold savings appear when `agent_arbitrage` samples exist.

**Legacy Deck:** optional 3D AST topology (`TopologyScene3D.svelte`) from
[[ADR-0023-reality-deck-3d-renderer]]. It is lazy-loaded only in development
when both Deck compatibility flags are enabled.

There is **no** shipped `TopologyPanel.svelte` 2D canvas fallback.

Supporting panels (not primary):

- Collapsed log (honesty: vendor CLI owns the session)
- Exact prepared-file diff and Apply history inside Mission Review
- Mission history table

## Constraints

- Svelte 5 + Tauri v2 only; **no direct filesystem access** from UI ([[presentation-passive-telemetry]])
- IPC → `pytxo-orchestrate` / `pytxo-store` only
- No embedded multi-pane raw terminal grid (anti-pattern per [[product-vision]])

## Repo

Implementation: `apps/desktop`. Theme tokens as CSS variables; Void and Light only.

## Status

| Feature | Status |
|---------|--------|
| Ops / Missions / Approvals loop | **default** (quiet instrument IA) |
| Mission Plan / Live / Review | shipped with reviewed Apply |
| Signal arbitrage as an evidence line | shipped when `agent_arbitrage` samples exist |
| Void + teal (no spectrum default) | shipped |
| 2D `TopologyPanel.svelte` | **removed / never present in current tree** |
| 3D AST topology | legacy shell only (`TopologyScene3D.svelte`) |
| Closed-loop retry telemetry | shipped — runner emits a `signal-retry` WAL event |

Capture marketing stills under `docs/_attachments/` when needed — not inline base64.
