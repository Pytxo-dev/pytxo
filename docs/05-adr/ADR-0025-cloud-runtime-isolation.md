---
title: ADR-0025 Cloud runtime isolation
slug: adr-0025-cloud-runtime-isolation
status: accepted
tags: [adr, cloud, security]
audience: [human, agent]
layer: security
created: 2026-06-24
updated: 2026-06-24
related: [[ADR-0020-cloud-sandbox-runtime]], [[cloud-sandbox-service]]
---

# ADR-0025: Cloud runtime isolation

## Status

Accepted (2026-06-24)

## Context

[[ADR-0020-cloud-sandbox-runtime]] shipped real Docker-backed sandboxes but containers ran with default Docker privileges, in-memory scaffold cache only, and no egress policy. Max-tier cloud workers execute untrusted agent code; the reference `pytxo-cloud-sandbox` service must harden container runtime defaults and support shared cache for multi-worker deploys.

## Decision

1. **Container hardening** — `docker run` uses `--user 1000:1000`, `--memory 512m`, `--cpus 1`, `--security-opt no-new-privileges`, `--read-only` rootfs with `tmpfs` mounts for `/workspace` and `/tmp`.
2. **Egress deny-by-default** — sandboxes attach to a Docker `--internal` network (`pytxo-sandbox-internal` by default). Optional `CLOUD_EGRESS_ALLOWLIST` (comma-separated hostnames) documents production iptables/nftables allow rules; internal network blocks egress without host-side rules.
3. **Redis scaffold cache** — when `REDIS_URL` is set, `/v1/cache/scaffold` uses the `redis` crate; otherwise in-process `HashMap` (single-worker dev).
4. **Load test** — concurrent cache lease test in `pytxo-cloud-sandbox` validates worker pool under parallel requests.

## Consequences

- Cloud deploys without Docker socket cannot start sandboxes (unchanged fail-closed).
- Read-only rootfs limits in-container package installs; agents sync workspace via delta API only.
- Operators with multi-replica cloud service should set `REDIS_URL` for cache coherence.
- `CLOUD_EGRESS_ALLOWLIST` is advisory until host firewall rules are applied; internal network alone blocks all egress.

## Alternatives considered

- gVisor / Kata runtime — deferred; Docker flags sufficient for reference service v1.
- Per-sandbox iptables from the service — rejected (requires `CAP_NET_ADMIN` in service container; document host rules instead).
