---
title: Pytxo architecture audit
slug: pytxo-architecture
status: active
tags: [context, audit, pytxo]
audience: [human, agent]
layer: meta
created: 2026-09-16
updated: 2026-09-16
related: ["[[PYTXO_CANONICAL_CONTEXT]]", "[[PYTXO_CURRENT_STATE]]", "[[PYTXO_DECISIONS]]"]
---

# Pytxo architecture audit

Scope: inspected working tree at 2026-09-16, HEAD `7287970` plus inherited changes. Evidence IDs resolve in [[PYTXO_SOURCE_LEDGER]]. The architecture is a Rust workspace with Svelte 5/Tauri v2 Desktop and Next.js Web. “Implemented” means executable source paths are present; current tests and runtime limits are separately recorded.

## Implemented architecture

### Local control and presentation

`pytxo-core` defines configuration, task/review types, permission profiles, execution domains, provider metadata, and billing abstractions. It is not the whole control plane. `pytxo-orchestrate` coordinates public run/stop/review/apply operations; `pytxo-runner` implements process/workspace/verification mechanics; `pytxo-store` persists their state.

Desktop calls Rust through typed Tauri IPC and backend wrappers. `DesktopShell.svelte` exposes Work, History, Setup, workspace switching, approvals, active work, and mission docking. `RunReviewScreen.svelte` displays prepared files and checks rather than trusting terminal success text. The preview backend supplies fixture data for browser testing; it is not native execution evidence. The CLI exposes mission/run/status/Stop and tools; its mission command directs users to Desktop History/Run Review for the complete review journey. CLI presence is not proof of parity with every Desktop review action.

`pytxo-planner` has heuristic, Signal-backed, and optional model planning. Explicit local selection wins over model opt-in. Model planning requires `PYTXO_PLANNER_LLM=1` and a resolvable configured endpoint. `flow.rs` persists drafts and reviewed plans and validates dispatch scope. `pytxo-scheduler` checks task IDs/dependencies, cycles, and path overlap and assigns bounded waves. `max_agents` limits parallelism, not requested task count. Mock/recovery environment switches exist in DAG code; ordinary fail-closed behavior must not be inferred for test/recovery modes.

Execution domains bind a repository to its registry/store slice. `pytxo-orchestrate/src/hypervisor.rs` holds a `HypervisorRegistry` of `DomainState` values containing repository/data paths, swarm/process registries, HITL queue and MCP hub. The catalog tracks projects/workspaces and flow drafts. A workspace may represent multiple folders at the product level, but reviewed run Apply rejects multi-root tasks. Do not conflate workspace selection with cross-root atomic integration.

### Harnesses, models, tools, and execution

`AdeCliSpec` / `CliAdapter` and provider registries are existing integration seams. The inspected catalog names Claude Code, Antigravity, Codex, Cursor Agent, OpenCode, Gemini, Copilot, Aider, Grok Build, Factory Droid, Cline, Goose, Qwen, and Kimi. Qwen is detection-only in source. Other catalog entries marked Ready mean a reviewed command exists, not that every version/auth/permission/Stop path has been exercised. Authentication policies distinguish verified session, vendor-managed, and provider-managed cases.

Flow currently creates instances of one selected harness; lower-level agent manifests are broader. PTY is the default execution backend; subprocess is an explicit alternative. Separate workspace backends include worktree/overlay mechanisms with platform-specific implementations. Race's `SwarmRegistry` mediates claims and stdin; the scheduler serializes represented path/dependency conflicts. Different paths do not imply independence of schemas, ports, caches, GPUs, databases, or semantics.

Signal uses structural parsing/context reduction. This is not an established reduction in token bills or improved task quality. MCP server/hub code and audited mediated tools exist (`pytxo-mcp`, `mcp_hub.rs`, orchestration MCP entry points). Skills/native harness instructions are inputs, not new authority. MCP integration is not a plugin marketplace or a universal monitor of every tool a child can use.

### Candidate, verification, Review, Apply, recovery

`prepare_review_package` assembles claimed changes into durable blobs and a manifest. `CandidateVerification::prepare` snapshots the included repository inventory, overlays stored targets, and inserts a Git-discovery boundary. The orchestrator runs each required command with original task authority capped by current trust, then calls `check_unchanged` after each command. `finish` records base/candidate inventories, exclusions, recipe evidence, and time, and recomputes the package digest.

`require_candidate_verification` requires a v3 package and nonempty successful checks with identity/authority fields and a reconstructable composition. The orchestrator compares the recorded recipe and receipts with the saved plan and domain. Nonempty changes need checks for every planned task; no-change manifests have an explicit shortcut.

Apply obtains a domain mutation lease, checks completed/ready status, domain/profile, durable package digest, recipe, preimages and included base inventory. The lease uses an OS file lock held by the process; it coordinates participating processes, not arbitrary external writers. Runner writes a journal/backups before mutation, rechecks each target preimage, applies stored bytes, verifies included post-state, and records commitment. This is filesystem Apply; do not describe it as necessarily creating a Git commit or merging a branch.

On failure, rollback restores only provable states. Reconciliation identifies committed/rolled-back/recovery-required outcomes. Missing journal evidence or post-crash drift can require manual attention. Refresh rebuilds from current primary inputs and frozen reviewed targets, then reverifies; it does not quietly copy unrelated stale worker files back.

SQLite WAL records runs, actors, events, contracts, domain changes, receipts and errors; disk package/journal state complements the DB. Stop persists cancellation in the process registry and checks it during later dispatch/verifier waits. Process cleanup is platform/mechanism dependent. Crash recovery and Stop are distinct; neither constitutes universal undo.

**A1 — September 16 source fix:** `backend.applyRunChanges(run.id, domainId, expectedPackageDigest)` → required Tauri argument → Core comparison under the domain mutation lease binds new Apply authorization to the displayed package. Empty/mismatched identities fail closed before the Apply claim and record `review-authorization-refused`; B remains ready. Package identity includes candidate/base inventories and verification evidence. Production plan/enforcement contracts are created at dispatch for a new run; refresh reuses them rather than editing their authority. Any future legitimate mutation of those fields must revisit identity coverage. Existing journal reconciliation remains the recovery of prior authority, not permission to Apply the latest package. Core/IPC/browser regressions pass; native packaged proof remains BLOCKED.

### Optional hosted/account/commerce services

Keep these separate from local mission truth:

- **Pytxo Link** (`services/pytxo-link`, v0.3.3): optional account/control API. Auth supports configured API keys and Clerk/JWKS; routes include entitlement status, org policy/seats/audit, usage/wallet and run accounting, plus Paddle/Dodo ingress. Public-bind/auth checks exist. No live deployment configuration was inspected.
- **pytxo-proxy** (v0.3.3): separate managed inference transport. Core contains managed transport/wallet/usage machinery; this does not imply universal spend accounting across subscription-backed CLIs.
- **cloud-sandbox** (v0.3.3): separate service with sandbox start/sync/exec/teardown and scaffold-cache routes plus Docker helpers. Cloud verifier code refuses execution without a cancellable verifier contract. Do not equate service presence with parity for local reviewed Apply or a working deployment.
- **Web** (package v0.1.0): marketing/public docs, Clerk-associated account/checkout integration and server routes. Checkout requests use an MBCZ broker. Browser return parameters are not entitlement authority.

Dirty Link source contains provider adapters → normalized subscription events → independent grants → Pytxo-owned entitlement projection. `commerce.rs` has a Postgres transaction path for event deduplication, subscription binding/order, grant updates and projection. `dodo.rs` verifies exact raw-body HMAC with timestamp tolerance and accepts any valid supplied v1 signature against the configured secret; this is signature-list support, not proof of full operational secret rotation. Active/grace products must map through the server catalog; metadata tier cannot grant access. Terminal events can restrict an existing binding even if product mapping is unavailable.

Migration 008 carries legacy Paddle records/grants forward and preserves old tables. `db.rs` now propagates SQLx migrator failure instead of deleting checksum evidence. Web's Dodo route forwards exact bytes and selected signature headers to Link. Link requires its configured secret/catalog/durable commerce store. These are actual source paths, but SQL migration, real provider payload compatibility, concurrent DB lifecycle behavior and deployment are unvalidated here.

## Safety-critical boundary audit

| Boundary | Classification | Exact limitation |
|---|---|---|
| Task validity and represented ownership/waves | ENFORCED within mediated scheduler | Path claims are not semantic/resource independence or host-wide locks. |
| Separate workspace preparation | ENFORCED within supported local execution path | Child cwd/overlay is not syscall filesystem confinement. |
| Orbit/Galaxy host filesystem | ADVISORY | Child retains host-user reach outside mediated path gates. |
| Orbit/Galaxy network | ADVISORY | Recognized command/HITL gates do not intercept arbitrary socket calls. |
| DeepSpace network/non-flush | ENFORCED when required mechanism and egress probe succeed | Refuses unavailable mechanism; not asserted tested on this host during audit. |
| Supernova | ENFORCED policy selection; deferred boundary bypassed | Explicit host-direct behavior, no reviewed Apply protection. |
| Candidate identity/recipe/freshness | ENFORCED in inspected reviewed path | Included inventories/exclusions, not semantic freshness; local metadata not adversarially attested. |
| Human approval → displayed candidate | ENFORCED in source; native verification pending | A1: required reviewed digest compared in Core under lease before a new claim; stale-client regression passes. |
| Apply/recovery | ENFORCED within single-root mediated transaction | Process-scoped lease, per-file writes, conservative recovery; no cross-system atomicity/universal rollback. |
| Durable Stop | ENFORCED in supported registry/lifecycle path | Full current packaged/platform matrix not rerun. |
| Event/audit persistence | PARTIAL | Durable local records; no guarantee against same-user hostile tampering or all external effects. |
| Hosted model egress consent | ENFORCED at planner opt-in | Does not enforce every child harness's egress; full local-only workflow remains unvalidated. |
| Routing policy | PARTIAL | Membership validator exists; eligibility discovery and cross-component policy pipeline absent. |
| Plugin containment | UNKNOWN / not implemented | String fields and Tauri plugins do not establish Pytxo extension security. |
| Provider → entitlement projection | PARTIAL | Source plus memory/route tests; real Postgres/provider lifecycle unproven. |
| Cloud production posture | UNKNOWN | No fresh service/account/config audit. |

## Partially implemented architecture

The untracked `coordinator.rs` is imported and configured. It defines replaceable provider/model/direct-or-managed selection, intent enums, route proposals, snapshots and membership rejection. Planner actually uses explicit endpoint resolution; direct support currently requires OpenAI-compatible protocol and supports local endpoints such as Ollama/LM Studio.

**The broader router is not implemented.** Symbol search found snapshot/proposal validation definitions and tests, no production route-selection caller or catalog-building/filtering pipeline. Membership in separate lists does not prove combination compatibility, budget sufficiency, plugin safety, or local-only policy. Existing `billing/router.rs` resolves worker/provider transport; it is not the proposed execution-profile intelligence.

Commerce has executable source but lacks runtime closure. Local inference support has config/transport seams, not an exercised local-worker/offline golden path. The current frontend contains broad setup/mission features and older legacy components; source existence does not make all of them exposed, complete, or supported in Beta.

## Documented / designed architecture

ADR-0034 and current vision document immutable review and durable Apply; implementation is described above with A1's qualification. ADR-0036/general effect contracts describe authority, postconditions, compensation and production boundaries beyond current repository effects.

ADR-0040 and ADR-0041 are **proposed**, despite their “Decision” prose. Their intended commerce/coordinator boundaries are useful, but do not grant implementation or deployment authority. Execution-profile routing, a Core-filtered catalog, typed proposal acceptance and provider-neutral plugin permissions are designs whose complete operational path is missing.

## Proposed future architecture

H2/H3 propose a RuntimeAdapter lifecycle interface, a CapacityPool independent of harness identity, local empirical competence, bounded routing escalation, plugin management/SDK, and operational Tasks/Code topology. These are not obligations to build them now. Intended near-term architecture should retain the existing control path and finish its proof; it does not need a new kernel.

The existing `TopologyScene3D.svelte` belongs to the legacy developer-deck path; `App.svelte` gates that shell, and navigation redirects old topology routes to review. Its existence does not implement the new semantic-zoom operational map.

## Explicitly rejected / superseded architecture

Do not rebuild a generic approval-popup product, make an AI coordinator authoritative, hardwire DeepSeek as a protocol dependency, make local Core unnecessarily depend on Cloud, or scatter Dodo tiers through product capability checks. The city/raw-AST primary UI was rejected; full 3D operational work and marketplace/enterprise expansion are deferred. Automatic fan-out was rejected for the existing increment, not prohibited forever. See the precise statuses and revisiting conditions in [[PYTXO_DECISIONS]].
