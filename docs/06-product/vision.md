---
title: Product vision
slug: product-vision
status: active
tags: [product, vision, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-09-07
related: [[commit-layer]], [[mission-loop]], [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains]], [[pytxo-commit-layer-alignment]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0036-effect-contract-commit-boundary]]
---

# Product vision

**Pytxo is an agent hypervisor.** It prepares missions, manages existing agent
instances and their resources, observes execution, verifies candidate results,
and controls their authorized integration. Agents do the work; Pytxo controls
what becomes real within the boundaries it actually mediates.

The shipping beachhead is repository work: Pytxo coordinates the coding-agent CLIs a
developer already uses, runs their work in isolated spaces, verifies the run,
and prepares exact repository bytes for review and Apply. One harness and one
worker are first-class. The commit boundary is an important hypervisor primitive,
not its entire identity. Generalized production effects remain a proposed
expansion, not a requirement for the next release or a capability claim.

## The control point

Model routing, agent harnesses, sandboxes, durable workflows, approvals, and
traces are useful inputs. None alone answers the question Pytxo owns:

> Which state transition may become real, under whose authority, with what
> evidence, and with what recovery path?

Pytxo therefore sits at the boundary between a proposed action and its
consequential effect. The agent proposes; an external control path prepares,
authorizes, commits, verifies, and records the result ([[commit-layer]]).

## Product contract

```text
observe -> propose -> prepare -> authorize -> commit -> verify -> receipt
                                      failure -> compensate or contain -> re-verify
```

The proposed generalized [[commit-layer|effect contract]] describes a state transition:
principal and delegation lineage, resources, preconditions, allowed effects,
budgets, commit policy, postconditions, evidence, and recovery class. Authority
is short-lived and bound to that exact effect. Verification observes the
post-state independently of the acting agent wherever deterministic checks are
available.

Effects are classified before commit:

| Class | Meaning | Required behavior |
|-------|---------|-------------------|
| Reversible | Prior state can be restored with high confidence | Tested rollback may run automatically |
| Compensatable | An inverse action reduces harm but cannot erase history | Tested compensation plus residual-risk record |
| Irreversible | Disclosure, communication, human, or physical effects cannot be undone | Strong approval, containment plan, explicit acknowledgment |

Pytxo does not promise a universal undo button.

## The first implementation

The current mission loop is a repository-scoped specialization of this
contract:

```text
one mission -> approved plan -> isolated waves -> verification
-> immutable review package -> affected-path revalidation -> Apply receipt
```

For Orbit and Galaxy, the immutable package binds reviewed bytes to one
execution domain and one repository root. Apply checks affected-path preimages,
uses only the stored package, journals mutation, and reconciles interrupted
attempts ([[ADR-0034-immutable-review-package-and-durable-apply]]). This is real
commit-layer evidence for code. It is not yet a production API gateway,
customer-VPC effector, cross-system evidence graph, or external compensation
engine.

## Architecture direction

Pytxo will deepen the boundary rather than rebuild commodity layers:

| Build deeply | Integrate or consume |
|--------------|----------------------|
| Typed Mission and Effect Contracts | Models and agent harnesses |
| Prepare/commit gateway | Sandboxes and cloud computers |
| Effect-bound authority and delegation lineage | IAM and policy languages |
| Independent post-state verification | Durable workflow engines |
| Causal evidence and recovery state | Trace storage and dashboards |
| Certified semantic effect adapters | MCP, A2A, OpenAPI, and SDK transport |

The existing controls remain implementation primitives:

- **Signal Core** supplies bounded structural observation and context.
- **Blast Shield** supplies prepared isolation and the first reviewed commit
  boundary.
- **Race Shield** supplies ownership and concurrency control before effects
  contend.
- **Permission profiles** describe the local execution boundary; effect-bound
  production authority is a separate, stricter contract.

The plausible long-term moat is not generic orchestration. It is accumulated
depth in semantic adapters, authority-to-effect provenance, injected-failure
fixtures, independent verification, and honest recovery behavior.

## Beachhead and expansion

The proposed next beachhead is agentic production change management: code,
CI/CD, Kubernetes, one cloud surface, feature flags, and observability or
incident tooling. Pytxo should first run read-only and in shadow mode. Gated
production execution is earned only after the buyer, GitOps-substitution,
verification, latency, and adapter-economics tests in
[[pytxo-commit-layer-alignment]] pass.

## Non-goals

- Another general agent builder, chat UI, or proprietary model router
- A new sandbox provider or generic workflow engine
- A generic connector or MCP marketplace
- An observability dashboard whose evidence ends at the tool-call log
- A proprietary identity or policy language
- Cross-root or cross-system atomicity claims
- Unqualified rollback, exactly-once, or production-assurance claims
- Letting a worker expand its authority or rewrite the enforcement kernel

Back: [[MOC-home]] · [[commit-layer]] · [[mission-loop]]
