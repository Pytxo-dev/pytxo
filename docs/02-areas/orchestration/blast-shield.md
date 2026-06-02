---
title: Blast Shield (CoW memory sandbox)
slug: blast-shield
status: active
tags: [orchestration, filesystem, moat]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[sparse-overlay-fs]], [[ADR-0003-sparse-overlay-not-ram-cow]], [[ADR-0005-worktree-isolation-for-mvp]], [[product-vision]]
---

# Blast Shield (CoW memory sandbox)

**Blast Shield** is Pytxo’s **copy-on-write memory sandbox** for agent side effects.

## Goal

- Isolate agent **bash** and **file writes** in memory-backed layers
- **Flush to physical disk only** on explicit user approval
- Enable **sub-5ms rollback** of an agent’s blast radius

## Model

```text
Physical disk (source of truth)
        ↑ flush on approve
Memory-mapped / overlay layer (per agent or per swarm)
        ↑ all agent writes & shell cwd
Headless PTY agents
```

## Relationship to other isolation designs

| Mechanism | Role |
|-----------|------|
| **Blast Shield** (north star) | Approval-gated CoW; fastest rollback |
| [[sparse-overlay-fs]] | Bounded-RAM virtual workspace for huge monorepos (FUSE / ProjFS) |
| Git worktrees (MVP) | Process isolation today ([[ADR-0005-worktree-isolation-for-mvp]]) |

[[ADR-0003-sparse-overlay-not-ram-cow]] rejects copying entire `node_modules` trees into RAM. Blast Shield targets **hot source paths** and agent mutations, not materializing full dependency graphs per agent.

## UX tie-in

Reality Deck shows **structural blast radius** ([[reality-deck-visual-system]]) — which symbols and modules an agent touched — before the user approves a flush.

## Status

- **MVP:** git worktrees + optional worktree cleanup on `pytxo stop`
- **Next:** overlay / mmap layer behind runner write hooks
