---
title: ADR-0036 Effect-contract commit boundary
slug: adr-0036-effect-contract-commit-boundary
status: proposed
tags: [adr, orchestration, security, effects]
audience: [human, agent]
layer: orchestration
created: 2026-08-27
updated: 2026-08-27
adr_id: ADR-0036
related: [[product-vision]], [[commit-layer]], [[mission-loop]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[pytxo-commit-layer-alignment]]
---

# ADR-0036: Effect-contract commit boundary

## Status

Proposed. Acceptance requires the Phase 0 evidence gates in
[[pytxo-commit-layer-alignment]].

## Context

Pytxo ships a strong repository boundary: isolated agent work becomes one
immutable review package, and journaled Apply mutates one repository root from
those reviewed bytes. The wider market increasingly supplies agent routing,
sandboxes, durable sessions, approvals, and traces. Rebuilding that horizontal
bundle would broaden Pytxo while weakening its differentiation.

Consequential agent work also crosses systems that do not share a transaction.
A trace can show that a tool was called without proving that fresh authority
covered this exact effect, that the effect happened once, that its post-state
matched the objective, or that recovery restored an acceptable state.

## Proposed decision

1. Define Pytxo's product north star as the commit layer for autonomous work.
2. Generalize the repository Apply contract into typed **Effect Contracts**
   with resources, preconditions, allowed effects, budgets, commit policy,
   postconditions, evidence requirements, and recovery class.
3. Enforce an `observe -> propose -> prepare -> authorize -> commit -> verify
   -> receipt` lifecycle outside the acting model or harness.
4. Keep standing production credentials out of model runtimes. Execute through
   customer-controlled, narrowly scoped effectors and revalidate authority at
   commit time.
5. Integrate existing IAM, policy languages, durable workflow engines,
   sandboxes, and telemetry stores. Do not invent proprietary replacements.
6. Accept adapters only when they declare idempotency, verification,
   reversibility, compensation, risk, evidence, and failure-injection fixtures.
7. Preserve the local agent hypervisor as an execution yard and code-change
   beachhead, not the complete category claim.

## Consequences

This direction narrows the company thesis while expanding the eventual effect
surface. It makes external semantics, verification, availability, and adapter
economics primary engineering risks. It also makes GitOps substitution an
existential test: Pytxo must prove material assurance beyond existing Git and
infrastructure controls.

No production gateway, external adapter, exactly-once guarantee, or universal
rollback claim follows from this proposal. A later Accepted ADR must pin the
contract version, trust boundary, fail-open/fail-closed policy, storage model,
and first enforced adapter set after shadow evidence exists.

## Links

- Extends rather than supersedes: [[ADR-0034-immutable-review-package-and-durable-apply]]
- Concept: [[commit-layer]]
- Evidence plan: [[pytxo-commit-layer-alignment]]
