---
title: Pytxo product and architecture history
slug: pytxo-history
status: active
tags: [context, audit, pytxo]
audience: [human, agent]
layer: meta
created: 2026-09-16
updated: 2026-09-16
related: ["[[PYTXO_DECISIONS]]", "[[PYTXO_SOURCE_LEDGER]]", "[[PYTXO_CANONICAL_CONTEXT]]"]
---

# Pytxo product and architecture history

This is a reconstruction of ideas, not a transcript or an invented chronology. H1–H4 identify supplied handoffs in [[PYTXO_SOURCE_LEDGER]]. H3 explicitly covers September 15–16; H4 September 16. H1/H2 are undated. Repository ADR/commit dates anchor implementation history, not necessarily the dates of conversations. H2's reported work matches a proposed ADR dated September 15. Confidence is higher in each handoff's internal sequence than in a strict ordering of all four.

## 1. Execution yard and coordination primitives

The repository's earlier model contains Signal structural context, Blast separate working state, Race ownership/concurrency, execution domains, and permission profiles. These are the substrate on which later product framing rests, not proof of every “moat” claim. The earlier Reality Deck/3D presentation survives in a development-only legacy shell.

**Previous idea:** headless agent execution and coordination differentiated through named controls.
**Problem:** primitives and impressive visualization do not by themselves explain a user's completed job or demonstrate unique value.
**Surviving direction:** use them to support bounded work and inspectable integration. Do not imply the entire product originated only with the supplied research; these summaries are incomplete history.

## 2. Review/approval research narrows the differentiated boundary

H1 asks whether developers need a separate review layer at all. Its reported research includes native permissions/sandboxes, Git branches/PR review, CI and automated review as substitutes. It also reports failure incidents, but does not retain enough original sources to treat incidents or statistics as reverified facts.

**Previous idea:** another review/approval layer.
**Problem:** existing workflows already ask for permission and review; repeated approvals may become friction with little information.
**New idea:** an independent boundary that captures resulting state and evidence and controls what enters the repository.
**Why it changed:** separation of the worker's output claim from the mechanism deciding whether it may land.
**Still survives:** yes as a key trust primitive. Generic approval-popup positioning was rejected; review itself was not.

H1 immediately identifies the limit: host-file deletion, production-data mutation or network effects may already have happened before repository Apply. Native sandboxing remains necessary. This is a scope correction, not proof that Pytxo implements all external side-effect governance.

## 3. Immutable reviewed work and generalized effect ambitions diverge

Repository ADR-0034 (accepted August 1) records immutable review packages and durable Apply. ADR-0036 (proposed August 27) broadens the concept toward typed effect contracts, post-state verification and honest compensation. These dates establish documented architecture milestones; they do not date H1.

**Previous idea:** conceptual exact-state approval.
**Problem:** mutable workspaces, stale inputs, ambiguous recovery and worker-local checks undermine a trustworthy boundary.
**Implemented direction:** stored blobs/manifests, candidate identity, independent combined checks, preimage/base verification, journaling and conservative recovery.
**Future direction:** generalized production effects, semantic adapters and compensation classes.
**Still survives:** repository specialization is implemented; generalized contracts remain proposed. A1 in [[PYTXO_CURRENT_STATE]] qualifies the current user-approval binding claim.

Commits `f21acfb` and `a4f5828` on September 6, followed by `2964d5a`, `402c599`, `f64a0b0` on September 7 and `eb5f73d` on September 8, record candidate evidence, audited refresh/approvals, scoped execution, Stop-lock/lifecycle and Windows runtime hardening. Current source/tests, not those commit subjects alone, support the detailed implementation account.

## 4. Useful delegation broadens the product without discarding Apply

H2 explicitly starts with a commit/trust-layer framing, then argues that safely refusing work is too narrow. The stronger user outcome is handing off a task and returning to one coherent result with checks and unresolved problems visible.

**Previous idea:** “mainly the commit layer”; aha is catching an agent falsely saying done.
**Problem:** a pure gate may add supervision rather than reduce it.
**New idea:** coordination/delegation as the product outcome, exact Review → Apply as the trust boundary.
**Why it changed:** the product must help complete useful real work, not merely block errors.
**Still survives:** September 7 repository vision calls Pytxo an agent hypervisor and the commit boundary a primitive. This convergence does not establish H2 caused that dated edit.

H3 later returns to stale-state refusal as a five-minute demo. That is tension about emphasis, not evidence the useful-work outcome was abandoned. A coherent demonstration can show successful delegation, meaningful refusal, re-verification and successful Apply.

## 5. UI moves from internal machinery to the user's mission

The current Work/History/Setup shell, mission dock, prepared-content review and progressively disclosed evidence reflect a move away from a terminal/grid/topology-led product. September 13 commit `7287970` preserves structural UX; later dirty source contains first-use, scroll, catalog and identity work.

**Previous idea:** operators reconstruct work from runs, terminals, hashes and internal control labels.
**Problem:** those details obscure requested outcome, next action and readiness.
**New idea:** mission first; one integrated review; technical proof remains accessible.
**Still survives:** current components and fresh type/style checks support source state. Older browser fixtures and narrow native screenshots do not establish current complete runtime acceptance.

A September 13 report records a cross-run UI bug: header changed while package content remained from another run. Keying Review by domain/run fixed that reported fixture. This should not be confused with A1's separate same-run stale-package authorization concern.

## 6. Single-agent-first, cheap harnesses and restrained scaling

H2 rejects “running lots of agents” as the whole product. H3 explores fragmented cheap inference and many CLIs, then corrects two assumptions: inference is not the same as a harness, and different harnesses can share one provider/account quota.

**Previous idea:** cheap/free capacity makes a large swarm compelling.
**Problem:** dependencies, conflicts, integration checks, quotas and human review can dominate throughput.
**New idea:** keep one-worker value, test up to four, scale only if verified useful work improves.
**Still survives:** explicit plans/waves and single-harness Flow; 100 workers are deferred research, not release scope.

The inspected dirty source already recognizes 14 harnesses. Some handoff suggestions to “add a cheap runtime” therefore need to distinguish existing command wiring from missing runtime validation. Recognition/command wiring is not authenticated successful execution. Qwen remains detection-only.

## 7. Coordinator and routing get explicit authority limits

H2/H3 favor deterministic resolution first, optional model advice for ambiguity, deliberate escalation if justified. DeepSeek is an economical candidate in historical discussions, not an architectural requirement.

**Previous implementation described by ADR-0041:** ambient provider keys could influence model planner transport and managed selection was hardcoded.
**Problem:** accidental egress/provider dependence and a risk of duplicating model-owned control authority.
**New implementation:** explicit independent coordinator provider/model/transport, local selection precedence and model opt-in; typed route proposals omit permission/approval/Apply authority.
**Still survives:** dirty source and current tests support these seams. ADR-0041 remains proposed.

Routing should eventually select compatible execution profiles—harness, model, skills/tools/plugins, environment, policy and checks. The actual new validator checks membership against supplied lists. No operational eligible-catalog producer or autonomous rank/dispatch pipeline was found. Existing worker billing/transport routing must not be relabeled as that future system.

## 8. Local models and plugins become directions, not proven flows

H2 elevates local inference and optional Cloud. Local-only work must not disclose repository context to a cloud coordinator merely to choose a local route.

**Problem:** provider lock-in/privacy and future integration growth.
**New direction:** replaceable local/cloud endpoints and extensions that contribute capabilities without taking Core authority.
**Still survives:** local endpoint resolution/config tests exist; complete local-worker/offline operation is not shown. Plugin manager/SDK/runtime containment is not found.

There is an unresolved scope tension: H2 says plugin foundations belong in Beta, while the bounded-release direction defers broad platform plumbing. Full marketplace deferral is clear; minimum Beta plugin scope is not. Preserve that question rather than silently resolving it.

## 9. Topology exploration rejects visual spectacle as the product

In H3, a generated cyberpunk city was explicitly rejected. A literal giant raw AST was also rejected as the main surface because of noise/occlusion and weak operational value.

**New direction proposed:** restrained 2D/2.5D topology, Tasks and Code modes, semantic zoom, ownership/waiting/change evidence and drill-down.
**Why:** explain real operator questions; graph proximity cannot authorize execution or Apply.
**Still survives:** proposal only for the new map. Legacy 3D source exists. Generated revamps with unsupported ETA, counts, ready-state or green isolation are explicitly not specifications.

## 10. Commerce becomes provider-neutral while external status stays separate

H2's pasted Sol report describes provider-neutral commerce, Dodo ingress, preserved Paddle, explicit coordinator configuration and migration safety. Most named files now exist as dirty/untracked source.

**Previous idea:** provider-specific subscription state drives access.
**Problem:** provider metadata, replay/order changes and cancellation must not redefine product capabilities or revoke another valid provider grant.
**New architecture:** normalized events → independent provider grants → Pytxo-owned entitlement projection, separate from local mission state.
**Still survives:** current unit/route tests pass; actual Postgres migration and provider lifecycle remain unverified.

H4 corrects a separate portfolio redesign toward a portfolio **and** commercial surface. Stonkz's Paddle flow must remain; Dodo verification is not a global migration or live-billing claim. ProdVerdict is explicitly discontinued; other product status is not inferred. MBCZ umbrella/Pytxo brand structure is intent; “verified owner/approved brand” prose in proposed docs is not external account evidence.

## 11. Release and first-user strategy favor evidence over breadth

H2 distinguishes release-complete from vision-complete and proposes Founder Alpha → Private Alpha → Closed Beta → Public Beta → 1.0. Those labels are a suggested progression, not verified milestone attainment.

Older records establish successful native jobs and exact Apply for specific builds. Later source, UI packages and commerce changes complicate that evidence. September 12 release prose supersedes the blanket “CI billing blocks everything” claim, but only for its observed committed head. Current uncommitted source has no new hosted run here. September 13 MSI and September 15 narrow visual EXE are not one accepted final release.

**Surviving release principle:** a stranger can complete the important golden path on the actual distribution artifact; polished screenshots and large test counts cannot replace it.

## Research and benchmark families

| Family | Question | Evidence discipline / current status |
|---|---|---|
| SafetyBench | Do deterministic state transitions reject invalid work and allow valid work? | Include drift, candidate replacement, mutation, old decisions, Stop, crash/recovery and liveness. Named new suite proposed; overlapping current regression tests exist. |
| Agent/Pytxo ablation | What does Pytxo add beyond the same competent harness/model? | Same starting state, task, tool access, acceptance tests, effort and human-assistance policy. Existing observations are confounded, not a measured superiority result. |
| Coordinator experiment | Does advice improve outcomes with workers held fixed? | Separate coordinator resources/failures from worker quality; no validated comparative result here. |
| Routing experiment | Does eligible execution-profile selection improve accepted work? | Compare valid routes and explain exclusions; full router not present. |
| SwarmBench | Does useful verified output scale with workers? | Begin 1→4, retain failures/conflicts/review overhead; 100 is not a goal by itself. Proposed. |
| Business Gauntlet | Can the system perform a broad integration/content challenge? | Resource envelope, not dollar-only rigor. Building software is not customer demand. Proposed. |

H1 proposes 5–10 users and unique unsafe/out-of-scope catches with roughly under 10% added active-human time. H2 proposes five completions and three voluntary returns; H3 proposes five users with two real tasks. These are complementary learning designs, not accepted thresholds or observed outcomes. Measure both risk reduction beyond native/Git/CI controls and whether useful work becomes easier enough to repeat. Count all attempts, intervention time, actual spend, normalized inference cost and local compute separately. Hybrid frontier rescue is not a cheap-model-only success.

## Contradictions retained

The full Position A / Position B / chronology / source / interpretation records are C1–C10 in [[PYTXO_DECISIONS]]. Particularly unresolved: plugin foundations as a Beta gate, which first-session story leads, exact public positioning, and current external release/provider state. The historical thesis supports a refinement toward useful delegation with independent integration control; it does not support universal side-effect enforcement, proven demand, or a completed autonomous routing platform.
