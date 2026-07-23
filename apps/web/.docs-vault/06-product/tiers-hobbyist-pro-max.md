---
title: Subscription tiers
slug: tiers-hobbyist-pro-max
status: active
tags: [product, pricing]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-23
related: [[token-arbitrage]], [[gtm-open-source-loop]], [[pytxo-link-service]], [[pytxo-improvement-research]], [[hybrid-execution]]
---

# Subscription tiers

Pytxo monetizes **infrastructure efficiency and cloud offload**; the local core stays open-source for adoption. Commercial maturity and gates: [[pytxo-improvement-research]].

| Tier | Price | Highlights |
|------|-------|------------|
| **Pytxo Core** | $0 | OSS Rust CLI + desktop skin, tree-sitter scaffolding, **3-agent swarm limit**, BYOK |
| **Pytxo Pro Cloud** | $25/mo | Unlimited local agents, **Pytxo Link** (when Link is live), [[token-arbitrage|cloud context caching]] roadmap |
| **Pytxo Max Swarm** | $75/mo | Hosted [[hybrid-execution|cloud sandboxes]] **when dispatcher configured**, heavy parallel pipelines (10+ agents), 30k credits |
| **Pytxo Ultra** | $99/mo | Managed metered billing target; **local ledger ships today**; Link `link_reconcile` is **not** E2E on the default path (`NoopBillingReconciler`) — see ADR-0009 |

## Enterprise (capability-gated)

**Pytxo Team Space** — org-wide permission profile ceilings via Link `org_policies`, shared trusted domains, and [[regex-sanitization]] defaults — when Link entitlements are configured.

| Capability | Status |
|------------|--------|
| Local Ultra wallet / ledger | Shipped |
| Link entitlements + reconcile | **Local-ledger until Link E2E**; HTTP path still stub/ping-only on default |
| Org policy API (`/v1/orgs/{id}/policy`) | Implemented when Link backend is deployed |
| Deck / TUI org policy indicator | Phase 49 UI when entitlements present |
| Enterprise sandboxes | **Not GA by default** — advertise only with configured cloud dispatcher ([[hybrid-execution]]) |

Pricing and limits are product decisions; verify against live billing before customer-facing copy. Do not imply live meter or cloud sandboxes without the corresponding non-noop path.
