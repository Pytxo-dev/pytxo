---
title: ADR-0005 Worktree isolation for MVP
slug: adr-0005-worktree-isolation-for-mvp
status: accepted
tags: [adr, filesystem, mvp]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
adr_id: ADR-0005
related: [ADR-0003-sparse-overlay-not-ram-cow](/docs/adr-0003-sparse-overlay-not-ram-cow), [sparse-overlay-fs](/docs/sparse-overlay-fs)
---

# ADR-0005: Worktree isolation for MVP

## Status

Accepted

## Context

Phase 0–1 must ship a cross-platform control plane quickly. Sparse FUSE/ProjFS overlays ([ADR-0003-sparse-overlay-not-ram-cow](/docs/adr-0003-sparse-overlay-not-ram-cow)) are high complexity across Windows, macOS, and Linux.

## Decision

For Phase 1, isolate parallel agents using **git worktrees** under `.pytxo/worktrees/{run_id}/{agent_id}` with branches `pytxo/{run_id}/{agent_id}`.

Optional path ownership and task `paths` in `pytxo.toml` drive **scheduler preflight** (wave assignment), not kernel-level overlays.

## Consequences

**Positive**

- Ships on all platforms with only git + subprocess dependencies.
- Aligns with community practice (worktrees, directory boundaries).

**Negative**

- Merge step still required when combining agent branches.
- Does not solve RAM pressure from huge `node_modules` without later overlay work.

## Links

- Supersedes nothing.
- Long-term overlay: [sparse-overlay-fs](/docs/sparse-overlay-fs) / [ADR-0003-sparse-overlay-not-ram-cow](/docs/adr-0003-sparse-overlay-not-ram-cow) when benchmarks require it.
