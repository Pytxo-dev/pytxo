---
title: Reality Deck
---

# Reality Deck

The **Reality Deck** is an optional desktop app for **passive telemetry** — runs, topology, approvals, and diffs — not a multi-pane terminal grid.

Download installers from [pytxo.com/download](https://pytxo.com/download).

## At a glance

| Panel | What you see |
|-------|----------------|
| Runs & agents | Active swarms, waves, per-root labels |
| Topology | Structural graph of edited files and import edges |
| Approvals | Pending Galaxy HITL requests (with domain label) |
| Project paths | Roots, read-only badges, permission tiers |
| Fleet runs | Cross-repo fleet status |
| Logs & diff | Supporting detail — not the primary surface |

## Design principles

- **Obsidian void** palette with teal, violet, and gold accents
- Structural visualization over raw terminal walls
- No direct filesystem access from the UI — all data via local IPC

## Structural topology

During an active run, the topology view shows:

- **Nodes** — files and modules touched by agents
- **Edges** — import relationships between them
- **Glow** — recency of edits and Signal Core savings

This is a 2.5D blast-radius graph. Full 3D AST exploration remains a north-star goal.

## What it is not

- Not a replacement for your IDE
- Not the primary place to read agent stdout (logs are supporting panels)

## Requirements

Install the Pytxo CLI first. The Deck talks to the same hypervisor and SQLite stores as `pytxo run`.

Back: [What is Pytxo?](/docs/concepts/what-is-pytxo) · [Galaxy approvals](/docs/concepts/galaxy-approvals)
