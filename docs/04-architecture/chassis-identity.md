---
title: Chassis visual identity
slug: chassis-identity
status: superseded
tags: [presentation, ui, design]
audience: [human, agent]
layer: presentation
created: 2026-08-25
updated: 2026-08-25
related: [[chroma-ribbon-identity]], [[desktop-visual-system]], [[ADR-0029-chroma-shared-design-tokens]], [[product-vision]]
---

# Chassis visual identity

**Superseded by [[chroma-ribbon-identity]].** Keep this note for history. Do not use Bezel / Aluminum / no-rainbow rules for new UI.

One family for marketing, docs, and Desktop. The metaphor is a **server faceplate**, not a nebula startup and not an ADE cockpit.

This note is historical. Palette, type, and signature now live in [[chroma-ribbon-identity]]. It does not change [[ADR-0029-chroma-shared-design-tokens]] (shared token package); Chroma ribbon is the current contents of `packages/chroma`.

## Palette

| Token | Hex | Use |
|-------|-----|-----|
| Bezel | `#14171C` | Desktop and marketing night |
| Plate | `#1E232B` | Panels, code wells |
| Aluminum | `#E6E8EC` | Docs paper; manuals are read in the light |
| Hairline | `#3D4450` / `#C5CAD3` | Borders |
| Live | `#3F8F7A` | Running only; desaturated |
| Cue | `#E06A3A` | Needs-you and Apply |

Two functional colors. Everything else is metal. No violet, gold, magenta, or cyan rainbow.

## Type and chrome

IBM Plex Sans + IBM Plex Mono, self-hosted. Radius: 2px controls, 4px panels. Motion 120–180ms, state only.

**Signature:** a 6px status lamp (idle / live / needs-you) with no glow. The other signature object is the **Apply receipt** (digest, root, profile, path counts, waiting/applied), reused on marketing, docs, and Review.

## Layout

- Hero: left spec, right orthographic Desktop capture in a bezel.
- Marketing loop: `plan | live | review | apply`. Internal moat names stay in docs.
- Docs sit on Aluminum of the same metal. Desktop is the same chassis in the dark.

## Reject

Cream+serif overcorrection, ethereal glass, numbered 01/02/03 eyebrows, clay/terracotta/coral agent-tool cluster, Cursor orange, Primer green, Linear indigo, tiled terminals, 3D Deck as primary.
