---
title: Presentation layer — passive telemetry
slug: presentation-passive-telemetry
status: active
tags: [architecture, svelte, tauri]
audience: [human, agent]
layer: presentation
created: 2026-06-02
updated: 2026-06-02
related: [three-tier-model](/docs/three-tier-model), [reality-deck-visual-system](/docs/reality-deck-visual-system), [execution-domains](/docs/execution-domains), [permission-profile-engine](/docs/permission-profile-engine), [product-vision](/docs/product-vision)
---

# Presentation layer — passive telemetry

The Pytxo desktop shell is a **passive telemetry skin** built with **Svelte 5** and **Tauri v2** — see [reality-deck-visual-system](/docs/reality-deck-visual-system) for the space-console aesthetic and 3D AST topology target.

## Constraints

- **No direct filesystem access** from the UI.
- All workspace state flows through Tauri v2 **IPC** into `pytxo-orchestrate` / `pytxo-store`.
- **Not** a multi-pane embedded terminal wall ([product-vision](/docs/product-vision)).

## Implementation notes

- **Svelte 5 Runes** (`$state`, `$derived`, `$effect`) for high-frequency streams.
- Log panel: bounded scrollback, ~60 Hz poll budget (not unbounded raw text as the hero).
- **xterm.js** for supporting log view; **Monaco** or inline diff for approve-path review.
- **Primary (target):** 3D AST dependency graph driven by [signal-core](/docs/signal-core) parse output.

## IPC and permission intents

The UI sends **intents**; orchestration enforces [permission profiles](/docs/permission-profile-engine) and mutates disk:

| Intent (examples) | UI role | Orchestration role |
|-------------------|---------|---------------------|
| `list_domains`, `tail_events(domain_id)` | Poll telemetry per project | Route to correct SQLite WAL ([execution-domains](/docs/execution-domains)) |
| `dispatch(repo, task)` | Start swarm on a repo | `HypervisorRegistry::ensure_domain` |
| `commit_workspace` | Approve Blast flush | Validate **Orbit+** profile; call `IsolationBackend::flush` |
| `hitl_respond` | Answer Galaxy prompt | Unblock agent in Race Shield HITL queue |

Never open `pytxo.db` or repo files from Svelte—Tauri commands only ([phase-2-reality-deck](/docs/phase-2-reality-deck), ADR-0001).

## Reality Deck

Live execution visualization for local and cloud runs. Shipped panels: runs, waves, logs, diff. North-star UI: structural blast radius on the topology graph. Multi-project: independent poll channels per `domain_id`—no global interleaved log stream.

Repo path: `apps/desktop` (crate `pytxo-desktop`).

Orchestration policy never lives in this layer ([three-tier-model](/docs/three-tier-model)).
