---
title: Product vision
slug: product-vision
status: active
tags: [product, vision, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-08-13
related: [[agent-os-vs-virtual-workspace]], [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[desktop-visual-system]], [[competitive-benchmarks]], [[pytxo-improvement-research]], [[beyond-the-ade]], [[pytxo-vs-github-copilot-app]], [[mission-loop]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0035-desktop-2-quiet-instrument-ia]]
---

# Product vision

**Pytxo** ([pytxo.com](https://pytxo.com)) coordinates the coding-agent CLIs
you already use. It assigns task paths and dependencies, runs agents in
isolated workspaces, and prepares one exact result for review.

The operator owns the plan and the final repository decision
([[mission-loop]]). Pytxo does not replace the editor with a terminal wall or
tie orchestration to one agent vendor.

## Problem

Separate worktrees handle unrelated tasks well. Connected changes need ordered
handoffs, explicit ownership, isolation, and verification after integration.
Pytxo supplies that local coordination across Claude Code, Codex, OpenCode, and
other terminal agents.

## Primary loop

```text
one mission -> proposed plan -> human edit/approve -> isolated waves
-> verification -> immutable review package -> Apply
```

CLI: `pytxo mission "…"`. Desktop: Ops → Missions (plan / live / review) → Approvals ([[ADR-0035-desktop-2-quiet-instrument-ia]]).

For Orbit and Galaxy, a successful run prepares an immutable package for one
execution domain and one repository root. Run Review displays the package
digest, exact additions/edits/deletions, ownership, plan, permission profile,
and enforcement receipt. Apply validates affected-path preimages and reads only
that package. Unrelated dirty files are allowed; drift on an affected path
requires a new review
([[ADR-0034-immutable-review-package-and-durable-apply]]).

## Language at the product boundary

| Internal term | User-facing explanation |
|---------------|-------------------------|
| Race Shield | Path ownership and dependency-aware scheduling |
| Blast Shield | Isolated workspaces and reviewed repository changes |
| Signal Core | Syntax structure before full source |
| DAG / wave | Task dependencies / execution stage |
| Flush | Apply changes |
| Galaxy HITL | Approval required |

Deep architecture docs keep the codenames. First-run surfaces explain what each
control does.

## Projects, Workspaces, and roots

Pytxo can run separate repositories concurrently. Each canonical repository
root is an [[execution-domains|execution domain]] with its own scheduler,
registry, SQLite WAL, and [[permission-profile-engine|permission profile]].

A [[modular-projects|Workspace]] can attach multiple path roots for coordinated
planning and execution. Each repository root keeps its own Apply boundary.
Project and fleet ordering does not create a cross-root transaction.

```text
IDE / CLI -> MCP hub -> Rust orchestration -> PTY agents
                          |
                          v
                  Pytxo Desktop
```

## Three technical controls

| What it does | Codename | Shipping mechanism | Open boundary |
|--------------|----------|--------------------|---------------|
| Starts with smaller code structure | [[signal-core]] | `tree-sitter` AST skeletons; measured scaffold-byte pin | Billable-token and task-outcome proof |
| Holds writes for review | [[blast-shield]] | Git worktrees or sparse copy-layer; immutable package; journaled single-root Apply | Kernel ProjFS/FUSE; cross-root transaction |
| Prevents independent path collisions | [[race-shield]] | Path claims, dependency waves, stdin buffering, Galaxy HITL | Runtime claim updates after profiling |

Signal Core's current 82.9% figure is a weighted scaffold-byte reduction across
185 tracked production files. It is not a model-token, cost, or task-success
claim ([[competitive-benchmarks]]).

## Pytxo Desktop

Optional **mission-review and intervention** UI ([[desktop-visual-system]], [[ADR-0035-desktop-2-quiet-instrument-ia]]). Formerly Reality Deck. Operations, Workspaces, and Settings remain primary destinations. The interactive 3D Deck is development-only legacy code.

Desktop sends intents through Tauri IPC. Orchestration owns permission checks,
package preparation, Apply, and recovery.

## Cloud and credentials

Local PTY execution is the default. Cloud execution appears only when a
non-noop dispatcher is configured ([[hybrid-execution]]). Supported cloud
surfaces use BYOK; Pytxo does not store provider keys in plaintext.

## Non-goals

- Embedded multi-pane terminal walls
- Cross-root Apply in v1.1
- Partial-file acceptance in v1.1
- Power-loss ACID or cross-filesystem durability claims
- Kernel-isolation claims where the enforcement receipt is advisory
- Duplicating IDE editing surfaces

Back: [[MOC-home]] · [[mission-loop]] · [[beyond-the-ade]]
