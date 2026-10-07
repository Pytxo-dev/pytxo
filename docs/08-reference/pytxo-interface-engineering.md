---
title: Pytxo interface engineering
slug: pytxo-interface-engineering
status: active
tags: [reference, desktop, website, accessibility, motion]
audience: [agent, human]
layer: presentation
created: 2026-09-14
updated: 2026-09-14
related: [[desktop-interaction-audit-2026-09-13]], [[modular-project-safety-contract-2026-09-13]], [[pytxo-presentation-references-2026-09-14]]
---

# Pytxo interface engineering

This is the versioned product contract used alongside the installed
`pytxo-interface` and `make-interfaces-feel-better` skills. Current source and
CHECKPOINT evidence override historical screenshots. It is not release approval.

## Composition and language

Lead with the requested outcome, actual state, and next meaningful action.
Then show responsibilities, changed files, verification, and blockers. Put exact
IDs, digests, receipts, and logs one labeled interaction away. Never disclose a
blocking failure only inside a closed panel. Use Project for the selected
codebase; keep Work, History, and Setup as the destinations. Internal types do
not automatically deserve controls or product nouns.

Preserve Chroma Aperture: Void/Aluminum surfaces, fine separators, compact
4–10px radii, Sora-compatible UI type and IBM Plex Mono for machine evidence.
Align controls and aim for consistent 40px desktop targets. Support long paths,
text enlargement, keyboard focus, and smaller windows before adding decoration.
Use empty space to establish hierarchy, not to stretch every control. Avoid
glowing cards, gratuitous gradients, giant rounded panels, and ornamental motion.
Website copy should explain the same product without pretending receipts are
security guarantees or folder attachment is coordinated modular Apply.

## State is authoritative

Outcome → isolated work → combined verification → human review → explicit
Apply → confirmed result. Inspection may happen earlier. Agent completion does
not verify the combined candidate. Unknown checks are not passed checks.
Empty candidates remain **Nothing to Apply**; stale, failed, recovery-required,
and applying states must retain their existing synchronous guards. Animation
must never enable an action, delay disabling it, or complete a transaction.

New work preserves drafts and explanatory disabled actions. Active work reports
real events, not invented percentages or ETAs. Keep task positions stable while
people read. Loading, reconnecting, cancellation, and failure need explicit
language, not merely a color or spinner. Review leads with changes and eligibility;
exact provenance remains reachable without competing with the decision.

## Motion and docking

Choose no motion for routine refreshes, CSS for simple feedback, Motion's free
DOM runtime for a lifecycle-managed interaction, and Anime.js only for an
exceptional timeline/SVG need. Kokonut is a pattern reference, not authority to
import React into Desktop. Bklit needs a real data-visualization requirement.

The bounded evidence action is `apps/desktop/src/lib/evidence-motion.ts`:
200ms opacity/8px translation on its own wrapper, current-frame retargeting,
generation-guarded cleanup, OS plus in-app reduced motion, and immediate hiding.
Do not animate the native child-preview wrapper: its bounds are synchronized
outside CSS. Pause panel motion during dragging/resizing. No exit animation may
retain another run's evidence. Keep focus and disclosure state independent.

Dock placement must come from a real committed layout change. Exercise sparse
pointer events as well as smooth gestures; targets must exist before hit-testing
and release coordinates must be evaluated. Demonstrate a move both directions,
keyboard alternative, Escape, resize, persistence, and scoped/pinned evidence.
A browser reproduction is not a native root-cause determination.

## Acceptance

Use focused checks while editing, then the final relevant combined matrix.
Inspect rendered screenshots; a green test can miss a malformed control.
Bind native evidence to an exact executable hash and dirty-source manifest.
Test the current viewport and a smaller supported size; browser zoom is not a
second Windows DPI result. Respect native handoff/interruption boundaries.
Do not call UI work accepted until the final native candidate is exercised.
Keep source captures, edited footage, previews, installed builds, and public
release gates separate. Demo fixtures and browser mocks never replace genuine
verification → Review → Apply evidence.
