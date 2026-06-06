---
title: Deterministic delta sync
slug: delta-sync
status: active
tags: [cloud, sync]
audience: [human, agent]
layer: cloud
created: 2026-06-02
updated: 2026-06-02
related: [sandbox-dispatch](/docs/sandbox-dispatch), [sparse-overlay-fs](/docs/sparse-overlay-fs)
---

# Deterministic delta sync

Cloud hybrid runs minimize network transfer by syncing **only changed file structures**, not full monorepo copies.

## Behavior

1. Compute delta from local overlay / workspace state ([sparse-overlay-fs](/docs/sparse-overlay-fs)).
2. Transmit structural changes to the cloud sandbox.
3. Run compilation, dependency resolution, and model loops remotely.
4. Stream results back; apply approved writes to the physical local workspace.

## Developer experience

Real-time visualization on the Reality Deck; final artifacts land on disk after explicit or policy-driven approval ([pytxo-link-signing](/docs/pytxo-link-signing) for remote approve paths).

Parent: [hybrid-execution](/docs/hybrid-execution).
