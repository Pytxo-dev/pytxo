---
title: Cloud context caching (token arbitrage)
slug: token-arbitrage
status: active
tags: [product, tokens]
audience: [human, agent]
layer: cloud
created: 2026-06-02
updated: 2026-07-29
related: [[tiers-hobbyist-pro-max]], [[adaptive-semantic-scaffolding]], [[ADR-0009-ultra-managed-metering]]
---

# Cloud context caching (token arbitrage)

Pytxo ships **local token-arbitrage profiling** today: it compares raw context
estimates with the Signal Core scaffold sent to an agent. The Pro Cloud
**server-side context cache is a capability-gated target**, not a default
hosted service. It requires a configured cache and cloud path.

## Pytxo Ultra: billed vs sent

Ultra runs use a dual ledger (see [[ADR-0009-ultra-managed-metering]]):

| Metric | Source |
|--------|--------|
| **tokens_in_billed** | Raw workspace files (public list rate basis) |
| **tokens_in_sent** | Signal Core scaffold actually egressed to the model |
| **saved_tokens** | Arbitrage yield persisted in `arbitrage_samples` |

Rust: `ArbitrageProfiler` in `pytxo-runner`, `UsageMeter` / `TokenWallet` in `pytxo-store`.

## Relationship to scaffolding

Local [[adaptive-semantic-scaffolding]] governs fidelity per agent. When a
server-side cache is configured, that cache can optimize **repeated**
structural payloads across swarm members.

## Honest positioning

Parallel agents still incur provider costs (see [[cost-and-swarm-limits]]).
Shipping local scaffolding mitigates duplication; configured caching can reduce
repeated payload work. Neither eliminates spend.

Part of [[tiers-hobbyist-pro-max]].
