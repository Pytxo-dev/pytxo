---
title: Sparse overlay filesystem
slug: sparse-overlay-fs
status: active
tags: [orchestration, filesystem]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-07-23
related: [[ADR-0003-sparse-overlay-not-ram-cow]], [[blast-shield]], [[pytxo-improvement-research]]
---

# Sparse overlay filesystem

Complements **[[blast-shield]]** for huge monorepos. Mapping entire repos (`node_modules`, build artifacts, large binaries) into RAM is not viable.

## Shipping today (Phase 69)

**Production interim** is a sparse **copy-layer** upper directory (plus git worktrees), selected via `prefer_kernel_overlay` — labeled `projfs-sparse-copy-v2` on Windows. This is **not** a kernel ProjFS provider and **not** an mmap CoW FS.

| Feature | Platform | Behavior |
|---------|----------|----------|
| Default overlay path | all | Sparse copy-layer upper dir ([[blast-shield]]) |
| `overlay-fuse-kernel` | Linux | Kernel `overlay` mount when available |
| `overlay-projfs` | Windows | Provider stub; falls back to copy-layer / worktree |
| Git worktrees | all | Still valid isolation backend ([[ADR-0005-worktree-isolation-for-mvp]]) |

## North star (dated roadmap — not marketing)

| OS | Mechanism |
|----|-----------|
| macOS / Linux | Full **FUSE** overlay provider |
| Windows | Full **Projected File System (ProjFS)** provider |

Heavy deps → read-only virtual links; hot agent writes → approval-gated layers. Until that ships, do not advertise FUSE/ProjFS as the default customer path. See [[pytxo-improvement-research]].

## Outcome

Active memory stays bounded (target: low hundreds of MB for overlay metadata + hot paths) while local compiles remain fast. Measure with [[competitive-benchmarks]] overlay scripts.

ADR: [[ADR-0003-sparse-overlay-not-ram-cow]].
