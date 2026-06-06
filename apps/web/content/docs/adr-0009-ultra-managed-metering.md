---
title: ADR-0009 Ultra managed metering
slug: adr-0009-ultra-managed-metering
status: accepted
tags: [adr, billing, ultra]
audience: [human, agent]
layer: architecture
created: 2026-06-02
updated: 2026-06-02
related: [token-arbitrage](/docs/token-arbitrage), [tiers-hobbyist-pro-max](/docs/tiers-hobbyist-pro-max), [signal-core](/docs/signal-core)
---

# ADR-0009: Pytxo Ultra managed metering

## Status

Accepted

## Context

Pytxo Core/Pro use **BYOK**. **Pytxo Ultra** ($99/mo base) adds hosted sandboxes, Pytxo Link proxy, and **managed metered billing** where users are billed at public list rates on raw context while Signal Core reduces tokens actually sent to providers — yielding measurable arbitrage margin.

## Decision

1. **Traits in `pytxo-core`:** `TokenWallet`, `UsageMeter`, `TokenEstimator`, `ModelRouter`, `ManagedTransport`.
2. **Persistence in `pytxo-store`:** migration 003 tables (`wallet_accounts`, `wallet_reservations`, `arbitrage_samples`, `usage_records`) via `SharedStore`.
3. **Runner hooks:** `ArbitrageProfiler` during `prepare_agent_context`; `ManagedTransport::apply` on child CLI env when `billing.mode = ultra`.
4. **Orchestrate:** hybrid wallet — local reserve/debit + `NoopBillingReconciler` stub until Pytxo Link HTTP exists.

## Billing model

| Ledger | Meaning |
|--------|---------|
| `tokens_in_billed` | Estimated from raw workspace files (market rate basis) |
| `tokens_in_sent` | Estimated from scaffolded payload (actual egress) |
| `saved_tokens` | `billed - sent` (arbitrage yield) |

## Consequences

- Ultra runs require `wallet_accounts` balance or `initial_balance_microcredits` in config for local dev.
- BYOK path unchanged when `billing.mode = byok` (default).
- Tiktoken estimator deferred behind future `tiktoken` feature on `pytxo-core`.

## Supersedes

None.
