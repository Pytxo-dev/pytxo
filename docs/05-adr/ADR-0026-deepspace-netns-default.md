---
title: ADR-0026 DeepSpace netns default on Linux
slug: adr-0026-deepspace-netns-default
status: accepted
tags: [adr, security, network]
audience: [human, agent]
layer: security
created: 2026-06-24
updated: 2026-06-24
related: [[ADR-0022-deepspace-network-namespace]], [[deepspace-network-v2]]
---

# ADR-0026: DeepSpace netns default on Linux

## Status

Accepted (2026-06-24)

## Context

[[ADR-0022-deepspace-network-namespace]] gated Linux `unshare -n` behind the `deepspace-netns` Cargo feature and `PYTXO_DEEPSPACE_NETNS=1`. Operators missed the opt-in, leaving DeepSpace agents on stub markers without real socket isolation. Doctor reported policy-blocked TCP but did not verify runtime behavior.

## Decision

1. **Linux default-on** — `isolate_deepspace_network` prefixes spawn with `unshare -n` on Linux via `cfg(target_os = "linux")` without requiring the `deepspace-netns` feature. Opt out with `PYTXO_DEEPSPACE_NETNS=0` for debugging.
2. **Doctor socket probe** — `pytxo-runner` spawns an isolated child that attempts TCP connect to `1.1.1.1:443`; doctor `deepspace_network_isolation` requires both policy denial and probe failure.
3. **Windows** — stub documents WFP path via `PYTXO_NETWORK_ISOLATION=1` AppContainer attempt marker; full loopback-only WFP remains future work.
4. **Feature `deepspace-netns`** — retained as empty alias for downstream manifests; no longer gates netns behavior.

## Consequences

- DeepSpace on Linux requires `unshare` in PATH (util-linux); missing binary surfaces spawn error.
- CI and doctor on Windows/macOS may report socket probe not blocked where platform stub applies — honest per platform matrix.
- ADR-0022 consequence about manual feature enable is superseded for Linux default path.

## Alternatives considered

- Keep opt-in env only — rejected (security default should be isolated).
- iptables per agent cgroup — deferred (higher ops burden than netns prefix).
