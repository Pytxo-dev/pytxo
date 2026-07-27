---
title: ADR-0032 Desktop 2 Focus and Flow as primary surfaces
slug: adr-0032-desktop-2-focus-flow-primary
status: accepted
tags: [adr, desktop, presentation]
audience: [human, agent]
layer: presentation
created: 2026-07-27
updated: 2026-07-27
adr_id: ADR-0032
related: [[desktop-visual-system]], [[presentation-passive-telemetry]], [[mission-loop]], [[ADR-0023-reality-deck-3d-renderer]], [[ADR-0028-desktop-product-name]]
---

# ADR-0032: Desktop 2 Focus and Flow as primary surfaces

## Status

Accepted

## Context

[[ADR-0023-reality-deck-3d-renderer]] decided that Three.js 3D topology is the primary center viewport. Product direction since Desktop 2 (v0.5.0+) made **structural Focus** and **Flow / Approvals / Run Review** the default mission-review surfaces; 3D remains only on the legacy shell (`desktop_shell_v1=true`). ADR-0028 renamed Reality Deck → Pytxo Desktop but left ADR-0023’s technical choice unchanged, creating doc/code drift.

## Decision

1. **Default Desktop surface** is Desktop 2: Flow (mission plan), Ops, Focus (structural list/graph), Approvals, and Run Review.
2. **Three.js `TopologyScene3D`** remains supported only when `desktop_shell_v1=true` (legacy). It is not the product-primary viewport.
3. Desktop’s job in the mission loop is **review and intervention** — proposed plan, evidence, approvals, flush — not decorative agent-count visualization.
4. [[ADR-0023-reality-deck-3d-renderer]] remains historically Accepted for the 3D renderer choice when the legacy shell is enabled; **this ADR supersedes its “primary viewport” claim**.

## Consequences

**Positive**

- Vision, presentation, and Desktop UX docs align with shipping defaults.
- Mission-loop UX can prioritize Flow / Run Review without 3D obligations.

**Negative / tradeoffs**

- Legacy shell maintainers must keep reading ADR-0023 for renderer details.

## Links

- Supersedes (primary-viewport portion of): [[ADR-0023-reality-deck-3d-renderer]]
- Related: [[mission-loop]], [[product-vision]]
