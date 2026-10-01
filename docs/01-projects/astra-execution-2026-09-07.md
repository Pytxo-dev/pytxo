---
title: ASTRA product improvement decisions and evidence
slug: astra-execution-2026-09-07
status: active
tags: [project, desktop, verification, research]
audience: [human, agent]
layer: orchestration
created: 2026-09-07
updated: 2026-09-10
related: [[beta-readiness-plan-2026-09-05]], [[beta-competitor-research-2026-09-05]]
---

# ASTRA execution

## Current ownership/Git repair package — September 10

[[astra-native-finish-2026-09-10]] is the current implementation and acceptance note.
MSI `9676d38f…` / extracted EXE `e0eeb950…` contains recorded runtime worker labels
and console-free background Git commands. All 361 frozen/current inputs match;
build, MSI import inspection, 65 affected UI tests, 193 Rust library tests,
21 integration checks, strict Clippy and formatting passed. Binaries are unsigned.
The initial fast-verifier timeout is retained; its cause has not been proven.

The preceding 6acc package completed native Cancel/Apply/restart with 11 project
and 26 independent checks. Its build-specific Bench record is
`tooling/benchmarks/results/astra-native-review-2026-09-10.json`; it cannot certify
the repaired executable. After the latest physical Escape, e0ee native acceptance
and final-build video remain unexecuted. The Windows packet and source handoff
are under `target/astra-native-finish-20260910/`. Clean Windows, current-source CI
and public download remain open. Game-like mission/crew and Focus/Control mockups
remain proposals. No Actions, spending, account changes or publication occurred.

## Preserved native follow-up — September 10

The f968 host mission passed Cancel, exact Apply, 11 project tests, 26 independent
checks and restart. Its 94-second actual-native walkthrough retains and labels
the netsh startup console found by denser frame review. The follow-up read-only
query repair passed a genuine detached-parent red-to-green regression, all 77
runner tests, strict Clippy and formatting. MSI 7d30141b… / EXE 8910d153… passed
build/dependency inspection; all 359 frozen inputs match. Only network_isolation.rs
differs from f968, and the frontend retains its 157-test evidence.

Physical Escape stopped native control before 891 launched. Wait for explicit
resumption before using native tools or capturing again. Final native proof and
smooth demo remain open for those bytes. The clean-Windows packet is ready with all 21 ZIP/ISO entries verified; guest RAM/direct owner sign-in remain practical constraints. PR #31's
12 green jobs cover eb5f73d, not current dirty source. The read-only billing check
still shows zero net charges and the $0 stop budget, without a remaining-allowance
field. Public release remains v1.2.1. No Actions or publication occurred.
See [[astra-evidence-2026-09-07]] and CHECKPOINT for exact artifacts and next steps.

## Preserved selected increment — September 9, before native resumption

The current candidate includes the September 9 menu/workflow pass: MSI
`26ef95bc…`, extracted executable `073a1a30…`, with 359 frozen source inputs.
It adds the native background-probe console repair. Local build, packaged
dependency inspection, 26 Desktop library tests and strict Clippy passed; the
preceding 154 browser tests / 42 stories / 64 screenshots cover unchanged frontend
inputs. It has not completed native launch, recording or clean-Windows acceptance.
The predecessor fcd reached onboarding/Work before physical Escape; it supplied
the console-flash reproduction, not completed mission evidence. The e775/7abe polish and c77 mission
and film remain historical evidence. See [[astra-desktop-menus-2026-09-09]] for
current UI decisions and before/after evidence, and [[astra-evidence-2026-09-07]]
for exact identities and check scope. The refreshed 21-file Windows packet is
`target/astra-ux-20260909/windows-validation-probes/pytxo-windows-local-validation.zip`
(SHA256 `b198756b…`); preparation is not installation.

| Remaining outcome | Current evidence and next action |
| --- | --- |
| Final-build native workflow and demo | Native control attached, then physical Escape stopped it again. The repaired payload is unlaunched. Recordly HUD targeting failed; fallback frames establish static framing only. After explicit resumption, verify a timestamped concurrent UI/capture transition, then the exact-build mission and DEMO plan. |
| Clean Windows validation | New 21-file packet staged; existing guest history is explicit. Run the retained install/workflow/recovery protocol when guest access is available. Prior guest sign-in execution was rejected by tool policy; do not bypass it. |
| Current-source CI | Twelve green jobs cover `eb5f73d`, before the polish. Preserve the existing PR31 authorization and included-allowance boundary; consolidate after native proof to avoid redundant runs. |
| Distribution | No publication approval or anonymous-download evidence for this candidate. Use [[astra-release-proposal-2026-09-07]]; source-equivalent release rebuilds need their own artifact validation. |
| Larger customization proposal | Focus/Control is reviewable but unapproved. Existing Work/History/Setup polish is implemented; no preview-only feature is represented as shipping. |

No further runtime architecture or platform expansion is selected. The research,
competent direct baseline, unsuccessful native attempts and narrow two-job pilot
plan remain in [[astra-product-decisions-2026-09-07]]. New implementation must
address an observed gap, not prolong work while an input request is unanswered.

## Earlier implementation — clean Windows runtime dependency

The actual clean guest installed the frozen MSI with exit 0, then failed its
first Start-menu launch with missing `MSVCP140.dll`. This reopens packaging;
it does not invalidate the earlier development-host mission within its scope.
The replacement now builds with a consistent Windows x64 static CRT using an
opt-in Cargo configuration and explicit target; ordinary developer builds retain
their defaults. The release check of every executable/DLL extracted from the
actual MSI passes, alongside 41 tooling and 12 native voice contract tests.
Replacement MSI `1f47f34d673b…` installed in the preserved guest with exit 0 and
rendered its Start-menu welcome screen on continuation. The guided example then
exposed an unhelpful missing-Git error. The corrected preflight now passes four
native tests and four browser journeys; Desktop check and distribution-feature
Clippy also pass. Final MSI `cf8db2e0b41c…` is built and packaged-import verified,
with all 354 frozen source inputs unchanged. Its separate host rehearsal passed
three-task review/Apply, 11 repository tests, 26 independent checks, Cancel and
receipt persistence after native restart. The final 52-second silent film and
website evidence checks passed. Continue guest acceptance from its existing
disk using the final packet once memory and vendor sign-in are available.
One consolidated CI run subsequently passed all twelve jobs on `eb5f73d`;
no retry or publication occurred for these repairs.

The final MSI subsequently installed with exit 0 in the preserved Windows guest,
and its installed EXE hash matches the final payload. The normal Start-menu
launch was interrupted by host memory pressure before the interface rendered.
The resource guard escalated ACPI shutdown to QMP quit after fifteen seconds;
this does not establish clean Windows shutdown or application acceptance.
The guest remains stopped pending actual host headroom. Root requested permission
to close the visible Minecraft client normally; no user app was closed. Retain
the final CI result and complete guest acceptance without an unnecessary rebuild.

Chosen over manually adding a guest prerequisite or copying one runtime DLL:
the current MSI should start without a separately installed VC++ runtime.
Rust's static CRT feature is communicated to native build scripts; Microsoft's
compiler guidance requires consistent linked-module runtime selection.
Sources: [Rust linkage](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes),
[Microsoft runtime options](https://learn.microsoft.com/en-us/cpp/build/reference/md-mt-ld-use-run-time-library?view=msvc-170).
Static linkage requires rebuilding to service embedded runtime fixes. Generated
native flags, final PE imports, clean launch and actual app behavior must be
checked. Permission profiles, execution domains and reviewed Apply logic are
unchanged. Publication still requires approval.

## Contract and source

Improve the agent hypervisor's first and repeated delegated job while preserving
PR31's candidate, cancellation, refresh, and audit invariants. Starting source:
`ff0b88fb71224d579267e697261ae4d831f0cd76`, `codex/beta-candidate-verification`.
Continue on this branch without switching or discarding the prior work. No other
active Pytxo writer was found in the app inventory; other projects are running.
The lead owns integration, source and native validation. Bounded specialists
completed independent product/release reviews and isolated runtime repairs;
each returned source ownership after its focused tests and adversarial review.
Builds run sequentially with one Cargo job because host memory is constrained.

Preserve the private master brief, existing untracked captures, `.verify`,
`desktop-e2e.log`, and `docs/superpowers`. The later Team-upgrade instruction
authorizes ordinary org-profile edits and continued PR31 CI pushes within the
included allowance. Releases, deployment-triggering merges, paid budget/security
changes, new system features and major redesign retain separate approval gates.

## Phase 1: Inspect and decide

- [x] Read instructions, checkpoint, PR31 baseline and prior audits.
- [x] Capture browser first launch, one-harness composer and plan at 1440px.
- [x] Complete independent product, runtime, release and adversarial review.
- [x] Consolidate primary-source findings in [[astra-product-decisions-2026-09-07]].

## Phase 2: Implement coherent improvements

- [x] Correct concrete authority/context issues only after narrow reproductions
  (`crates/pytxo-core/src/project.rs`, `crates/pytxo-runner/src/context.rs`,
  `crates/pytxo-planner/src/lib.rs`).
- [x] Improve scoped mission examples, draft workspace restoration, readiness
  recovery and readable plan ownership/checks within the existing layout
  (`apps/desktop/src/components/desktop2/FlowScreen.svelte` and shared CSS).
- [x] Align website, first mission and support claims to actual single-harness
  value and release availability (`apps/web/src/components/site`, public docs).
- [x] Prepare final-build demo/Bench/release evidence using existing tooling.

## Phase 3: Verify and resolve gates

- [x] Fresh meaningful regression tests; final combined checks after review.
- [x] Desktop/mobile screenshots and keyboard/state journeys; native workflow.
- [x] Earlier MSI provenance, native demo and Bench evidence, retained by identity.
- [x] Replacement MSI native workflow, demo and Bench evidence; host scope only.
- [x] Hosted CI at `f64a0b0`: twelve jobs passed in run `34112393479`.
- [x] Hosted CI for the subsequent static-runtime and onboarding repairs:
  all twelve jobs passed on `eb5f73d` in run `34181828835`, September 8.
- [ ] Clean Windows installation and public download gates; VM preparation is
  authorized and in progress, while publication still requires approval.

## Coverage and choices

| Area | Evidence / gap | Disposition and acceptance |
| --- | --- | --- |
| PR31 hardening | REPORTED passing source/runtime evidence; preserved at baseline | ALREADY DONE; rerun affected invariants after integration |
| First/repeated mission | REPRODUCED vague templates, missing-CLI noise; source shows draft keeps current workspace | IMPROVE NOW; explicit scope, recover readiness, restore intended workspace and rebuild authority |
| Chroma Aperture | REPRODUCED existing Work/History/Setup and two-column composer | IMPROVE NOW within existing structure; compare same states at 1440/390px |
| Visual alternatives | Keep current styling only; improve hierarchy within structure; rebuild navigation | Choose hierarchy and workflow clarity; styling alone misses friction, navigation rewrite lacks evidence and approval |
| Planner economics | Source chooses cloud from unrelated inherited provider keys | IMPROVE NOW after regression; explicit opt-in, local choice wins, existing explicit provider configuration remains supported |
| Runtime/context | REPRODUCED traversal/link/fidelity, stale context and Windows PTY cwd failures | IMPLEMENTED with 26 boundary and 37 launch/verification checks; the CI-corrected MSI's native run and exact Apply passed 11 fixture tests and 26 independent acceptance checks; no profile expansion |
| Competitive position | Fresh primary docs show coordination, worktrees, checks and review overlap | Choose recurring bug/regression job; DEFER acquisition claims pending second-use trial; no exclusive verification or speed claim |
| Context/skills/protocols | Existing native instructions and Signal | DEFER new platform; improve current context correctness |
| CI | All twelve jobs in `34112393479` passed on `f64a0b092b02af917fea52180a4e76458e2f1934` | CLOSED for that source; Windows workspace/transport, audit and feature checks passed after the test-only correction; earlier failures remain historical |
| Clean install | Initial MSI failed with missing MSVCP140.dll; static-runtime replacement installed and rendered welcome with checked VC++ runtime DLLs absent | IN PROGRESS; final onboarding candidate and complete workflow pending; Windows evaluation/servicing limitations recorded separately |
| Publication/download | Public latest v1.2.1; v1.2.2 absent | HUMAN-BLOCKED; prepare source/artifacts before requesting specific publication |
| Demo/Bench | Final MSI's three-task native run, 11 repository and 26 independent post-checks, restart receipt and 52-second film | VERIFIED on the development host for the named bytes; direct baseline and unsuccessful attempts retained separately; no controlled comparative result |

## Acceptance commands and evidence index

Focused Rust regressions first, then `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, build CLI/MCP,
`cargo fmt --all -- --check`. Desktop `npm run check`, affected repository
Playwright journeys, `npm run build:native`, MSI build. Website lint, link/asset
checks, production build and browser journeys. Demo validation uses its existing
scripts. An unavailable command or environment remains unverified.

Working specialist reports: `target/astra-research/{product,runtime,release}.md`.
Decisions and primary sources: [[astra-product-decisions-2026-09-07]].
Before screenshots: `target/astra-ui/before/`. Browser preview fixtures establish
presentation behavior only. Final source/build/installed/public evidence are
separate acceptance layers. The goal remains active.

## Verification checkpoint

[[astra-evidence-2026-09-07]] is the current test, native-attempt and artifact
ledger. It distinguishes the full workspace run before the final receipt repair
from the full affected orchestration suite and combined lint/build checks after
it. Desktop production browser coverage is 128 checks. The final native run,
three-hash Apply, eight independent tests, 52-second silent film, 23 website
browser checks and portable Windows packet are complete and recorded there.
The September 7 Team-upgrade resumption produced commit `2964d5ad…` and exposed
three CI failure causes. The corrections pass all 468 local workspace tests and
21 tooling tests. A rebuilt MSI completed a separate credentials-path follow-up:
three reviewed/applied hashes, 11 fixture tests, 26 independent checks and native
restart persistence. The preceding observation and direct baseline are retained.
The consolidated correction push is authorized using included Actions. Final
hosted results must identify its actual commit; no release, merge or deployment
is authorized by that step.

The completion audit closed stale native-status prose and final website metadata
coverage. All 45 sitemap routes now have verified crawlable page identities;
robots, redirects, 404s and social-image metadata were checked. Eight local timing
observations carry explicit production/field-performance limits in the ledger.

The implementation includes the measured context, planner, Windows cwd and
prompt transport repairs, plus the original task permission-cap repair. Every
reopened boundary had a concrete reproduction. No permission profile or
execution domain was widened. CI cancellation affects stale PR runs only;
main and reusable release runs retain their prerequisites and separate groups.

The failed native attempts and competent direct worktree observation remain
separate records. [[astra-release-proposal-2026-09-07]] defines the owner-only
budget, clean-machine and staged-publication sequence. Local success cannot
substitute for those external gates.
