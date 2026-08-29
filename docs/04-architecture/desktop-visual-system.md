---
title: Pytxo Desktop visual system
slug: desktop-visual-system
status: active
tags: [presentation, ui, design]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-08-29
related: [[ADR-0038-epistemic-state-contract]], [[ADR-0039-evidence-ledger-visual-contract]], [[chroma-aperture-identity]], [[presentation-passive-telemetry]], [[product-vision]], [[signal-core]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0035-desktop-2-quiet-instrument-ia]], [[ADR-0037-chroma-aperture-visual-contract]]
---

# Pytxo Desktop visual system

**Pytxo Desktop** is the optional control UI: a precise operator instrument for planning a run, watching it execute, reading the exact package it prepared, and deciding whether to Apply. (Formerly called Reality Deck.)

## Aesthetic

Canonical tokens live in `packages/chroma`. State semantics are
[[ADR-0038-epistemic-state-contract]]; the visual system that carries them is
[[ADR-0039-evidence-ledger-visual-contract]]. Desktop night is Void chrome.
Hue is reserved for verified, refuted, and attention. Unknown carries no hue.

| Token | Value | Use |
|-------|-------|-----|
| **Void** | `#050507` | App background |
| **Panel** | `#111113` | Panels |
| **Ink** | `#EDEDEF` | Type and primary buttons |
| **Verified** | `--state-verified` | Mechanically enforced or confirmed |
| **Refuted** | `--state-refuted` | Evidence of failure or circumvention |
| **Attention** | `--state-attention` | Work that needs a human decision |

Typography: Satoshi + IBM Plex Mono. Operator-read text floors at **11px**;
column heads may be 10px. Desktop type scale: 10, 11, 12, 14, 18, 24. Ledger
rows are **34px**. Radius: **4px** controls, **6px** panels. Motion only for
state transitions of 120–160ms, with zero infinite loops. Motion pauses when
reduced motion is requested.

Spectrum is retired from state duty. It is retained in the brand mark only. Do
**not** ship nebula meshes, glow-on-every-heading, Terminal skins, sparkle CTAs,
fake waveforms, orbit rings, decorative CSS graphs, or a fabricated progress
fill. Shared tokens are `packages/chroma` ([[ADR-0029-chroma-shared-design-tokens]]).

## Primary surface (shipping)

**Desktop 2** (default): three destinations — **Work**, **History**, **Setup** —
plus a title-bar workspace switcher and an approvals overlay. Work is the
focused run: a wave-grouped ledger and a persistent commit-boundary panel with
the enforcement receipt. History is the run inventory. Setup holds Appearance,
Providers, Workspaces, Agents, Voice, Privacy, and Account. Per
[[ADR-0038-epistemic-state-contract]] and
[[ADR-0039-evidence-ledger-visual-contract]]. The six-item nav in
[[ADR-0035-desktop-2-quiet-instrument-ia]] is retired.

**Legacy Deck:** optional 3D AST topology (`TopologyScene3D.svelte`) from
[[ADR-0023-reality-deck-3d-renderer]]. It is lazy-loaded only in development
when both Deck compatibility flags are enabled.

There is **no** shipped `TopologyPanel.svelte` 2D canvas fallback.

Supporting panels (not primary):

- Collapsed log (honesty: vendor CLI owns the session)
- Exact prepared-file diff and Apply history inside Run Review
- History table

## Constraints

- Svelte 5 + Tauri v2 only; **no direct filesystem access** from UI ([[presentation-passive-telemetry]])
- IPC → `pytxo-orchestrate` / `pytxo-store` only
- No embedded multi-pane raw terminal grid (anti-pattern per [[product-vision]])
- No element renders a quantity the orchestrator does not compute

## Repo

Implementation: `apps/desktop`. Theme tokens as CSS variables; Void and Light only.

## Status

| Feature | Status |
|---------|--------|
| Work / History / Setup | **default** |
| Run Review from History | shipped with reviewed Apply |
| Enforcement receipt on Work and History | shipped |
| Epistemic state chips (verified / claimed / unknown / refuted) | shipped |
| Void + one static spectrum on the website hero only | shipped |
| 2D `TopologyPanel.svelte` | **removed / never present in current tree** |
| 3D AST topology | legacy shell only (`TopologyScene3D.svelte`) |
| Closed-loop retry telemetry | shipped — runner emits a `signal-retry` WAL event |

Capture marketing stills under `docs/_attachments/` when needed — not inline base64. They are mirrored from the deterministic Desktop capture set.
