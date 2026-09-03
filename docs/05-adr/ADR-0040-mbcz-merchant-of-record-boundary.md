---
title: ADR-0040 MBCZ account and Dodo merchant-of-record boundary
slug: ADR-0040-mbcz-merchant-of-record-boundary
adr_id: ADR-0040
status: proposed
tags: [adr, billing, commercial, security]
audience: [human, agent]
layer: cloud
created: 2026-09-03
updated: 2026-09-03
related: [[ADR-0021-billing-source-of-truth]], [[ADR-0009-ultra-managed-metering]], [[dodo-mor-integration]]
---

# ADR-0040: MBCZ account and Dodo merchant-of-record boundary

## Status

Proposed. This does not change the v1.2.1 runtime or authorize a live billing
cutover.

## Context

Pytxo is a customer-facing product under MBCZ. The web application already
creates checkout sessions through an MBCZ-owned broker
(`apps/web/src/lib/billing/mbcz-checkout.ts`). Pytxo Link still contains a
Paddle-specific webhook parser, signature verifier, price catalog, event table,
and subscription table.

Dodo Payments acts as Merchant of Record and legal seller. It supports several
brands under one verified business account while keeping KYC, payouts, and bank
details at the business level. Its webhook
contract follows Standard Webhooks, includes a unique webhook ID, retries
delivery, and warns that subscription events can arrive out of order. Those
properties make direct provider metadata an unsafe entitlement source.

## Decision

Dodo is the legal seller and Merchant of Record. MBCZ is the verified Dodo
business-account owner and payout beneficiary. Pytxo is the approved
customer-facing brand and product. Provider API keys, parent or guardian
identity, payout details, and compliance documents stay in Dodo or an MBCZ
secret store. They never enter this repository, Pytxo configuration, Desktop,
logs, or receipts.

Pytxo Web continues to request checkout from the MBCZ broker. It sends a Pytxo
plan key, the authenticated Clerk user ID, and customer email. The broker owns
the mapping from that plan key to an approved Dodo brand, product, and price.
The browser never supplies a tier that Link treats as authority.

Dodo sends webhooks to a Pytxo-domain endpoint. Pytxo Web forwards the raw body
and the `webhook-id`, `webhook-signature`, and `webhook-timestamp` headers to a
Dodo adapter in Link. Link verifies the provider signature and freshness,
applies business, brand, product, and price allowlists, then persists the event,
customer binding, per-subscription grant, and effective entitlement in one
transaction. The normalized event contract contains:

- issuer, schema version, event ID, provider occurrence time, and lifecycle state;
- Pytxo user or organization identity established during checkout;
- provider, merchant brand ID, customer ID, and subscription ID;
- canonical plan key and lifecycle state;
- original provider event reference for audit, without raw payment data.

Link owns the canonical mapping from plan key to Pytxo capabilities. Payment
events may lower or restore a commercial entitlement, but can never bypass
folder trust, organization policy, permission ceilings, approval, execution
domains, or reviewed Apply.

## Reliability and security rules

Event IDs are unique. Duplicate delivery returns success without repeating the
entitlement effect. Link must not assume Dodo supplies a monotonic sequence.
It uses documented event/object timestamps, lifecycle precedence, and, where
ambiguity remains, a current-subscription lookup before changing access. Unknown
or stale events never upgrade access.

Each provider subscription produces its own grant. Link recomputes the effective
user or organization tier across active grants, so cancelling a legacy Paddle
subscription cannot revoke a valid Dodo subscription. Scheduled cancellation,
on-hold, recovery, expiry, refund, and dispute states have explicit policies;
`subscription.cancelled` is not treated as immediate loss when service remains
valid through the paid period. Logs carry opaque IDs and outcomes, never billing
payloads or personal data.

Paddle and Dodo can coexist during migration, but both adapters must call the
same reconciliation service and generic tables. Cutover requires test-mode
checkout, signed webhook, retry, replay, out-of-order, plan-change,
cancellation, and recovery evidence.

## Consequences

The Pytxo repository loses provider-specific checkout pricing authority, while
Link retains the canonical plan-to-capability mapping. Link needs a generic
billing-event/grant migration and a Dodo adapter. The existing Paddle endpoint
stays available until the dual-run comparison and rollback window complete.

The current parent or guardian account arrangement is an external compliance
matter. Official Dodo guidance requires the guardian's documents and bank
details for an under-18 merchant. Pytxo code should know only the verified MBCZ
merchant and Pytxo brand identifiers.

## Alternatives

Direct Dodo checkout calls from browser-facing Pytxo code were rejected because
they duplicate merchant credentials and product mapping in every MBCZ product.
An MBCZ-normalized webhook gateway was rejected for the first migration because
it adds a new durable queue and acknowledgement boundary outside this repository.
It may be reconsidered when multiple MBCZ products need the same normalization.
Treating redirect query parameters or checkout metadata as entitlement proof
was rejected because neither is a verified payment lifecycle.
