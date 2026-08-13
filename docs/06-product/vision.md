---
title: Product vision
slug: product-vision
status: active
tags: [product, vision, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-08-13
related: [[agent-os-vs-virtual-workspace]], [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[desktop-visual-system]], [[competitive-benchmarks]], [[pytxo-improvement-research]], [[beyond-the-ade]], [[pytxo-vs-github-copilot-app]], [[mission-loop]]
---

# Product vision

**Pytxo** ([ptyxo.com](https://ptyxo.com)) is a local, **inspectable workflow engine** that coordinates the coding agents you already use — Claude Code, Codex, OpenCode, and similar — through explicit plans, dependencies, isolation, verification, and one reviewable apply.

Grounded thesis: turn **one messy engineering mission** into **safe parallel work and one verified result** — not the product with the most agents on screen, and not another opaque autonomous swarm ([[mission-loop]]).

It is **not** an ADE with walls of terminals, and **not** a single-vendor agent desktop (see [[pytxo-vs-github-copilot-app]], [[beyond-the-ade]]).

Maturity and honesty program: [[pytxo-improvement-research]]. Mission Loop Phase 1: [[mission-loop]], [[ADR-0031-mission-planner-byok-scout]].

## Problem

Independent worktrees are good for unrelated tasks. Interconnected work needs an execution order, handoffs, isolation, and checks after integration. Vendor control centers solve supervision inside one ecosystem; ADE terminal walls optimize for demos. Pytxo’s job is **cross-CLI local orchestration** with an inspectable plan the operator owns.

## Primary loop

```text
one mission → proposed plan → human edit/approve → isolated execution
→ verification → one reviewable result
```

CLI: `pytxo mission "…"`. Desktop: Ops → Missions (plan / live / review) → Approvals ([[ADR-0035-desktop-2-quiet-instrument-ia]]).

## Plain language at the product boundary

| Internal | User-facing |
|----------|-------------|
| Race Shield | Conflict-aware scheduling |
| Blast Shield | Isolated changes |
| Signal Core | Codebase map |
| DAG / wave | Task dependencies / execution stage |
| Flush | Apply changes |
| Galaxy HITL | Approval required |

Deep docs keep moat codenames; first-run UX should not require them.

## Workspaces (multi-project)

**Productivity max** has two layers:

1. **Many projects at once** — dispatch independent swarms on different directories without leaking scheduler, registry, or log state. Each repo is an [[execution-domains|execution domain]] with its own WAL and [[permission-profile-engine|permission profile]] (default **Orbit**).
2. **Workspaces (modular projects)** — one Pytxo **Workspace** can include **multiple path roots**. See [[modular-projects]].

Mission Phase 1 is **single-domain**; multi-root stays `project` / `fleet` until a later phase.

```text
IDE / CLI  →  MCP hub  →  Orchestration (Rust)  →  Execution yard (PTY agents)
                              ↓
                    Pytxo Desktop (mission review + evidence)
```

## Three technical moats

All future orchestration code should route through these layers — not around them.

| What it does | Codename | Function | Target |
|--------------|----------|----------|--------|
| **Smarter context** | [[signal-core]] | `tree-sitter` AST skeletons on file read | Aspirational ~60% on body-heavy files; measure with `signal-reduction` ([[competitive-benchmarks]]) |
| **Safe sandbox until you approve** | [[blast-shield]] | Worktree or sparse copy-layer until explicit approval | Flush on approve; kernel ProjFS/FUSE remains north star |
| **No write collisions** | [[race-shield]] | Locked swarm registry + stdin buffering | No cross-agent write collisions; lock-free shards only after profiling |

| Moat | Shipping today | North star gap |
|------|----------------|----------------|
| Signal Core | 8+ language skeletons, closed-loop retry, mission codebase map, 185-file scaffold-byte pin | Billable-token and task-outcome proof; broader grammars |
| Blast Shield | Git worktrees + sparse copy-layer default | Full kernel ProjFS provider + FUSE default |
| Race Shield | Registry, path claims, PTY stdin, Galaxy HITL | Lock-free / path-prefix shards after profiling; runtime claim updates (Phase 2) |

## Pytxo Desktop

Optional **mission-review and intervention** UI ([[desktop-visual-system]], [[ADR-0035-desktop-2-quiet-instrument-ia]]). Formerly Reality Deck.

**Desktop 2** (default): Flow, Ops, Focus, Approvals, Run Review. Interactive **3D** topology remains **legacy shell only** (`desktop_shell_v1=true`).

## Pytxo Cloud

Hosted sandboxes — **capability-gated** until a non-noop cloud dispatcher is configured ([[hybrid-execution]]). Strictly **BYOK** on supported surfaces.

## Non-goals

- Multi-pane embedded terminal walls
- Storing provider API keys in plaintext
- Duplicating IDE editing surfaces
- Claiming unique multi-agent / unique sandbox / invented worktrees vs 2026 peers
- Out-feature-matching Warp Oz across cloud fleets, schedules, and teams
- Semantic cross-file “contract conflict” engines before verification-after-integration works

Back: [[MOC-home]] · [[mission-loop]] · [[beyond-the-ade]]
