---
title: Pytxo Jev routing implementation work packages
slug: 2026-09-22-jev-routing-rollout
status: draft
tags: [plan, routing, experiments, verification]
audience: [human, agent]
layer: orchestration
created: 2026-09-22
updated: 2026-09-22
related: ["[[2026-09-22-jev-routing-design]]", "[[2026-09-22-jev-routing-contracts]]", "[[2026-09-22-jev-routing-stress-review]]", "[[2026-09-22-jev-routing-benchmark]]"]
---

# Implementation work packages

**Goal:** determine whether a replaceable Jev advisor improves task routing while Core retains all execution and Apply authority.

**Architecture:** Core records and pure policy; one orchestration controller; one-attempt runner; immutable outputs/handoffs; optional dedicated proxy/Link service. Rust/Tokio/SQLite, current PTY infrastructure, existing Svelte/Tauri UI.

This is the requested implementation-ready plan, not authorization to code, spend, contact a vendor, commit or deploy. The source-backed change map is [[2026-09-22-jev-routing-stress-review]]. One implementation worker owns the changes unless the user later asks for delegation. Preserve unrelated dirty work.

## Fixed boundaries

V1a: one repository/domain, Codex, Orbit, local PTY, one worker; two qualified model/settings recipes, fixed approved tools/skills. Core controls the initial attempt plus at most one repair. Jev initially influences only ambiguous initial selection. V1b separately qualifies cross-harness transfer, broader permission profiles, optional local inference and two-worker concurrency.

No run-time DAG mutation, hidden retry, API-billing conversion, automatic provider fallback, new approval authority or synthetic success. The user-selected hosted disclosure is workspace opt-in for a redacted packet. Local rules/manual remains accountless.

Every package includes meaningful failure/progress tests. Commands below are planned; none was run for this documentation task.

## 0. Cheap feasibility gate before runtime refactoring

First identify two actual launchable profiles using the current harness and a disposable manual qualification task. Freeze the proposed packet/template and compare R0 versus Jev demand labels on the 300-packet development/validation/holdout collection under a separately authorized tiny evaluation budget. Existing account ownership and task disclosure still apply.

This can reject an unusable semantic signal before building a controller, service or UI. It cannot prove coding-cost savings: the full end-to-end trial still needs identical attempt accounting and checks. Do not manufacture an orchestration subsystem merely to run packet evaluations. If the profile pair is unavailable, the packet leaks required private context, or the question adds no useful discrimination, stop this proposal at the research stage.

## 1. Add exact profile and policy contracts

**Depends on:** none.

**Existing:** crates/pytxo-core/src/coordinator.rs, task.rs, ade_registry.rs, config.rs, lib.rs and billing/router.rs; crates/pytxo-orchestrate/src/flow.rs.

**New:** crates/pytxo-core/src/routing.rs; crates/pytxo-orchestrate/src/routing/{mod,catalog,policy}.rs; crates/pytxo-orchestrate/tests/routing_policy.rs.

- Add the recipe/binding/observation/task/authorization records from the contract. Validate complete combinations, minimum identity evidence and immutable content.
- Keep ModelRouter's existing transport role; the routed path must produce an explicit exact ModelRoute/launch descriptor without its defaults.
- Implement R0 and the pure optional RJ substitution as defined, with an unevaluated RJ manifest shadow-only.
- Extend reviewed Flow plan metadata with routing mode, approved profiles, check and policy digests. Older plans/runs keep legacy semantics; missing fingerprints cannot silently opt them into routing.
- Pin effective skills/tools before selection. Unknown auth/effective-model support remains explicit.

**Acceptance:** incompatible but individually known model/harness is excluded; detected-only CLI is excluded; changed executable/skill/binding invalidates launch; both eligible and no-eligible cases behave deterministically; manual selection stays pinned. This package need not call a real model.

## 2. Introduce durable task/attempt admission

**Depends on:** 1.

**Existing:** crates/pytxo-store/src/{schema,migrate,store,catalog}.rs; crates/pytxo-orchestrate/src/{lib,hypervisor}.rs; crates/pytxo-runner/src/process_registry_file.rs.

**New:** crates/pytxo-orchestrate/src/routing/{controller,capacity}.rs; crates/pytxo-orchestrate/tests/routing_state.rs.

- Add domain task_state, attempts, route_decisions, control_events and advice_requests with unique constraints and CAS transitions. Allocate migration numbers from the actual checkout when implementing.
- Retain existing active_run ownership; add routed recovery handling and a supervisor generation/fence. An old callback cannot write under a new owner.
- Add a small host catalog resource reservation primitive. V1a uses an exclusive routed-worker host slot; V1b adds qualified account/GPU/external-resource pools. No new daemon.
- Implement provisional host reservation → domain admission → bound launch-token protocol and explicit compensation. Never claim cross-database atomicity.
- Keep process-registry cancellation authoritative during a store failure. Reconcile all unresolved attempts before clearing routed ownership, including terminal legacy run statuses.
- Preserve legacy history without inventing past attempt records or filling unknown usage with zero.

**Acceptance:** duplicate event/request, concurrent admission, Stop at each boundary, same task ordinal twice, two domain contenders, lost owner, missing domain store, stale PID and crash-before/after admission all have asserted final states. Include successful recovery of a provably unlaunched attempt. No real worker required yet.

## 3. Expose one-attempt runner and winner projection

**Depends on:** 1–2.

**Existing:** crates/pytxo-runner/src/{run,process_registry_file,context}.rs; crates/pytxo-orchestrate/src/{lib,flow}.rs.

**New:** crates/pytxo-runner/src/profile.rs; crates/pytxo-orchestrate/tests/routing_attempts.rs.

- Extract reusable one-attempt execution from run_one_agent without creating a second wave controller. Keep execute_plan for legacy callers.
- Lower exact adapter-owned launch arguments to current PTY/subprocess machinery. Preserve prompt transport and minimal child environment.
- Add routed no-hidden-retry mode; keep Signal context preparation. Any richer-context retry uses ordinal 2 in the new controller.
- Bind attempt ID to agent/process key and OS start identity. Disallow unapproved cloud fallback or implicit route resolution.
- Emit typed lifecycle observations and seal/verify exact output before winner publication. Retain raw terminal events only as evidence.
- Adapt final TaskResolution winners to existing AgentWorkspaceInput. All attempts contribute usage/history; only final task failures determine run failure.

**Acceptance:** fake worker invocation counter proves two attempts means at most two launches, including Signal failure cases. A failed first attempt plus successful repair yields one candidate source and complete usage; a final failure blocks candidate preparation. Real PTY/subprocess Stop tests must confirm no later worker/check/dependent launch. An orphan child requires recovery, not respawn.

## 4. Freeze handoff and dependency artifacts

**Depends on:** 3.

**Existing:** crates/pytxo-runner/src/{change_set,context,candidate_verification}.rs and run.rs integration; existing candidate/Apply tests.

**New:** crates/pytxo-runner/src/handoff.rs; crates/pytxo-runner/tests/routing_handoff.rs.

- Implement [[2026-09-22-jev-routing-handoff]] using canonical manifests and local content-addressed blobs.
- Require quiescence evidence; project frozen successful prerequisites and optional failed repair input into a new sandbox.
- Validate paths, preimages, hashes, modes, protected configuration and receiving capabilities. Keep trusted verification recipes separate from worker content.
- Persist winner output identity before dependent admission; never reopen a consumed winner.
- Retain the existing DAG waves and composition conflict rules.

**Acceptance:** same-harness second attempt survives a controller restart without mutable workspace dependence. Corrupt/missing blobs, stale base, duplicate task winners, instruction-file changes, path escapes/case collisions and lingering writers reject safely. Independent tasks progress while failed descendants block.

**V1b gate:** qualify a real second harness and at most two workers before changing Desktop Beta blockers. Demonstrate cross-harness repair and clean restart plus shared resource conflicts. A same-harness test cannot satisfy this gate.

## 5. Add replaceable advisor and offline stress fixtures

**Depends on:** 1–4.

**Existing:** crates/pytxo-sanitize/src/lib.rs; routing controller from earlier packages.

**New:** crates/pytxo-orchestrate/src/routing/{packet,advisor}.rs; crates/pytxo-orchestrate/tests/routing_advisor.rs; tooling/benchmarks/routing/{README.md,corpus.jsonl,profiles.json,policy.json,questions.json,protocol.json}.

- Define provider-neutral AdviceEnvelope and the single fixed initial-demand Choice template.
- Construct an allowlisted hosted packet with local-only/consent enforcement; validate serialized payload, not just individual field sanitizers.
- Enforce two-second caller deadline, immutable request/context binding, zero upstream retry and deterministic fallback.
- Separate model/template/threshold release manifest from authorization. Changes require reevaluation, not silently updated aliases.
- Build local fake-service fixtures for valid, missing, malformed, adversarial, stale, cancelled and overloaded replies. No paid call is needed for authority tests.

**Acceptance:** turning off consent yields zero network calls, including after restart. Late replies cannot launch work. Service outage selects exactly R0 under the same local facts. Advisor cannot affect permissions, graph, checks, billing source or Apply.

## 6. Run the routing comparison before building subsidy scale

**Depends on:** 5 and explicit profile/corpus/spending authorization for real execution.

**New:** tooling/benchmarks/routing/run-routing.ps1 and analysis/report artifacts, using the current environment's established runtimes; do not invent a second worker orchestrator for the benchmark.

Follow [[2026-09-22-jev-routing-benchmark]] exactly: 300 packet cases; 24 tasks × four arms screening; then a separately frozen held-out R0/RJ trial with power disclosure. The benchmark may use a private test credential directly in its adapter; production Desktop must not embed Pytxo's provider key.

**Acceptance:** report all assigned tasks, every attempt, advisor failures, paired quality/cost uncertainty and versioned artifacts. Stop active-advice expansion if RJ loses or remains inconclusive. Rules-only execution is a useful stopping point. No service deployment is required to establish whether the semantic signal is worthwhile.

## 7. Add fixed-purpose routing sponsorship

**Depends on:** 5; scale/promotion depends on 6 and provider/budget prerequisites.

**Existing:** services/pytxo-link/src/{auth,jwt,main,limits,inference}.rs; services/pytxo-proxy/src/main.rs and metering.rs. Existing paid paths keep their semantics.

**New:** services/pytxo-link/src/routing_allowance.rs; services/pytxo-proxy/src/routing.rs; dedicated Link migration and isolated service integration tests.

- Implement scoped account tokens and the fixed endpoint from [[2026-09-22-jev-routing-service]].
- Add atomic account/global reservations, one-send ownership, conservative unknown settlement, expiring answer recovery and nonreplayable expired IDs.
- Reject development/shared identities, client account headers, arbitrary questions/URLs, model aliases and payload overflow.
- Disable raw payload logging and automatic upstream retries. Separate sponsor nano-USD from worker/planner ledgers.
- Establish a verified provider billing bound or actual prepaid/spending control before claiming hard subsidy enforcement.

**Acceptance:** multi-replica admission cannot overbook the ledger; token rotation cannot reset allowance; crash-after-send does not resend; malformed/late output cannot affect Core. Retention and log inspection use an isolated database. Prepare an exact deployment/retention/budget payload before any separately authorized launch.

## 8. Expose minimal product control and promote

**Depends on:** qualified local behavior; hosted activation additionally needs 7.

**Existing:** apps/desktop/src-tauri/src/{ipc_flow,ipc}.rs; apps/desktop/src/lib/{types,desktop-backend}.ts; desktop2/{MissionDock,WorkActive,HistoryScreen,SettingsScreen}.svelte.

**New:** apps/desktop/e2e/routing.spec.ts.

Expose Manual/Balanced, workspace disclosure preview, selected profile/reason, full attempt chain, fallback reason, consent revocation and actual/estimated/unknown cost. Keep Work/History/Setup and current Review → Apply. No major visual redesign.

**Acceptance:** a disposable native mission demonstrates plan → bounded attempts → exact review candidate → existing Apply; history shows failed attempts without treating them as final task failure. Provider outage/allowance exhaustion does not block local rules, Stop or review. Browser tests alone are insufficient native evidence.

## Commands and rollback

Existing test targets verified to exist during this design review:

```powershell
cargo test -p pytxo-core coordinator
cargo test -p pytxo-planner
cargo test -p pytxo-scheduler
cargo test -p pytxo-store
cargo test -p pytxo-orchestrate --test flow --test flow_concurrency --test candidate_run --test run_apply
cargo test -p pytxo-runner --test dependency_outcomes --test run_change_set
cargo test -p pytxo-proxy
cargo test -p pytxo-link
npm --prefix apps/desktop run check
```

Add each new meaningful test target when its package creates it. Before promotion, run workspace tests/clippy and the affected native/package journey using repository instructions. Keep unrelated failures distinct; do not rewrite evidence to turn a blocked check green.

Rollback first disables active Jev and uses the same R0 controller. Disabling new routed dispatch must still reconcile existing attempts, cancellation and review packages; never fall back to legacy execution midway through an active routed task. Additive schema/history stays readable. Re-enable only a pinned qualified policy, never “latest.”

Open activation inputs: actual E/S recipes, sufficient corpus and funded execution budget, provider billing/retention terms, and evaluated thresholds. Cross-harness, local model and two-worker expansion each require their own evidence. No unrecorded design choice is delegated to Jev.
