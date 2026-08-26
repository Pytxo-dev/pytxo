---
title: Pytxo Desktop visual system
slug: desktop-visual-system
status: active
tags: [presentation, ui, design]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-08-25
related: [[chroma-ribbon-identity]], [[chassis-identity]], [[presentation-passive-telemetry]], [[product-vision]], [[signal-core]], [[pytxo-improvement-research]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0035-desktop-2-quiet-instrument-ia]]
---

# Pytxo Desktop visual system

**Pytxo Desktop** is the optional control UI — a quiet operator instrument for composing missions, watching execution, reviewing prepared changes, and deciding whether to Apply. (Formerly called Reality Deck.)

## Aesthetic

Canonical tokens live in [[chroma-ribbon-identity]]. Desktop night is Void chrome with a 1px spectrum ribbon; Ops surfaces stay monochrome.

| Token | Value | Use |
|-------|-------|-----|
| **Void** | `#050507` | App background |
| **Panel** | `#111113` | Panels |
| **Ink** | `#EDEDEF` | Type and primary buttons |
| **Live** | `#7BE07A` | Running lamps only |
| **Cue** | `#FF6A3D` | Needs-you, Apply |

Typography: Sora + IBM Plex Mono; body/rows **13px**, meta **12px**, kbd **11px**; **tabular numbers** for money and counts. Radius: **6px** controls, **8px** panels. Motion only for selection, open/close, and status change (150ms). Ribbon travel 14s, paused when reduced motion. Signature: chroma ribbon on titlebar and active nav; 6px status lamp (no glow) and the Apply receipt.

Do **not** ship nebula meshes, glow-on-every-heading, Terminal skins, sparkle CTAs, fake waveforms, orbit rings, or decorative CSS graphs. Shared tokens are `packages/chroma` ([[ADR-0029-chroma-shared-design-tokens]]).

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
| Void + chroma ribbon on chrome (lamps stay Live/Cue) | shipped |
| 2D `TopologyPanel.svelte` | **removed / never present in current tree** |
| 3D AST topology | legacy shell only (`TopologyScene3D.svelte`) |
| Closed-loop retry telemetry | shipped — runner emits a `signal-retry` WAL event |

Capture marketing stills under `docs/_attachments/` when needed — not inline base64.
