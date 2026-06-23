---
title: ADR-0020 Cloud sandbox runtime
slug: adr-0020-cloud-sandbox-runtime
status: accepted
tags: [adr, cloud, commercial]
audience: [human, agent]
layer: architecture
created: 2026-06-22
updated: 2026-06-22
related: [[ADR-0019-ultra-managed-inference-proxy]], [[cloud-sandbox-service]]
---

# ADR-0020: Cloud sandbox runtime

## Status

Accepted (2026-06-22)

## Context

`pytxo-cloud-sandbox` returned synthetic `cloud-stub` stdout when Docker was unavailable, masking production failures. Delta sync was a no-op. Teardown did not remove containers. Max-tier users need real isolated execution with entitlement gating.

## Decision

1. **Fail-closed exec** — without a live container, `/exec` returns HTTP 503; no stub success path.
2. **Delta sync** — `sync_sandbox` stages files via local tar archive + `docker cp` + in-container `tar xf` under `/workspace`.
3. **Teardown** — `DELETE /v1/sandboxes/{id}` runs `docker rm -f` for the leased container.
4. **TTL sweeper** — background task removes sandboxes older than `CLOUD_SANDBOX_TTL_SECS` (default 3600).
5. **Entitlement gate** — optional `LINK_BASE_URL` + `x-pytxo-user-id` header; start/exec require Link `cloud_enabled`.
6. **Content hash** — `pytxo-core::cloud::content_hash` uses SHA-256 (not `DefaultHasher`) for cross-process cache keys.

## Consequences

- Deck cloud badge derives from WAL `cloud-delta` / `cloud-exec` events, not stdout sniffing.
- Local dev without Docker gets explicit errors instead of false-positive cloud runs.
- Production cloud service requires Docker socket and Link entitlements for Max tier.

## Alternatives considered

- Firecracker microVMs — deferred; Docker POC sufficient for v1 reference service.
- Stub mode behind env flag — rejected (violates fail-closed telemetry honesty).
