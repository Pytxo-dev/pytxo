---
title: Dodo merchant-of-record integration
slug: dodo-mor-integration
status: active
tags: [project, billing, dodo, mbcz]
audience: [human, agent]
layer: cloud
created: 2026-09-03
updated: 2026-09-03
related: [[ADR-0040-mbcz-merchant-of-record-boundary]], [[ADR-0021-billing-source-of-truth]], [[pytxo-link-service]]
---

# Dodo merchant-of-record integration

## Product and account model

Dodo is the Merchant of Record and legal seller. MBCZ is the verified Dodo
business-account owner and payout beneficiary. Pytxo is one customer-facing
brand with its own name, logo, website, support address, statement descriptor,
products, and checkout appearance. This matches Dodo's multi-brand model:
business KYC and payouts remain shared, while products and transactions carry a
brand ID.

For an under-18 operator, official Dodo guidance says the business must use a
parent or guardian's details, documents, and bank account. Keep that legal layer
consistent in Dodo. Do not add a guardian's name, ID, phone, email, bank data,
domain invoice, or KYC artifact to Pytxo source control. Dodo's current FAQ says
domain registration must be transferred to the guardian; reconfirm that and all
account requirements with Dodo compliance during onboarding.

A factual Pytxo brand description for review:

> Pytxo is software for running coding agents behind a reviewed repository
> boundary. It isolates proposed work, records verification evidence, requires
> approval before applying reviewed changes, and keeps a durable run history.
> Paid plans provide higher coordination limits and optional hosted services.

This avoids claiming universal rollback or production-system effects that the
current product does not ship.

## Current repository flow

```text
Pytxo Plans
  -> /api/billing/checkout-redirect
  -> MBCZ /api/checkout/sessions
  -> current merchant checkout

current Paddle webhook
  -> pytxo.com proxy
  -> Link /v1/webhooks/paddle
  -> Paddle signature + price allowlist + identity binding
  -> Postgres entitlement
  -> CLI/Desktop /v1/entitlements/status
```

The checkout side is already MBCZ-shaped. The coupling is inside Link:
`src/paddle.rs`, `AppState` Paddle fields, the Paddle route, three price
variables, and migration 007. The entitlement record and client status contract
are provider-neutral and should remain stable.

## Target flow

```text
Pytxo Web -> MBCZ checkout broker -> Dodo Pytxo brand checkout
                                      |
                                      v
                              Dodo webhook
                                      |
                                      v
                    pytxo.com raw-body/header proxy
                                      |
                                      v
                       Pytxo Link Dodo adapter
                                      |
                  generic event + binding + entitlement transaction
                                      v
                     unchanged CLI/Desktop entitlement status
```

Redirects are navigation only. A success query parameter must never unlock a
plan. Link changes access only after a verified, allowlisted lifecycle event.

## Delivery phases

### Phase 0: merchant setup

Complete MBCZ verification and add Pytxo as a secondary brand. Use
`pytxo.com`, a monitored support address, accurate product copy, and only plans
that exist in the product. Before live review, publish actual numeric prices,
Terms, Privacy, Refund/Cancellation, and a working contact route; the current
Pytxo web app does not yet provide those review surfaces. Record Dodo brand,
product, and price IDs in server-side configuration, not source control. Create
separate test and live credentials.

Exit evidence: Dodo approves the Pytxo brand, test checkout displays Pytxo
branding, receipts name the correct merchant/brand, and no live charge occurs.

### Phase 1: generic Link reconciliation

Add migration 008 with `billing_events` and `billing_subscriptions`. Key every
event by `(provider, event_id)` and every binding by
`(provider, subscription_id)`. Model one grant per provider subscription and
recompute effective access across active grants. Store lifecycle state, provider
occurrence/object time, canonical plan key, Pytxo principal, and opaque external
IDs. Any provider cursor is optional and must never be assumed. Do not store
card, address, tax, or full webhook bodies.

Extract Paddle's transaction logic into a provider-neutral reconciler. Keep the
existing Paddle parser and signature verifier as one adapter. Add tests before
changing production behavior.

Exit evidence: the existing 22 Link tests pass against the generic service,
schema migration upgrades production-shaped fixtures, and replay or out-of-order
events cannot grant the wrong tier.

### Phase 2: Dodo adapter and product-domain ingress

Add a Pytxo web route that proxies the unmodified request body plus
`webhook-id`, `webhook-signature`, and `webhook-timestamp` to
`POST /v1/webhooks/dodo` in Link. Link uses the official SDK or a maintained
Standard Webhooks library, timestamp tolerance, constant-time verification,
business/brand/product/price allowlists, maximum body size, and event uniqueness.
Rotate Dodo webhook credentials independently from Link admin access.

Link maps allowlisted external IDs to a canonical Pytxo plan. The proxy does not
parse, provision, or acknowledge success until Link has durably accepted the
event. Shared proxy rate limits must account for webhook bursts from one egress
address.

Exit evidence: unsigned, stale, replayed, malformed, wrong-brand, wrong-product,
unknown-plan, and reordered events fail closed. Duplicate delivery returns 2xx
only after confirming the original effect is durable.

### Phase 3: test-mode end to end

Switch the MBCZ checkout broker's Pytxo test catalog to Dodo. Exercise new
subscription, renewal, plan change, scheduled cancellation, end-of-period
cancellation, on-hold, recovery, expiry, refund, and dispute outcomes. Verify
Clerk identity binding and observe Link,
CLI, Desktop, email/receipt, and customer portal behavior.

Run retries and intentionally reorder captured test events. Verify that a paid
redirect arriving before its webhook shows a pending state rather than paid
access. Measure webhook latency and alert on unprocessed or dead-letter events.

### Phase 4: shadow and cutover

For existing subscribers, retain Paddle as the authority until their migration
path is explicit. Dodo's documented migration is support-assisted; card tokens
may be transferable, wallet tokens are not, and payment history is not migrated.
Confirm the exact Pytxo account path with Dodo and Paddle, export or retain the
required history, and do not promise a seamless migration before that approval.
New Pytxo checkouts can move behind a server-side MBCZ flag.
Compare provider state with Link entitlement state without letting a shadow
event mutate access. Keep a rollback switch to the prior checkout provider.

Cut over only after a full billing cycle or a deliberately approved shorter
window. Remove Paddle credentials and routes in a later release after refunds,
chargebacks, cancellations, and historical support no longer need them.

## Required Pytxo changes

- `services/pytxo-link`: generic reconciler, Dodo adapter, migration 008,
  redacted audit fields, metrics, and negative tests.
- `apps/web`: raw webhook proxy, pending/success/cancel states, provider-neutral
  authenticated customer-portal redirect, and checkout error handling. Keep
  `mbcz-checkout.ts`; do not import the Dodo SDK into the browser-facing app.
- `.env.example` and Railway docs: add only variable names and rotation notes.
- Public Terms, Privacy, Refund, pricing, and support copy: replace Paddle
  references only when Dodo live mode and the actual customer contract change.
- MBCZ checkout repository: Dodo checkout client and server-side Pytxo
  brand/product catalog. It does not provision Pytxo access.

## Release decision

Do not put the provider cutover in v1.2.1. That release already changes the
entitlement security boundary and must ship from the verified candidate. This
document and the proposed ADR may ship as an honest design record. Runtime work
starts after Dodo test credentials, approved brand and product IDs, webhook
ownership, legal/pricing pages, and a migration window are available.

## Official references

- [Dodo multi-brand setup](https://docs.dodopayments.com/features/multi-brands)
- [Dodo Merchant of Record model](https://docs.dodopayments.com/features/mor-introduction)
- [Dodo subscription webhook events](https://docs.dodopayments.com/developer-resources/webhooks/intents/subscription)
- [Dodo webhook event guide](https://docs.dodopayments.com/developer-resources/webhooks/intents/webhook-events-guide)
- [Dodo under-18 merchant FAQ](https://docs.dodopayments.com/miscellaneous/faq)
- [Dodo webhook CLI and preserved headers](https://docs.dodopayments.com/developer-resources/sdks/cli)
- [Migrate to Dodo](https://docs.dodopayments.com/migrate-to-dodo)
- [Subscription migration constraints](https://docs.dodopayments.com/miscellaneous/subscription-migration)
- [Dodo verification requirements](https://docs.dodopayments.com/miscellaneous/verification-process)
