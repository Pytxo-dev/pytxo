---
title: Pytxo current repository state
slug: pytxo-current-state
status: active
tags: [context, audit, pytxo]
audience: [human, agent]
layer: meta
created: 2026-09-16
updated: 2026-09-16
related: ["[[PYTXO_CANONICAL_CONTEXT]]", "[[PYTXO_ARCHITECTURE]]", "[[PYTXO_SOURCE_LEDGER]]", "[[PYTXO_OPEN_QUESTIONS]]"]
---

# Pytxo current repository state

**Observed 2026-09-16.** This document owns volatile facts for the context pack. Evidence IDs refer to [[PYTXO_SOURCE_LEDGER]]. Historical/source/test/runtime/release claims are deliberately separate.

## Post-audit implementation update — September 16

The audit snapshot below is retained as historical evidence. A1 is no longer
merely suspected: the existing fixture reproduced A authorizing a legitimately
refreshed B before repair. Desktop/IPC/Core now require the reviewed digest;
Core compares it under the domain mutation lease before a new Apply claim.
Stale refusal is audited and leaves B ready. Reconciliation of an already
authorized interrupted attempt is intentionally preserved before new authority.
Core, typed IPC and two-client browser regressions cover the repaired boundary;
native/package proof remains BLOCKED. Current code must not be described as
missing the caller digest based on the older audit sections below.

Flow also now persists `max_workers`; Desktop requests one worker without
rewriting config. Dispatch cannot widen it with a later configuration increase;
old previews without explicit worker authority require regeneration. Additional
per-run check commands go through fresh preview. Prompt-only save rejects check
edits instead of silently ignoring them. Full Beta feature gating and first-use
completion remain outstanding. RELEASE_READINESS owns current test results;
RELEASE_PLAN and CHECKPOINT own the approved remaining sequence. HEAD is unchanged;
the working tree contains this implementation in addition to inherited changes.

## Git

| Field | Observation |
|---|---|
| Root | `C:\pytxo` |
| Branch | `codex/beta-candidate-verification` |
| HEAD | `72879702f90f2b74eece888bb117df59608f9b56` |
| Baseline changes before this pack | **115 modified tracked files; 118 untracked files** with `-uall`; 233 file entries |
| Index/staged | Empty (`git diff --cached --name-only`) |
| Deleted tracked files | None in baseline status |
| Current source identity | HEAD **plus** inherited dirty/untracked code; HEAD alone is insufficient |
| Local beta remote-tracking ref | `origin/codex/beta-candidate-verification = eb5f73d`; no fetch performed |
| Main refs | Local `main = 79823f5`; local `origin/main = 9139a90`; not live remote observations |
| Local candidate tag | `local-v1.2.2-rc1` exists; annotated tag object `2436cc0`, not today's source identity |
| PR references | Release docs reference private PR31, historically open/draft with 12 successful jobs for eb5f73d; current remote PR state UNKNOWN |

### Worktrees

| Path | Commit / branch |
|---|---|
| `C:/pytxo` | 7287970 / codex/beta-candidate-verification |
| `C:/pytxo-v1-1` | 1be01d1 / codex/v1-1-release-hardening |
| `C:/pytxo-v1-1-1` | 7d3c6db / codex/v1-1-1-desktop-polish |
| `C:/Users/mattbaconz/.config/superpowers/worktrees/pytxo/pytxo-desktop-2` | 71a8a81 / codex/release-desktop-macos-target-fix; Git reports prunable; untouched |

Other relevant local branches include `codex/release-1.2.2-integrity`, `codex/release-1.2.1`, `codex/desktop-readiness`, `codex/pytxo-desktop-2`, `codex/v1-1-trustworthy-mission-control`, `release/1.2.0`, `release/1.1.1`. Branch names indicate history, not preferred source or release authority.

### Recent relevant commits

| Commit/date | Subject |
|---|---|
| 7287970 / Sep 13 | checkpoint: preserve validated Desktop structural UX |
| eb5f73d / Sep 8 | fix: remove Windows runtime prerequisite and verify final candidate |
| f64a0b0 / Sep 7 | test: diagnose Windows transport lifecycle timeouts |
| 402c599 / Sep 7 | fix: release registry lock before terminating workers |
| 2964d5a / Sep 7 | fix: harden scoped agent execution and reviewed Apply |
| ff0b88f / Sep 6 | docs: record final beta review and hosted CI gate |
| a4f5828 / Sep 6 | fix: keep candidate refresh and approvals auditable |
| f21acfb / Sep 6 | fix: bind reviewed work to verified candidate evidence |

Dirty work groups: Desktop structure/catalog/assets/E2E and IPC metadata; Web marketing/docs/billing ingress; demo/media; planner/coordinator/config/CLI/shell; Link commerce/entitlements/migration; release/tooling/research/checkpoint documentation. Untracked coordinator, commerce, Dodo and migration files are imported/used by tracked changes. They must not be dropped as disposable scaffolding.

Only the seven requested `docs/context/*.md` files were created by this mission. Build/test outputs are generated verification artifacts; no product source, existing docs, index, branch, credentials or external state was changed. Baseline file hashes were captured for preservation verification.

## Version/build state

| Surface | Current source value / evidence |
|---|---|
| Rust workspace, Desktop package/Tauri, demo-video, npm wrapper | 1.2.2 (`Cargo.toml`, respective package manifests, Tauri config) |
| Link / proxy / cloud-sandbox packages | Each 0.3.3; independent service versioning |
| Web package | 0.1.0, private; not a public product release number |
| Chroma package | 0.3.4; separate design package |
| Public release declaration | `apps/web/src/lib/site.ts`: PUBLISHED_VERSION 1.2.1, CANDIDATE_VERSION 1.2.2 |
| Historical public observation | H3 reports v1.2.1 observed in public releases Sep 15–16, published/updated Sep 3; **not refreshed online here** |
| Current build check | Selected Rust tests compiled successfully; Desktop Svelte/CSS check passed. No fresh native executable/MSI or full Web build made. |

### Existing artifacts actually found

These are the newest relevant artifacts found through current checkpoint/release pointers, not an exhaustive drive scan. Hashes below were recalculated during this audit.

| Artifact | Identity | What it proves |
|---|---|---|
| `target/ui-audit-2026-09-15/pytxo-desktop-cline-logo-final.exe` | 37,699,072 bytes; SHA256 `F164E774308C8EB1D8CD8086DE1C3D26A64C8453895D29E304366D437E818532` | File exists and matches saved receipt. Receipt reports 1282×802 Void/Light icon inspection; not rerun, not whole mission acceptance or installer. |
| `target/astra-review-identity-20260913/package/candidate/Pytxo Desktop_1.2.2_x64_en-US.msi` | 11,087,872 bytes; SHA256 `322489A0F8858E4FAEEC2C20E09875F24D5B85920EC8D363402E4D8445CA57B3` | Newer referenced MSI than integrated AE6D; saved receipt says not installed/native-accepted/frozen/published. Later UI source supersedes it. |
| Same candidate directory, `pytxo-desktop.exe` | SHA256 `E5ED40A55AB1F5C47838E7AFE6E2092BE004AB7000DA93207C5039B740A75806` | Standalone EXE identity only. Saved packaged payload is separately identified as F84BF980…; do not substitute hashes. |
| `target/astra-integrated-20260913/package/candidate/Pytxo Desktop_1.2.2_x64_en-US.msi` | SHA256 `AE6D704B3E366EF2745867CBAD4DA8C56676463F9CB5355BDE5835E9009F5C41` | Older retained MSI; prior to cross-run Review identity repair. |

The September 15 source receipt has manifest digest `7c2ede2070bbcd231de09b246463f5eda5241ccb079a95d2c76c56d1e3d7a9b1`. Its full source manifest was **not** compared against every current file. Matching executable hash does not establish matching current source.

Release readiness: **NOT ESTABLISHED for the current whole tree.** Older “locally verified” conclusions are snapshot-specific. No install, signing verification, clean-Windows run, current-source hosted CI, downloaded-public-asset check or publication occurred here.

## Tests

All commands ran on the dirty working tree, without modifying product code or dependencies. Rust used `--locked --offline` to avoid lockfile changes/network downloads. Approximately 14.2 GB free on C: was observed before compilation; no cargo/rustc process was active at that check. Existing Node processes were left alone.

### CURRENTLY RUN

| Command (repository root unless noted) | Actual result | Scope |
|---|---|---|
| `npm run check` in `apps/desktop` | PASS; 0 Svelte errors/warnings; CSS lint exit 0 | Static frontend check, not browser/native acceptance |
| `cargo test -p pytxo-runner --test candidate_verification_scope --locked --offline -- --test-threads=1` | **8 passed**, 0 failed | Combined-candidate failure, exact bytes/base, verifier mutation, exclusions, Git discovery boundary |
| `cargo test -p pytxo-orchestrate --test run_apply --test candidate_run --test flow_concurrency --locked --offline -- --test-threads=1` | **13 passed**: 11 Apply + 1 candidate + 1 concurrency | Stored digest mismatch, domain/actor receipts, missing/corrupt package, recovery, drift, real fixture commands, frozen refresh, task conservation |
| `cargo test -p pytxo-link --locked --offline -- --test-threads=1` | **37 passed**, 0 failed | Memory reconciliation and Axum route contracts; signatures, replay/order, binding, grants, expiry; no real Postgres |
| `cargo test -p pytxo-core coordinator --locked --offline -- --test-threads=1` | **5 passed**, 77 filtered | Config/default/local/direct selection and route membership rejection |
| `cargo test -p pytxo-planner --locked --offline -- --test-threads=1` | **17 passed**, 0 failed; 0 doctests | Opt-in/local precedence, bounded/redacted brief, task conservation/dependencies/check inference |
| Artifact SHA256 reads, Git inventory | PASS | Read-only identities, no release conclusion |
| Context-pack link/frontmatter/content/preservation checks | See final audit completion note below | Documentation scope only |

**Total: 80 Rust tests passed**, plus Desktop static checks. No current test demonstrated A1's stale-displayed-package scenario. Existing stored-package mismatch tests should not be credited as that proof.

### HISTORICALLY REPORTED

- H2's pasted Sol run: 136 targeted Core/Link/planner tests, full workspace tests, clippy/format/diff/ESLint/TypeScript passes. These are historical report counts, not this audit's result.
- September 13 release record: full workspace/clippy, 197 production-preview Desktop tests, 1 development test, 6 reference captures, Web build/lint and 23 tests, demo checks and 44 release-tool tests; tied to its older snapshot.
- September 13 cross-run review repair: 30 focused browser tests and a newer MSI.
- September 15 checkpoint: 5 focused harness/scroll tests, native build and narrow logo/spacing inspection. Those checks did not dispatch a mission or Apply.
- September 8 `tooling/benchmarks/results/astra-final-native-2026-09-08.json`: actual Codex 0.153.4 three-task/two-wave candidate and reviewed Apply on a recorded v1.2.2 MSI; source HEAD f64a0b0 plus dirty state. This is historical runtime evidence, not a comparative win or current-binary certificate.

### NOT RE-RUN

Full workspace suite, clippy/rustfmt, all Desktop/browser/Storybook suites, Web build/TypeScript/ESLint, release toolchain/MSI, provider/harness authentication/execution probes, actual native window/UI inspection, Stop platform matrix, benchmark experiments and remote CI/release checks.

### BLOCKED / not established

Real Dodo/Postgres migration/checkout/lifecycle was historically blocked by unavailable Docker daemon and no psql/initdb in H2. This audit neither rechecked that host prerequisite nor attempted activation; call the validation **missing**, not “Docker is currently broken.” Clean-Windows/current-artifact acceptance and publication remain missing evidence. No command attempted during this audit failed due to an external blocker.

## Feature state and historical-claim reconciliation

“VERIFIED CURRENT” below means inspected executable source, with specific current tests where listed; it never means universally runtime-proven.

| Feature / historical claim | State | Evidence | Notes |
|---|---|---|---|
| Mission flow | VERIFIED CURRENT | R2; planner + Flow tests | Draft → reviewed plan → dispatch; full packaged journey not rerun |
| Execution | VERIFIED CURRENT / NEEDS RUNTIME VERIFICATION for catalog breadth | R3/R7 | PTY/subprocess and CLI adapters; fixture processes are not paid harness runs |
| Workspace isolation | VERIFIED CURRENT | R3 enforcement/blast | Separate workspace; host filesystem advisory, Supernova bypass |
| Runtime support | VERIFIED BUT CHANGED | R7 catalog, dirty changes | 14 recognized, Qwen detection-only; more names than historically validated runs |
| Ownership and waves | VERIFIED CURRENT | R2/R3; concurrency test | Deterministic represented overlap/dependencies; not semantic independence |
| Candidate assembly / exact identity | VERIFIED CURRENT | R4; 8 runner tests | Stored targets + v3 inventory/digest; not just worker-local state |
| Integrated verification | VERIFIED CURRENT | R4; candidate_run | Required recipes run on combined source; recipe adequacy separate |
| Freshness / mutation detection | VERIFIED CURRENT | R4; current tests | Whole included base inventory and after-each-check mutation rejection |
| Approval binding | PARTIALLY PRESENT | R4/R5, finding A1 | Current stored digest checked; no caller-reviewed digest |
| Review | VERIFIED CURRENT | R5/R10; static check | Chunk identity matching and UI content; no fresh browser/native journey |
| Apply | VERIFIED CURRENT within scope | R4; 11 Apply tests | Single root/domain Orbit/Galaxy, ready completed run and recipe, journaled writes |
| Rollback | VERIFIED CURRENT source / NEEDS RUNTIME VERIFICATION broadly | R4 journal code | Failure compensation, not arbitrary undo of a successful external effect |
| Recovery | VERIFIED CURRENT in selected cases | R4; missing journal/recovered commit tests | Refuses uncertain/post-crash drift; full fault matrix not rerun |
| Durable Stop | VERIFIED CURRENT source / NEEDS RUNTIME VERIFICATION | R3 cancellation; dated native record | Persisted cancellation prevents later dispatch; no fresh platform matrix |
| History/audit | VERIFIED CURRENT | R6 | SQLite/WAL and local receipts; not hostile-user attestation |
| Coordinator | PARTIALLY PRESENT | R7; 5 Core/17 planner passes | Real planner transport/config; broader route/diagnosis system not implemented |
| DeepSeek configuration | VERIFIED CURRENT config only | R7 | Default string deepseek-flash, model egress opt-in; remote model facts UNKNOWN |
| Routing | PARTIALLY PRESENT | R7 symbol/caller search | Existing transport router plus unused higher-level membership validator; no autonomous profile pipeline |
| Generic RuntimeAdapter / CapacityPool | NOT FOUND in bounded source search | R7/H3 | Proposed pseudocode cannot establish implementation |
| Plugins | DOCUMENTED ONLY for proposed Pytxo framework | H2; R7 | Route plugin names ≠ manager/runtime/SDK/containment; no public marketplace |
| Local models | PARTIALLY PRESENT | R7/planner tests | Local endpoint seams; no actual local-model mission/offline proof |
| Commerce / Dodo adapter | VERIFIED BUT CHANGED; NEEDS RUNTIME VERIFICATION | R8; 37 Link tests | Older “documentation only” release prose stale; real dirty source now exists |
| Dodo exact raw bytes/freshness/signature-list handling | VERIFIED CURRENT source/tests | R8/dodo tests | Single configured secret; no real provider rotation/delivery run |
| Dodo catalog/replay/order/cancellation/grace/coexistence | VERIFIED CURRENT source/memory tests; NEEDS RUNTIME VERIFICATION | R8 | Live identities, payloads, DB transactions and expiry behavior not exercised together |
| Migration safety | VERIFIED CURRENT source; NEEDS RUNTIME VERIFICATION | R8/db.rs + SQL008 | Checksum failure propagated; migration/legacy-data upgrade not run |
| Entitlements | VERIFIED CURRENT source/memory tests | R8 | Provider-independent grants project to Pytxo tiers; not arbitrary generic capability marketplace |
| Link/proxy separation | VERIFIED CURRENT source | R9 | Optional account/control versus managed transport; live health UNKNOWN |
| Desktop UX/docking | VERIFIED BUT CHANGED | R10/R11 | Implemented and dirty follow-ups; not only screenshot concept |
| Topology | VERIFIED BUT CHANGED / DOCUMENTED ONLY for new map | R10/H3 | Legacy development-only 3D exists; new operational map not implemented |
| Windows packaging | VERIFIED BUT CHANGED / NEEDS RUNTIME VERIFICATION | R11 + existing hashes | Multiple old artifacts; latest source not frozen/clean-installed |
| Beta readiness | STALE if treated as complete | R11 | Snapshot-specific successes, outstanding final gates |
| Cloud services | PARTIALLY PRESENT | R9/service source | No current deployed security/health or local-Apply parity proof |
| Unique value / first-user return | UNKNOWN | H1–H3/R12 | Research/experiments, no independent repeat-use result found |

## Findings and known blockers

### A1 — Caller-reviewed package identity is not part of Apply authorization

**Observed:** `RunReviewScreen.svelte:168` calls `applyRunChanges(run.id, domainId)`. `desktop-backend.ts:73`, `ipc.ts:175`, Tauri `ipc.rs:886`, and orchestration `lib.rs:687` accept run/domain/config without an expected digest. The orchestrator checks its current persisted digest against its current package. Prepared-content chunk reads do carry identity checks.

**Implication/inference:** if a second legitimate operation refreshes/reprepares the same run while the first UI still displays old evidence, the Apply call cannot distinguish the old viewing decision from authorization of the newly stored package. A refresh can change inputs/check evidence even when target bytes remain frozen. This is a concrete missing API binding, **not a reproduced unsafe Apply exploit**. Current tests do not close it. Same-run UI/key handling and mutation-lease timing need a targeted adversarial regression.

### Technical risks

- A1 weakens the strongest exact-review claim until proven/repaired.
- Host filesystem/network are advisory for Orbit/Galaxy. Same-user local metadata is not a hostile-worker security boundary.
- Correct checks can still omit semantic requirements; historical native work needed documentation-task guidance despite passing code checks.
- New route contracts lack operational eligibility/compatibility enforcement; avoid exposing them as autonomous routing.
- Dirty commerce runtime/migration is incomplete evidence, not a finished activation.

### Runtime-validation blockers

- No fresh complete current-binary mission/Review/Apply/recovery journey.
- Clean Windows installation and native scaling/current artifact acceptance absent.
- Expanded harness catalog and local-model path lack a current version-specific execution matrix.
- Dodo/Postgres lifecycle and Web → Link → entitlement end-to-end not exercised.

### Release blockers

- No one frozen current-source artifact with combined acceptance.
- Current-source hosted CI not established; old CI quota/billing block is superseded, current allowance UNKNOWN.
- Public release/download verification and release authorization not performed.
- Checkpoint/provenance records an Antigravity logo approval-or-omit public-distribution gate; asset presence is not clearance. Other signing/media/provider-term gates require current evidence, not assumptions.

### Product-validation blockers

No inspected evidence establishes unique incremental catches, reduced supervision, first-user task completion/voluntary return, or useful scaling beyond a competent native harness. Historical benchmark observations disclose confounding and cannot establish a speed/cost win.

### External/provider blockers

Real Dodo status/catalog, MBCZ checkout compatibility, provider quotas/terms and current hosted service posture are UNKNOWN here. They are external validation prerequisites, not failures proved by this audit. Do not couple them unnecessarily to accountless local Core.

## Recommended next mission — exactly one

**Objective:** close and prove the user-reviewed-candidate identity boundary for existing Desktop Review → Apply.

**Why now:** the product's strongest promise is applying precisely the state the user reviewed. A1 is narrower and more consequential than starting routing, plugins, topology or another commerce subsystem.

**Scope:** reproduce the same-run stale-view scenario using two logical clients. If the source gap is confirmed, carry an expected package digest (and existing run/domain identity) from displayed review through the typed frontend/backend/IPC path to the deterministic mutation boundary. Compare it under the same lease/state claim used for Apply, before mutation; return a clear stale-review error requiring reload/review. Preserve current candidate/recipe/drift/recovery checks and handle every exposed reviewed-Apply caller consistently. Inspect lower-level callers so no silent “use latest package” fallback undermines the fix.

**Non-goals:** new authorization service, cryptographic hostile-host attestation, scheduler/router/plugin redesign, arbitrary external-effect governance, commerce activation, broad UX redesign, release publication.

**Acceptance criteria:**

1. Client views package A; another supported operation legitimately replaces/refreshes it to B; A's Apply is refused before repository mutation or commitment.
2. The rejection also covers refreshed base/check evidence with unchanged target bytes.
3. Refresh/review B allows an explicit B-bound Apply and records its exact digest/attempt/result.
4. Wrong run/domain/digest, missing identity and stale retry cannot reuse old authorization.
5. Ordinary Apply, preimage/base drift refusal, required recipe, task authority, Stop and conservative recovery remain intact; unrelated operator files survive.
6. UI explains stale review, cancels stale confirmation, reloads evidence, and requires a fresh explicit decision.
7. On one exact newly built native artifact, demonstrate stale refusal then fresh successful Apply on a disposable fixture; record source manifest, binary hash, package identities, resulting hashes and receipt. Do not claim clean-install release acceptance from that test.

**Evidence/tests required:** regression that fails before repair; focused orchestration/runner tests; typed IPC/caller test; browser test for open confirmation during same-run package refresh; Desktop type/style check; native fixture proof. If reproduction disproves the suspected path because another guard already prevents it, record that guard and regression before deciding whether any API change is necessary.

**Likely files/subsystems:** `RunReviewScreen.svelte`, `desktop-backend.ts`, `desktop-backend.preview.ts`, `ipc.ts`, `apps/desktop/src-tauri/src/ipc.rs`, `crates/pytxo-orchestrate/src/lib.rs`, relevant store contract/claim code, `run_apply.rs`, `candidate_run.rs`, Desktop review E2E.

**Risks:** digest changes include verification evidence, not only file bytes; backward compatibility can accidentally fail open; frontend checks alone leave a race; API changes must preserve recovery/one-domain lease ordering.

**Expected user value:** the confirmation means “apply the exact package and evidence I reviewed,” including after concurrent refresh, with a clear recovery path when the view is stale.

This is a recommendation, not implementation authorization. No part was implemented during the audit.

## Baseline dirty-file inventory

Exact `git status --porcelain=v1 -uall` file inventory, captured before adding this pack. Paths are repository-relative. Ignored build outputs are intentionally excluded.

<details>
<summary>115 tracked modifications and 118 untracked files</summary>

```text
 M .env.example
 M CHECKPOINT.md
 M DEMO.md
 M README.md
 M RELEASE_PLAN.md
 M RELEASE_READINESS.md
 M apps/demo-video/README.md
 M apps/demo-video/package.json
 M apps/demo-video/scripts/validate-master.mjs
 M apps/demo-video/src/Root.tsx
 M apps/desktop/e2e/astra-workflow.spec.ts
 M apps/desktop/e2e/beta-workflow.spec.ts
 M apps/desktop/e2e/marketing-captures.spec.ts
 M apps/desktop/e2e/menu-workflow.spec.ts
 M apps/desktop/e2e/motion-scroll.spec.ts
 M apps/desktop/e2e/polish.spec.ts
 M apps/desktop/e2e/review-depth.spec.ts
 M apps/desktop/e2e/review-hierarchy.spec.ts
 M apps/desktop/e2e/shell.spec.ts
 M apps/desktop/e2e/ui-feedback.spec.ts
 M apps/desktop/package-lock.json
 M apps/desktop/package.json
 M apps/desktop/public/ade/PROVENANCE.md
 M apps/desktop/src-tauri/src/ipc_meta.rs
 M apps/desktop/src/components/desktop2/AdeIdentity.svelte
 M apps/desktop/src/components/desktop2/AgentsScreen.svelte
 M apps/desktop/src/components/desktop2/DesktopShell.svelte
 M apps/desktop/src/components/desktop2/FlowScreen.svelte
 M apps/desktop/src/components/desktop2/HistoryScreen.svelte
 M apps/desktop/src/components/desktop2/MissionDock.svelte
 M apps/desktop/src/components/desktop2/MissionsScreen.svelte
 M apps/desktop/src/components/desktop2/RunReviewScreen.svelte
 M apps/desktop/src/components/desktop2/SettingsScreen.svelte
 M apps/desktop/src/components/desktop2/WorkActive.svelte
 M apps/desktop/src/components/desktop2/desktop2-shared.css
 M apps/desktop/src/components/setup/SetupStepAgents.svelte
 M apps/desktop/src/deck-scroll.css
 M apps/desktop/src/lib/desktop-backend.preview.ts
 M apps/desktop/src/lib/types.ts
 M apps/web/content/docs/compare/index.mdx
 M apps/web/content/docs/compare/meta.json
 M apps/web/content/docs/concepts/blast-shield.mdx
 M apps/web/content/docs/concepts/galaxy-approvals.mdx
 M apps/web/content/docs/concepts/meta.json
 M apps/web/content/docs/concepts/race-shield.mdx
 M apps/web/content/docs/concepts/signal-core.mdx
 M apps/web/content/docs/concepts/three-moats.mdx
 M apps/web/content/docs/concepts/what-is-pytxo.mdx
 M apps/web/content/docs/developers/architecture.mdx
 M apps/web/content/docs/developers/contributing.mdx
 M apps/web/content/docs/developers/v1-1-architecture.mdx
 M apps/web/content/docs/getting-started/desktop-setup.mdx
 M apps/web/content/docs/getting-started/first-mission.mdx
 M apps/web/content/docs/getting-started/first-three-agent-run.mdx
 M apps/web/content/docs/getting-started/install.mdx
 M apps/web/content/docs/getting-started/meta.json
 M apps/web/content/docs/index.mdx
 M apps/web/content/docs/meta.json
 M apps/web/content/docs/reference/changelog.mdx
 M apps/web/content/docs/reference/cli.mdx
 M apps/web/content/docs/reference/permission-tiers.mdx
 M apps/web/content/docs/troubleshooting/meta.json
 M apps/web/e2e/docs.spec.ts
 M apps/web/e2e/marketing.spec.ts
 M apps/web/package-lock.json
 M apps/web/package.json
 M apps/web/pnpm-lock.yaml
 M apps/web/src/app/(marketing)/download/page.tsx
 M apps/web/src/app/(marketing)/evidence/page.tsx
 M apps/web/src/app/(marketing)/page.tsx
 M apps/web/src/app/(marketing)/plans/page.tsx
 M apps/web/src/app/layout.tsx
 M apps/web/src/components/site/boundary-section.tsx
 M apps/web/src/components/site/compatibility-section.tsx
 M apps/web/src/components/site/download-desktop.tsx
 M apps/web/src/components/site/footer.tsx
 M apps/web/src/components/site/get-it-section.tsx
 M apps/web/src/components/site/hero.tsx
 M apps/web/src/components/site/measured-evidence.tsx
 M apps/web/src/components/site/product-section.tsx
 M apps/web/src/lib/site.ts
 M crates/pytxo-cli/src/main.rs
 M crates/pytxo-cli/src/mission.rs
 M crates/pytxo-core/src/ade_registry.rs
 M crates/pytxo-core/src/billing/link_reconciler.rs
 M crates/pytxo-core/src/billing/providers/registry.rs
 M crates/pytxo-core/src/billing/router.rs
 M crates/pytxo-core/src/config.rs
 M crates/pytxo-core/src/lib.rs
 M crates/pytxo-orchestrate/src/flow.rs
 M crates/pytxo-planner/src/lib.rs
 M crates/pytxo-shell/src/session.rs
 M distribution/railway/README.md
 M docs/00-meta/MOC-home.md
 M docs/01-projects/astra-evidence-2026-09-07.md
 M docs/01-projects/astra-execution-2026-09-07.md
 M docs/01-projects/astra-product-decisions-2026-09-07.md
 M docs/01-projects/astra-release-proposal-2026-09-07.md
 M docs/01-projects/dodo-mor-integration.md
 M docs/05-adr/ADR-0040-mbcz-merchant-of-record-boundary.md
 M docs/05-adr/index.md
 M docs/08-reference/pytxo-link-service.md
 M docs/08-reference/pytxo-toml.md
 M pytxo.toml.example
 M services/pytxo-link/README.md
 M services/pytxo-link/docker-compose.yml
 M services/pytxo-link/openapi.yaml
 M services/pytxo-link/railway.json
 M services/pytxo-link/src/db.rs
 M services/pytxo-link/src/entitlements.rs
 M services/pytxo-link/src/main.rs
 M services/pytxo-link/src/paddle.rs
 M services/pytxo-link/src/state.rs
 M tooling/scripts/verify-release-version.mjs
 M tooling/scripts/verify-release-version.test.mjs
?? .impeccable/critique/2026-09-14T05-23-57Z__c-components-desktop2-desktopshell-svelte-d94b2b85.md
?? PYTXO_ASTRA_MASTER_GOAL.md
?? THIRD_PARTY_NOTICES.md
?? apps/demo-video/aperture-props.json
?? apps/demo-video/public/product/aperture/.gitignore
?? apps/demo-video/public/product/flow-1600x1000.png
?? apps/demo-video/scripts/test-aperture-privacy.mjs
?? apps/demo-video/scripts/validate-aperture.mjs
?? apps/demo-video/src/aperture/PytxoApertureFilm.tsx
?? apps/demo-video/src/aperture/components/Frame.tsx
?? apps/demo-video/src/aperture/components/NativeFootage.tsx
?? apps/demo-video/src/aperture/components/Optics.tsx
?? apps/demo-video/src/aperture/config.ts
?? apps/demo-video/src/aperture/scenes/01Opening.tsx
?? apps/demo-video/src/aperture/scenes/02Mission.tsx
?? apps/demo-video/src/aperture/scenes/03Structure.tsx
?? apps/demo-video/src/aperture/scenes/04Review.tsx
?? apps/demo-video/src/aperture/scenes/05Apply.tsx
?? apps/demo-video/src/aperture/scenes/06Result.tsx
?? apps/demo-video/src/aperture/scenes/07End.tsx
?? apps/demo-video/src/aperture/style.ts
?? apps/desktop/e2e/harness-catalog.spec.ts
?? apps/desktop/e2e/presentation-pass.spec.ts
?? apps/desktop/e2e/setup-scroll-ownership.spec.ts
?? apps/desktop/public/ade/aider.png
?? apps/desktop/public/ade/anthropic.svg
?? apps/desktop/public/ade/antigravity.png
?? apps/desktop/public/ade/cline.svg
?? apps/desktop/public/ade/copilot.svg
?? apps/desktop/public/ade/cursor-on-dark.svg
?? apps/desktop/public/ade/cursor-on-light.svg
?? apps/desktop/public/ade/factory-droid.svg
?? apps/desktop/public/ade/gemini.svg
?? apps/desktop/public/ade/goose.svg
?? apps/desktop/public/ade/grok-on-dark.svg
?? apps/desktop/public/ade/grok-on-light.svg
?? apps/desktop/public/ade/kimi.svg
?? apps/desktop/public/ade/openai-on-dark.svg
?? apps/desktop/public/ade/openai-on-light.svg
?? apps/desktop/public/ade/opencode.svg
?? apps/desktop/public/ade/qwen.svg
?? apps/desktop/src/lib/ade-status.ts
?? apps/desktop/src/lib/evidence-motion.ts
?? apps/web/captures/site/docs-1440.png
?? apps/web/captures/site/download-1440.png
?? apps/web/captures/site/evidence-1440.png
?? apps/web/captures/site/hero-1280.png
?? apps/web/captures/site/hero-1920.png
?? apps/web/captures/site/hero-390.png
?? apps/web/captures/site/hero-768.png
?? apps/web/captures/site/home-1440.png
?? apps/web/captures/site/home-1920.png
?? apps/web/captures/site/home-390.png
?? apps/web/captures/site/home-full-1440.png
?? apps/web/captures/site/home-full-390.png
?? apps/web/captures/site/page-download.png
?? apps/web/captures/site/page-evidence.png
?? apps/web/captures/site/section-boundary-section.png
?? apps/web/captures/site/section-compatibility-section.png
?? apps/web/captures/site/section-evidence-section.png
?? apps/web/captures/site/section-get-it-section.png
?? apps/web/captures/site/section-marketing-hero.png
?? apps/web/captures/site/section-product-section.png
?? apps/web/captures/site/section-situation.png
?? apps/web/content/docs/compare/bridgemind.mdx
?? apps/web/content/docs/compare/conductor.mdx
?? apps/web/content/docs/compare/factory.mdx
?? apps/web/content/docs/getting-started/review-and-apply.mdx
?? apps/web/e2e/presentation.spec.ts
?? apps/web/public/evidence/astra-aperture-native-2026-09-08.json
?? apps/web/scripts/.verify/approvals.png
?? apps/web/scripts/.verify/flow.png
?? apps/web/scripts/.verify/history.png
?? apps/web/scripts/.verify/operations.png
?? apps/web/scripts/.verify/run-review.png
?? apps/web/scripts/.verify/setup.png
?? apps/web/scripts/.verify/work-agent-selected.png
?? apps/web/scripts/.verify/work.png
?? apps/web/scripts/.verify/workspaces.png
?? apps/web/src/app/api/billing/dodo/webhook/route.ts
?? apps/web/src/components/site/product-walkthrough.tsx
?? crates/pytxo-core/src/coordinator.rs
?? desktop-e2e.log
?? docs/01-projects/pytxo-final-three-harness-logo-research-2026-09-15.md
?? docs/01-projects/pytxo-harness-catalog-research-2026-09-14.md
?? docs/01-projects/pytxo-interface-upstream-research-2026-09-13.md
?? docs/01-projects/pytxo-presentation-acceptance-2026-09-14.md
?? docs/01-projects/pytxo-presentation-references-2026-09-14.md
?? docs/01-projects/pytxo-ui-audit-2026-09-14.md
?? docs/05-adr/ADR-0041-advisory-coordinator-and-routing-boundary.md
?? docs/08-reference/pytxo-interface-engineering.md
?? docs/demo/README.md
?? docs/demo/fixture/.gitignore
?? docs/demo/fixture/.pytxo-demo-sentinel
?? docs/demo/fixture/package.json
?? docs/demo/fixture/pytxo.toml
?? docs/demo/fixture/server.mjs
?? docs/demo/fixture/src/app.js
?? docs/demo/fixture/src/index.html
?? docs/demo/fixture/src/model.mjs
?? docs/demo/fixture/src/style.css
?? docs/demo/fixture/test/model.test.mjs
?? docs/demo/reset.ps1
?? docs/demo/setup.ps1
?? docs/superpowers/plans/2026-08-27-chroma-aperture-desktop.md
?? docs/superpowers/plans/2026-08-27-chroma-aperture-web.md
?? docs/superpowers/specs/2026-08-27-chroma-aperture-design.md
?? services/pytxo-link/migrations/008_provider_neutral_commerce.sql
?? services/pytxo-link/src/commerce.rs
?? services/pytxo-link/src/dodo.rs
?? tooling/benchmarks/results/astra-aperture-demo-2026-09-09.json
?? tooling/benchmarks/results/astra-aperture-native-2026-09-08.json
?? tooling/benchmarks/results/astra-aperture-plan-2026-09-08.json
?? tooling/benchmarks/results/astra-native-menus-2026-09-09.json
?? tooling/benchmarks/results/astra-native-menus-2026-09-10.json
?? tooling/benchmarks/results/astra-native-menus-video-2026-09-10.json
?? tooling/benchmarks/results/astra-native-review-2026-09-10.json
?? tooling/benchmarks/results/astra-native-review-video-2026-09-10.json
```

</details>

## Audit completion note

All seven documents were reread and reconciled for chronology, implementation/evidence scope, rejection versus deferral, and the A1 qualification. Automated checks found exactly the seven requested files, required frontmatter, valid intra-pack wikilinks, balanced code fences, no trailing whitespace, and no conflict/base64-image markers. The complete baseline inventory was compared against fresh SHA256/status readings: all 233 inherited dirty/untracked files were unchanged, with only these seven context documents added. HEAD remained 72879702f90f2b74eece888bb117df59608f9b56 and the index remained empty. The audit did not implement its recommended mission.
