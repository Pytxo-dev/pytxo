---
title: Pytxo canonical context
slug: pytxo-canonical-context
status: active
tags: [context, audit, pytxo]
audience: [human, agent]
layer: meta
created: 2026-09-16
updated: 2026-09-16
related: ["[[PYTXO_CURRENT_STATE]]", "[[PYTXO_DECISIONS]]", "[[PYTXO_ARCHITECTURE]]", "[[PYTXO_SOURCE_LEDGER]]"]
---

# Pytxo canonical context

Audit snapshot: **2026-09-16**, `C:\pytxo`, branch `codex/beta-candidate-verification`, HEAD `72879702f90f2b74eece888bb117df59608f9b56`, plus substantial inherited dirty/untracked work. This is a reconciled entry point, not a timeless release certificate. Current implementation means the inspected working tree, including uncommitted code; it does not mean published, installed, or independently secure.

## Product

Pytxo is a vendor-neutral local control plane for delegated coding work. It prepares bounded missions, schedules existing agent processes in separate workspaces, assembles their repository changes, runs checks on the combined candidate, presents one result for human review, and applies a stored package through a guarded, journaled repository boundary. Its intended users are solo maintainers and small teams already using coding agents who find supervision, integration, and review burdensome. The current wedge is **useful delegation with an inspectable repository trust boundary**.

The value proposition is fewer manual handoffs and a coherent, checkable result—not simply more agents or another permission popup. One harness and one worker remain useful: scope, separate working state, combined verification, drift detection, review, controlled Apply, and durable history do not require a swarm. Flow currently selects one harness for its worker instances; lower-level configuration can describe different agents. That is not proof of polished heterogeneous automatic routing.

Pytxo is not a replacement coding model, a syscall sandbox, a generic terminal grid, a universal undo engine, or a production-effects gateway. “Agent hypervisor” is product language for its control responsibilities, not a hardware virtualization guarantee. General effect contracts, learned routing, a plugin marketplace, and cheap-capacity swarms remain designs or experiments.

## Why this thesis exists

The supplied research argued that permissions, branches, PR review, CI, and agent-native controls already cover much of generic approval. It refined the thesis toward independent enforcement of what can enter a repository, then acknowledged that a late repository gate cannot undo arbitrary earlier host or network effects. The first-user discussion broadened the product outcome: help complete useful delegated work, retaining exact Review → Apply as a differentiator.

This evolution is supported by the handoffs, but their absolute ordering is incomplete. Review research and first-user handoffs have no explicit conversation dates; their internal sequences are clearer than their cross-file chronology. September 15–16 cheap-harness discussion re-emphasizes stale-Apply refusal as a demonstration without reversing the need for successful work. See [[PYTXO_HISTORY]] and contradiction records in [[PYTXO_DECISIONS]].

## Core workflow

The source-supported path is:

1. Choose a repository and a supported harness; inspect detection/authentication limitations and folder trust.
2. Describe bounded work; local heuristic/Signal planning or explicitly enabled model planning proposes tasks, paths, dependencies, and checks.
3. Review the plan. Deterministic scheduling preserves tasks while bounding concurrent workers.
4. Dispatch through orchestration into isolated workspaces using PTY by default, with subprocess fallback. Worker completion and worker checks alone do not establish integrated correctness.
5. Prepare stored blobs and a manifest. For nonempty reviewed changes, build a combined source snapshot, run every task's required recipe against it, and reject included-source mutation or changed base inputs.
6. Desktop Review displays prepared content, checks, limitations, and package identity.
7. Apply validates the current stored contract/package, recipe, execution domain, and repository inputs, then journals filesystem writes and verifies the resulting inventory. Reject/discard and explicit refresh/re-verification are separate actions.
8. Persist result/receipt; reconcile interrupted Apply conservatively.

Sources: `crates/pytxo-orchestrate/src/{flow.rs,lib.rs}`, `crates/pytxo-runner/src/{candidate_verification.rs,change_set.rs,run.rs}`, Desktop `RunReviewScreen.svelte`, `src-tauri/src/ipc.rs`, and `pytxo-store/src/store.rs`.

**September 16 implementation update:** A1 was reproduced with a legitimate same-run refresh. Apply now requires the displayed package digest and compares it in Core under the domain mutation lease before new authorization. Stale/missing identities fail closed without invalidating the replacement candidate. Automated Core, IPC and two-client browser regressions pass; exact-build native/package acceptance remains **BLOCKED**, not proven. See [[PYTXO_CURRENT_STATE]] and the root RELEASE_READINESS ledger.

The full install → real task → reviewed result → Apply → recovery journey on one frozen current release artifact remains an acceptance goal. Historical successful native runs are evidence for their specific older artifacts, not today's entire tree.

## Trust boundary and critical invariants

“Core” here means the deterministic Rust control path across Core, planner validation, scheduler, runner, orchestration, and store—not solely the `pytxo-core` crate.

| Invariant | Current support / limit |
|---|---|
| Probabilistic systems suggest; deterministic mechanisms own transitions | Planner suggestions enter existing control paths. Coordinator configuration exists; route proposal validation is only a foundation. |
| Worker self-report is not verification | Separate combined-candidate checks exist. A passing recipe proves its scope, not all behavior or semantics. |
| Checks cover the exact candidate | Version-3 package records base/candidate inventories and ordered checks; mutation detection exists. Empty/no-change packages are a special case. |
| Review authorization binds to Apply state | Caller-reviewed digest is required and compared under Core's mutation lease; same-run stale-client regressions pass. Exact native/package proof pending. |
| Relevant drift invalidates evidence | Current implementation compares the whole included base inventory, not a semantic dependency model. Exclusions narrow coverage. |
| Advisory must not appear enforced | Per-surface receipts exist. Orbit/Galaxy host filesystem and arbitrary network access remain advisory. |
| Runtime/model/provider/plugin cannot take Core authority | Current proposal types omit approvals/Apply authority; plugin runtime containment is not implemented. Local store is not hostile-worker tamper-proof. |
| Worker limit bounds concurrency, not task count | Planner and wave scheduling preserve tasks; focused concurrency test exists. |
| Preserve operator/unrelated state | Preimages, frozen-target refresh, journal rollback and post-crash drift refusal exist. No universal concurrent-writer guarantee. |
| Recovery fails safely under uncertainty | Missing/corrupt journal state can require manual recovery; never infer rollback merely from failure. |
| Local work should not require Pytxo Cloud | Local planning/execution and accountless BYOK paths exist; selected harness/provider may require internet. Local inference is not automatically offline workflow. |
| Payment providers do not define capabilities | Dirty Link adapters normalize into Pytxo-owned grants and tier projection. Runtime migration/lifecycle proof remains absent. |
| Generated UI is not specification | Historical concepts contain unsupported ETA, counts, and enforcement. Use current semantics. |
| Historical context is not implementation evidence | Source plus named current checks establishes this snapshot; reports remain dated evidence. |
| Complement native sandboxing | Reviewed Apply mediates repository writes through Pytxo. It cannot prevent every external effect or direct host access by same-user processes. |

Reviewed run Apply supports **Orbit/Galaxy, one execution domain, one repository root**. DeepSpace is non-flushable and requires an available, probed network mechanism. Supernova is intentionally host-direct and bypasses deferred Apply. “Workspace enforced” means prepared separate workspace and mediated integration, not an OS-wide filesystem jail. The process-scoped mutation lease is not cross-system atomicity or protection from every external writer.

## Current product status

Source is v1.2.2; web source declares public v1.2.1. That public declaration and historical release observations were not refreshed online during this audit. Real scheduling, isolated execution, candidate verification, prepared-content review, guarded Apply, journal recovery, Stop, and SQLite history exist.

The working tree also contains uncommitted provider-neutral commerce, Dodo ingress, explicit coordinator selection, and a larger harness catalog. These are not all released or runtime-validated. Full routing, generic RuntimeAdapter/CapacityPool systems, plugin management, and the newly proposed operational topology are not established implementations. Legacy 3D topology code does exist.

Readiness remains fragmented across source snapshots and native packages. Clean-Windows current-artifact acceptance, current-source hosted CI, publication/download verification, commerce lifecycle evidence, and real repeat-user value are not established here. A1 has a source fix and automated proof, with native acceptance outstanding. Pricing, quotas, external model claims, and legal/provider status in historical sources are not refreshed facts.

The two validation frameworks complement each other: measure unique catches and added human overhead **alongside** useful-task completion, coordination burden, and voluntary task-two use. Neither internal tests nor building an app proves demand.

Business context stays outside local Core: MBCZ is the intended umbrella identity; Pytxo is a distinct product. Preserve existing Paddle integrations. Dodo website preparation, approval, adapter code, test-mode operation, and live billing are five different states.

## Agent Start Here

1. Read this document, then [[PYTXO_CURRENT_STATE]] for the snapshot and bounded next mission.
2. Inspect current branch, HEAD, worktrees, status, applicable instructions, and checkpoint before editing.
3. Reverify drift-prone claims and distinguish committed source, dirty source, tests, packaged runtime, and publication.
4. Preserve approval, candidate, scope, recovery, and advisory/enforced invariants; preserve A1's reviewed-digest requirement and complete exact-build acceptance before broadening trust claims.
5. Consult [[PYTXO_DECISIONS]] before reviving fan-out, router, topology, marketplace, or cloud proposals.
6. Work only on the currently authorized bounded mission. This audit authorizes no product implementation, deployment, provider changes, or release.
