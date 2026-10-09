---
title: Pytxo interface engineering
slug: pytxo-interface-engineering
status: active
tags: [reference, desktop, website, accessibility, motion]
audience: [agent, human]
layer: presentation
created: 2026-09-14
updated: 2026-10-09
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

## Terminal language

Approved by Matt on 2026-10-09 ("add your own UI style"). Pytxo Desktop and
pytxo.com share a terminal vocabulary on top of Chroma Aperture:

- **Panes** (`.tui-pane`, `.tui-legend` in `apps/desktop/src/app.css`): a
  hairline frame with its title cut into the top border. Set `--tui-bg` to the
  surface behind the pane. Fleet workers, the fleet monitor, plan tasks and the
  New work fleet preview use them; the website run record mirrors the look.
- **Spinner** (`.tui-spin`): `| / - \`, four visible frames so a still capture
  never shows a blank. It marks a live worker, never progress.
- **Fleet monitor** (`FleetBoard.svelte`): one row per task with its CLI, an
  activity sparkline and an elapsed clock. The sparkline is output volume per
  time slice with a few seconds' decay on one fleet-wide scale; it is not
  progress and must never be read as percent complete. A live worker silent for
  45 s reads **quiet**, because Pytxo cannot tell quiet from stuck.
- **ASCII scope** (`FleetRadar.svelte`): the aperture as an ASCII sphere, one
  orbit per step and one numbered blip per task. Live blips travel and the sweep
  turns only while something is live. It is decoration over real state; the
  rows beside it carry the text and the accessible names.
- Mono text uses `--pytxo-font-mono`, which falls back to Cascadia Mono or
  Consolas for box-drawing and block glyphs that IBM Plex Mono lacks.
- Glow is limited to live activity marks. Every animation stops under OS or
  in-app reduced motion, and canvases pause when hidden or off screen.

The browser preview can play a whole fleet run (`pytxo-preview-fleet-script-v1`)
for demos and the website clip. It is an illustrative fixture, never evidence.

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
