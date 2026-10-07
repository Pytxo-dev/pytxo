---
title: Website ASCII motion and clarity proposal
slug: website-ascii-motion-and-clarity-plan-2026-09-24
status: draft
tags: [project, website, design, marketing]
audience: [human, agent]
layer: presentation
created: 2026-09-24
updated: 2026-09-24
related: [[chroma-aperture-identity]], [[desktop-visual-system]], [[market-ready-polish-research]], [[product-vision]]
---

# Website ASCII motion and clarity proposal

## Judgment and evidence boundary

The local website has strong typography, honest version labels and a real-product-centered walkthrough, but its later sections repeat the same repository Review and Apply explanation. The September 21 R8 native Desktop captures show a credible operator interface and a distinctive character glyph. A one-worker canvas leaves much unused space; long requests and dense Review content remain hard to scan. These are visual observations, not a fresh usability study or proof of current release readiness. This proposal changes the website only.

## Direction to review

Use the **product-first** study, `../design/mockups/website-ascii-product-first-concept-2026-09-24.png`, as the composition reference. `../design/mockups/website-ascii-hero-first-concept-2026-09-24.png` is the more expressive alternate; its large glyph competes with product proof. Keep the actual logo, present copy where accurate, and verified Desktop captures. Both ImageGen boards contain invented words and placeholder application chrome; none of that is implementation content or execution evidence.

The glyph should be coded from the existing `ApertureGlyph.svelte` character geometry and Chroma tokens. The two transparent concept PNGs in `docs/design/mockups/` have real alpha but are noisy and oversaturated; keep them as exploration, not production artwork. The final still frame should come from the renderer itself. Details and generation prompts are recorded in `../design/mockups/website-ascii-generation-2026-09-24.md`.

## Content reduction

| Current surface | Proposed treatment |
|---|---|
| Hero and three-step `ProductSection` | Keep the concrete task in the hero. Let the three walkthrough tabs explain execution, review and outcome; remove the separate three-step block after checking that its unique details remain available. |
| `BoundarySection` | Keep one short, prominent “Prepared is not applied” note and the host/network limitation link. Remove the second full explanation of candidate and Apply. |
| `CompatibilitySection` | Show a compact supported path: Windows Desktop, Codex CLI, one repository. Keep detailed requirements on Download and in setup docs. |
| Seven homepage FAQs | Retain only the objections that change a first-use decision: account, what checks prove, and interrupted Apply. Keep the other answers reachable in docs. |
| `GetItSection` and header auth | Keep one closing Download/first-mission action. Remove repeated support navigation. Put configured-deployment account access in Plans/Account rather than competing with Download in the public header; preserve the auth routes. |
| Download intro | Consolidate repeated v1.2.1/v1.2.2 sentences into one explicit public-release-versus-preview notice. |

## Three implementation phases, after direction approval

1. **Content and layout.** Update `apps/web/src/app/(marketing)/page.tsx`, `components/site/hero.tsx`, `product-walkthrough.tsx`, the five section components above, `header.tsx`, and Download copy. Preserve source-backed product screenshots, provenance labels, links, scope caveats and the current published version. Capture 1440px and 390px prototypes before adding motion.
2. **ASCII motion.** Add a website `AsciiAperture` component and pure geometry/renderer module using the Desktop glyph and Chroma token values as references. One hero character volume gets restrained ambient movement only while visible; a short settling transition may accompany the walkthrough. The spectrum line remains static. Motion is decorative and never reports run, verification or Apply state. Provide a static SVG frame, reduced-motion behavior, hidden-tab/offscreen pause and an accessible pause control if the hero loops beyond five seconds. No raster frames or new heavy 3D dependency.
3. **Verification.** Rework affected Playwright expectations instead of deleting coverage. Run `pnpm exec next build` (avoid the mutating `prebuild` asset sync during review), `pnpm lint`, `pnpm check:links`, `node scripts/verify-product-assets.mjs`, focused site E2E and the site audit. Inspect 390/768/1280/1440/1920 layouts, keyboard and reduced-motion paths, contrast, no horizontal overflow, image readability, no layout shift, and animation frame cost/long tasks. Verify retained claims against the actual capture and published download. No deployment or release is in this phase.

**Acceptance:** a visitor can understand the task, supported beta path, Review/Apply boundary, preview provenance and next action in the first screenful; the real product is more prominent than decoration; removing copy loses no material limitation; the site remains usable with motion disabled. Desktop changes require a separate scoped pass.

This is a design proposal under Pytxo's major-redesign gate. Local implementation and public deployment are separate decisions.
