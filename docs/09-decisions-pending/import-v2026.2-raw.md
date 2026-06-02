---
title: Import — Pytxo Architecture v2026.2 (raw)
slug: import-v2026.2-raw
status: archived
tags: [import, archive]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[MOC-home]]
---

# Archived import

This note records that the original **Pytxo Architecture & Strategy Specification (v2026.2)** was imported from the author’s Downloads folder on 2026-06-02.

Content was **atomized** into linked notes under `docs/02-areas/`, `docs/04-architecture/`, `docs/06-product/`, and related paths. Fixed architectural choices were promoted to ADRs in `docs/05-adr/`.

The source document contained embedded base64 PNG images for formulas and metrics. Those were **not** copied into git; use text descriptions in atomic notes or add replacements under `docs/_attachments/` when diagrams are re-exported.

## Source sections (decomposition index)

| § | Topic | Atomic notes |
|---|--------|----------------|
| 1 | Executive / philosophy | [[beyond-the-ade]], [[agent-os-vs-virtual-workspace]] |
| 2 | Architecture | [[three-tier-model]], [[presentation-passive-telemetry]], [[mcp-hub-integration]] |
| 3 | Bottlenecks | [[adaptive-semantic-scaffolding]], [[sparse-overlay-fs]], [[dag-flow-engine]], [[sqlite-wal-logging]], [[closed-loop-fidelity]] |
| 4 | Cloud | [[hybrid-execution]], [[sandbox-dispatch]], [[delta-sync]] |
| 5 | Security | [[regex-sanitization]], [[pytxo-link-signing]] |
| 6 | Business | [[tiers-hobbyist-pro-max]], [[token-arbitrage]] |
| 7 | GTM | [[gtm-open-source-loop]], [[competitive-benchmarks]] |

Do not treat this file as living documentation; edit the atomic notes and ADRs instead.
