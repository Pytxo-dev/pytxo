---
title: Pytxo Desktop
---

# Pytxo Desktop

**Pytxo Desktop** is an optional control UI for runs, structure, approvals, and diffs. It is not a wall of terminal panes. (Older docs called this Reality Deck.)

Download installers from [pytxo.com/download](https://pytxo.com/download).

## At a glance

| Panel | What you see |
|-------|----------------|
| Workspaces | One or more project folders under one coordinated run |
| Runs & agents | Active swarms, waves, per-folder labels |
| Topology | Graph of edited files and import edges |
| Approvals | Pending Galaxy requests (with domain label) |
| Project paths | Roots, read-only badges, permission tiers |
| Fleet runs | Cross-repo fleet status |
| Logs & diff | Supporting detail, not the main surface |

## Design principles

- Dark UI with teal, violet, and gold accents
- Structural views over raw terminal walls
- No direct filesystem access from the UI; all data comes through local IPC

## Structural topology

During an active run, the topology view shows:

- **Nodes:** files and modules agents touched
- **Edges:** import relationships between them
- **Glow:** how recent the edits are, plus context savings

This is a live map of how agent edits touch your code, not a wall of terminal text.

## What it is not

- Not a replacement for your IDE
- Not the primary place to read agent stdout (logs are supporting panels)

## Requirements

Install the Pytxo CLI first. Desktop talks to the same orchestrator and SQLite stores as `pytxo run`.

Back: [What is Pytxo?](/docs/concepts/what-is-pytxo) · [Galaxy approvals](/docs/concepts/galaxy-approvals)
