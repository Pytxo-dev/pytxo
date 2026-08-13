---
title: ADR-0033 Reviewed run atomic apply
slug: adr-0033-reviewed-run-atomic-apply
status: accepted
tags: [adr, orchestration, blast-shield, desktop]
audience: [human, agent]
layer: orchestration
created: 2026-07-31
updated: 2026-07-31
adr_id: ADR-0033
related: [[mission-loop]], [[blast-shield]], [[permission-profile-engine]], [[ADR-0017-overlay-flush-contract]], [[ADR-0032-desktop-2-focus-flow-primary]]
---

# ADR-0033: Reviewed run atomic apply

## Status

Accepted

## Context

[[ADR-0017-overlay-flush-contract]] defines how one overlay copies its upper
tree into the physical repository. The mission loop later exposed each
successful agent workspace as a separate Desktop action. Applying those
workspaces sequentially could leave the primary checkout partially changed if
a later workspace conflicted or failed. The contract also did not preserve one
run-level record of its plan, permission enforcement, reviewed bytes, and final
filesystem result.

## Decision

1. The unit of review and Apply for a single-domain mission is the **run**, not
   an individual agent workspace.
2. Dispatch persists a run contract containing the pinned Git base revision,
   validated task DAG, requested and effective permission profiles, and an
   enforcement receipt.
3. A run becomes Apply-eligible only after every planned task has a completed
   agent result with exit code zero. Orbit and Galaxy use reviewed Apply;
   DeepSpace is non-flushable; Supernova writes directly and has no Apply step.
4. Apply prepares one deterministic change set across all agent workspaces.
   Every add, modification, and delete must match a declared path claim.
   Independent tasks changing the same path conflict; a dependency-ordered
   downstream task may supersede its ancestor.
5. The prepared set pins SHA-256 digests for each primary-repository base file
   and each agent result. Apply rejects a dirty checkout, changed `HEAD`, base
   drift, result drift, unsafe path, symlink source, or non-file destination.
6. Primary-repository writes form one local transaction. Existing files are
   backed up under `.pytxo/apply/<transaction-id>`, changes are applied in
   deterministic order, and any failure restores the complete pre-Apply state.
7. SQLite owns the Apply lifecycle (`pending`, `ready`, `applying`, `applied`,
   `failed`, `non_flushable`, `not_applicable`, `unsupported`) and prevents two
   callers from claiming the same run.
8. Desktop remains a passive presentation tier. `apply_run_changes` sends one
   run-scoped intent; orchestration validates the persisted contract and
   returns the path-level manifest.
9. Multi-root atomic Apply is explicitly unsupported until a cross-filesystem
   transaction or compensating protocol is accepted.

## Consequences

**Positive**

- A failed final write cannot leave an earlier agent result applied.
- Review evidence and mutation evidence share one persisted source of truth.
- Deletes, human edits after dispatch, and agent-result drift fail safely.

**Tradeoffs**

- The primary checkout must be clean at dispatch and Apply.
- Rollback is process-level in v1.1. An abrupt process termination or power
  loss during the write window is not yet recovered from a durable journal and
  may require Git or manual recovery.
- v1.1 does not offer partial path selection, automatic rebasing, retry after a
  failed contract, or atomic multi-root Apply.
- Copy-layer and worktree backends still own workspace creation; this ADR
  supersedes ADR-0017 only for the reviewed mission flush path.

## Links

- Supersedes (reviewed mission flush portion): [[ADR-0017-overlay-flush-contract]]
- Related: [[mission-loop]], [[blast-shield]], [[permission-profile-engine]]
