---
title: Market-ready polish research
slug: market-ready-polish-research
status: active
tags: [project, research, desktop, marketing, ux]
audience: [human, agent]
layer: meta
created: 2026-07-23
updated: 2026-07-30
related: [[pytxo-improvement-research]], [[desktop-ui-improvement-backlog]], [[desktop-visual-system]], [[product-vision]], [[competitive-benchmarks]]
---

# Market-ready polish research

Primary-source UX table stakes for agent control planes (2026) plus Desktop 2 / marketing audits. Companion to [[pytxo-improvement-research]] Phase 74.

## Product test

A user opening Pytxo Desktop must answer in ~10 seconds: **what’s running, what needs me, what’s sandboxed, what’s it costing?**

## Market table stakes (primary sources)

| Stake | Peer bar | Sources |
|-------|----------|---------|
| Live session roster | Copilot Sessions / Cursor agents list | https://docs.github.com/en/copilot/how-tos/github-copilot-app/agent-sessions · https://cursor.com/docs/cloud-agent |
| Approvals + containment | Codex sandbox ≠ approval; Copilot Plan mode | https://developers.openai.com/codex/concepts/sandboxing · https://developers.openai.com/codex/agent-approvals-security |
| Cost visibility | Cursor spend limit; Claude teams token warning | https://cursor.com/docs/cloud-agent · https://code.claude.com/docs/en/agent-teams |
| Honest empty / onboarding | Copilot getting started | https://docs.github.com/en/copilot/how-tos/github-copilot-app/getting-started |

**Do not** copy landing habits into Desktop: cinematic heroes, moat feature cards, fake div screenshots, glassmorphism, invented “10×” stats.

## Design-taste split

| Surface | Rules |
|---------|--------|
| Marketing (`apps/web`) | design-taste-frontend: brand-first hero, product imagery, proof band, restrained motion |
| Desktop 2 | Dense supervision UI: Chroma void + teal/violet/gold; low motion; high scanability; no landing composition |

## Desktop 2 gap map (pre-Phase 74)

| Gap | Impact |
|-----|--------|
| Mount-once snapshot (no live poll) | Ops “Live” is fake during runs |
| Approvals: no row selection; local resolved without refresh | Blast path feels broken |
| Workspace/recents don’t select domain | Catalog disconnected from Ops/Flow |
| Settings “Coming soon” / MCP stub | Trust hit |
| `snapshot.fleets` unused | Docs vs product mismatch |
| Focus structural list unstyled | Vision oversells “graph” |
| Flow `domain_id` from `repo_root` | Wrong domain wiring |

## Marketing product-truth re-audit

The original Phase 74 note marked the product-led hero shipped before the
implementation matched that claim. A 2026-07-29 re-audit found a logo-only
hero and 27 route aliases that all repeated the Operations render at each
resolution.

The corrective pass now uses current Desktop evidence:

- the hero leads with the mission loop and a real Operations capture;
- Flow, Operations, and Approvals form the Plan / Run / Approve story;
- `capture:marketing` renders all nine Desktop routes at 1600×1000,
  1280×800, and 960×640 from the preview backend;
- `verify:product-assets` rejects missing, incorrectly sized, duplicated, or
  detached marketing captures before the web build;
- the Signal proof reads **82.9% weighted scaffold-byte reduction across 185
  tracked production files**, while model-token and task impact remain
  explicitly unmeasured.

Web Playwright coverage fixes the desktop viewport at 1440×900 and mobile at
390×844. It checks the hero hierarchy, current product image, overflow,
reduced-motion static story, GSAP pinning, and agent accordion behavior.
The release gate also pins Next.js 16.2.12 and forces Next's nested PostCSS
and Sharp packages to patched 8.5.24 and 0.35.3 releases. The high-severity
production audit passes; the remaining moderate Clerk wallet-adapter chain
requires a breaking auth change and stays outside this marketing pass.

## Ranked program (this cycle)

1. Live snapshot poll + Approvals selection/refresh + Ops→Run Review — **shipped**
2. Active domain / recents + Flow domain_id + Integrations/Settings honesty + thin Fleet + Focus CSS — **shipped**
3. Marketing mission hero + truthful product evidence + proof + restrained
   scroll story + product-led OG image — **verified 2026-07-29**
4. Backlog / Phase 74 docs sync — **shipped**
5. Plans claim-integrity closeout — capability-gated copy, always-visible
   maturity details, exact 2×2 plan grid, Focus/Ops wording instead of
   topology, and Playwright + browser QA at 1440×900 / 390×844 —
   **verified 2026-07-29**
6. Release demo + real-repo proof — 63-second Remotion composition using real
   Desktop captures, ElevenLabs-ready scene script, production-source Signal
   corpus, and fixed Race/Blast control-plane workload — **verified
   2026-07-30** ([[v0.13-demo-benchmark-readiness]])

Back: [[MOC-home]] · [[pytxo-improvement-research]]
