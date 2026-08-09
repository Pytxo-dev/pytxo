---
title: Mission loop
slug: mission-loop
status: active
tags: [orchestration, product, mission]
audience: [human, agent]
layer: orchestration
created: 2026-07-27
updated: 2026-07-31
related: [[product-vision]], [[ADR-0031-mission-planner-byok-scout]], [[ADR-0032-desktop-2-focus-flow-primary]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[signal-core]], [[blast-shield]], [[race-shield]], [[dag-flow-engine]], [[pytxo-improvement-research]]
---

# Mission loop

Pytxo’s primary product loop turns one engineering mission into an inspectable
plan, isolated agent waves, real verification, and one reviewable run-level
Apply.

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
| Desktop Flow | Compose, dispatch, active state, history, and completed-run review |
| Run Review in Flow | Immutable package, exact changes, enforcement evidence, and Apply history |

## Planner

Default: bounded repository brief plus Signal-backed decomposition
([[ADR-0031-mission-planner-byok-scout]]). Optional BYOK scout output must use
unique task IDs, explicit safe paths, known dependencies, and an acyclic graph.
If safe ownership cannot be inferred, planning blocks instead of claiming the
repository root.

## Verification

Optional `verify` shell commands on tasks/mission run in the isolation bubble.
Failed verifies block Apply eligibility. After a successful Orbit or Galaxy
run, Pytxo stores an immutable package containing the exact reviewed bytes and
their ownership. Apply validates the preimage of each affected path; unrelated
dirty files are allowed. A changed affected path marks the review stale.
Interrupted Apply attempts are reconciled from their journal before another
run or Apply in the execution domain
([[ADR-0034-immutable-review-package-and-durable-apply]]).

## Scope (Phase 1)

Reviewed Apply covers one execution domain and one repository root. Multi-root
and fleet missions do not share a transaction. DeepSpace is non-flushable;
Supernova writes directly to the host tree.

ADRs: [[ADR-0031-mission-planner-byok-scout]],
[[ADR-0032-desktop-2-focus-flow-primary]],
[[ADR-0034-immutable-review-package-and-durable-apply]].

Back: [[MOC-home]] · [[architecture-index]]
