---
title: ADR-0039 Evidence ledger visual contract
slug: adr-0039-evidence-ledger-visual-contract
status: accepted
tags: [adr, presentation, website, desktop]
audience: [human, agent]
layer: presentation
created: 2026-08-29
updated: 2026-08-29
adr_id: ADR-0039
related: [[ADR-0038-epistemic-state-contract]], [[ADR-0037-chroma-aperture-visual-contract]], [[ADR-0035-desktop-2-quiet-instrument-ia]], [[ADR-0029-chroma-shared-design-tokens]], [[desktop-visual-system]]
---

# ADR-0039: Evidence ledger visual contract

## Status

Accepted. Supersedes the state-semantics, density, and marketing-motion items of
[[ADR-0037-chroma-aperture-visual-contract]].

## Context

[[ADR-0037-chroma-aperture-visual-contract]] assigned the chroma spectrum to
"execution state, dependency flow, active navigation, progress, and commit
boundaries". That mapping had no data to bind to, because the orchestrator
computes no per-agent progress and no per-lane energy. The contract therefore
degraded in implementation exactly where it was least verifiable: hue became
array index, progress became a constant, and a perpetual gradient loop ran in
persistent navigation chrome.

The lesson is not that the identity was wrong to be expressive. It is that a
visual language keyed to "liveness" invites fabrication, whereas one keyed to
provability cannot be satisfied without real data.

[[ADR-0038-epistemic-state-contract]] establishes the semantics. This decision
sets the visual system that carries them.

## Decision

1. **Spectrum is retired from state duty.** It is retained in the brand mark and
   in exactly one static website identity moment. It never animates and never
   encodes execution state, navigation, or progress.

2. **Three semantic hues only,** all from `packages/chroma`:
   `--state-verified` (desaturated green-teal), `--state-refuted` (desaturated
   red-orange), `--state-attention` (brand accent). Unknown carries no hue.
   Everything else is monochrome. One hue has exactly one meaning; the previous
   dual use of amber for both "needs your decision" and "no evidence available"
   is prohibited.

3. **Structure carries what colour used to.** Hairline dividers over cards,
   panels only at real boundaries, tabular numerals for machine values. Hatch is
   legal only inside a state chip, a surface row, or a progress track. It is
   never a background or a divider.

4. **Density serves prolonged expert reading.** Desktop uses 34px table rows in
   place of 88px lanes. Operator-read text floors at 11px, replacing the previous
   9px definition lists. Desktop type scale: 10 for column heads only, then 11,
   12, 14, 18, 24.

5. **Typography.** One grotesk for interface and display, one mono for all
   machine values: identifiers, digests, paths, counts, timestamps. Mono is
   semantic, not decorative.

6. **Signature components** are the enforcement receipt table, the run ledger
   with wave grouping, and the state chip. The identity lives in these objects
   rather than in a colour effect.

7. **Motion.** Desktop is limited to state transitions of 120 to 160ms with zero
   infinite loops. The website permits entrance reveals and at most one pinned
   sequence, and every animation must encode a state change or a narrative step.
   Reduced motion collapses all of it to static. GSAP scroll pinning and the
   scrubbed text reveal permitted by ADR-0037 item 5 are withdrawn.

8. **Product captures appear at readable scale or as deliberate crops,**
   annotated with at most four callouts. A capture below legible scale is deleted
   rather than shrunk. Published captures are mirrored from the deterministic
   Desktop capture set, never hand-copied.

9. Items of ADR-0037 that remain accepted: the near-black monochrome
   foundation, Satoshi with IBM Plex Mono, the 4px control and 6px panel corner
   scale, reduced-motion and keyboard behaviour, both Desktop themes, and the
   prohibition on rainbow-filled buttons, gradient headings, glow fields,
   generic AI imagery, glassmorphism, and 3D topology as a default view.

## Consequences

The product looks quieter and reads denser. Recognisability now depends on the
receipt table and the state vocabulary rather than on colour, which is a slower
identity to acquire but one that cannot decay into decoration.

Because hue is scarce, any future request for a new colour meaning has to
displace an existing one or go without, which is the intended pressure.

The website loses two scroll set pieces and a marquee. It gains one capture large
enough to read, which was the point of using real product imagery in the first
place.
