---
title: Pytxo Desktop
---

# Pytxo Desktop

**Pytxo Desktop** is an optional control UI for planning, supervising, and reviewing agent work. It is not a wall of terminal panes. (Older docs called this Reality Deck.)

Download installers from [pytxo.com/download](https://pytxo.com/download).

## At a glance

| Panel | What you see |
|-------|----------------|
| Operations | Active runs, agent health, approvals, failures, and structured activity |
| Workspaces | One or more project folders under one coordinated run |
| Runs & agents | Active swarms, waves, per-folder labels |
| Flow | Mission drafts, reviewable plans, execution waves, ADE assignments, and dispatch |
| Run Review | Semantic changes, validation, completion evidence, and recovery |
| Topology | Structural Focus graph of edited files and import edges (Desktop 2 default; 3D only in legacy shell) |
| Approvals | Pending Galaxy requests (with domain label) |
| Integrations | ADE availability; MCP hub is CLI/IDE (`pytxo-mcp`), not in-app connection config |
| Project paths | Roots, read-only badges, permission tiers |
| Fleet runs | Cross-repo fleet status via CLI today; Desktop 2 fleet panel planned (Phase 74) |
| Logs & diff | Supporting detail, not the main surface |

## Pytxo Flow

Flow is the mission-to-plan workflow. Enter a text mission, select an execution domain, and
review the resulting tasks, waves, path claims, ADE assignments, permissions, isolation, cost
estimates, warnings, and blockers before dispatch.

Planning and dispatch policy remain in Rust. Desktop sends intent; Signal Core, Race Shield,
Blast Shield, permission ceilings, and execution-domain validation decide whether a plan can run.
There is no quick-run path: every Flow mission must be previewed first.

## Pytxo Voice

Voice captures speech and turns the transcript into an editable Flow mission. Version 0.4 uses
local transcription with an on-demand model. Raw microphone audio remains in bounded memory and
is cleared on completion, cancellation, device loss, or shutdown; it is never stored in Pytxo's
catalog, logs, events, or diagnostics.

Voice can create mission text only. It cannot navigate Desktop, approve actions, stop runs, or
dispatch work automatically.

## Design principles

- Dark UI with teal, violet, and gold accents
- Structural views over raw terminal walls
- No direct filesystem access from the UI; all data comes through local IPC
- Plan preview before dispatch
- Keyboard-accessible capture, navigation, and decisions

## Structural topology

During an active run, Topology Focus shows:

- **Nodes:** files and modules from the Signal Core structural graph
- **Edges:** import relationships between them
- **Arbitrage bar:** per-run saved-token totals from `agent_arbitrage` when samples exist

This is a structural map of how agent edits touch your code, not a wall of terminal text. Interactive 3D topology is legacy-shell only.

## What it is not

- Not a replacement for your IDE
- Not the primary place to read agent stdout (logs are supporting panels)

## Requirements

Install the Pytxo CLI first. Desktop talks to the same orchestrator and SQLite stores as `pytxo run`.

Back: [What is Pytxo?](/docs/concepts/what-is-pytxo) · [Galaxy approvals](/docs/concepts/galaxy-approvals)
