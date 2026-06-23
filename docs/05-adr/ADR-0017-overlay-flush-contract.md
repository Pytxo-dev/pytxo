---
title: ADR-0017 Overlay flush contract
slug: adr-0017-overlay-flush-contract
status: accepted
tags: [adr, orchestration, blast-shield]
audience: [human, agent]
layer: orchestration
created: 2026-06-19
updated: 2026-06-19
adr_id: ADR-0017
related: [[blast-shield]], [[ADR-0003-sparse-overlay-not-ram-cow]], [[ADR-0005-worktree-isolation-for-mvp]]
---

# ADR-0017: Overlay flush contract

## Status

Accepted

## Context

Orbit's north star is approval-gated CoW via overlay ([[blast-shield]]). The copy-layer POC (`overlay-fuse`) prepared upper directories but `flush` was a no-op for non-git overlays.

## Decision

1. **Copy-layer flush:** When `IsolationMode::Overlay` uses a copy-layer upper (`branch` empty), `flush` recursively copies the upper tree into the physical `repo_root`.
2. **Git worktree path unchanged:** Overlay backends that delegate to worktrees still use `merge_agent_branch` on flush.
3. **Doctor:** `overlay_isolation` probe reports copy-layer or kernel mount availability per platform.

## Consequences

- Non-git read-only roots can participate in modular projects once overlay is enabled.
- Rollback remains `remove_dir_all` on the overlay layer directory.
- Kernel FUSE / ProjFS paths keep their platform-specific rollback hooks.
