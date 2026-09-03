---
title: Public v1 core and reliability release audit
slug: release-audit-core
status: active
tags: [project, release, audit, orchestration, reliability]
audience: [human, agent]
layer: orchestration
created: 2026-08-31
updated: 2026-08-31
related: [[vision]], [[commit-layer]], [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0038-epistemic-state-contract]]
---

# Public v1 core and reliability release audit

## Verdict

**Do not publish the current Rust control plane as a dependable public v1 yet.** No P0 data-loss exploit was reproduced, and the immutable reviewed Apply boundary is unusually strong. However, ten P1 gaps remain in the ordinary run lifecycle: the default PTY cannot be durably stopped while it is live, Stop can report success without proving termination, failed dependencies still feed downstream agents, verification bypasses the selected permission profile, startup failures can return an orphan run ID, concurrent/crashed domain runs are not reconciled, Flow contradicts its approved plan, failure can still exit successfully, Fleet can strand its ledger, and old migrations can claim success after ignored errors.

This is a code-and-test audit of commit `f31a75a` on `release/1.2.0`. “Verified” below means exercised by a passing test in this audit, not inferred from documentation. P0 is immediate release stop/data or host compromise; P1 is a credible release blocker; P2 is important but can ship only with an explicit limitation; P3 is hardening.

## Subsystem map

| Subsystem | Current implementation | Audit state |
|---|---|---|
| Core contracts/config | Typed tasks, plans, permission profiles, review manifests, child environment policy | Implemented; unit-tested; config deserialization has a P1 false-success default |
| Scheduler / Race Shield | Waves, dependency ordering, path conflict detection, in-memory claims, buffered stdin | Implemented and existing tests pass; dependency *success* is not enforced |
| Runner / execution yard | PTY default, subprocess fallback, cloud dispatcher, isolated workspaces, HITL, MCP child hub | Implemented but not release-safe under stop, cancellation, error unwind, or verifier execution |
| Blast Shield / reviewed Apply | Immutable byte package, digests/chunks, preimage validation, domain mutation lease, journal, rollback/recovery | Implemented and verified by extensive fault-injection tests; strongest release surface |
| Store / evidence | SQLite WAL, run/agent/event ledger, transactional Apply state machine, domain change cursor | Implemented and partially verified; migrations and evidence-corruption handling need work |
| Orchestrate / domains | Run facade, singleton domain registry, async dispatch, active marker, Flow, Fleet, projects | Partially implemented; lifecycle ownership is not durable or concurrency-safe |
| Signal / permission receipts | Tree-sitter scaffolds; per-surface enforcement receipts; DeepSpace fail-closed probe | Implemented and tested in isolation; MCP can bypass DeepSpace fidelity and raw-read limits |
| CLI / MCP | Run/status/log/stop/project/fleet and stdio MCP tools | Implemented; several paths can misstate outcome or expose an incomplete tool contract |

## What is implemented and verified

- The review contract is typed around exact file kinds, before/after digests, bounded chunks, and a package digest (`crates/pytxo-core/src/review.rs:3-71`). Package preparation writes exact content-addressed bytes and synchronizes the durable manifest (`crates/pytxo-runner/src/change_set.rs:259-399`); loading revalidates version, package digest, paths, blobs, and chunks (`crates/pytxo-runner/src/change_set.rs:402-497`).
- Apply runs under an execution-domain mutation lease, revalidates affected preimages immediately before mutation, persists journal progress, and rolls back or enters recovery-required rather than claiming an ambiguous commit (`crates/pytxo-runner/src/change_set.rs:13-61`, `706-989`, `1355-1414`, `1417-1722`). Protected paths, traversal, symlinks, and special files fail closed (`crates/pytxo-runner/src/change_set.rs:1738-1932`).
- Store transitions for prepare/apply/recovery use checked transactional updates (`crates/pytxo-store/src/store.rs:299-370`, `432-590`) and the database enables WAL (`crates/pytxo-store/src/store.rs:93-101`).
- Enforcement receipts do not pretend Orbit/Galaxy have a syscall sandbox: host filesystem and ordinary network controls are labeled advisory, while DeepSpace requires a recognized isolation mechanism and a passing socket probe (`crates/pytxo-runner/src/enforcement.rs:21-45`, `48-140`). Signal Core produces AST scaffolds and explicitly returns raw content only for high fidelity or unsupported languages (`crates/pytxo-signal/src/lib.rs:19-58`).
- The focused command `cargo test -p pytxo-core -p pytxo-scheduler -p pytxo-store -p pytxo-signal -p pytxo-runner -p pytxo-orchestrate` passed on this worktree. That includes 32 `run_change_set` cases, 9 orchestration Apply cases, runner enforcement/network/HITL/worktree tests, store state-machine tests, and Flow/hypervisor tests. A scan found no `TODO`, `FIXME`, `todo!`, or `unimplemented!` markers in the audited crates. Passing tests do **not** cover the contradicted paths below.

## P1 release blockers

### P1.1 — The supervision plane cannot reliably stop the default execution backend

The configured default is PTY (`crates/pytxo-core/src/config.rs:155-157`, `171-189`). PTY obtains the child PID at spawn but blocks on `child.wait()` and returns the PID only after exit (`crates/pytxo-runner/src/pty.rs:118-124`, `171-187`). The runner persists and registers that PID only after `run_pty_session`/`run_command_streaming` returns (`crates/pytxo-runner/src/run.rs:787-843`, `1159-1180`). While a default PTY child is actually live, durable Stop therefore has no PID to address. The existing live-stop integration explicitly selects `ExecutionBackend::Subprocess` and waits for that backend's registry entry (`crates/pytxo-orchestrate/tests/stop_exact.rs:156-183`), so it does not verify the default.

Stop also suppresses every per-PID kill error and always returns success (`crates/pytxo-runner/src/kill.rs:29-34`); Windows does not request tree termination and Unix ignores a failed KILL fallback (`crates/pytxo-runner/src/kill.rs:7-25`). Entries are removed immediately after this unverified attempt (`crates/pytxo-runner/src/run.rs:1344-1359`). `stop --all` clears the marker and prints success without settling any run rows (`crates/pytxo-orchestrate/src/lib.rs:1876-1883`); single-run Stop warns but still succeeds if cancellation persistence fails (`crates/pytxo-orchestrate/src/lib.rs:1956-1972`). `--cleanup-worktrees` passes a newly empty in-memory registry and is consequently a no-op (`crates/pytxo-orchestrate/src/lib.rs:1909-1955`). The durable entry records only a bare PID, not process start identity (`crates/pytxo-runner/src/process_registry_file.rs:14-22`), so a stale registry can target an unrelated reused PID after a crash.

**Release fix:** return a live child/process-tree handle at spawn, persist PID plus start identity before waiting, store a cancellable handle in the domain, terminate the full tree, wait and verify exit, and keep registry evidence until confirmation. Settle every stopped run transactionally and drive cleanup from durable workspace records. Add default-PTY and crash/PID-reuse stop tests on every supported OS.

### P1.2 — A failed upstream task is treated as valid dependency output

Every normal `AgentRunResult` is inserted into the `completed` dependency map without checking `exit_code` (`crates/pytxo-runner/src/run.rs:192-217`). Non-zero process or verifier exits are returned as ordinary results (`crates/pytxo-runner/src/run.rs:965-1001`), and later waves compose their workspaces from that map (`crates/pytxo-runner/src/run.rs:175-186`). Downstream agents can therefore consume partial output from an agent that failed verification or exited non-zero. No focused test covers a failing upstream dependency.

**Release fix:** model task outcomes separately from process completion; schedule a dependent only when all prerequisites completed successfully and passed verification. Persist a `blocked_by_dependency` terminal state and test partial writes, non-zero exit, and failed verification.

### P1.3 — Error unwind can leak claims, sessions, sandboxes, workspaces, and children

An agent claims Race paths and prepares its workspace before many fallible policy, cloud, MCP, HITL, and spawn operations (`crates/pytxo-runner/src/run.rs:475-539`, `646-843`). The only explicit claim release, MCP deregistration, and cloud teardown occur near the success tail (`crates/pytxo-runner/src/run.rs:956-963`), so earlier `?` returns bypass them. At wave level, the first task/join error immediately returns from the `JoinSet` loop (`crates/pytxo-runner/src/run.rs:192-195`); dropping async tasks cannot cancel an already-running `spawn_blocking` child. This can leave an agent executing with no supervising future and keep Race claims until process restart.

**Release fix:** introduce an idempotent per-agent lifecycle guard that owns every acquired resource, plus cooperative cancellation and wait for all wave members before returning. Inject failures after each acquisition stage and assert the domain is empty, children are dead, and durable status is settled.

### P1.4 — Verification commands bypass permission profiles and enforcement receipts

After an agent exits zero, each verifier is launched directly through host `cmd /C` or `sh -lc`, inheriting the parent environment, host filesystem, and network, with no timeout (`crates/pytxo-runner/src/run.rs:965-985`, `1004-1038`). It does not use the bounded child environment, HITL, DeepSpace shell wrapper, cloud sandbox, or network policy. This contradicts a DeepSpace receipt that says network isolation is enforced and Apply is non-flushable (`crates/pytxo-runner/src/enforcement.rs:82-125`).

**Release fix:** execute verification as a typed actor under the same or stricter profile, sandbox, environment filter, network boundary, timeout, and output cap; record a separate verification receipt. Test that verifier commands cannot read stripped secrets, reach egress, or mutate outside the isolated workspace.

### P1.5 — Async dispatch can return a run ID that never becomes a ledger row

Hypervisor dispatch spawns a detached future, logs any error, and immediately returns its generated run ID (`crates/pytxo-orchestrate/src/hypervisor.rs:169-210`). The background body performs Git, entitlement, policy receipt, DeepSpace probe, recovery, and checkout checks before inserting the run (`crates/pytxo-orchestrate/src/lib.rs:1267-1401`, `1411-1422`). Any early failure leaves the caller holding an ID that status can never find. Flow then marks its draft `dispatched` after this acceptance (`crates/pytxo-orchestrate/src/flow.rs:471-498`), while Mission polls for the missing row until a two-hour timeout (`crates/pytxo-cli/src/mission.rs:95-116`).

**Release fix:** synchronously create a durable `starting` run and Flow linkage before returning; transition that same row to `running` or structured `failed_startup`. Retain the task handle/cancel token in `DomainState` and expose startup failure to callers.

### P1.6 — Active-run state is a racy singleton with no crash reconciliation

`DomainState` holds registries but no active run task handles (`crates/pytxo-orchestrate/src/hypervisor.rs:16-27`), and dispatch does not exclude another run in the same domain. Every run overwrites one non-atomic `active_run.json` (`crates/pytxo-orchestrate/src/lib.rs:1520-1522`, `2099-2108`). Stop can address only the marker-selected run (`crates/pytxo-orchestrate/src/lib.rs:1886-1908`). Normal Rust unwind settles through `RunFinalizer::Drop`, but process death bypasses Drop (`crates/pytxo-orchestrate/src/lib.rs:105-140`); no startup routine reconciles stale `running` rows, process entries, and workspaces.

**Release fix:** replace the singleton marker with a transactional per-domain active-run set containing lease/heartbeat and process identity. On startup, reconcile the WAL against live process identities and settle unknown/refuted runs. Either serialize one run per domain or explicitly supervise multiple handles. Add two simultaneous non-overlapping runs and kill/restart recovery tests.

### P1.7 — Flow preview approves staged overlap that dispatch always rejects

Preview intentionally allows overlapping claims when the scheduler puts them in different waves and labels them `path_claim_staged` (`crates/pytxo-orchestrate/src/flow.rs:218-274`). Dispatch later rejects *any* result from `find_conflicts`, without comparing waves (`crates/pytxo-orchestrate/src/flow.rs:450-455`). The passing staged-overlap test stops at preview (`crates/pytxo-orchestrate/tests/flow.rs:101-130`). A reviewed dependency plan can therefore be impossible to dispatch.

**Release fix:** rebuild the exact execution plan during dispatch and reject only overlaps in the same approved wave. Add an integration test that saves, reviews, dispatches, and completes a dependency-staged overlap.

### P1.8 — Config drift allows a failed run to return process success

`fail_fast` uses plain `#[serde(default)]`, so a present `pytxo.toml` that omits it deserializes to `false`, while `PytxoConfig::default()` and the reference both say `true` (`crates/pytxo-core/src/config.rs:52-53`, `171-178`; `docs/08-reference/pytxo-toml.md:17-24`). With false, orchestration records `failed` but returns `Ok(run_id)` (`crates/pytxo-orchestrate/src/lib.rs:1641-1656`), and the CLI prints `Run ... finished` and exits successfully (`crates/pytxo-cli/src/main.rs:380-396`).

**Release fix:** use `#[serde(default = "default_true")]`, add compatibility fixtures for minimal/old configs, and make the CLI/MCP return an explicit failed terminal result regardless of whether orchestration continues other tasks.

### P1.9 — Fleet failure paths strand catalog state and “waves” run serially

Fleet inserts a run and marks nodes `dispatching`, then executes every node with awaited `run_blocking` inside a serial loop (`crates/pytxo-orchestrate/src/fleet.rs:151-212`). A default fail-fast node error propagates at lines 181-195 before the node or fleet is settled, so `continue_on_error` never gets to interpret the failed database status. Catalog writes are also discarded as best effort (`crates/pytxo-orchestrate/src/fleet.rs:153-155`, `170-178`, `197-204`, `235-265`).

**Release fix:** execute each wave concurrently with owned node results, convert every dispatch/run error into an explicit terminal node record, and settle the fleet in a guard/finally path. Make required catalog writes fatal or emit an evidence-gap state. Test both continue policies with one failing and one slow sibling.

### P1.10 — Upgrade migrations can advance past failed schema changes

Migrations 2, 4, and 5 ignore every statement error and then advance `user_version` (`crates/pytxo-store/src/migrate.rs:22-29`, `39-55`). The comment mentions duplicate columns, but missing tables, malformed historical schema, and other failures are treated identically. Migration 7 demonstrates the correct transaction pattern (`crates/pytxo-store/src/migrate.rs:73-85`); existing migration tests focus on v7.

**Release fix:** transactionally apply each migration, tolerate only a positively identified already-present column, verify the resulting schema, and update `user_version` in the same transaction. Test fixtures from every released schema plus injected failures.

## P2 findings

### P2.1 — MCP contracts and permission routing are incomplete

All advertised tools expose an empty input schema even when arguments are required (`crates/pytxo-mcp/src/main.rs:100-150`, `216-221`). Successful text is sanitized, but error text is returned raw (`crates/pytxo-mcp/src/main.rs:157-177`). Status and logs ignore repo/project arguments and open the current domain (`crates/pytxo-mcp/src/main.rs:270-283`). More seriously, the shared permission engine caps DeepSpace at low fidelity (`crates/pytxo-core/src/moat/permission/mod.rs:97-102`), but MCP scaffold reads accept caller-selected high fidelity and raw reads accept `raw=true` without applying that cap (`crates/pytxo-mcp/src/main.rs:310-325`, `422-434`; `crates/pytxo-orchestrate/src/lib.rs:1986-2046`). High fidelity returns raw source (`crates/pytxo-signal/src/lib.rs:19-23`). Treat the DeepSpace bypass as P1 if MCP is enabled in the public profile.

**Fix:** publish real JSON Schemas, sanitize both success and error envelopes, route every call through the selected execution domain, and enforce `PermissionEngine::max_fidelity` with no raw override under DeepSpace.

### P2.2 — Default project run reports all writable roots but executes only the primary

`project_run` returns every writable root label but invokes `execute_run_body` once with `tasks: None` in the primary domain (`crates/pytxo-orchestrate/src/project.rs:232-281`). Synthetic tasks have `root: None` (`crates/pytxo-orchestrate/src/lib.rs:1659-1672`, `2088-2095`), while MCP describes this as running “across ... writable roots” (`crates/pytxo-mcp/src/main.rs:139-141`). The current dry-run test validates the returned label list, not markers in each root.

**Fix:** synthesize one root-scoped task per writable root, or rename the operation and return only roots actually scheduled. Verify an end-to-end marker per root without custom tasks.

### P2.3 — Evidence failures can be silently converted into success or invented data

Live event persistence discards lock/append failures (`crates/pytxo-orchestrate/src/lib.rs:1438-1455`), and agent rows are inserted only after the whole plan returns (`crates/pytxo-orchestrate/src/lib.rs:1522-1563`). The schema declares event-to-agent foreign keys, but `open` enables WAL without enabling foreign-key enforcement (`crates/pytxo-store/src/schema.rs:10-32`; `crates/pytxo-store/src/store.rs:93-101`). Malformed stored review/error JSON becomes `None` (`crates/pytxo-store/src/store.rs:253-284`), and an invalid evidence timestamp is replaced with the current time (`crates/pytxo-store/src/store.rs:1010-1014`). These violate the epistemic rule that missing/corrupt evidence must become unknown/refuted, never fabricated.

**Fix:** insert agents before events, enable/verify foreign keys, expose append gaps, and return typed corruption/unknown states instead of dropping parse errors or manufacturing timestamps.

### P2.4 — Release identity is internally inconsistent

The workspace version is `1.2.0` (`Cargo.toml:25`), the checkpoint calls the release `v1.0.0` (`CHECKPOINT.md:1`), and the root README still labels install `v0.3.0` and status `v0.1.0` (`README.md:23`, `README.md:74`). A public release must choose one version and align binaries, packages, docs, release notes, and update paths.

## P3 hardening

- Race overlap is deliberately conservative but uses raw string prefixes, so sibling names such as `src/a` and `src/ab` conflict (`crates/pytxo-scheduler/src/overlap.rs:3-18`). This is safe but can serialize unrelated work. Normalize path components and glob semantics when throughput matters.

## Required verification gates

Run these from a clean checkout after the P1 fixes:

```powershell
cargo fmt --all -- --check
cargo test -p pytxo-core -p pytxo-scheduler -p pytxo-store -p pytxo-signal -p pytxo-runner -p pytxo-orchestrate
cargo test -p pytxo-cli -p pytxo-mcp
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p pytxo-cli -p pytxo-mcp
cargo run -p pytxo-cli -- status --json
```

Add named release-gate scenarios, not only unit tests:

1. Dispatch a blocking **default PTY** run, observe a durable process identity before exit, stop it, prove the full process tree exited, the run is cancelled, and the workspace is cleaned.
2. Crash the orchestrator during spawn, running, review preparation, Apply mutation, and Stop; restart and prove each run resolves to verified/unknown/refuted with no stale “running” claim.
3. Run two non-overlapping jobs in one domain and two overlapping jobs in separate waves; prove exact-run Stop, claims, and active state remain isolated.
4. Fail an upstream task after partial writes and fail its verifier; prove dependents never start and receive `blocked_by_dependency`.
5. Under each permission profile, attempt verifier and MCP env/file/network escape; compare observed enforcement to the stored receipt.
6. Save/review/dispatch a staged-overlap Flow; run a two-node Fleet with one failure under both continuation policies; run a project without custom tasks and verify actual roots.
7. Upgrade database fixtures from every prior public schema and inject a migration error; the old version must remain intact and startup must fail closed.

The additional `cargo build -p pytxo-cli -p pytxo-mcp` attempt during this audit was inconclusive because another workspace Cargo process held the shared artifact lock; it was cancelled without changing repository state. Full build/clippy remains a release-owner gate, not a verified result of this note.
