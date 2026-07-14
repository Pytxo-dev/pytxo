---
title: ADR-0028 Pytxo Desktop product name
slug: adr-0028-desktop-product-name
status: accepted
tags: [adr, desktop, naming, product]
audience: [human, agent]
layer: presentation
created: 2026-07-09
updated: 2026-07-09
adr_id: ADR-0028
related: [[ADR-0023-reality-deck-3d-renderer]], [[desktop-visual-system]], [[product-vision]]
---

# ADR-0028: Pytxo Desktop product name

## Status

Accepted — supersedes the **product naming** in [[ADR-0023-reality-deck-3d-renderer]] only. The Three.js 3D topology decision in ADR-0023 remains in force.

## Context

The optional presentation app at `apps/desktop` was branded **Reality Deck**. That name was evocative but opaque for new users and competed with simpler “Desktop” language already used in architecture docs (“desktop shell,” “desktop telemetry”).

Market peers (BridgeSpace, Emdash, Warp Oz) use plain ADE / control-plane language. Pytxo’s wedge is clearer when the UI product is named for what it is: the desktop control surface for the agent hypervisor.

## Decision

1. User-facing product name is **Pytxo Desktop** (not Reality Deck).
2. Canonical visual-system note is [[desktop-visual-system]] (replaces `reality-deck-visual-system` slug for new links).
3. ADR-0023’s technical choice (Three.js topology as primary viewport) is unchanged.
4. Internal identifiers (`com.pytxo.reality-deck`, `pytxo-deck://`, `deck-*` CSS/classes, legacy installer filenames) may remain until a dedicated identity migration; they are not user-facing copy.

## Consequences

- Marketing, public docs, Desktop UI chrome, and vault prose use **Pytxo Desktop**.
- Glossary may note “formerly Reality Deck” once for continuity.
- A future ADR may rename Tauri identifier / deep-link scheme if update channels require it.

## Alternatives rejected

- Keep Reality Deck as the public brand (rejected: too much jargon for first-run clarity).
- Rename Tauri identifier in the same change (rejected: breaks keyring / deep-link / updater continuity).
