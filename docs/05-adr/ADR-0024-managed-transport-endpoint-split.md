---
title: ADR-0024 Managed-transport endpoint split
slug: adr-0024-managed-transport-endpoint-split
status: accepted
tags: [adr, billing, commercial]
audience: [human, agent]
layer: architecture
created: 2026-06-24
updated: 2026-06-24
related: [[ADR-0019-ultra-managed-inference-proxy]], [[ADR-0021-billing-source-of-truth]]
---

# ADR-0024: Managed-transport endpoint split

## Status

Accepted (2026-06-24)

## Context

Phase 41 shipped `services/pytxo-proxy` as a dedicated Ultra inference proxy (ADR-0019), but `billing.proxy_url` in `pytxo.toml` defaulted to `https://link.pytxo.com`. `ManagedTransport` used that value for `ANTHROPIC_BASE_URL`, so managed inference traffic pointed at Link instead of the proxy. Link handles entitlements and run ledger; the proxy handles provider API forwarding only.

## Decision

1. **`billing.proxy_url`** — Pytxo Link base URL for entitlements, run reconciliation, and org policy (`https://link.pytxo.com`).
2. **`billing.inference_proxy_url`** — Ultra managed-inference proxy base URL (`https://proxy.pytxo.com`). `ManagedTransport` injects `{inference_proxy_url}/anthropic` etc.
3. `pytxo doctor` gains separate `link_reconcile` and `inference_proxy_health` checks.
4. Deploy the proxy as a third Railway service with custom domain `proxy.pytxo.com`.

## Consequences

- Existing configs that set only `proxy_url` continue to work for Link reconcile; inference defaults to `https://proxy.pytxo.com`.
- ADR-0019 consequence #3 is superseded: `proxy_url` is Link, not the inference proxy.
- Ultra dry-run must show `ANTHROPIC_BASE_URL` pointing at the proxy host.

## Alternatives considered

- Single URL with path routing on Link — rejected (ADR-0019 blast-radius isolation).
- Rename `proxy_url` to `link_url` — rejected (breaking change for shipped configs); add `inference_proxy_url` instead.
