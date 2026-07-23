---
title: Presentation layer — passive telemetry
slug: presentation-passive-telemetry
status: active
tags: [architecture, svelte, tauri]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-07-23
related: [[three-tier-model]], [[desktop-visual-system]], [[execution-domains]], [[permission-profile-engine]], [[product-vision]], [[pytxo-improvement-research]]
---

# Presentation layer — passive telemetry

The Pytxo desktop shell is a **passive telemetry skin** built with **Svelte 5** and **Tauri v2** — see [[desktop-visual-system]] for the space-console aesthetic. Default product surface is **Desktop 2 structural Focus**, not a 3D canvas.

## Constraints

- **No direct filesystem access** from the UI.
- All workspace state flows through Tauri v2 **IPC** into `pytxo-orchestrate` / `pytxo-store`.
- **Not** a multi-pane embedded terminal wall ([[product-vision]]).

## Implementation notes

- **Svelte 5 Runes** (`$state`, `$derived`, `$effect`) for high-frequency streams.
- Log panel: bounded scrollback, ~60 Hz poll budget (not unbounded raw text as the hero).
- **xterm.js** for supporting log view; **Monaco** or inline diff for approve-path review.
- **Primary (shipping):** Desktop 2 structural Focus graph driven by [[signal-core]] parse output (`FocusScreen.svelte`).
- **Legacy only:** interactive 3D AST (`TopologyScene3D.svelte`) when `desktop_shell_v1=true`.

## IPC and permission intents

The UI sends **intents**; orchestration enforces [[permission-profile-engine|permission profiles]] and mutates disk:

| Intent (examples) | UI role | Orchestration role |
|-------------------|---------|---------------------|
| `list_domains`, `tail_events(domain_id)` | Poll telemetry per project | Route to correct SQLite WAL ([[execution-domains]]) |
| `dispatch(repo, task)` | Start swarm on a repo | `HypervisorRegistry::ensure_domain` |
| `commit_workspace` | Approve Blast flush | Validate **Orbit+** profile; call `IsolationBackend::flush` |
| `hitl_respond` | Answer Galaxy prompt | Unblock agent in Race Shield HITL queue |

Never open `pytxo.db` or repo files from Svelte—Tauri commands only ([[phase-2-reality-deck]], ADR-0001).

## Pytxo Desktop

Live execution visualization for local runs (cloud when a dispatcher is configured — [[hybrid-execution]]). Shipped: Ops (live poll), Flow, Approvals, Focus / Run Review, thin Fleet panel from `snapshot.fleets` (Phase 74 — [[market-ready-polish-research]]). Multi-project: independent poll channels per `domain_id`—no global interleaved log stream.

Repo path: `apps/desktop` (crate `pytxo-desktop`).

Orchestration policy never lives in this layer ([[three-tier-model]]).
