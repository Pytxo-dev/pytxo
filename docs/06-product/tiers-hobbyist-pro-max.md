---
title: Subscription tiers
slug: tiers-hobbyist-pro-max
status: active
tags: [product, pricing]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-21
related: [[token-arbitrage]], [[gtm-open-source-loop]], [[pytxo-link-service]]
---

# Subscription tiers

Pytxo monetizes **infrastructure efficiency and cloud offload**; the local core stays open-source for adoption.

| Tier | Price | Highlights |
|------|-------|------------|
| **Pytxo Core** | $0 | OSS Rust CLI + desktop skin, tree-sitter scaffolding, **3-agent swarm limit**, BYOK |
| **Pytxo Pro Cloud** | $25/mo | Unlimited local agents, **Pytxo Link**, [[token-arbitrage|cloud context caching]] |
| **Pytxo Max Swarm** | $75/mo | Hosted [[hybrid-execution|cloud sandboxes]], heavy parallel pipelines (10+ agents), 30k credits |
| **Pytxo Ultra** | $99/mo | Managed metered billing, Pytxo Link proxy (`link_reconcile` on by default), frontier models; [[token-arbitrage|Signal Core arbitrage]] on billed vs sent tokens (ADR-0009) |

## Enterprise (GA)

**Pytxo Team Space** — org-wide permission profile ceilings via Link `org_policies`, shared trusted domains, and [[regex-sanitization]] defaults. Set `PYTXO_ULTRA_SESSION` + org id; orchestration caps local `permission_profile` to the org ceiling.

| Capability | Status |
|------------|--------|
| Link entitlements + reconcile | Shipped |
| Org policy API (`/v1/orgs/{id}/policy`) | Shipped (Postgres) |
| Deck / TUI org policy indicator | Shipped (Phase 49) |
| Enterprise sandboxes | GA via `pytxo-cloud-sandbox` worker pool |

Pricing and limits are product decisions; verify against live billing before customer-facing copy.
