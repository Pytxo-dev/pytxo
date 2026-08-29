---
title: ADR-0037 Chroma Aperture visual contract
slug: adr-0037-chroma-aperture-visual-contract
status: superseded
tags: [adr, presentation, website, desktop]
audience: [human, agent]
layer: presentation
created: 2026-08-27
updated: 2026-08-29
adr_id: ADR-0037
superseded_by: ADR-0039
related: [[ADR-0039-evidence-ledger-visual-contract]], [[ADR-0038-epistemic-state-contract]], [[chroma-aperture-identity]], [[desktop-visual-system]], [[ADR-0029-chroma-shared-design-tokens]], [[ADR-0035-desktop-2-quiet-instrument-ia]], [[product-vision]]
---

# ADR-0037: Chroma Aperture visual contract

## Status

Superseded in part by [[ADR-0039-evidence-ledger-visual-contract]].

Item 2 (spectrum encodes execution state, flow, navigation, progress, and commit
boundaries), item 5 (GSAP scroll pinning and scrubbed text reveal), and item 6's
Operations composition are withdrawn. Spectrum-as-state was not verifiable
against any computed quantity and degraded into index-assigned hue and a
constant progress fill in implementation; see
[[ADR-0038-epistemic-state-contract]] for the semantics that replace it.

Items 1, 3, 4, 7, and 8 remain accepted.

## Context

[[ADR-0035-desktop-2-quiet-instrument-ia]] correctly reduced Pytxo Desktop to six durable product areas and removed ornamental topology from the primary experience. Its presentation contract also made the product extremely quiet: flat monochrome panels, low type contrast, sparse Operations composition, and limited motion. The subsequent [[chroma-ribbon-identity]] reintroduced a spectrum hairline, but kept the same spatial system. The website and Desktop consequently share tokens without expressing Pytxo's operational advantage or decision boundary clearly enough.

This decision is presentation-only. It does not change mission semantics, permission profiles, execution domains, reviewed Apply, or the accepted six-part information architecture.

## Decision

1. Adopt **Chroma Aperture** as the shared website and Desktop visual contract.
2. Keep a near-black monochrome foundation. Use the chroma spectrum only for execution state, dependency flow, active navigation, progress, and commit boundaries.
3. Use Satoshi for display and interface text and IBM Plex Mono for commands, identifiers, timestamps, and telemetry.
4. Use 4px control corners and 6px panel corners. Prefer tonal nesting over borders around every region.
5. Let marketing use editorial asymmetry, real product imagery, large spatial chapters, GSAP scroll pinning, and scrubbed text reveal.
6. Keep Desktop motion functional and CSS-based. Operations becomes a denser three-region instrument with navigation, live mission lanes, and a commit-boundary inspector.
7. Preserve reduced-motion behavior, keyboard navigation, semantic status colors, and both dark and light Desktop themes.
8. Never use rainbow-filled buttons, gradient-filled headings, purple glow fields, generic AI imagery, decorative glassmorphism, or 3D topology as the default view.

## Supersession

This ADR supersedes only item 4, the visual contract, and the typography implication in item 5 of [[ADR-0035-desktop-2-quiet-instrument-ia]]. Its navigation, mission object, aliases, and token-isolation decisions remain accepted.

It supersedes [[chroma-ribbon-identity]] as the current identity note. [[ADR-0029-chroma-shared-design-tokens]] remains accepted; its shared package is updated rather than replaced.

## Consequences

Pytxo gains a recognisable identity that is still restrained enough for an operational product. The website and Desktop require coordinated capture verification, and Desktop now carries more visible information at once. Chroma semantics must stay disciplined or the system will regress into decorative rainbow UI.
