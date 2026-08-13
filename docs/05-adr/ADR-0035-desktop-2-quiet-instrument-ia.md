---
title: "ADR-0035: Desktop 2 quiet instrument IA"
slug: adr-0035-desktop-2-quiet-instrument-ia
status: accepted
tags: [adr, desktop, presentation]
audience: [human, agent]
layer: presentation
created: 2026-08-13
updated: 2026-08-13
adr_id: ADR-0035
related: [[ADR-0032-desktop-2-focus-flow-primary]], [[ADR-0033-reviewed-run-atomic-apply]], [[desktop-visual-system]], [[pytxo-desktop-2-flow-voice]], [[mission-loop]], [[product-vision]]
---

# ADR-0035: Desktop 2 quiet instrument IA

## Status

Accepted

## Context

[[ADR-0032-desktop-2-focus-flow-primary]] made Desktop 2 the default shell and retired 3D topology as the primary viewport. The shipping IA still treated **Flow**, **Runs**, **Topology Focus**, and **Run Review** as sibling destinations, with Integrations as a seventh catalog. The shell duplicated status (AppBar glance, Ops KPI tiles, nav badges), defaulted to a spectrum accent, and offered Terminal/Nebula skins. That costume fought the product: a local decision instrument for one mission.

Presentation-only. This ADR does not change Blast Shield, Race Shield, planner, or the reviewed Apply contract in [[ADR-0033-reviewed-run-atomic-apply]] and [[ADR-0034-immutable-review-package-and-durable-apply]].

## Decision

1. **Primary nav** is six items: Ops, Missions, Approvals, Workspaces, Agents, Settings.
2. **Mission is the core object.** New mission is an action (composer). Detail has three panes: Plan, Live, Review. Review holds files touched, checks, cost, and Apply/Discard. Structural graph is a file list inside Review, not a destination named Topology Focus.
3. **Hash aliases** keep deep links: `flow` → compose, `runs` → missions list, `integrations` → agents, `topology-focus` / `run-review` → mission detail.
4. **Visual contract** is calm Chroma: Void + Light, Geist, teal as the only action/live accent, gold for needs-you and cost. No spectrum hairline, no Terminal/Nebula skins, no 8px type, no KPI tile row.
5. Desktop token overrides stay in `apps/desktop` CSS. Do not restyle `packages/chroma` in a way that silently restyles the marketing site.

[[ADR-0032-desktop-2-focus-flow-primary]] remains historically Accepted for retiring 3D as the default viewport. **This ADR supersedes its “Flow / Focus as primary surfaces” claim.**

## Consequences

### Positive

- Cold start answers what is running, what needs you, what is sandboxed, and what it costs without duplicate widgets.
- Outcome → plan → live → review/apply no longer requires Focus or a separate Runs table.

### Negative

- Capture filenames (`flow-*`, `integrations-*`) remain for marketing asset verification even though those labels left the nav.

## Links

- Supersedes (primary-surface portion of): [[ADR-0032-desktop-2-focus-flow-primary]]
- Related: [[desktop-visual-system]], [[pytxo-desktop-2-flow-voice]], [[mission-loop]]
