---
title: Market-ready polish research
slug: market-ready-polish-research
status: active
tags: [project, research, desktop, marketing, ux]
audience: [human, agent]
layer: meta
created: 2026-07-23
updated: 2026-07-23
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

## Marketing gap map

| Gap | Impact |
|-----|--------|
| Hero = logo panel | Reads brand mark, not product |
| Unused `desktop-topology-hero.png` | Asset ready, unused |
| Thin trust/proof | No logo cloud / measured strip under hero |
| Moat accents use cyan | Triad is teal / gold / violet |
| Load-time section fade only | Weak scroll presence |

## Ranked program (this cycle)

1. Live snapshot poll + Approvals selection/refresh + Ops→Run Review — **shipped**
2. Active domain / recents + Flow domain_id + Integrations/Settings honesty + thin Fleet + Focus CSS — **shipped**
3. Marketing hero product imagery + proof + triad + scroll motion + download/OG — **shipped**
4. Backlog / Phase 74 docs sync — **shipped**

Back: [[MOC-home]] · [[pytxo-improvement-research]]
