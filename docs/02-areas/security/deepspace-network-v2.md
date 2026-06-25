---
title: DeepSpace network block v2 spike
slug: deepspace-network-v2
status: draft
tags: [security, network, deepspace]
audience: [human, agent]
layer: security
created: 2026-06-21
updated: 2026-06-21
related: [[permission-profile-engine]], [[network-policy-v1]], [[ADR-0008-local-permission-profile-four-tiers]]
---

# DeepSpace network block v2 spike

**DeepSpace** (tier 1) must keep agents air-gapped from outbound network. Phase 32 shipped **spawn-time** egress classification for Orbit/Galaxy ([[ADR-0018-network-and-mcp-policy]]). This note scopes **v2** platform hooks for true DeepSpace isolation.

## Goals

1. Block outbound sockets for DeepSpace child processes (not only known network binaries at spawn).
2. Preserve BYOK path: cloud workers hold provider keys; local DeepSpace agents never see them.
3. Document platform limits honestly — full syscall-level block is OS-specific.

## Platform matrix (spike)

| OS | Mechanism | Feasibility | Notes |
|----|-----------|-------------|-------|
| Linux | Network namespace (`unshare --net`) or `iptables`/`nftables` per agent cgroup | **High** | Fits `portable-pty` + subprocess yard; requires root or `CAP_NET_ADMIN` for namespace create |
| macOS | `sandbox-exec` profile denying `network-outbound` | **Medium** | Requires signed sandbox profile; conflicts with some CLIs |
| Windows | WFP / AppContainer loopback-only | **Low–Medium** | ProjFS overlay path separate; needs elevation for WFP filters |

## Proposed orchestration contract

```rust
// pytxo-core::moat::permission::NetworkPolicy
fn isolate_deepspace_network(&self, child: &mut Command) -> Result<()>;
```

- **DeepSpace:** call before spawn; no-op on Orbit+.
- Runner records `network-isolation: deepspace-v2` WAL event when hook applied.
- Doctor probe: attempt TCP connect to `1.1.1.1:443` from probe child — expect failure under DeepSpace.

## Non-goals (v2 spike)

- Per-domain DNS filtering (Enterprise Team Space).
- Cloud sandbox network policy (handled in `pytxo-cloud-sandbox` worker image).

## Next steps

1. Linux `unshare -n` default-on for DeepSpace ([[ADR-0026-deepspace-netns-default]]); opt out with `PYTXO_DEEPSPACE_NETNS=0`.
2. ADR if namespace lifecycle (create/teardown per agent) becomes default.
3. Update [[permission-profile-engine]] capability table when POC lands.

Parent: [[permission-profile-engine]].
