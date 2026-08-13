---
title: Presentation layer — passive telemetry
slug: presentation-passive-telemetry
status: active
tags: [architecture, svelte, tauri]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-08-13
related: [[three-tier-model]], [[desktop-visual-system]], [[execution-domains]], [[permission-profile-engine]], [[product-vision]], [[pytxo-improvement-research]], [[ADR-0034-immutable-review-package-and-durable-apply]]
---

# Presentation layer — passive telemetry

The Pytxo desktop shell is a **passive telemetry skin** built with **Svelte 5** and **Tauri v2** — see [[desktop-visual-system]] for the quiet-instrument aesthetic. Default product surface is **Desktop 2 Ops / Missions / Approvals**, not a 3D canvas. Orchestration remains the only layer that enforces policy or mutates repository files.

## Constraints

- **No direct filesystem access** from the UI.
- All workspace state flows through Tauri v2 **IPC** into `pytxo-orchestrate` / `pytxo-store`.
- **Not** a multi-pane embedded terminal wall ([[product-vision]]).

## Implementation notes

- **Svelte 5 Runes** (`$state`, `$derived`, `$effect`) for local view state.
- Flow contains Compose, Active, History, and contextual Run Review.
- Run Review renders the persisted immutable manifest and exact diff content;
  it never computes a change set from an agent workspace.
- Native mutations emit `pytxo://domain-changed`; a monotonic domain cursor
  catches up CLI and external changes.
- Full snapshots are used for initial load, domain switch, reconnect, cursor
  reset, and an infrequent integrity refresh.
- The interactive 3D Deck is lazy-loaded only in development with both legacy
  flags enabled.

## IPC and permission intents

The UI sends **intents**; orchestration enforces [[permission-profile-engine|permission profiles]] and mutates disk:

| Intent (examples) | UI role | Orchestration role |
|-------------------|---------|---------------------|
| `list_domains`, `tail_events(domain_id)` | Read telemetry for one project | Route to the correct SQLite WAL ([[execution-domains]]) |
| `dispatch(repo, task)` | Start swarm on a repo | `HypervisorRegistry::ensure_domain` |
| `apply_run_changes` | Apply the reviewed package | Claim a ready Orbit/Galaxy review, validate affected paths, and run the journaled single-root Apply |
| `refresh_run_review`, `discard_run_review` | Replace a stale package or discard staged blobs | Prepare/store a new package or remove staged data while retaining audit history |
| `reconcile_run_recovery` | Retry automatic recovery | Reconcile the durable Apply journal and persist the proven state |
| `hitl_respond` | Answer Galaxy prompt | Unblock agent in Race Shield HITL queue |

Never open `pytxo.db` or repo files from Svelte—Tauri commands only ([[phase-2-reality-deck]], ADR-0001).

## Pytxo Desktop

Desktop shows local execution and configured cloud runs
([[hybrid-execution]]). Flow, Operations, Workspaces, Settings, Approvals, and
Integrations share one backend contract. Each execution domain has its own
cursor and event stream; Desktop does not interleave raw logs across roots.

Repo path: `apps/desktop` (crate `pytxo-desktop`).

Orchestration policy never lives in this layer ([[three-tier-model]]).
