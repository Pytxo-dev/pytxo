---
title: Beta core architecture and reliability audit
slug: beta-core-audit-2026-09-05
status: active
tags: [project, beta, audit, reliability, orchestration]
audience: [human, agent]
layer: orchestration
created: 2026-09-05
updated: 2026-09-05
related: [[release-audit-core]], [[commit-layer]], [[permission-profile-engine]], [[execution-domains]], [[beta-ux-audit-2026-09-05]]
---

# Beta core architecture and reliability audit

## Evidence boundary

The August 31 [[release-audit-core]] is historical input, not the current
verdict. Current source already contains typed dependency outcomes, verifier
environment/network boundaries and timeouts, durable startup reservations,
active-run locks and reconciliation, staged Flow revalidation, and transactional
migrations. Repeating the old ten-blocker list would misrepresent this checkout.

This review covers local execution in each permission profile (DeepSpace, Orbit,
Galaxy, Supernova), with process state isolated by the owning execution domain's
data directory. The exercised fixtures use Orbit in a temporary Windows Git
repository. No cross-domain, cloud, macOS, or Linux enforcement proof is inferred
from those fixtures. Cloud verification continues to fail closed.

## Fixed: Stop did not own verification

Before this change, `run_one_agent` called blocking verification directly on its
async worker. Verifiers were never entered into the durable process registry.
Consequently Stop could target an already-exited agent while its verifier kept
running; on a single-thread Tokio runtime, even the task handling Stop could be
starved. This was established from the verifier spawn and registry paths in
`crates/pytxo-runner/src/run.rs` and orchestration's `stop_impl`.

Verification now runs through `spawn_blocking`, records its PID and OS creation
identity, and checks durable cancellation before commands and during execution.
The existing process-registry document gains a backward-compatible
`cancelled_runs` field. Stop records cancellation under the same lock used for
spawn registration, preventing subsequent task/retry/verifier registrations from
escaping a stopped run. Failed termination retains both cancellation and process
evidence; it does not report success. Very short checks retain their observed
exit instead of failing merely because the process exited before registration.

Output capture now includes pipe completion in the verification deadline. A
descendant retaining a pipe can no longer make an unbounded `JoinHandle::join`
hold the runner forever. Missing output or deadline expiry returns an error.

Verified commands:

- `cargo test -p pytxo-runner --lib verification_boundary_tests -- --nocapture`:
  **9 passed**, covering durable live Stop, pre-spawn cancellation, failed-kill
  evidence retention, fast checks, output deadlines, timeout, secret filtering,
  egress gating, and cloud refusal.
- `cargo test -p pytxo-runner --test dependency_outcomes -- --nocapture`:
  **4 passed**, including real Stop during verification on a single-thread Tokio
  runtime, failed verification blocking dependents, and sibling survival after
  startup failure.

## Remaining limits and release decisions

Sep 6 resolution: items 1 and 4 below are fixed by v3 combined verification and
orchestrated Stop-all settlement. Item 3 is also fixed: agent rows are created on
their first event, before SQLite's event foreign key is used. Failed event writes
are counted; settlement records an `evidence-gap` and refuses Apply-ready status.
The integration test injects a real SQLite rejection of `verify-boundary`, checks
the visible failure contract, and verifies normal boundary events persist. Final
workspace and packaging checks are being rerun after this bookkeeping correction.
Item 2 remains a stated platform limitation; no universal containment is claimed.

1. **Task checks are not combined-candidate verification.** Checks execute in
   individual workspaces before `prepare_review_package` composes changes.
   `PreparedRunManifest` binds file bytes, not a test result for the assembled
   candidate. Two separately passing tasks may interact badly after composition.
   The UI and demo must call this task-check evidence until a digest-bound
   candidate verification pass is implemented and exercised.
2. **Detached descendants need stronger supervision proof.** Windows Stop uses
   `taskkill /T`; Unix enumerates descendants. Neither proves that an already
   orphaned or deliberately detached descendant was stopped. Bounded output
   returns an honest error but does not establish universal process containment.
   Add actual orphan/pipe inheritance fixtures before claiming this boundary.
3. **Event evidence can still be lost.** The orchestration event callback ignores
   `append_event` errors and inserts agent rows after execution. Failures must
   eventually surface as evidence gaps rather than silently missing receipts.
4. **Stop-all requires orchestration settlement.** Runner cancellation covers
   tracked identities. The orchestration facade must also cancel an active run
   between child processes and settle its ledger before removing its marker.

Reject generic effect adapters, universal recovery, and automatic skill expertise
as Beta promises. The defensible near-term workflow is one selected harness,
inspectable bounded tasks, successful dependency output, explicit task checks,
an exact reviewed package, and Apply with honest recovery evidence. Broad Beta
readiness still requires live harness and distribution evidence beyond this
focused core audit.
