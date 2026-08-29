---
title: Commit layer
slug: commit-layer
status: active
tags: [orchestration, security, effects, verification]
audience: [human, agent]
layer: orchestration
created: 2026-08-27
updated: 2026-08-27
related: [[product-vision]], [[mission-loop]], [[blast-shield]], [[permission-profile-engine]], [[pytxo-commit-layer-alignment]], [[ADR-0036-effect-contract-commit-boundary]]
---

# Commit layer

The **commit layer** is the externally enforced boundary between an agent's
proposal and a consequential change to a real system. It does not decide how
the agent reasons. It decides whether a specific state transition may become
real, performs it through a bounded effector, verifies the result, and records
the evidence and recovery state.

## Lifecycle

```text
Observe -> Propose -> Prepare -> Authorize -> Commit -> Verify -> Receipt
                                             |
                                             v
                         Classify -> Compensate or contain -> Re-verify
```

The actor does not receive a standing production credential. At commit time,
the control path revalidates the principal, delegation lineage, live mission,
budget, policy, resource version, and idempotency identity. The effector holds
the narrow credential inside the customer's trust boundary.

## Effect contract

An **effect contract** is a typed commitment about a state transition, not a
natural-language goal or tool call. At minimum it identifies:

- the human principal, agent workload, and delegation chain;
- typed resources and allowed effects;
- observed resource versions, preconditions, and invariants;
- time, cost, and blast-radius budgets;
- commit-time approval and reauthorization rules;
- independently observable postconditions;
- reversible, compensatable, or irreversible recovery behavior; and
- evidence retention and receipt requirements.

The contract is stable across models, harnesses, and workflow engines. MCP,
A2A, OpenAPI, and CLIs are transports into the boundary, not substitutes for
the contract.

## Evidence levels

Pytxo must keep four claims separate:

| Evidence | What it proves |
|----------|----------------|
| Attempt | A call was issued |
| Effect | The external system accepted or exposed a mutation |
| Outcome | Declared invariants held after the observation window |
| Recovery | Compensation or containment restored an acceptable state |

High-risk verification should prefer deterministic remote observation. A
semantic evaluator may assist when necessary, but it must be configured
independently and must not merely repeat the actor's conclusion.

## Relationship to the shipping product

The immutable repository review package is Pytxo's first effect contract. It
binds exact add, modify, and delete bytes to a base revision, ownership,
permission evidence, a package digest, affected-path preconditions, and a
journaled Apply. That is a real but narrow commit boundary
([[ADR-0034-immutable-review-package-and-durable-apply]]).

External production effects are not shipped. Pytxo does not yet intercept
cloud, Kubernetes, CI/CD, flag, or incident APIs; issue effect-bound
capabilities; verify their post-state; or compensate cross-system failure. The
proposed expansion is governed by [[ADR-0036-effect-contract-commit-boundary]]
and the falsifiable gates in [[pytxo-commit-layer-alignment]].

Back: [[product-vision]] · [[mission-loop]] · [[MOC-home]]
