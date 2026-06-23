---
title: ADR-0022 DeepSpace network namespace isolation
slug: adr-0022-deepspace-network-namespace
status: accepted
tags: [adr, security, network]
audience: [human, agent]
layer: security
created: 2026-06-22
updated: 2026-06-22
adr_id: ADR-0022
related: [[deepspace-network-v2]], [[ADR-0018-network-and-mcp-policy]], [[ADR-0008-local-permission-profile-four-tiers]]
---

# ADR-0022: DeepSpace network namespace isolation

## Status

Accepted

## Context

[[ADR-0018-network-and-mcp-policy]] blocks known network binaries at spawn for Orbit and DeepSpace via `spawn_egress_allowed`. DeepSpace (tier 1) additionally requires **process-level** outbound isolation so agents cannot open sockets even when the command line is benign ([[deepspace-network-v2]]).

## Decision

1. **`isolate_deepspace_network`:** `pytxo-runner` applies a platform hook to the child `Command` (subprocess) or wraps the PTY one-liner before spawn when `permission_profile = deep_space` only.
2. **Linux:** Behind `deepspace-netns` feature; when `PYTXO_DEEPSPACE_NETNS=1`, prefix spawn with `unshare -n`. Otherwise set `PYTXO_NETWORK_ISOLATION=deepspace-v2-stub` env marker.
3. **macOS:** Wrap with `sandbox-exec` profile denying `network-outbound`.
4. **Windows:** Stub sets `PYTXO_NETWORK_ISOLATION=deepspace-v2-stub` until WFP / AppContainer loopback-only lands.
5. **Runtime policy:** Spawn path calls `NetworkPolicyEngine::egress_allowed("1.1.1.1", 443)` for commands that imply egress; DeepSpace must return `false`.
6. **Telemetry:** Runner emits WAL event `network-isolation: deepspace-v2:<mechanism>` when hook applies.
7. **Doctor:** `deepspace_network_isolation` check reports mechanism label and `tcp_probe_expect_blocked=true`.

## Consequences

- DeepSpace agents get honest platform-specific isolation with documented stubs where full syscall block is not yet available.
- Orbit/Galaxy/Supernova are unchanged; isolation hook is DeepSpace-only.
- Operators enable Linux netns POC with `PYTXO_DEEPSPACE_NETNS=1` and `deepspace-netns` feature on `pytxo-runner`.
