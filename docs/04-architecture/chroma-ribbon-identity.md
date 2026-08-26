---
title: Chroma ribbon identity
slug: chroma-ribbon-identity
status: active
tags: [presentation, ui, design]
audience: [human, agent]
layer: presentation
created: 2026-08-25
updated: 2026-08-25
related: [[chassis-identity]], [[desktop-visual-system]], [[ADR-0029-chroma-shared-design-tokens]], [[product-vision]]
---

# Chroma ribbon identity

One family for marketing, docs, and Desktop. The metaphor is the **folded rainbow lambda** on Vercel-quiet black chrome — not a nebula mesh and not a metal faceplate.

This note supersedes [[chassis-identity]] for palette, type, and signature. It does not change [[ADR-0029-chroma-shared-design-tokens]] (shared token package); Chroma ribbon is the current contents of `packages/chroma`.

## Palette

| Token | Hex | Use |
|-------|-----|-----|
| Void | `#050507` | Marketing, docs, and Desktop night |
| Panel | `#111113` | Cards, sidebar, code wells |
| Ink | `#EDEDEF` | Body, headings, primary buttons |
| Line | `#2A2A2E` | Hairlines |
| Magenta | `#C44BFF` | Spectrum start (logo left stroke) |
| Cyan | `#3EE0D0` | Spectrum end (logo tip) |

Ribbon stops: Magenta → Pink `#FF4B9A` → Orange `#FF6A3D` → Gold `#F5C542` → Green `#7BE07A` → Cyan. **Live** is the green stop (running). **Cue** is the orange stop (needs-you / Apply). Primary buttons are Ink on Void. Never fill a button with the rainbow.

## Type and chrome

Sora (display and body) + IBM Plex Mono (commands), self-hosted. Radius: 6px controls, 8px panels. UI motion 150ms. Ribbon travel 14s, paused when `prefers-reduced-motion`.

**Signature:** a 1px chroma ribbon under the header / titlebar, on the active nav indicator, and at most one hero word. Status lamps stay a single color. No glow, no nebula background.

## Layout

- Hero: left spec, right real Ops capture with a spectrum edge.
- Marketing loop: `plan | live | review | apply`. Internal moat names stay in docs.
- Docs sit on Void, same night as marketing. Desktop uses the ribbon on chrome only; Ops density does not change.

## Reject

Chassis metal-only, cream+serif, Geist-on-black clones, glow-on-every-heading, numbered 01/02/03 eyebrows, tiled terminals, 3D Deck as primary.
