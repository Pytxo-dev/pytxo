---
title: Cloud context caching (token arbitrage)
slug: token-arbitrage
status: active
tags: [product, tokens]
audience: [human, agent]
layer: cloud
created: 2026-06-02
updated: 2026-06-02
related: [[tiers-hobbyist-pro-max]], [[adaptive-semantic-scaffolding]], [[ADR-0009-ultra-managed-metering]]
---

# Cloud context caching (token arbitrage)

**Pro Cloud** tier includes **cloud context caching**: semantic code signatures stored on fast cloud infrastructure to reduce repeated LLM input tokens during large multi-agent refactors.

## Pytxo Ultra: billed vs sent

Ultra runs use a dual ledger (see [[ADR-0009-ultra-managed-metering]]):

| Metric | Source |
|--------|--------|
| **tokens_in_billed** | Raw workspace files (public list rate basis) |
| **tokens_in_sent** | Signal Core scaffold actually egressed to the model |
| **saved_tokens** | Arbitrage yield persisted in `arbitrage_samples` |

Rust: `ArbitrageProfiler` in `pytxo-runner`, `UsageMeter` / `TokenWallet` in `pytxo-store`.

## Relationship to scaffolding

Local [[adaptive-semantic-scaffolding]] still governs fidelity per agent. Cache layer optimizes **repeated** structural payloads across swarm members.

## Honest positioning

Parallel agents still incur provider costs (see [[cost-and-swarm-limits]]). Caching and scaffolding mitigate duplication; they do not eliminate spend.

Part of [[tiers-hobbyist-pro-max]].
