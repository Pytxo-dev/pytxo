---
title: ASTRA product improvement decisions and evidence
slug: astra-execution-2026-09-07
status: active
tags: [project, desktop, verification, research]
audience: [human, agent]
layer: orchestration
created: 2026-09-07
updated: 2026-09-08
related: [[beta-readiness-plan-2026-09-05]], [[beta-competitor-research-2026-09-05]]
---

# ASTRA execution

## Current implementation — clean Windows runtime dependency

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
disk using the final packet. No extra Actions run or publication has occurred
for these repairs.

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
- [ ] Hosted CI for the subsequent static-runtime and onboarding repairs.
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
