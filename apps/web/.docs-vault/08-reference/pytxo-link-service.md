---
title: Pytxo Link service (private repo)
slug: pytxo-link-service
status: active
tags: [billing, ultra, reference]
audience: [human, agent]
layer: product
created: 2026-06-05
updated: 2026-06-05
related: [[ADR-0009-ultra-managed-metering]], [[github-organization]], [[pytxo-toml]]
---

# Pytxo Link service (private repo)

Ultra-tier runs reconcile usage against **Pytxo Link**, a small HTTP service hosted outside this monorepo.

## Monorepo client (shipped)

The control plane ships [`HttpBillingReconciler`](../../crates/pytxo-core/src/billing/link_reconciler.rs):

- Builds `POST /runs/start` and `POST /runs/end` envelopes.
- With the `link-http` feature on `pytxo-core`, sends JSON via `ureq`.
- Without the feature, `send()` validates configuration only (offline CI default).

Configure in `pytxo.toml`:

```toml
[billing]
mode = "ultra"
link_reconcile = true
proxy_url = "https://link.pytxo.com/v1"
```

## Service repo (reference implementation)

The Link service itself (auth, webhooks, staging deploy) will live in the private **`pytxo-link`** repository under the Pytxo GitHub org. A **reference implementation** for local Ultra dev ships at [`services/pytxo-link`](../../services/pytxo-link/) in this monorepo until that repo is published. See [[github-organization]] for public vs private split.

Back: [[ADR-0009-ultra-managed-metering]] · [[MOC-home]]
