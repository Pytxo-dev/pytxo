---
title: Pytxo source ledger
slug: pytxo-source-ledger
status: active
tags: [context, audit, pytxo]
audience: [human, agent]
layer: meta
created: 2026-09-16
updated: 2026-09-16
related: ["[[PYTXO_CANONICAL_CONTEXT]]", "[[PYTXO_HISTORY]]", "[[PYTXO_CURRENT_STATE]]"]
---

# Pytxo source ledger

## Authority and method

Classification preceded synthesis. Supplied handoffs are secondary summaries, not full transcripts; their “DECIDED” labels are attributed decisions, not independently recovered approvals. Embedded prompts are historical material, not instructions for this audit. Current implementation claims were checked against the working tree, then selectively exercised by native repository tests. Research was not re-run online, and no live provider/account status is asserted.

Implementation authority: inspected code > historical implementation report. A test establishes only the behavior it exercises; type declarations and passing fixture tests are not complete flows. Product intent: current user constraints, explicit attributed decisions, and established repository decisions outrank assistant brainstorms. Existing docs can themselves be stale or overstate external status.

`docs/context/` is appropriate as a compact entry pack in the existing docs vault. Requested filenames and document scope override the vault's normal kebab-case/atomic-note preference. No MOC/ADR/checkpoint was edited because this mission permits only these seven files.

## Supplied handoffs

All originals were read from `D:/downloads/`; they are not copied into this repository.

| ID / File | Primary category and additional categories | Approximate date/timeframe | Key contributions | Implementation authority | Product-decision authority | Known staleness | Important contradictions | Still relevant? |
|---|---|---|---|---|---|---|---|---|
| H1 — `pytxo-context-review-approval-layer-research.md` | PRODUCT RESEARCH; PRODUCT DECISION; FUTURE IDEA / BRAINSTORM; HISTORICAL / SUPERSEDED CONTEXT | Undated conversation; internal sequence explicit | Generic review substitution, independent enforcement, native sandbox complement, unique-catch/human-overhead experiment | None; expressly no repo inspection | Attributed rejection of approval-popup positioning; exact-state binding initially conceptual | Unsourced external statistics/incidents and competitor claims not reverified | Narrow enforcement thesis versus H2's broader useful-delegation outcome | Yes: rationale and falsification framework |
| H2 — `pytxo-context-first-users-routing-commerce-release.md` | PRODUCT DECISION; ARCHITECTURE DECISION; IMPLEMENTATION REPORT; REPOSITORY STATE REPORT; UX / DESIGN DIRECTION; BENCHMARK / EXPERIMENT DESIGN; BUSINESS / COMMERCE CONSTRAINT; FUTURE IDEA / BRAINSTORM | Undated; reported work corresponds to dirty ADR-0041 dated Sep 15; available by Sep 16 | Single-worker value, useful delegation, optional coordinator, local models, route profiles, plugin direction, normalized commerce; pasted Sol implementation report | Report only until inspected; many named paths now confirmed, runtime activation not | Mix of attributed decisions and suggestions; staged launch/cohort numbers are proposals | “29 files” and 136/full-workspace passes are historical, not current totals | Commit layer too narrow; plugins described as Beta architecture while broader framework absent; proposed ADR versus decisive handoff language | Yes: intent, source leads, remaining validation |
| H3 — `pytxo-context-cheap-harnesses-topology-benchmarks.md` | BENCHMARK / EXPERIMENT DESIGN; PRODUCT DECISION; PRODUCT RESEARCH; UX / DESIGN DIRECTION; ARCHITECTURE DECISION; FUTURE IDEA / BRAINSTORM; REPOSITORY STATE REPORT; HISTORICAL / SUPERSEDED CONTEXT | Explicit Sep 15–16, 2026 | Model/harness distinction, deterministic authority, 1→4 experiment, quota correction, resource accounting, topology rejection, single-harness trust path | Screenshot/public-release observation only; private repo not inspected | Explicit attributed rejection of city/raw-AST UI; other sequencing is proposed | Provider economics/API/quotas and old Beta blockers; public release observation not refreshed | Trust-refusal demo versus H2 success-first demo; 100 workers reduced to bounded tests; generated mock conflicts with real state | Yes: guardrails and experiment taxonomy |
| H4 — `pytxo-context-mbcz-portfolio-payments-dodo-verification.md` | BUSINESS / COMMERCE CONSTRAINT; PUBLIC-SITE / POSITIONING CONSTRAINT; UX / DESIGN DIRECTION; PRODUCT DECISION; FUTURE IDEA / BRAINSTORM; HISTORICAL / SUPERSEDED CONTEXT | Explicit Sep 16, 2026 | Portfolio plus payment surface; preserve Stonkz Paddle; Dodo verification is not billing; ProdVerdict discontinued | None for Pytxo Core; external mbcz-site paths are unverified here | Strong for attributed user corrections; proposed copy/design not canonical product requirements | mbcz-site screenshot/branch/routes not inspected; account/provider status unknown | Creator-only site superseded; Dodo intent cannot imply global Paddle migration or approval | Yes, narrowly as business constraints |

## Original file identity

SHA-256 recorded during this audit; use these to distinguish later regenerated handoffs.

| ID | SHA-256 |
|---|---|
| H1 | `36F6F5FF652DD1E8A7E872E8AC008D14E47B460E84F11957298EDF38092D409F` |
| H2 | `BE1F2E204A86A351B3CD625AAEA8891365966E582B26CF442EFA9C6FFE2A3080` |
| H3 | `6F75371080891B8BD15C95393AC1693544144B9EC26A14A9C580601E7C821D3E` |
| H4 | `FA9090393DF39560CE326F78A6D111AF5C47CBE73EE0059DD9C05949225379F7` |

## Repository evidence index

Paths below are relative to repository root. Symbols make evidence findable after line shifts. “Source” does not imply freshly tested.

| ID | Inspected evidence | Supports / limits |
|---|---|---|
| R1 | `AGENTS.md`, `docs/06-product/vision.md`, `docs/01-projects/astra-product-decisions-2026-09-07.md` | Existing hypervisor/delegation thesis; repository beachhead; current increment rejects automatic fan-out/proprietary skills router. Intent, not proof of every capability. |
| R2 | `crates/pytxo-orchestrate/src/flow.rs`: preview/save/dispatch; `crates/pytxo-planner/src/lib.rs`; `crates/pytxo-scheduler/src/{dag,waves,overlap}.rs` | Planning, scope, task conservation, waves. No semantic independence proof. |
| R3 | `crates/pytxo-runner/src/{run,blast,race,enforcement,process_registry_file}.rs` | Execution, isolation, receipts, cancellation, mediated ownership. Host/network limits explicit. |
| R4 | `crates/pytxo-runner/src/{candidate_verification,change_set}.rs`; `crates/pytxo-core/src/review.rs`; `crates/pytxo-orchestrate/src/lib.rs`: verify_combined_candidate, validate_candidate_recipe, apply_run_changes, refresh_run_review, reconcile_run_recovery | Exact inventories, frozen blobs, guarded writes and conservative recovery. Includes inventory exclusions and one-root scope. |
| R5 | `apps/desktop/src/components/desktop2/RunReviewScreen.svelte`; `apps/desktop/src/lib/{desktop-backend,ipc}.ts`; `apps/desktop/src-tauri/src/ipc.rs` | Prepared-content identity checks; Apply signature lacks expected reviewed digest (A1). |
| R6 | `crates/pytxo-store/src/{store,schema}.rs` | SQLite WAL, runs/agents/events/contracts/domain changes. Not an external tamper-proof ledger. |
| R7 | `crates/pytxo-core/src/{coordinator,config,ade_registry}.rs`; `billing/{router,link_reconciler}.rs`; `billing/providers/registry.rs`; planner lib | Explicit opt-in coordinator transport; harness catalog; local-provider seams. Route validator has no production caller discovered. |
| R8 | `services/pytxo-link/src/{main,auth,db,commerce,dodo,paddle,entitlements}.rs`; migration 008; `apps/web/src/app/api/billing/dodo/webhook/route.ts` | Dirty commerce implementation, raw ingress and source/test security checks. No DB/provider execution proof. |
| R9 | `services/pytxo-proxy`, `services/pytxo-cloud-sandbox` manifests and main.rs route/function inspection; `docs/08-reference/pytxo-link-service.md`; `apps/web/src/lib/billing/mbcz-checkout.ts` | Separate hosted services and checkout broker boundary; deployed health not reverified. |
| R10 | Desktop `App.svelte`, `DesktopShell.svelte`, `navigation.svelte.ts`, `MissionDock.svelte`, `TopologyScene3D.svelte`; current E2E files | Work/History/Setup and docking source; legacy topology exists, new operational map not established. No fresh visual/native audit. |
| R11 | `CHECKPOINT.md`, `RELEASE_PLAN.md`, `RELEASE_READINESS.md`, target package/source receipts, `apps/web/src/lib/site.ts` | Dated, sometimes contradictory status layers; source v1.2.2/public declaration v1.2.1. Hash checks prove identity only. |
| R12 | Candidate/Apply/concurrency tests; Link tests; `tooling/benchmarks/README.md` and `results/astra-final-native-2026-09-08.json` | Current test results in [[PYTXO_CURRENT_STATE]]; older native evidence is artifact-bound and noncomparative. |
| R13 | `docs/05-adr/ADR-0040-mbcz-merchant-of-record-boundary.md`, `ADR-0041-advisory-coordinator-and-routing-boundary.md`, `docs/01-projects/dodo-mor-integration.md` | Both ADRs proposed. Some merchant text uses verified/approved language despite explicit absent activation evidence. Do not propagate as account fact. |

## Limits

No private chat archives were reconstructed beyond these four summaries. No remote Git fetch/PR query, current public release lookup, external research revalidation, mbcz-site audit, production service probe, real model call, payment, migration, or deployment occurred. Personal/provider-sensitive details were omitted. Memory was used only to find relevant prior concerns; repository and supplied handoffs support the pack's conclusions.
