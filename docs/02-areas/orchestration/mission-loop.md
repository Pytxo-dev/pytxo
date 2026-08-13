---
title: Mission loop
slug: mission-loop
status: active
tags: [orchestration, product, mission]
audience: [human, agent]
layer: orchestration
created: 2026-07-27
updated: 2026-08-13
related: [[product-vision]], [[ADR-0031-mission-planner-byok-scout]], [[ADR-0035-desktop-2-quiet-instrument-ia]], [[signal-core]], [[blast-shield]], [[race-shield]], [[dag-flow-engine]], [[pytxo-improvement-research]]
---

# Mission loop

Pytxo’s primary product loop turns one engineering mission into an inspectable plan, isolated agent waves, real verification, and one reviewable apply (flush).

```text
one mission → proposed plan → human edit/approve → isolated execution
→ verification → one reviewable result
```

## User-facing language

| Internal | User-facing |
|----------|-------------|
| Race Shield | Conflict-aware scheduling |
| Blast Shield | Isolated changes |
| Signal Core | Codebase map |
| DAG / wave | Task dependencies / execution stage |
| Flush | Apply changes |
| Galaxy HITL | Approval required |

## Surfaces

| Surface | Role |
|---------|------|
| `pytxo mission "…"` | CLI dogfood path — preview, approve, dispatch, report |
| Desktop Flow | Same orchestration; edit prompts; dispatch |
| Approvals / Run Review | HITL flush and evidence before apply |

## Planner

Default: Signal-backed decompose ([[ADR-0031-mission-planner-byok-scout]]). Optional BYOK scout LLM when a provider key is present. Offline fallback is heuristic with low-confidence warnings.

## Verification

Optional `verify` shell commands on tasks/mission. Runner executes them in the isolation bubble. Failed verifies block flush eligibility. Orbit still requires human approve-to-flush — verification does not silently merge.

## Scope (Phase 1)

Single execution domain (one repo). Multi-root / fleet missions remain `pytxo project` / `pytxo fleet` until a later phase.

ADRs: [[ADR-0031-mission-planner-byok-scout]], [[ADR-0035-desktop-2-quiet-instrument-ia]] (supersedes the Flow/Focus primary-surface claim in [[ADR-0032-desktop-2-focus-flow-primary]]).

Back: [[MOC-home]] · [[architecture-index]]
