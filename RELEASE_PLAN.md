# Pytxo v1.2.2 release control plan

**Owner:** lead release engineer
**Started:** 2026-08-30
**Target:** credible public v1.2.2 release
**Current recommendation:** **NOT READY — corrective release candidate under verification**

This file is the source of truth for the release effort. A checkbox is not
enough to establish completion; every release claim must carry one of the
evidence states below.

| State | Meaning |
|---|---|
| **IMPLEMENTED** | Code or documentation exists, but the release behavior has not yet been exercised. |
| **VERIFIED** | The behavior was exercised and the evidence is recorded here or in `RELEASE_READINESS.md`. |
| **PARTIALLY VERIFIED** | Some required environments or paths passed; the remaining boundary is explicit. |
| **UNVERIFIED** | No current evidence supports the claim. |
| **BLOCKED** | An external dependency or required decision prevents verification. |

## 1. Product understanding

Pytxo is the commit layer for autonomous work. Its shipping beachhead is a
repository-scoped control boundary around existing headless coding-agent CLIs:

```text
mission -> plan -> isolated waves -> verification -> immutable review package
        -> human authorization -> affected-path revalidation -> journaled Apply
        -> evidence / recovery state
```

The intended first user is a developer or engineering operator who wants to
use heterogeneous coding agents without granting them an unreviewed path from
natural-language intent to repository mutation. The product must make the
distinction between proposed, authorized, committed, verified, and recovered
state visible. It must not claim the broader production-effect gateway as
shipping today.

The public-release proof is therefore not “many agents on a dashboard.” It is:
one real mission produces isolated work, exact reviewable bytes, truthful
enforcement evidence, a guarded Apply, independently observed post-state, and
an honest recovery outcome when Apply cannot be proven.

## 2. Current architecture summary

| Layer | Primary paths | Release responsibility |
|---|---|---|
| Presentation | `apps/desktop`, `apps/web`, `apps/demo-video` | Work / History / Setup UI, approval and review surfaces, public docs/site, demo evidence |
| Orchestration | `crates/pytxo-orchestrate`, `crates/pytxo-runner`, `crates/pytxo-scheduler`, `crates/pytxo-store` | Domain-scoped execution, PTYs, DAG waves, isolation, review packaging, Apply/recovery journal, durable ledger |
| Policy and context | `crates/pytxo-core`, `crates/pytxo-signal`, `crates/pytxo-sanitize` | Permission profiles, effect/enforcement DTOs, AST read skeletons, secret redaction |
| Interfaces | `crates/pytxo-cli`, `crates/pytxo-mcp`, `crates/pytxo-shell`, `crates/pytxo-tui` | CLI/TUI, editor/agent MCP access, status and control intents |
| Optional services | `services/*` | Capability-gated Link, sandbox, and managed inference; not the default local v1 proof |
| Distribution | `packages/pytxo`, `.github/workflows`, `distribution/` | npm wrapper, multi-platform CLI, Desktop installers/updater, checksums and release notes |

The architectural invariants for this release are one execution domain and
one repository root per reviewed Apply; Orbit/Galaxy use isolated work plus an
immutable review package; DeepSpace is non-flushable; Supernova is host-direct;
accepted ADRs are not edited.

## 3. Current state

- **VERIFIED:** v1.2.1 merged through PR #29; its required CI run
  `33731373527` passed 12/12 jobs and release run `33781868097` passed every
  required build, publication, Desktop, mirror, and npm job.
- **VERIFIED:** the canonical docs and exercised runtime implement an immutable reviewed-byte Apply
  contract and explicit recovery states (ADR-0034).
- **VERIFIED:** Desktop v1.2 information architecture is Work, History, and
  Setup with workspace switching in the title bar and approvals as an overlay.
- **IMPLEMENTED / PARTIALLY VERIFIED:** repository metadata, lockfiles, npm
  wrapper, Desktop, demo, web downloads, installers, public docs, and release
  notes identify v1.2.2. The expanded parity test passes; the hosted release
  matrix has not yet run on this corrective candidate.
- **VERIFIED:** public v1.2.0 already points to the earlier main merge, so this
  audited candidate was retargeted to v1.2.1 instead of rewriting a published
  tag or package.
- **VERIFIED:** v1.2.1 was published to the private and public GitHub release,
  npm, Vercel, and all three Railway services. Production health endpoints and
  the final hosted workflow were checked.
- **REOPENED / FIX IN PROGRESS:** an adversarial post-publication audit found
  obsolete tracked root `dist/` files that the Desktop mirror uploaded over the
  public Windows CLI and checksum manifest. v1.2.1 was repaired and verified,
  but its public artifact provenance became mutable. v1.2.2 is the fresh-tag
  recovery release and must not publish until clean staging and exact inventory
  gates pass locally and in hosted CI.
- **VERIFIED (local):** the complete v1.2.2 patch-scope Rust, Desktop native,
  web, demo, npm wrapper, workflow, formatting, and version gates pass. The
  unchanged packaged product workflows remain supported by the v1.2.1 runtime
  evidence recorded in `RELEASE_READINESS.md`.
- **VERIFIED:** a packaged Windows Desktop completed a real deterministic Orbit
  mission, exact review, explicit Apply, post-state test, and History flow; a
  separate packaged run refused affected-path drift without partial mutation or
  a false success receipt.
- **PARTIALLY VERIFIED:** the v1.2.2 npm wrapper exercised its production
  checksum/version-bound postinstall path against the rebuilt local v1.2.2 CLI;
  installation from the published npm tarball remains part of the final audit.
- **WORKTREE NOTE:** pre-existing untracked `apps/web/captures/`,
  `apps/web/scripts/.verify/`, `desktop-e2e.log`, and `docs/superpowers/` are
  user-owned until explicitly classified; they must not be overwritten or
  accidentally shipped.

## 4. Release blockers

| Priority | Blocker | State | Exit evidence |
|---|---|---|---|
| P0 | Public release staging could publish stale tracked files over freshly built assets and checksums. | **IMPLEMENTED / PARTIALLY VERIFIED** | Obsolete root `dist/` files are removed and `/dist/` is ignored. CLI/private/public and Desktop stages start empty and require exact non-empty inventories; signed and unsigned Desktop cases have regression tests. Local tests and `actionlint` pass; hosted release evidence remains required. |
| P0 | Current v1.2 critical workflow has no fresh end-to-end evidence. | **VERIFIED** | Packaged run `54924278-5e91-40d0-a7de-6f8e99cde4a3` completed isolation -> verification -> exact three-file review -> confirmation -> path-specific Apply -> post-state tests (3/3) -> committed History. Packaged run `47da6a06-a3f3-4b66-8425-838efd8a7809` refused affected-path drift, left only the operator edit, kept baseline tests 2/2, and recorded Apply failed with no success receipt. |
| P0 | Full Rust, Desktop, web, demo, packaging, and repository gates have not run on the current tree. | **VERIFIED (patch scope)** | Full workspace tests/clippy/builds, Desktop check/native build, web gates/build, demo static gates, npm tests/dry-pack, version/inventory regressions, workflow lint, and whitespace checks pass on v1.2.2. Browser/Storybook/runtime flows are inherited from v1.2.1 because product source is unchanged; hosted CI remains a separate publication gate. |
| P0 | Fresh install/package behavior and embedded version parity are unproven. | **PARTIALLY VERIFIED** | v1.2.2 wrapper tests pass against the rebuilt release CLI, including checksum mismatch and embedded-version rejection; the package still dry-packs exactly six files. A clean install from the actually published `pytxo@1.2.2` is required before this closes. |
| P0 | Repository-controlled per-agent profiles could exceed out-of-band trust ceilings. | **VERIFIED** | Every project root requires readable explicit trust; trusted-primary/untrusted-secondary Supernova is rejected before execution; signed-in offline Supernova/elevated-agent requests fail instead of dropping the org ceiling; focused fixtures and the final workspace regression pass. |
| P0 | Cloud sync and context-cache uploads could transmit raw secrets or reinterpret a policy denial as local execution. | **PARTIALLY VERIFIED (fix)** | Dual consent, manifest, content/path denial, no-follow handle reads, policy-vs-transport classification, separate host consent for local fallback, and remote `/workspace` request tests pass. A live sandbox sync→exec test is still required before this row is fully verified. |
| P0 | Paddle entitlement webhook accepted unsigned/unbounded tier changes and had no durable identity/replay binding. | **VERIFIED (targeted fix)** | 21 Link tests prove fail-closed secret/signature freshness, event allowlist, configured price mapping, durable subscription/customer/user binding, conflict rejection, transactional replay, cancellation ownership, and idempotent duplicate success; the duplicate web provisioner was replaced by a fail-closed Link proxy. Production Postgres migration remains part of deployment verification. |
| P0 | The documented Windows echo smoke previously printed `OK` after the real run failed folder-trust enforcement, so prior smoke evidence could be false-positive. | **VERIFIED** | Checked native invocation helper, explicit trust, and current-tree binary smoke exit zero; deliberate native failure is rejected and cannot reach the success marker. |
| P1 | README install/status language was materially stale and undermined onboarding. | **VERIFIED** | README reconciled to v1.2.0; the packed npm artifact installed from a staged checksum-bound release in a clean prefix; its CLI version and quick doctor passed. |
| P1 | The release lacked a reproducible 2–4 minute commit-boundary walkthrough and full CLI -> Desktop -> Apply evidence. | **VERIFIED** | `DEMO.md`, isolated fixture/catalog/WebView state, and the exact packaged candidate completed mission -> review -> confirmation -> Apply -> checkout verification -> History; a separate stale-path run proved refusal. |
| P1 | Completed native runs omitted their agents/waves and recomputed isolation from configured intent, so Work could hide real tasks and report `worktree · worktree` for a persisted ProjFS overlay receipt. | **VERIFIED (runtime fix)** | Snapshots now include agents for every returned run and prefer the persisted receipt mechanism; targeted native tests pass and the packaged Desktop shows `1 of 1 settled · 1 passed`, the exact agent, and `overlay · projfs-sparse-copy-v2`. |
| P1 | Demo-only `PYTXO_HOME` isolation did not isolate WebView local storage, allowing unrelated recent workspaces into a recording. | **VERIFIED (demo fix)** | Demo preparation now assigns a fresh `WEBVIEW2_USER_DATA_FOLDER` inside the demo home; packaged onboarding exposed exactly one current demo recent. |
| P1 | Reviewed Apply executed from one click while `DEMO.md` promised consequence confirmation. | **VERIFIED (runtime fix)** | Run Review now uses a focus-contained, Escape-cancellable confirmation naming the exact path count and package digest; 18 focused browser tests pass and the packaged Desktop required `Apply exact package` before mutation. |
| P1 | Narrated demo publishing remains dependent on approved/licensed local audio assets. | **BLOCKED** | Approved media, retimed captions, and `validate:narrated` pass; otherwise ship an explicitly silent demo only |
| P0 | The 52-second launch video mixed the current Work/History/Setup shell with retired six-destination Flow/Operations captures; the old validator passed the contradiction. | **VERIFIED (silent master)** | Composition, narration, captions, and shot names now use current Work/Run Review; seven unused retired captures were removed; validation rejects retired destinations; the fresh 52-second 1920×1080/30fps BT.709 silent master and transition sheet passed automated and visual review. Narrated publishing remains separately blocked on approved media. |
| P1 | Storybook rendered light semantic state tokens on a dark shell and its catalog advertised retired destinations; current screens also contained near-threshold contrast failures. | **VERIFIED (automated accessibility)** | Story catalog now exposes Work/History/Setup and their states, Storybook fixes the release dark theme, muted/component colors meet AA targets, and all 41 Storybook interaction/accessibility tests pass against the final static build. |
| P1 | Desktop can display runs from one execution domain under another workspace and then request review/recovery using the selected workspace domain. | **VERIFIED** | Run/agent DTOs carry owning domains; Work filters by that identity; History/review/recovery use it directly; two-domain native ownership, 104 browser tests, and final workspace regression pass. |
| P1 | Native snapshot loading silently skips per-domain config/store/run errors and returns success, so omitted evidence can look like a verified empty state. | **VERIFIED** | Per-domain config/store/run/contract/agent diagnostics make the snapshot explicitly partial; native failure tests, Desktop checks, and final workspace regression pass. |
| P1 | Flow can retain and dispatch a previous ready plan after mission/ADE/domain changes or a failed preview, while hiding safety fields and blocker reasons. | **VERIFIED** | Plan-input binding/invalidation and visible safety contract were exercised at 1600×1000 and 390×844; the final 104-test release suite passes. |
| P1 | Approval overlay guesses the related run from repository recency because HITL records lack causal run/agent/effect identifiers. | **VERIFIED** | HITL DTO/audit carries exact domain/run/agent/action identity; UI refuses substitute runs; focused approval and final browser/workspace suites pass. |
| P1 | Browser release tests primarily use fabricated preview data and cannot certify native IPC, PTY execution, package generation, Apply, or recovery. | **VERIFIED (test boundary)** | Packaged Windows E2E using a deterministic local adapter and real repository assertions |
| P1 | Default PTY Stop previously lacked durable live identity and could leave the Windows worker blocked after its process tree exited. | **VERIFIED** | Live PID+creation identity is persisted at spawn; PID reuse is refused; Stop kills and confirms the tree; ConPTY cancellation settles the worker and preserves `cancelled`; exact Windows descendant and final workspace tests pass. |
| P1 | Nonzero or verification-failed upstream tasks were inserted into the dependency-completed map, allowing dependents to consume failed partial output. | **VERIFIED** | Successful-outcome dependency gating, `blocked_by_dependency` evidence, partial-write/nonzero/verifier tests, and final workspace regression pass. |
| P1 | Early agent/wave errors can bypass Race/MCP/cloud/workspace cleanup and dropping `spawn_blocking` futures can orphan children. | **PARTIALLY VERIFIED (fix)** | Agent futures are supervised, wave siblings are drained, lifecycle errors become durable failed results, Race/MCP release is explicit, and cloud sandbox teardown is RAII; acquisition-error/sibling test passes. Panic/process fault injection and retained-workspace cleanup audit remain. |
| P1 | Verification commands previously ran as raw host shells with inherited environment, no timeout, and no permission/network/DeepSpace enforcement. | **VERIFIED** | Dedicated verifier receipt, minimal environment, profile/network/HITL gates, process-tree timeout, bounded output, cloud fail-closed behavior, targeted timeout/secret/egress tests, and final regression pass; host filesystem enforcement remains honestly platform-dependent. |
| P1 | Detached dispatch returns a run ID before durable insertion; early startup failure can leave Flow “dispatched” to an ID that status never finds. | **VERIFIED** | Dispatch atomically persists `starting` plus active ownership before returning; detached startup errors settle `failed_startup`; four lifecycle and final workspace tests pass. |
| P1 | Per-domain active state is a racy singleton marker without supervised handles or crash reconciliation. | **VERIFIED** | Locked create-new ownership, PID creation identity, guarded terminal transitions, concurrent-dispatch refusal, crashed-owner reconciliation, and final regression pass. |
| P1 | Flow preview approves dependency-staged overlap but dispatch rejects every overlap, making an approved plan undispatchable. | **VERIFIED** | Dispatch recomputes and requires the exact approved waves while allowing staged overlaps; unit, preview, browser, and final workspace tests pass. |
| P1 | Omitted `fail_fast` deserialized false and failed runs could settle as failed while CLI/MCP returned success. | **VERIFIED** | Default-true compatibility, nonzero terminal-error, and final workspace tests pass after durable settlement. |
| P1 | Fleet runs waves serially and can strand node/fleet catalog state when a node errors. | **PARTIALLY VERIFIED (fix)** | Nodes now dispatch concurrently inside each wave; a durable guard settles unfinished rows on error/cancellation; success and failure/skip tests pass; explicit continue-on-error and cancellation tests remain |
| P1 | Store migrations 2/4/5 previously ignored arbitrary statement errors and still advanced schema version. | **VERIFIED** | Transactional, schema-verified upgrades from every released version plus partial/malformed/falsely-advanced failure fixtures; 24 store tests and the final workspace regression pass. |
| P1 | Public MCP reads could request raw/high-fidelity source under DeepSpace and error envelopes were unsanitized. | **PARTIALLY VERIFIED (fix)** | DeepSpace fidelity cap/raw denial and success+error sanitization tests pass; domain-routing and complete tool-schema audit remain required |
| P1 | Release workflow could publish a partial platform matrix and then publish a cross-platform npm wrapper whose postinstall exited successfully after a missing download. | **VERIFIED LOCALLY / HOSTED RUN PENDING** | Exact five-CLI/one-Windows-Desktop asset gates and fail-hard atomic checksum/version installer are implemented; four package tests, six-file pack inspection, and exact staged install pass. Hosted matrix execution requires publication authority. |
| P1 | CLI/Desktop installers lacked end-to-end integrity checks; updater metadata could be partial. | **VERIFIED LOCALLY / SIGNING PENDING** | CLI installers verify SHA-256 and version before atomic replacement; exact npm tarball install passes; Windows-only updater fixture requires exactly its declared platform. Signed hosted updater rehearsal requires credentials. |
| P1 | Tag/dispatch publication was not gated on tests or protected-main reachability and used mutable Actions. | **VERIFIED BY STATIC/LOCAL GATES / HOSTED RUN PENDING** | Canonical release reuses full CI, checks version/main ancestry, requires exact artifacts/tokens, pins every workflow Action, and removes the bypass Desktop workflow. External negative workflow runs require publication authority. |
| P1 | Hosted Link and cloud-sandbox authentication defaults could fail open when public bind variables existed but auth secrets were omitted. | **VERIFIED LOCALLY / DEPLOYMENT PENDING** | Both services refuse unauthenticated public binds and auth-without-credentials; final workspace includes 22 Link and 6 cloud-sandbox passing tests. Hosted deployment is optional and external. |
| P1 | Web account handoff put a reusable Clerk bearer in a custom-scheme URL that current Desktop intentionally rejects. | **VERIFIED** | Bearer creation/custom link removed and v1.2 limitation shown explicitly; full web lint, production build, and 17-page browser suite pass. |
| P1 | Core, UX/demo, and security/release audits are complete; their P0/P1 findings are being remediated. | **VERIFIED** | Three audit notes under `docs/01-projects/release-audit-*.md` |

## 5. High-priority improvements

### Phase 1 — Audit, reproduce, and lock release scope

- [x] Complete core/reliability, security/release, and UX/demo audits
  (`docs/01-projects/release-audit-*.md`).
- [x] Build a traceable blocker ledger in this file with exact paths, failure
  modes, proposed fixes, and verification commands.
- [x] Establish a deterministic v1.2 mission fixture and both success and
  failure/recovery acceptance scenarios (`examples/`, `tooling/`).
- [x] Run cheap baseline gates to separate existing failures from introduced
  failures.

**Acceptance:** every P0/P1 finding has an owner, state, and exit test; no
substantial code change starts from an unverified assumption.

### Phase 2 — Fix and integrate the release-critical paths

- [x] Fix P0/P1 core truthfulness, state-transition, recovery, security, and
  packaging defects without widening the one-root v1 contract.
- [x] Fix first-run/onboarding/documentation drift and any Desktop dead ends or
  misleading epistemic states.
- [x] Create `DEMO.md` and repeatable demo setup that showcases control,
  approval, evidence, and recovery rather than static screens.
- [x] Update release notes and changelog sections using `[added]`, `[changed]`,
  and `[fixed]`.

**Acceptance:** targeted tests reproduce each fixed defect before or alongside
the fix; the primary workflow and controlled failure are exercised at runtime.

### Phase 3 — Verify, adversarially falsify, and package

- [x] Run the complete local release matrix listed in section 11.
- [x] Verify Desktop at desktop and compact widths with screenshots and actual
  navigation/actions.
- [x] Stage and inspect release artifacts; test clean install and embedded
  version; scan for secrets and development-only material.
- [x] Perform a fresh adversarial audit against the claim “Pytxo is ready for
  public release,” fix important findings, and rerun affected gates.
- [x] Create `RELEASE_READINESS.md` with an evidence-backed READY / READY WITH
  KNOWN RISKS / NOT READY recommendation.

**Acceptance:** no P0/P1 blocker remains open; every release criterion is
verified or explicitly accepted as a bounded risk; no public publish/deploy is
performed without user approval.

## 6. Medium/low-priority improvements

- P2: reconcile stale historical control files (`CHECKPOINT.md`, root
  `PLAN.md`) so maintainers do not mistake v1.0/v1.1 gates for current proof.
- P2: reduce duplicate public-doc surfaces (`apps/docs` vs
  `apps/web/content/docs`) or label the non-canonical surface clearly.
- P2: improve evidence retention/cleanup guidance for review blobs, Apply
  journals, worktrees, logs, and captures.
- P2: verify accessibility basics beyond automated checks: keyboard-only
  approval/review path, visible focus, reduced motion, status announcements,
  and non-colour epistemic state.
- P3: prune clearly obsolete generated release staging only after it is proven
  unreferenced and the user authorizes removal.

## 7. Reliability risks

| Risk | Current assessment | Required evidence |
|---|---|---|
| Review package differs from reviewed bytes | **VERIFIED:** stored manifest/blobs, exact chunks, digest, and three applied paths matched the packaged runtime review | Preserve manifest/blob and packaged-runtime regression |
| Duplicate Apply or double-submit | **VERIFIED (integration):** one contract claim/attempt wins; retries follow durable state | Retain concurrent claim and UI loading guards |
| Interrupted Apply produces false success | **VERIFIED (integration):** journal reconciliation yields committed, rolled back, or recovery required | Retain interruption-boundary and restart tests |
| Affected-path drift is missed | **VERIFIED (packaged runtime):** unrelated drift allowed; affected drift refused with no partial Apply | Preserve both demo rehearsals |
| Scheduler/path ownership collision | **VERIFIED:** Race contention, staged overlaps, exact-wave revalidation, and fleet/domain tests pass | Retain load and multi-root coverage |
| Failed upstream output reaches dependents | **VERIFIED:** dependents receive `blocked_by_dependency`; independent later work may continue | Retain partial-write/nonzero/verifier fixtures |
| Stop does not stop a live default PTY | **VERIFIED (Windows runtime):** durable identity, descendant kill, exit confirmation, and `cancelled` state | Keep exact process-tree test |
| Startup returns an orphan run ID | **VERIFIED:** durable `starting`, failed-startup settlement, ownership refusal, crash reconciliation | Keep four lifecycle tests |
| Resource leak on early error | **PARTIALLY VERIFIED:** futures are supervised, siblings drained, explicit cleanup and RAII teardown pass acquisition-error coverage | Additional panic/process fault injection is accepted P2 work |
| Failed migration claims schema success | **VERIFIED:** transactional upgrade matrix and malformed/partial/falsely-advanced fixtures pass | Retain 24 store tests |
| PTY/subprocess/platform behavior diverges | **VERIFIED for Windows primary scope:** both backends and PTY/Stop paths pass; Unix CLI builds remain hosted | Keep claims platform-bounded |
| Durable state/cursor becomes stale | **VERIFIED:** restart, cursor reset, active-owner, persisted History, and packaged runtime evidence pass | Retain reconnect tests |
| Cleanup removes evidence too early | **PARTIALLY VERIFIED:** review packages survive workspace removal and runtime History retained evidence | Broader retention/cleanup policy remains P2 documentation work |

## 8. Security risks

| Risk | Current assessment | Required evidence |
|---|---|---|
| Secret leakage into logs/MCP/receipts | **VERIFIED WITH LIMIT:** sanitizer/MCP/error/child-env tests pass; regex sanitization remains best effort | Retain representative secret fixtures and bounded claims |
| Permission receipt overstates enforcement | **VERIFIED:** persisted four-surface receipt and UI mapping expose enforced/advisory/unavailable/bypassed; packaged ProjFS mechanism matched | Retain platform-specific assertions |
| Unsafe path/symlink/special-file Apply | **VERIFIED (integration):** traversal, control paths, ancestors, symlinks, blob mutation, and drift fail closed | Retain Apply/change-set suite |
| Trust/config/profile escalation | **VERIFIED:** every root and agent is capped by explicit folder trust and fresh entitlement ceiling | Retain project-root/offline tests |
| Per-agent trust-ceiling bypass | **VERIFIED:** repository Supernova overrides cannot exceed Orbit trust, including untrusted secondary roots | Retain absolute ceiling tests |
| Shell/agent command injection | **VERIFIED WITH INHERENT RISK:** ADE prompt text is not interpolated; child environment is allowlisted; configured commands remain authorized executable input | Preserve config/argument tests and permission profiles |
| Verifier bypasses permission boundary | **VERIFIED WITH PLATFORM LIMIT:** separate receipt, minimal env, timeout, network/HITL/profile gates; host filesystem boundary can remain advisory | Preserve verifier boundary tests and honest receipts |
| DeepSpace MCP raw-read bypass | **VERIFIED:** permission-engine routing caps fidelity and denies raw override; success/error envelopes sanitize | Complete schema/domain inventory remains P2 |
| Supply-chain or committed secret exposure | **VERIFIED WITH ACCEPTED WARNINGS:** 0 unignored Rust vulnerabilities, 0 production npm vulnerabilities, tracked matches are synthetic fixtures | Preserve audit/scan and exact artifact inspection |
| Updater/signing ambiguity | **VERIFIED LOCALLY / SIGNING BLOCKED:** Windows-only manifest gates pass; signing requires external key | Never claim signed updater without hosted signed artifacts |
| Unsigned entitlement mutation | **VERIFIED:** missing/invalid/stale signatures and unbound identity fail closed; replay/idempotency tests pass | Production migration remains deployment evidence |
| Secret-bearing cloud upload | **VERIFIED LOCALLY:** dual consent, manifest, path/content/symlink/special-file denial, and policy-vs-transport tests pass | Live hosted sync/exec remains optional external evidence |

## 9. UX/demo problems

- **VERIFIED:** onboarding and public copy use the v1.2 commit-boundary thesis,
  Work/History/Setup navigation, and Windows-first Desktop scope.
- **VERIFIED:** desktop and compact browser suites cover loading, empty, error,
  stale, success, approval, Stop, review, and epistemic-state behavior; the
  packaged app exercised first-run, workspace switching, Work, Run Review,
  explicit Apply confirmation, stale refusal, and History.
- **VERIFIED:** verified/claimed/unknown/refuted styling and persisted receipt
  mechanisms are tested; advisory enforcement is not rendered as mechanical
  proof.
- **VERIFIED:** `DEMO.md` provides the 2–4 minute reproducible live narrative;
  the separate 52-second silent master uses only current Work/Run Review assets.
- **BLOCKED:** narrated publishing requires approved/licensed audio. The silent
  master is the only release-ready video claim.

## 10. Testing gaps

- Hosted five-platform CLI jobs, signed updater publication, production
  PostgreSQL migration, and live external cloud-sandbox execution remain
  unverified because they require external infrastructure or credentials.
- Browser/Storybook suites use `PreviewDesktopBackend`; native commit behavior
  is supported separately by the two packaged Desktop rehearsals and repository
  assertions, not inferred from those preview tests.
- Apply interruption, rollback, recovery-required, duplicate claim, and
  affected-path drift have integration coverage; only stale-path refusal was
  exercised through the packaged UI in this release rehearsal.
- Panic/process fault injection after every possible acquired resource and
  exhaustive MCP tool-schema/domain routing remain P2 hardening opportunities.

## 11. Verification requirements

Run from the repository root unless the command changes directory. Record
exact result, duration, and relevant artifact/log in `RELEASE_READINESS.md`.

### Repository and Rust

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p pytxo-cli --release
cargo build -p pytxo-mcp --release
cargo run -p pytxo-cli -- status --json
cargo run -p pytxo-cli -- --version
git diff --check
```

### Desktop

```powershell
Push-Location apps/desktop
npm ci
npm run check
$env:PLAYWRIGHT_CHANNEL='chromium'
npm run e2e
npm run storybook:build
npm run storybook:test:ci
npm run build:native
Pop-Location
cargo test -p pytxo-desktop
```

### Website and public docs

```powershell
Push-Location apps/web
pnpm install --frozen-lockfile
pnpm run lint
pnpm run verify:product-assets
pnpm run check:links
pnpm run build
pnpm run e2e -- e2e/marketing.spec.ts --workers=1 --reporter=line
pnpm audit --prod --audit-level=high
Pop-Location
```

### Demo

```powershell
Push-Location apps/demo-video
npm ci
npm run typecheck
npm run compositions
npm run test:audio-qa
npm run validate:publishing
npm run stills
npm run poster
npm run render:silent
npm run validate:silent
npm run transition-sheet
Pop-Location
```

Narrated validation is required only if narrated media is claimed or published;
otherwise the limitation must be explicit.

### Release artifact and security checks

- Verify version parity across Cargo, CLI binary, npm wrapper, Tauri config,
  installer names, release notes, updater metadata, and website download data.
- Scan tracked source and staged artifacts for secrets and absolute developer
  paths.
- Install the staged npm wrapper and Windows CLI into a clean temporary prefix;
  run `pytxo --version`, `pytxo doctor`, and the guided dry run.
- Install and launch the staged Windows Desktop artifact; exercise first-run,
  workspace trust, Work, approval, Run Review, Apply, History, and Setup.
- Verify checksums from the exact staged bytes.

## 12. Demo requirements

- Create root `DEMO.md` with a rehearsable 2–4 minute sequence.
- Use a deterministic, dependency-light workspace with a passing baseline.
- Narrative: unsafe direct autonomy problem -> mission proposal -> isolated
  execution and ownership -> human review/authorization -> exact Apply ->
  independent post-state evidence -> honest failure/recovery variant.
- Include narration, exact clicks/commands, expected visible states, fallback
  steps, recording checklist, shots/scenes, and short release/social copy.
- Prefer tooling that resets and validates the fixture; never rely on hidden
  cached state or fabricated UI data.
- Rehearse once from a clean fixture and retain screenshots/log evidence.

## 13. Documentation requirements

- Reconcile `README.md`, `RELEASE.md`, `CHECKPOINT.md`, `PLAN.md`, public docs,
  release notes, config example, and guided example with v1.2 behavior.
- Quick-start commands must be executed exactly as written.
- Public docs must distinguish the shipping repository boundary from the
  production-effect north star and state Windows-first Desktop limits honestly.
- Document required tools, agent/provider authentication boundaries, folder
  trust, permission profiles, artifact signing/updater limits, and recovery
  operator actions.
- Add `DEMO.md` and `RELEASE_READINESS.md`; link them from maintainers’ release
  material where useful.

## 14. Release checklist

### Product and engineering

- [x] Primary mission -> review -> Apply -> evidence workflow is **VERIFIED**.
- [x] Controlled Apply failure/recovery is **VERIFIED**.
- [x] No known P0/P1 defect remains in the claimed local/Windows-first scope.
- [x] Critical state transitions fail closed and diagnostics are actionable.
- [x] Receipts distinguish attempted, committed, verified, and recovered state.

### UX and onboarding

- [x] Fresh user can install, trust a workspace, run the guided scenario, review
  exact changes, authorize Apply, and find the receipt/history.
- [x] Desktop navigation and all critical empty/loading/error/success states are
  **VERIFIED** at desktop and compact widths.
- [x] Public copy and README match actual behavior and platform availability.

### Build, test, security, and distribution

- [x] All locally executable section 11 commands pass on the final tree.
- [x] Fresh staged install and embedded version parity pass.
- [x] Tracked secret/development-artifact scan passes; matches are synthetic fixtures.
- [x] Release notes, changelog, checksums, installer metadata, and updater policy
  are coherent.
- [x] Silent demo master matches the claimed publishing state; narration remains blocked.

### Final audit and authority

- [x] Fresh adversarial audit completed after implementation.
- [x] Audit findings fixed and affected gates rerun.
- [x] `RELEASE_READINESS.md` contains the honest final recommendation.
- [x] User approval obtained before external tag, npm publish, upload,
  deployment, updater-channel change, and public release.

## 15. Completed work

| Date | Work | State | Evidence |
|---|---|---|---|
| 2026-08-30 | Read project instructions, checkpoint, prior release plan, canonical vision, commit-layer contract, release workflow, v1.2 notes, and existing demo/guided example. | **VERIFIED** | Repository files inspected on `release/1.2.0` at merge head `f31a75a` (first-parent feature commit `a0137a2`) |
| 2026-08-30 | Established v1.2 release control document and three-phase acceptance plan. | **IMPLEMENTED** | `RELEASE_PLAN.md` |
| 2026-08-30 | Dispatched independent core, security/release, and UX/demo audits. | **IMPLEMENTED** | Audit notes pending under `docs/01-projects/` |
| 2026-08-30 | Ran cheap current-tree baseline gates. | **VERIFIED** | `cargo fmt --all -- --check`; Desktop `npm run check` (0 errors/warnings plus stylelint); web `pnpm run lint`; demo `npm run typecheck` — all exit 0 |
| 2026-08-30 | Exercised the immutable review package and low-level Apply/recovery contract. | **VERIFIED** | `cargo test -p pytxo-runner --test run_change_set` — 32 passed, 0 failed |
| 2026-08-30 | Exercised the orchestration-level reviewed Apply contract. | **VERIFIED** | `cargo test -p pytxo-orchestrate --test run_apply` — 9 passed, 0 failed |
| 2026-08-30 | Checked shared source version parity for the target. | **VERIFIED** | `node tooling/scripts/verify-release-version.mjs 1.2.0` — Rust, Desktop, Tauri, demo, npm all 1.2.0 |
| 2026-08-31 | Reproduced a Windows smoke false success: folder trust rejected the run, `status` showed no runs, but the script printed `OK` and exited 0. Added checked native invocations and explicit trust to Windows smoke/benchmark paths. | **IMPLEMENTED / PARTIALLY VERIFIED** | Corrected echo smoke completed on the installed CLI; current-tree release binary and deliberate-failure checks remain |
| 2026-08-31 | Completed Desktop/web/demo UX release audit. | **VERIFIED** | `docs/01-projects/release-audit-ux-demo.md`; Desktop check + 100 browser tests + 19 native tests, web lint/link/assets/build, and demo static gates passed in the audit; packaged-native behavior remains unverified |
| 2026-08-31 | Completed Rust core/reliability release audit. | **VERIFIED** | `docs/01-projects/release-audit-core.md`; focused core package suite passed, including 32 change-set and 9 Apply tests; 10 ordinary lifecycle P1 blockers remain |
| 2026-08-31 | Completed security/release/distribution audit. | **VERIFIED** | `docs/01-projects/release-audit-security-release.md`; three P0 security boundaries and five P1 release/distribution blockers identified with primary-source/runtime evidence |
| 2026-08-31 | Fixed dependency outcome and failed-run false-success semantics. | **VERIFIED (targeted)** | `cargo fmt --all -- --check`; 11 core config tests, 2 dependency-outcome tests, and 1 durable failed-run integration test passed |
| 2026-08-31 | Ran the complete affected runner/orchestrator regression suite after the failure-semantics changes. | **VERIFIED** | `cargo test -p pytxo-runner -p pytxo-orchestrate` — all unit and integration targets passed, including 32 change-set tests and 9 Apply tests |
| 2026-08-31 | Made Flow preview authority input-bound, exposed its safety contract, distinguished settled from successful waves, and corrected profile-dependent containment copy. | **PARTIALLY VERIFIED** | Desktop `npm run check` passed with 0 diagnostics; live production-preview browser exercised onboarding and plan construction at 1600×1000 and 390×844; screenshots in `apps/desktop/output/playwright/` (local verification artifacts) |
| 2026-08-31 | Reproduced and fixed narrow-shell header collisions, clipped Flow selectors, misleading final onboarding copy, and missing Desktop favicon. | **VERIFIED (browser)** | Live browser before/after at 390×844; fixed screenshot `apps/desktop/output/playwright/flow-plan-mobile-fixed-390x844.png`; production Vite build passed and the post-reload favicon console error disappeared |
| 2026-08-31 | Implemented candidate folder-trust ceiling enforcement across global, per-agent, project-root, receipt, stdin, and MCP paths. | **PARTIALLY VERIFIED** | Focused core/runner/orchestrator tests and `-D warnings` clippy passed in the implementation workstream; independent review and final workspace regression pending |
| 2026-08-31 | Implemented candidate fail-closed cloud egress with dual consent, protected paths, secret-content checks, hash manifest, and cache/sync/delta tests. | **PARTIALLY VERIFIED** | Core/runner/cloud integration tests and focused `-D warnings` clippy passed in the implementation workstream; independent review and final workspace regression pending |
| 2026-08-31 | Implemented fail-closed Paddle secret/signature freshness/event allowlist/transactional replay controls. | **PARTIALLY VERIFIED** | `cargo test -p pytxo-link` — 14 passed and focused clippy passed; audit found server-side subscription/customer identity binding and duplicate web webhook bypass still unresolved, so P0 remains open |
| 2026-08-31 | Adversarially reviewed the candidate trust/cloud P0 fixes instead of accepting focused green tests. | **VERIFIED (review)** | Review reproduced remaining untrusted-secondary-root and offline-org-ceiling privilege paths, cloud policy fallback, symlink TOCTOU, and incorrect remote working-directory risks; remediation reopened |
| 2026-09-01 | Closed the remaining folder/org trust-ceiling bypasses. | **VERIFIED (targeted)** | `cargo test -p pytxo-orchestrate --test project_run` — 5 passed including trusted-primary/untrusted-secondary rejection; 8 entitlement tests passed including offline/expired-ceiling Supernova denial |
| 2026-09-01 | Closed the reviewed cloud policy-fallback, file-handle, and remote-working-directory defects. | **PARTIALLY VERIFIED** | 68 core tests, 46 runner tests, and 6 cloud HTTP tests passed; symlink fixtures and policy-denial/no-local-fallback tests pass; live hosted sync→exec remains pending |
| 2026-09-01 | Completed Paddle server-side price/identity/replay binding and removed the web-side provisioning bypass. | **VERIFIED (targeted)** | `cargo test -p pytxo-link` — 21 passed; duplicate delivery is an idempotent success; production Postgres migration rehearsal remains pending |
| 2026-09-01 | Made Flow zero-verification plans undispatchable, made unknown agent states non-terminal, tied Stop consequence copy to the enforcement receipt, and preserved compact status evidence. | **VERIFIED (automated)** | Desktop check passed with 0 diagnostics; `epistemic-state.spec.ts` — 12 passed; final compact screenshot rerun remains pending |
| 2026-09-01 | Hardened npm/direct installers and consolidated the public release path. | **PARTIALLY VERIFIED** | npm installer tests — 4 passed including checksum rejection and atomic replacement; `npm pack --dry-run` contains exactly 6 intended files; workflows use immutable upstream SHAs, exact matrices, version/ancestry/full-CI gates, and no alternate Desktop publisher; hosted/Unix runs pending |
| 2026-09-01 | Made the default PTY lifecycle durably stoppable and PID-reuse safe. | **VERIFIED (targeted)** | Spawn-time PID+Windows creation identity persistence; idempotent already-exited handling; `/T /F` descendant termination; explicit in-process cancellation settlement; `stop_exact::dispatched_blocking_run_stays_cancelled_after_exact_stop` passes with a real PowerShell descendant and durable `cancelled` state |
| 2026-09-01 | Bounded and evidenced the verifier execution boundary. | **VERIFIED (targeted)** | Runner unit suite — 50 passed, including verifier secret stripping, receipt emission, timeout/tree termination, cloud fail-closed, and Orbit egress rejection; dependency failure integration tests — 2 passed |
| 2026-09-01 | Made SQLite schema upgrades transactional and schema verified. | **VERIFIED (targeted)** | Store unit suite — 24 passed, including upgrades from versions 0–6, existing-column recovery, rollback on missing tables, malformed schema refusal, and falsely advanced version refusal |
| 2026-09-01 | Capped DeepSpace MCP reads and sanitized MCP error envelopes. | **VERIFIED (targeted)** | Orchestrator DeepSpace raw/high-fidelity denial fixtures compile; `cargo test -p pytxo-mcp` — 2 sanitization tests passed; complete MCP schema/domain audit remains open |
| 2026-09-01 | Removed reusable bearer tokens from Desktop custom-protocol handoff and made hosted service binds fail closed. | **VERIFIED (targeted)** | Focused web ESLint passed; Link startup matrix included in 22 passing tests; cloud-sandbox startup matrix included in 6 passing tests; public bind without auth and auth without credentials are rejected |
| 2026-09-01 | Added a deterministic real commit-boundary demo fixture and operator script. | **PARTIALLY VERIFIED** | `DEMO.md`; fixture baseline tests 2 passed, controlled agent mutation tests 3 passed, exact three-file diff and `git diff --check` passed; real package/review/Apply rehearsal remains open |
| 2026-09-02 | Made detached dispatch durably visible and single-owner before returning. | **VERIFIED (targeted)** | `dispatch_lifecycle` — 4 passed: immediate `starting`, startup failure settlement, concurrent owner refusal, and crashed-supervisor reconciliation |
| 2026-09-02 | Made fleet waves concurrent and fleet/node settlement fail closed. | **VERIFIED (targeted)** | `fleet_run` — 2 passed: cross-repository wave success and durable failed/skipped settlement; failed fleet commands return an error even when continuation is requested |
| 2026-09-02 | Bound Desktop run/agent/review operations to their exact execution domain and exposed partial snapshots. | **VERIFIED (targeted)** | Desktop native IPC tests — 21 passed; `npm run check` — 0 errors/warnings; focused browser approval/Stop flows — 2 passed; prior combined browser run — 69/70 before corrected scoped-focus expectation |
| 2026-09-02 | Added causal approval identity and removed recency-based substitute evidence. | **VERIFIED (targeted)** | HITL audit/DTO carries domain, run, agent, request, action and decision; UI shows only the exact run; focused approval browser test passes |
| 2026-09-02 | Reconciled staged-overlap preview and dispatch policy. | **VERIFIED (targeted)** | Orchestrator unit suite — 28 passed including exact-wave revalidation; Flow integration suite — 9 passed including staged-overlap preview |
| 2026-09-02 | Ran the full Rust workspace test gate after integrating all targeted fixes. | **VERIFIED** | `cargo test --workspace` — exit 0; all workspace unit, integration, and doc-test targets passed, including 32 change-set, 9 Apply, 4 dispatch-lifecycle, 3 dependency-outcome, 3 Stop, 22 Link, and 6 cloud-sandbox tests |
| 2026-09-02 | Ran the complete Desktop browser release and development suites. | **VERIFIED (browser)** | Production-preview release suite — 104 passed; development-only legacy-deck suite — 1 passed after making its checked-in Chrome-channel contract match release/Storybook; config contract rerun — 4 passed |
| 2026-09-02 | Closed the web dependency-lock and production-audit gate. | **VERIFIED** | Updated explicit `nanoid`/`browserslist` overrides to patched compatible releases; frozen install passes; production audit reports no known vulnerabilities; lint, 27-asset parity, 153-link check, Next production build (58 static pages), and all 17 browser tests pass. |
| 2026-09-02 | Built the native Windows Desktop release candidate. | **VERIFIED (build)** | Frontend production build transformed 6,366 modules; `cargo build -p pytxo-desktop --release --features custom-protocol` completed successfully; subsequent packaged runtime rehearsals are recorded below. |
| 2026-09-02 | Audited, corrected, rendered, and inspected the release-demo media. | **VERIFIED (silent master)** | Demo clean install and dependency audit pass with zero known vulnerabilities; typecheck/composition/asset/audio-parser/publishing validators pass; the rendered 52.000-second 1920×1080/30fps BT.709 silent master passes validation; the seven-shot contact sheet and generated transition sheet show one coherent current Work/Run Review shell. |
| 2026-09-02 | Exercised the npm wrapper release contract. | **VERIFIED (targeted)** | Four installer/platform/checksum/atomic-replacement tests pass; `npm pack --dry-run` contains exactly six intended files; the packed clean-prefix result is recorded below. |
| 2026-09-02 | Installed the real packed npm artifact from a staged release. | **VERIFIED** | `pytxo-1.2.0.tgz` installed into a clean prefix through the production postinstall download/checksum/version path; the wrapper reported `pytxo 1.2.0`, returned the isolated completed demo run, and quick doctor passed all five repository/directory checks. |
| 2026-09-02 | Rehearsed the complete commit boundary through the packaged Windows Desktop. | **VERIFIED** | Run `54924278-5e91-40d0-a7de-6f8e99cde4a3`, package `072f954fcaa27bc0c9d36e561139614223cac176e3871857ea1f423a4165f3b0`; Work showed the persisted agent/wave and ProjFS receipt, Run Review showed exactly three modified paths, explicit confirmation gated Apply, unrelated `operator-note.txt` survived, checkout tests passed 3/3, and History recorded one committed attempt. |
| 2026-09-02 | Rehearsed affected-path drift refusal through the packaged Windows Desktop. | **VERIFIED** | Run `47da6a06-a3f3-4b66-8425-838efd8a7809`, package `30b29840dc68225b7a521ce0b78ca144d7a65c8555bc71af39f781f784e086a9`; an operator edit to `src/risk-policy.mjs` caused explicit stale-review refusal, README/test were untouched, baseline tests passed 2/2, and History recorded Apply failed with no success receipt. |
| 2026-09-02 | Closed final dependency-audit findings without regressing the execution yard. | **VERIFIED** | Rust minimum is 1.88; Ratatui 0.30.2 resolves to patched `lru 0.18.3`; optional Linux FUSE resolves to patched `fuser 0.16.0`; an attempted `portable-pty 0.9` upgrade reproduced a PTY hang and was rejected, with 0.8 restored and its environment/Stop tests passing. `cargo audit` reports 0 unignored vulnerabilities. |
| 2026-09-02 | Reproduced and removed a parallel Windows HTTP test race found by the final gate. | **VERIFIED** | Accepted mock sockets now restore blocking mode, read complete requests, send complete close-delimited responses, and wait for client EOF; the complete 69-test core suite passed 10 consecutive times and the final workspace suite passed. |
| 2026-09-02 | Ran final integrated Rust/repository gates on the candidate. | **VERIFIED** | `cargo fmt --all -- --check`, `cargo test --workspace`, warnings-denied workspace clippy, optimized CLI/MCP builds, live status JSON, `pytxo 1.2.0`, and `git diff --check` all exit 0. |
| 2026-09-02 | Rebuilt and restaged final Windows/npm artifacts. | **VERIFIED** | Desktop check reports 0 errors/warnings and CSS lint passes; final frontend/native release build passes. Exact six-file npm tarball installed to `target/final-package-install-20260902-155602-514`, downloaded rebuilt digest `aab81760200de903cb06c35f3a5e324bdfbf241ee1e5fa6a882c131b09517bda`, reported 1.2.0, and passed all five quick-doctor checks. |
| 2026-09-02 | Completed final adversarial release audit. | **VERIFIED** | Dependency reachability, ignored advisories, tracked secrets/paths, generated artifacts, runtime truth, stale refusal, installer integrity, and flaky gates were challenged; important findings were fixed and affected gates rerun. Recommendation is READY WITH KNOWN RISKS. |
| 2026-09-03 | Retargeted the candidate from the already-published v1.2.0 tag/package to v1.2.1 and reran the complete local release gates. | **VERIFIED** | Version parity, formatting, full Rust tests, warnings-denied clippy, optimized CLI/MCP builds, live CLI version/status, Desktop checks/native build, web lint/links/assets/build, npm tests/dry-pack, demo typecheck/render validation, and repository whitespace checks passed. |
| 2026-09-03 | Recorded the Dodo MoR / MBCZ account / Pytxo brand model and a fail-closed provider migration design. | **IMPLEMENTED (design only) / UNVERIFIED (runtime)** | Proposed `ADR-0040` and `docs/01-projects/dodo-mor-integration.md`; no provider cutover, credentials, or entitlement mutation path changed in v1.2.1. |
| 2026-09-04 | Merged and published v1.2.1 through protected main, then deployed the website and Link/cloud/proxy services. | **VERIFIED** | PR #29; CI run `33731373527` passed 12/12 jobs; release run `33781868097` passed; npm `pytxo@1.2.1`, GitHub private/public releases, Vercel production, and all three Railway health endpoints were verified. |
| 2026-09-04 | Detected and repaired a v1.2.1 public asset collision during a fresh post-publication audit. | **VERIFIED (repair) / ACCEPTED AS MUTABLE HISTORY** | Public Windows CLI and five-entry `SHA256SUMS.txt` were restored byte-for-byte from the private release and independently downloaded/rehashed. Because v1.2.1 assets changed after publication, v1.2.2 is required for immutable provenance. |
| 2026-09-04 | Implemented v1.2.2 clean-staging and exact-inventory publication controls. | **IMPLEMENTED / PARTIALLY VERIFIED** | Stale tracked `dist/` files removed; root staging ignored; signed, unsigned, stale-file, empty-file, and missing-updater regression cases pass; expanded 27-surface version parity passes; hosted CI/release pending. |
| 2026-09-04 | Ran the complete local v1.2.2 corrective release gate. | **VERIFIED (local)** | `cargo fmt`, full workspace tests, warnings-denied clippy, optimized CLI/MCP builds, live v1.2.2 status/version, Desktop check/native build, web lint/links/assets/build, npm tests/dry-pack, demo static checks, workflow lint, release regressions, and whitespace checks exited 0. |
| 2026-09-04 | Removed the release icon pipeline's abandoned `to-ico` dependency and vulnerable legacy transitive graph. | **VERIFIED** | A local PNG-in-ICO encoder has deterministic header/offset/input tests; Desktop and web generators completed; generated assets passed quality checks; clean tooling install and audit reported 0 vulnerabilities. |

## 16. Remaining work

1. Finish the v1.2.2 local release gate, merge only after protected-hosted CI is
   green, and run the corrected five-platform CLI plus Windows MSI/updater
   release workflow from the merge commit.
2. Independently download and verify the exact v1.2.2 private/public assets,
   checksums, embedded versions, updater manifest/signature, npm package,
   website deployment, and all hosted-service health responses.
3. Add approved/licensed narration and rerun narrated validation only if a
   narrated master is to be published; otherwise ship the verified silent master.
4. Implement the proposed MBCZ/Dodo adapter only after approved Dodo test
   credentials, brand/product IDs, gateway ownership, and a migration window exist.

## 17. Known risks intentionally accepted for v1

These are bounded product limits, not verified release exceptions. They remain
accepted only if public copy and receipts state them accurately:

- Reviewed Apply covers Orbit and Galaxy, one execution domain, and one
  repository root; no cross-root transaction or partial-file acceptance.
- The journal is designed for process-crash reconciliation, not claimed
  power-loss ACID or cross-filesystem atomicity.
- Orbit/Galaxy isolation can be advisory on platforms without kernel-enforced
  filesystem/network boundaries; receipts must expose the actual mechanism.
- Supernova is intentionally host-direct and does not get a reviewed Apply.
- Desktop is Windows-first for v1.2; unbuilt older macOS/Linux artifacts must
  not be relabeled.
- Regex sanitization is best effort, not an authorization boundary or a proof
  that arbitrary agent output contains no secret.
- Optional cloud/Ultra services are capability-gated and are not part of the
  default local v1 release claim.
- A silent demo may ship if approved/licensed narration media is unavailable;
  narrated-demo readiness must then remain **BLOCKED**, not implied.
- The MBCZ/Dodo work in this release is an architecture record only. Paddle
  remains the current Link webhook adapter until the documented dual-run and
  rollback gates are verified.
- `portable-pty 0.8.1` retains an unmaintained `serial` transitive dependency;
  the maintained 0.9 upgrade was tested and rejected because it hung a real PTY
  lifecycle test. No RustSec vulnerability is attached to `serial`.
- RustSec informational warnings remain for Linux-only GTK3/Tauri dependencies,
  build-time Unicode crates, and an unenabled yanked `spin` dependency. v1.2
  publishes only Windows Desktop; the CLI release graph has no unignored known
  vulnerability.
- Windows notifications retain ignored `quick-xml 0.37.5` denial-of-service
  advisories, but that parser receives only Pytxo-generated notification XML,
  not remote/untrusted XML. The ignored `rsa` advisory is absent from the
  Windows target graph and Pytxo does not use it for key handling.
- GitHub Dependabot still reports alerts from development and legacy manifests;
  current shipped web production dependencies and the release icon tooling audit
  clean. Remaining alerts are tracked honestly and are not represented as a
  zero-alert repository state.

## 18. Release operation status

- **VERIFIED:** v1.2.1 passed hosted CI/release and was deployed. Its public
  Windows CLI/checksum collision was repaired, but immutable provenance is not
  claimed for that tag.
- **VERIFIED (local) / PENDING (hosted):** v1.2.2 parity covers 27
  release-facing source, lockfile, installer, web, docs, and notes surfaces.
  Exact-inventory and icon regressions, full Rust tests/clippy/builds, Desktop
  native build, web build, npm package tests, demo checks, workflow lint, and
  whitespace checks pass locally.
- **VERIFIED:** npm, public-release mirror, and Tauri updater-signing secrets
  exist in the private repository. No secret value was read or logged.
- **BLOCKED:** Windows Authenticode and local Git signing keys are not
  configured. Do not describe the MSI, commit, or tag as code-signed.
- **IN PROGRESS:** v1.2.2 full local gate, protected source merge, hosted release
  matrix, fresh-tag publication, independent artifact audit, and production
  website/services redeployment.

## Do not touch without explicit authority

- Accepted ADR contents; supersede with a new ADR if architecture must change.
- User-owned untracked captures, verification data, logs, or notes until they
  are classified.
- Credentials, signing keys, licensed media, and external-account settings.
  Publication authority does not authorize changing account security settings
  or inventing missing signing material.
