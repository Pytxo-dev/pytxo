---
title: Sparse overlay filesystem
slug: sparse-overlay-fs
status: active
tags: [orchestration, filesystem]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [ADR-0003-sparse-overlay-not-ram-cow](/docs/adr-0003-sparse-overlay-not-ram-cow), [blast-shield](/docs/blast-shield)
---

# Sparse overlay filesystem

Complements **[blast-shield](/docs/blast-shield)** for huge monorepos. Mapping entire repos (`node_modules`, build artifacts, large binaries) into RAM is not viable. Pytxo uses a **sparse overlay virtual filesystem** for read-mostly bulk paths while hot agent writes stay in approval-gated layers.

## Technology

| OS | Mechanism |
|----|-----------|
| macOS / Linux | **FUSE** overlay |
| Windows | **Projected File System (ProjFS)** |

## Sparse tracking

- Heavy deps and assets → read-only virtual links to physical disk.
- Active source trees → lightweight in-memory layer for agent writes.

## Outcome

Active memory stays bounded (target: low hundreds of MB for overlay metadata + hot paths) while local compiles remain fast.

ADR: [ADR-0003-sparse-overlay-not-ram-cow](/docs/adr-0003-sparse-overlay-not-ram-cow).

**MVP note:** Phase 1 uses git worktrees instead ([ADR-0005-worktree-isolation-for-mvp](/docs/adr-0005-worktree-isolation-for-mvp)).

**POC (2026-06-05):**

| Feature | Platform | Behavior |
|---------|----------|----------|
| `overlay-fuse` | all | Copy-layer upper dir ([blast-shield](/docs/blast-shield)) |
| `overlay-fuse-kernel` | Linux | Kernel `overlay` mount when available |
| `overlay-projfs` | Windows | ProjFS provider stub; falls back to copy-layer / worktree |
