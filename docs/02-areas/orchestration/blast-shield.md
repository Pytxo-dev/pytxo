---
title: Blast Shield (CoW memory sandbox)
slug: blast-shield
status: active
tags: [orchestration, filesystem, moat]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-07-18
related: [[sparse-overlay-fs]], [[ADR-0003-sparse-overlay-not-ram-cow]], [[ADR-0005-worktree-isolation-for-mvp]], [[permission-profile-engine]], [[product-vision]], [[pytxo-improvement-research]]
---

# Blast Shield (CoW memory sandbox)

**Blast Shield** isolates agent side effects until explicit user approval, then flushes to the physical tree.

## Goal

- Isolate agent **bash** and **file writes** in a per-agent bubble
- **Flush to physical disk only** on explicit user approval
- Fast rollback of an agent’s blast radius (worktree discard or overlay upper drop)

North-star language historically said “memory-mapped virtual FS” and “sub-5ms rollback.” Those remain **aspirational**; production today is **git worktrees** and **sparse copy-layer** overlays, not an mmap CoW FS.

## Model

```text
Physical disk (source of truth)
        ↑ flush on approve
Worktree or sparse copy-layer upper (per agent)
        ↑ all agent writes & shell cwd
Headless PTY agents
```

## Relationship to other isolation designs

| Mechanism | Role |
|-----------|------|
| **Blast Shield** (shipping) | Approval-gated isolation; worktree or sparse copy-layer |
| [[sparse-overlay-fs]] | Bounded virtual workspace for huge monorepos (FUSE / ProjFS north star; copy-layer interim) |
| Git worktrees | Process isolation ([[ADR-0005-worktree-isolation-for-mvp]]) |

[[ADR-0003-sparse-overlay-not-ram-cow]] rejects copying entire `node_modules` trees into RAM. Blast Shield targets **hot source paths** and agent mutations, not materializing full dependency graphs per agent.

## Permission profile binding

Blast Shield is the primary write-isolation mechanism for **`Orbit`** ([[permission-profile-engine]], [[ADR-0008-local-permission-profile-four-tiers]]):

- Agents read the physical repo; writes and shell cwd target the isolation bubble (`IsolationBackend::prepare`).
- **Approve-to-flush:** physical disk updates only after orchestration calls `IsolationBackend::flush`, triggered by a validated Tauri IPC intent (`commit_workspace` in [[phase-2-reality-deck]]). The UI never writes files directly ([[presentation-passive-telemetry]]).
- **DeepSpace** may forbid flush entirely; **Supernova** may bypass Blast and write to the host tree without CoW.

`IsolationMode` (`worktree` | `overlay`) selects the backend implementation; it does not replace `PermissionProfile`.

## UX tie-in

Pytxo Desktop shows **structural blast radius** ([[desktop-visual-system]]) — which symbols and modules an agent touched — before the user approves a flush. Default Desktop 2 uses a structural Focus list; legacy shell may show 3D topology.

## Status

- **Shipping (Phase 69):** git worktrees + sparse **copy-layer** default via `prefer_kernel_overlay` (`projfs-sparse-copy-v2` on Windows — not a kernel ProjFS provider)
- **North star:** full kernel FUSE / ProjFS provider on all platforms ([[sparse-overlay-fs]], [[ADR-0003-sparse-overlay-not-ram-cow]])
