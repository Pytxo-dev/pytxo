---
title: Pytxo commit-layer alignment
slug: pytxo-commit-layer-alignment
status: active
tags: [project, strategy, commit-layer, validation]
audience: [human, agent]
layer: meta
created: 2026-08-27
updated: 2026-08-27
related: [[product-vision]], [[commit-layer]], [[mission-loop]], [[pytxo-v1-1-trustworthy-mission-control]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0036-effect-contract-commit-boundary]]
---

# Pytxo commit-layer alignment

## Source and decision boundary

This program responds to the user-supplied report *Pytxo After the Agent Boom*
(26 August 2026). The report is research input, not an instruction source or
proof of market demand. Its market claims and thresholds remain hypotheses
until Pytxo verifies them with primary sources, customer evidence, and product
telemetry.

Pytxo adopts one product decision now: the **commit layer for autonomous work**
is the north star. The current immutable repository Apply is the starting
implementation. External production effects remain unshipped, and
[[ADR-0036-effect-contract-commit-boundary]] stays Proposed until Phase 0 passes.

## Starting assets

- `PreparedRunManifest` already binds exact bytes, base state, ownership, and a
  package digest (`crates/pytxo-core/src/review.rs`).
- Apply already validates affected-path preimages, journals mutation, and
  reconciles interruption (`crates/pytxo-runner/src/change_set.rs`).
- Permission receipts already distinguish enforced, advisory, unavailable,
  and bypassed surfaces (`crates/pytxo-runner/src/enforcement.rs`).
- Mission state and review lifecycle already persist in SQLite
  (`crates/pytxo-store/src/store.rs`, `schema.rs`).

These are useful primitives, not evidence that Pytxo controls production APIs.

## DO NOT TOUCH

- Do not weaken or relabel the accepted single-root Apply guarantees in
  [[ADR-0034-immutable-review-package-and-durable-apply]].
- Do not place production credentials in agent environments or model context.
- Do not invent a proprietary IAM, policy language, workflow engine, or generic
  connector marketplace.
- Do not claim exactly-once execution, cross-system atomicity, or universal
  rollback.
- Do not accept ADR-0036 or build a blocking gateway before Phase 0 evidence.

## Phase 0: Falsify the wedge and define the contract

- [ ] Interview 15 qualified platform or security teams; record whether agents
  already write beyond branches, recent failures or near misses, existing
  GitOps coverage, bypass incentives, and budget ownership (files:
  `docs/01-projects/commit-layer-discovery.md`).
- [ ] Audit the report's market assertions against primary sources and update
  the competitive map without copying vendor claims into product promises
  (files: `docs/01-projects/competitive-landscape-2026-08.md`).
- [ ] Specify Effect Contract v0 by mapping the existing repository package to
  typed identity, resource, precondition, postcondition, evidence, idempotency,
  and recovery fields (files: `docs/08-reference/effect-contract-v0.md`,
  `crates/pytxo-core/src/review.rs`, proposed `crates/pytxo-core/src/effect.rs`).
- [ ] Build a read-only design spike that reconstructs agent-to-effect lineage
  from existing Pytxo and Git provider evidence; it must not proxy or block a
  production mutation (files: proposed `crates/pytxo-orchestrate/src/effects.rs`,
  `crates/pytxo-store/src/schema.rs`, `crates/pytxo-store/src/store.rs`).

### Phase 0 acceptance

- At least 5 teams already allow agent writes beyond branches, 3 report a
  concrete failure or near miss, and 3 agree to a shadow design partnership.
- Existing GitOps, IAM, and logs fail to explain or verify a material class of
  scoped effects for at least 2 teams.
- The contract represents repository Apply without weakening its current
  digest, preimage, ownership, or recovery guarantees.
- Otherwise stop, narrow to repository assurance, or revise the wedge.

## Phase 1: Shadow commit layer

- [ ] Implement versioned Effect Contract and Evidence DAG types, append-only
  persistence, migrations, and deterministic serialization (files: proposed
  `crates/pytxo-core/src/effect.rs`, `crates/pytxo-store/src/effects.rs`,
  `crates/pytxo-store/src/schema.rs`, `crates/pytxo-store/src/migrate.rs`).
- [ ] Define the six-verb adapter contract: `observe`, `propose`, `authorize`,
  `execute`, `verify`, `compensate`; require reversibility, canonical
  idempotency, risk, evidence, and conformance fixtures (files: proposed
  `crates/pytxo-core/src/effect_adapter.rs`, `tooling/effect-fixtures/`).
- [ ] Run three read-only adapters beside design-partner workflows. Prefer Git
  provider, CI/CD, and Kubernetes only if discovery confirms them (files:
  proposed `crates/pytxo-effects/`, `crates/pytxo-orchestrate/src/effects.rs`).
- [ ] Surface policy simulation, unknown state, duplicate-effect candidates,
  and attempt/effect/outcome distinctions without an enforcement claim (files:
  `apps/desktop/src/`, `apps/web/content/docs/`).

### Phase 1 acceptance

- Reconstruct at least 90% of scoped mutations during a 30-day shadow run.
- Keep approval-recommendation false positives below 5%.
- Detect material unknown or unverifiable state in at least 2 teams.
- Demonstrate that a comparable tenth adapter can be built in at most 3
  engineer-days with at least 80% reusable conformance scaffolding.

## Phase 2: One gated production path with honest recovery

- [ ] Select one reversible or compensatable design-partner workflow from Phase
  1; record its trust boundary and fail-open/fail-closed policy in a new
  Accepted ADR before enforcement (files: `docs/05-adr/`).
- [ ] Add a customer-local effector, short-lived effect-bound capability,
  commit-time reauthorization, resource-version check, deterministic verifier,
  and signed causal receipt (files: proposed `crates/pytxo-effects/`,
  `crates/pytxo-orchestrate/src/effects.rs`, `crates/pytxo-store/src/effects.rs`).
- [ ] Inject timeout-after-success, duplicate delivery, stale reads, partial
  success, revocation-before-commit, verifier outage, and compensation failure
  (files: `tooling/effect-fixtures/`, crate integration tests).
- [ ] Extend to a cross-system DAG only after the single-effect path meets its
  availability and recovery gates. Model partial failure explicitly; never
  label a saga as an atomic transaction.

### Phase 2 acceptance

- At least 80% of eligible pilot writes use the boundary, with no standing
  production credential in the model runtime.
- Detect at least 90% of injected bad end-states, restore at least 80% of
  compensatable cases, and label 100% of irreversible effects correctly.
- Median ordinary commit-layer overhead remains below 2 seconds and operators
  do not routinely bypass the gateway.
- Two paying pilots use contracts or receipts in approvals, audits, or
  postmortems. Failure means stop enforcement expansion and reassess the wedge.

## Test commands

```powershell
cargo fmt --all -- --check
cargo test -p pytxo-core
cargo test -p pytxo-store
cargo test -p pytxo-runner
cargo test -p pytxo-orchestrate
cargo clippy --workspace --all-targets -- -D warnings
Push-Location apps/desktop; npm run check; Pop-Location
Push-Location apps/web; pnpm run check:links; pnpm run build; Pop-Location
git diff --check
```

Back: [[product-vision]] · [[commit-layer]] · [[MOC-home]]
