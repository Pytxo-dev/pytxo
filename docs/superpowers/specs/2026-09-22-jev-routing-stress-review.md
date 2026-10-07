---
title: Jev routing stress review and implementation change map
slug: 2026-09-22-jev-routing-stress-review
status: draft
tags: [review, routing, evidence, architecture]
audience: [human, agent]
layer: orchestration
created: 2026-09-22
updated: 2026-09-22
related: ["[[2026-09-22-jev-routing-design]]", "[[2026-09-22-jev-routing-contracts]]", "[[2026-09-22-jev-routing-rollout]]"]
---

# Stress review: integration risks in the first draft

Reviewed the current dirty working tree again at HEAD 72879702f90f2b74eece888bb117df59608f9b56 on September 22, 2026. Line references below describe that snapshot, not HEAD alone. These are design/integration findings; no runtime or deployment vulnerability is claimed from source inspection alone.

In the findings table, core/, runner/, orchestrate/ and store/ abbreviate crates/pytxo-core/src/, crates/pytxo-runner/src/, crates/pytxo-orchestrate/src/ and crates/pytxo-store/src/. Service paths are repository-relative.

## Findings and resolutions

| Severity | Evidence in current source | Failure if the first draft were implemented literally | Required resolution |
|---|---|---|---|
| P1 | runner/run.rs:1120, :1144 | Signal can relaunch within run_one_agent, reusing the task workspace and replacing the result. An outer two-attempt policy could spawn four workers and hide earlier usage/outcomes. | Routed single-attempt API disables the hidden retry; an explicit second attempt may request richer context. Preserve manual semantics. |
| P1 | orchestrate/lib.rs:1945–2007 | Any failed AgentRunResult marks the run failed; all results become candidate workspaces when successful. Returning historical attempts either poisons recovery success or duplicates task ownership. | Separate complete attempt history from one final TaskResolution per task. Candidate input is the winner projection only. |
| P1 | core/coordinator.rs:148; core/billing/router.rs:187; orchestrate/flow.rs:643 | Component-list membership is not compatibility. Flow builds a shared harness command; ModelRouter can silently default to Claude Opus. Changing metadata would not prove the selected model/harness actually launched. | Immutable recipe plus binding/observation; adapter-owned exact launch; routed misses are errors. No component Cartesian product or default fallback. |
| P1 | orchestrate/lib.rs:1797–1837; store/schema.rs:10–28 | Agent text events can be lost and are only agent-keyed. Reusing them as control events allows missing/duplicate/out-of-order events to corrupt attempt state. | Dedicated typed control journal plus conditional task/attempt transactions. Existing text events remain observations. |
| P1 | runner/process_registry_file.rs:9–25; orchestrate/lib.rs:2615–2676 | Process ownership lives in a separately locked JSON file. Startup reconciliation currently fails a dead-supervisor run. A naive SQLite outbox or terminal run label cannot prove no child was spawned or remains alive. | Mode-aware reconciliation, stable attempt/process start identities and recovery-required states. No blind respawn or lease expiry while ownership is uncertain. |
| P1 | services/pytxo-proxy/src/main.rs:295–345, :110–146; link/auth.rs:13–46 | Existing paid-proxy authorization and bearer/process-local limits do not establish a unique subsidized account or durable budget. With Link unset, a nonempty bearer can pass the proxy check. | Dedicated routing auth and Link-owned atomic account/global reservations; no shared/dev auth identities, tier reuse or bearer-keyed quota. |
| P1 | runner/run.rs:235, :266–381; runner/change_set.rs:1021, :1777 | Successful dependencies are carried as workspace paths in memory. Failed repair bytes, mutable prerequisites and repeated task IDs have different semantics from a single immutable winner. | Freeze dependency/repair manifests; immutable winners; receiving preimage checks and fresh sandbox; preserve existing composition conflict predicates. |
| P2 | orchestrate/flow.rs:734–756 | Desktop Beta explicitly requires Codex, one worker, PTY and Orbit. The first draft's two-worker/cross-harness/Galaxy v1 understated this gate. | V1a preserves it. V1b separately qualifies and deliberately updates the blocker when approved. |
| P2 | runner/race.rs:27; store/catalog.rs:15–62 | Race Shield is in-memory path coordination, and the catalog currently registers domains/fleets. Neither is a durable account/GPU semaphore across processes. | Small host catalog reservation primitive with reconciliation; retain current DAG waves. No distributed scheduler. |
| P2 | First draft's service and benchmark sections | A context limit was treated as a financial ceiling; 24 tasks were paired with a two-point quality gate they could rarely establish. | Explicit billing-bound activation gate, no automatic upstream retries, conservative unknown usage, and a separate fixed held-out benchmark with power disclosure. |

## Required changes versus preserved behavior

Paths below are relative to crates/ unless prefixed services/ or apps/.

| Surface | Required change | Preserve |
|---|---|---|
| pytxo-core | Add profile/binding/observation, TaskContract, decision and typed lifecycle records. RouteProposal can remain a draft compatibility input. | PermissionProfile, execution-domain identity, existing provider transport/billing routing roles. |
| pytxo-orchestrate/flow.rs | Bind reviewed plan to allowed exact recipes and task check contracts; add experimental routing mode. | Current review/freshness/permission checks and V1a beta restrictions. |
| pytxo-orchestrate/lib.rs | Delegate routed work to one controller; persist typed transitions; reconcile routed runs; prepare candidates from final task winners. | Legacy execute_plan path, stop mechanics, independent combined verification and Apply predicates. |
| pytxo-runner/run.rs | Extract/reuse one-attempt execution, exact launch descriptor, typed observations, explicit no-hidden-retry mode and quiescence evidence. | PTY/subprocess mechanisms, Signal read-context preparation, cancellation identity checks, existing legacy behavior. |
| pytxo-runner/change_set.rs and context.rs | Reuse safe inventory/composition helpers through frozen handoff/dependency inputs; provide a winner projection at the boundary. | Path/preimage/conflict rules and exact candidate freezing. Do not weaken checks to accept new routing. |
| pytxo-store | Add domain task/attempt/decision/control records and host catalog capacity reservations; distinct migrations and reconciliation. | Per-domain WAL separation, run/review history and immutable prepared-package identity. |
| services/pytxo-link and pytxo-proxy | Dedicated scoped auth, transactional sponsorship and fixed Jev adapter endpoint. | Existing paid worker inference, commerce, wallet semantics and customer credentials. |
| apps/desktop | Opt-in packet preview, Manual/Balanced control, attempt history, exclusions/fallback/usage provenance. | Work/History/Setup, Review → Apply, existing evidence language and visual direction. |

**No initial semantic rewrite:** scheduler DAG/wave algorithm, generative planner, permission engine, sanitizer, vendor authentication ownership, billing ModelRouter, Apply/rollback journal and accepted ADRs. Integration call sites and regression tests may need changes, but these authorities are not delegated to Jev.

## Complexity removed

- Split profile recipe from volatile account/readiness/mission authority rather than growing one universal profile object.
- One semantic Choice in V1a. No dynamic skills, runtime DAG edits, Jev ready-queue ranking, failure diagnosis, multi-agent supervisor or auto-installer.
- Existing wave barrier instead of an opportunistic scheduler. One local capacity reservation primitive, not a service mesh or cross-host lease system.
- Local JSON/blobs instead of a generalized vendor-transcript converter or package format.
- Existing proxy/Link modules with dedicated routing policy instead of a new standalone public routing platform.
- Zero upstream retries in the first sponsored service, and no prompt queue or replay-based event-sourcing rewrite.
- Manual profile qualification and a small packet-value gate before runtime refactoring; a useless semantic signal should not trigger a platform build.

## Failure drills that must pass before activation

Inject Stop after advice, after resource reservation, between spawn and PID receipt, and during verification. Crash after each durable transition. Duplicate worker exit and advisor replies. Reuse a PID with a different start identity. Change a profile executable/config between selection and launch. Leave a descendant writing after its parent exits. Attempt dependency consumption before verification. Corrupt a handoff blob, add a Windows case collision, or modify receiving instructions. Rotate account tokens during quota exhaustion and replay an expired request.

For every case, assert the specific final state, number of actual worker starts, held/released reservations, complete cost provenance and whether any candidate is eligible. Also prove positive recovery/progress; a system that blocks every run is not acceptable.

The known limitations remain explicit: same-user filesystem access is not a tamper-proof boundary, external vendor usage is outside local lease control, redaction is not anonymity, finite tests are not universal correctness, and Jev's superiority is unmeasured.

## Review validation

Documentation checks cover frontmatter, wikilink targets, paired code fences, whitespace and unresolved placeholders. Forty named existing source/test paths were verified; sixteen inspected implementation files retained identical SHA-256 hashes before and after this pass. Budget arithmetic and benchmark run counts were checked. No product source was edited, no test suite or benchmark was run, and no inference request or external write was made. Existing unrelated working-tree changes were preserved.
