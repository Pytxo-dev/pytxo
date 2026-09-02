---
title: Deterministic delta sync
slug: delta-sync
status: active
tags: [cloud, sync]
audience: [human, agent]
layer: cloud
created: 2026-06-02
updated: 2026-06-02
related: [[sandbox-dispatch]], [[sparse-overlay-fs]]
---

# Deterministic delta sync

Cloud hybrid runs minimize network transfer by syncing **only changed file structures**, not full monorepo copies.

## Behavior

1. Compute delta from local overlay / workspace state ([[sparse-overlay-fs]]).
2. Omit protected paths, scan remaining content locally, and emit a path/hash/size
   sync manifest. A likely secret aborts the complete batch.
3. Require `[cloud].upload_consent = true` plus an out-of-band host
   acknowledgement, then transmit the exact manifested files to the cloud
   sandbox. Repository configuration alone cannot grant consent.
4. Run compilation, dependency resolution, and model loops remotely.
5. Stream results back; apply approved writes to the physical local workspace.

The hard denylist includes `.env*`, `.pytxo`, `.git`, credential directories,
private-key containers, and common credential/token filenames. This boundary is
independent of the log/MCP `sanitize` toggle: cloud file uploads always fail
closed and source is never silently redacted into different semantics.

## Developer experience

Real-time visualization on the Pytxo Desktop; final artifacts land on disk after explicit or policy-driven approval ([[pytxo-link-signing]] for remote approve paths).

Parent: [[hybrid-execution]].
