---
title: Pytxo Link account and control API
slug: pytxo-link-service
status: active
tags: [billing, api, cloud, reference]
audience: [human, agent]
layer: product
created: 2026-06-05
updated: 2026-09-15
related: [[ADR-0009-ultra-managed-metering]], [[ADR-0040-mbcz-merchant-of-record-boundary]], [[ADR-0041-advisory-coordinator-and-routing-boundary]], [[dodo-mor-integration]], [[pytxo-toml]]
---

# Pytxo Link account and control API

Pytxo Link is the optional account/control API implemented in
[`services/pytxo-link`](../../services/pytxo-link/). It is the smallest shared
backend Pytxo currently needs; it is not a remote replacement for Pytxo Core.

## Ownership boundary

Link owns authenticated identity projection, organization policy and seats,
commerce-derived entitlements, hosted run-usage records, and managed-inference
accounting. Its API groups are:

- account state: `/v1/entitlements/status`, organization policy, seats, and audit;
- usage state: `/v1/runs/*`, `/v1/inference/usage`, and wallet balance;
- commerce ingress: signed provider webhooks normalized into Link-owned grants;
- administration: explicitly authenticated entitlement and organization writes.

Pytxo Core owns local permission profiles, execution domains, mission state,
verification, immutable candidate identity, approvals, and reviewed Apply. Link
cannot mark a candidate verified or authorize an Apply. The local default is
accountless and offline: `billing.mode = "byok"` leaves Link reconciliation off.

Keep adjacent services separate. Pytxo Web requests checkout through the MBCZ
broker and only proxies raw provider webhooks. `pytxo-proxy` holds managed model
credentials and serves inference traffic. A cloud sandbox owns remote execution
and repository transfer. Combining those workloads with Link would couple
billing availability to local execution and put unrelated secrets and data in
one failure domain.

## Commerce projection

Paddle and Dodo are adapters, not sources of Pytxo capability semantics. Each
adapter verifies the provider request and maps allowlisted external product or
price IDs into a canonical plan key. The provider-neutral reconciler persists
event identity, subscription binding, lifecycle state, and one grant per
subscription. Effective access is projected across all active grants, so ending
one provider subscription cannot revoke another valid grant. An explicit admin
provisioning record remains a higher-priority override; it is Pytxo business
state, not provider metadata.

The Dodo route follows Standard Webhooks and requires the exact raw body plus
`webhook-id`, `webhook-timestamp`, and `webhook-signature`. Link stores opaque
commerce IDs and normalized state, not card, tax, address, or full webhook
payload data. See [[dodo-mor-integration]] for test-mode and cutover gates.

## Local client

The control plane ships
[`HttpBillingReconciler`](../../crates/pytxo-core/src/billing/link_reconciler.rs),
which builds idempotent run-start and run-end envelopes. With the `link-http`
feature it sends them to Link; otherwise it stays offline-testable. Link is only
contacted when sign-in or explicitly configured reconciliation requires it.

Configure in `pytxo.toml`:

```toml
[billing]
mode = "ultra"
link_reconcile = true
proxy_url = "https://link.pytxo.com"
```

This opt-in URL does not make local planning or local-model coordination depend
on Link. Managed coordinator transport may reuse the entitlement token, but its
inference request goes to the separate proxy described in
[[ADR-0041-advisory-coordinator-and-routing-boundary]].

Back: [[ADR-0009-ultra-managed-metering]] · [[MOC-home]]
