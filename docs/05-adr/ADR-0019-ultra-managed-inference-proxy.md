---
title: ADR-0019 Ultra managed-inference proxy
slug: adr-0019-ultra-managed-inference-proxy
status: accepted
tags: [adr, billing, commercial]
audience: [human, agent]
layer: architecture
created: 2026-06-22
updated: 2026-06-22
related: [[ADR-0009-ultra-managed-metering]], [[ADR-0021-billing-source-of-truth]]
---

# ADR-0019: Ultra managed-inference proxy

## Status

Accepted (2026-06-22)

## Context

Ultra `ManagedTransport` injects `ANTHROPIC_BASE_URL={proxy}/anthropic` etc., but no proxy service existed. Clients cannot use managed inference without server-side provider key custody.

## Decision

1. Deploy **`services/pytxo-proxy`** as a dedicated inference proxy (blast-radius isolation from Link).
2. Routes: `/anthropic/*`, `/openai/*`, `/google/*`, `/deepseek/*`, `/openrouter/*`, `/agy/*`.
3. Provider API keys live only on the proxy host (env vars); Ultra clients use `PYTXO_ULTRA_SESSION` Bearer auth.
4. Session validation: optional fetch to Link `/v1/entitlements/status` for tier gate.
5. Streaming responses pass through unchanged (SSE).

## Consequences

- Link handles entitlements + ledger; proxy handles inference only.
- `billing.proxy_url` in `pytxo.toml` should point at proxy base (e.g. `https://proxy.pytxo.com`), not Link `/v1`.
- Production requires provider keys on proxy and `PROXY_REQUIRE_AUTH=1`.

## Alternatives considered

- Proxy routes on Link — rejected (key blast radius + coupling).
- Client-side BYOK for Ultra — rejected (product definition).
