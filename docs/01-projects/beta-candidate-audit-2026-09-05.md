---
title: Combined candidate verification audit
slug: beta-candidate-audit-2026-09-05
status: active
tags: [project, beta, verification, reliability]
audience: [human, agent]
layer: orchestration
created: 2026-09-05
updated: 2026-09-06
related: [[mission-loop]], [[beta-competitor-research-2026-09-05]], [[v1-1-architecture]], [[ADR-0034-immutable-review-package-and-durable-apply]]
---

# Combined candidate verification audit

## Sep 6 resolution

The historical finding below is now addressed by version 3 source-inventory and
recipe-bound candidate verification. Eight runner scope tests and the actual
orchestration negative control pass. The full workspace suite passes. Explicit
refresh reruns the recipe while preserving frozen effects and current operator
files; successful refresh can recover a failed review only after all tasks
succeeded and the run has no event-persistence evidence gap. Git-dependent checks fail within the snapshot rather than discovering
the primary repository. This is not an OS sandbox or dependency attestation.

One real Codex mission produced v3 evidence and then passed native Tauri
Review/Apply. Its three primary file hashes matched the frozen targets and four
post-Apply tests passed. The record is
`tooling/benchmarks/results/beta-single-codex-2026-09-06.json`.
The original two tests below document why the boundary was necessary; they are
not the current implementation verdict.

The final independent CLI review found three additional boundary defects:
Desktop refresh could block native approval handling, refresh could clear a
live-event evidence gap, and candidate approvals used an actor key with no
persisted agent row while audit errors were ignored. Refresh now runs off the
native event thread, rejects evidence-gap runs, and uses the originating
completed actor for candidate checks. Approval decisions are published only
after their audit write succeeds; failed writes leave the request pending.
A Galaxy regression injects an audit-write failure, confirms no authorization
escapes, and then confirms the same request succeeds with a persisted audit.
Follow-up static review found no additional production defects, but identified
a test-home isolation omission that was corrected. These changes preserve the
originating permission ceiling and single execution-domain scope.

**Finding:** independently passing task checks do not establish that the exact
combined review package passes. This is a behavioral verification gap, separate
from the existing immutable-byte Apply guarantees.

## Observed evidence

The reviewed path in [runner/run.rs](../../crates/pytxo-runner/src/run.rs)
executes `task.verify` in that task's mutable workspace, then emits `verify-ok`.
[Orchestration](../../crates/pytxo-orchestrate/src/lib.rs) calls
`prepare_review_package` after all successful tasks. Preparation in
[change_set.rs](../../crates/pytxo-runner/src/change_set.rs) composes deltas and
freezes blobs without rerunning checks. The inspected v2
[`PreparedRunManifest`](../../crates/pytxo-core/src/review.rs) contains file
identity, but no behavioral-check receipt.

Two executable negative controls live in
[`candidate_verification_scope.rs`](../../crates/pytxo-runner/tests/candidate_verification_scope.rs):

- Base has `a=0,b=0`; task A changes only `a=1`, task B only `b=1`.
  The real shell check rejects `a=1,b=1`. Both isolated task checks exit zero;
  preparation and exact-byte Apply succeed; the combined check exits one.
- A workspace passes with `a=1,b=0`, then `b` changes before preparation.
  Packaging accepts those later bytes. Re-preparation of the same run replaces
  package identity without rerunning the historical check.

`cargo test -p pytxo-runner --test candidate_verification_scope` passed **2/2**
on Windows on 2026-09-05. Passing these tests confirms the negative controls;
it does **not** mean combined verification is implemented. Both use temporary
repositories and no paid agents.

`refresh_run_review` rebuilds a package from retained workspaces without running
checks. `verify_manifest_preimages` examines changed paths only: an unchanged
test, configuration or source dependency can drift without invalidating those
preimages. A later successful verifier command can also change source after an
earlier check; packaging then freezes the unchecked result.

The original `agentState(completed)` UI mapping asserted “Verify passed” and a
verified tone without check evidence. That assertion must not return. Task
checks, process completion and combined-candidate checks need separate labels;
historical task events cannot attest refreshed package bytes.

## Bounded implementation recommendation

Permission scope is each originating task's **effective** profile, including
trust ceilings. Domain scope is one repository root. Reject unsupported cloud
verification; never upgrade a verifier's permissions during composition.

1. Freeze the review blobs. Snapshot the complete declared repository input
   inventory, including unchanged source/configuration and file modes. Copy
   those exact inputs into a dedicated isolated candidate workspace, then
   materialize additions, modifications and deletions from immutable blobs.
   Reject symlink escapes and a base that changes during copying.
2. Run the ordered task-check recipe there through the existing bounded,
   cancellable verifier lifecycle. Record command, originating task/profile,
   exit/result, boundary receipt and input inventory identity. Empty required
   recipes are unverified. Compare source inventories after every command;
   reject edits, additions, deletions or mode changes instead of silently
   freezing verifier output as verified source.
3. Persist successful candidate evidence inside a versioned manifest whose
   package digest covers the recipe, inventory and result. Avoid a circular
   package-digest field inside that receipt. Keep v2 packages readable as
   unverified; explicit refresh must run the entire candidate check again.
4. Before Apply, require matching evidence and recheck the complete bound base
   inventory under the domain mutation lease. After Apply, compare observed
   bound post-state. External-editor races can still occur: report drift and
   use existing honest recovery rather than claiming universal atomicity.

Use one new `candidate_verification.rs` module for inventory/materialization
and receipt coordination, with a small `run.rs` adapter to the existing verifier.
Core manifest types, package digest/loading, orchestrator initial/refresh paths
and store settlement require coordinated edits. No new scheduler is needed.

Explicitly record exclusions. Build outputs may be excluded, but dependency
trees, toolchains, environment and remote responses are not thereby verified.
Do not call this a full reproducible-build or hostile-code containment proof.
