---
title: Jev versus rules routing benchmark protocol
slug: 2026-09-22-jev-routing-benchmark
status: draft
tags: [routing, benchmark, experiments, evidence]
audience: [human, agent]
layer: orchestration
created: 2026-09-22
updated: 2026-09-28
related: ["[[2026-09-22-jev-routing-contracts]]", "[[2026-09-22-jev-routing-rollout]]", "[[jev-typesafe-research-2026-09-22]]"]
---

# Does Jev improve routing over rules?

**Primary hypothesis:** RJ reduces total cost per accepted task by at least 15% versus a competent frozen R0 policy, while accepted task quality is no more than two percentage points worse. RJ differs only in initial ambiguous everyday-versus-strong selection. Test this hypothesis; do not assume it because Jev inference is cheap.

This is an unexecuted protocol. No corpus, qualified profile pair, calibrated threshold, outcome or savings claim exists yet. Corpus curation and funded runs are explicit rollout deliverables.

## Freeze the comparison

Use one harness with two qualified profiles E (everyday) and S (strong), identical tools/skills, permissions, task inputs and verification. V1a uses the existing Codex/Orbit/local PTY boundary. Pin adapter/executable/settings and record requested/reported/provider model identities honestly. No harness changes in this experiment.

All arms share the same durable attempt controller, approved graph, handoff, second-attempt rule, hard blockers, wall limits and verified oracle. Freeze plans before assignment; exclude generative replanning. A first implementation/check failure may get S repair in every arm, with two admitted attempts maximum. Jev does not control repair.

Arm definitions:

| Arm | Initial route | Role |
|---|---|---|
| E | Everyday whenever eligible | Cheap-first reference in screening. |
| S | Strong whenever eligible | Strong-first reference in screening. |
| R0 | Ordered rules in the contracts note | Primary control; usable product fallback. |
| RJ | Same R0 plus the one bounded Jev signal | Treatment. Advisor failure/abstention uses R0 and remains in the results. |

R0 gets an explicit development pass using exactly the facts available to RJ; it is not deliberately weakened. Publish its final rules, tags, defaults, feature extraction and authorized fallback order. Count all selected rows, including tasks where RJ agrees with R0. Evaluating only disagreements or “easy wins” would bias the result.

## Corpus and reproducibility files

Proposed directory: tooling/benchmarks/routing/. Required artifacts before a paid run:

- corpus.jsonl: case_id, repository/snapshot digest, task-family/variant group, split, goal, reviewed task kind/provenance, claims, dependencies, input artifact hashes, exact reviewed redacted-packet digest, oracle digest, external-resource needs, timeout, billing/spend permit reference.
- profiles.json and policy.json: qualified recipe/binding/adapter versions, authorized E/S roles, policy/version/hash, model evidence level, integer-parts-per-million thresholds and eligibility rules. No secrets.
- questions.json and packets.jsonl: exact template/model version, packet serialization version and actual disclosed redacted packets. No hidden checks or expected fix in the routing packet.
- protocol.json: randomized order seed, fixed sample sizes, primary metrics/margins, exclusions, stopping rule, analysis version and authorized budget.
- runs.jsonl: case/repetition/arm, timestamps, selected profile/reasons, advice status/usage, every attempt and resource/input digest, outcome, raw usage provenance, oracle results, reviewer result and candidate hash.

Archive manifests and source identities before revealing holdout outcomes. Task variants from the same bug/template, adjacent commits and the same repository stay in one split. Keep hidden checks and review notes outside worker workspaces and the planner/advisor packet; use a separate verifier and probe their actual access boundary. If the permission receipt provides only logical/tool separation, report that limitation rather than claiming OS-enforced secrecy. No fixture with a prior solution exposed during development enters confirmation.

The offline freeze now binds a reviewed packet digest and a pretrial `planned_advice_opportunity` boolean to each case and arm (frozen schema v2; protocol schema v1). Store aliases each send's packet digest with the assignment's private key; the recorder freezes the matching alias and rejects a different Jev send. The freeze refuses a confirmation manifest whose planned Core advice-opportunity cases all have the same packet, even when other families vary. Keep the frozen schedule, study and analysis bundle private: the low-cardinality raw packet digest is enumerable and can reveal coarse task facts even without task text. This checks consistency, not provenance: an independent pre-outcome review must establish each opportunity label from the complete Core initial gate (authorized evaluated Live policy, first strong-default choice, complete diagnosed single-component work and both profiles eligible), plus each packet digest from the exact projection. Runtime eligibility may differ; the label is not a Core receipt. Synthetic or self-asserted digests cannot qualify a funded trial. Pre-register the expected opportunity count and repository spread from screening data; two distinct packet digests alone do not establish enough exposure or power.

Prepare 300 cases (180 development, 60 validation, 60 untouched packet holdout), including source tasks with missing context, non-English text, injected instructions, long descriptions and ambiguous demands. The current reviewed projection emits only a fixed goal and eight coarse categories, including two optional reviewer-declared diagnosis cues; test that source wording cannot leak into the outbound packet. Two independent reviewers label demand and leakage; disagreements are adjudicated before model answers are revealed. These labels calibrate the question, not prove coding success. If this sparse packet cannot improve on R0, report the negative result rather than expanding disclosure after seeing holdout outcomes.

The experimental one-file Claude proposal builder originally fixed every coarse category, making its hosted Shadow packet identical across accepted tasks. It now defaults conservatively to other/unknown demand, incomplete context, and unknown component scope, and accepts explicit coarse facts in Desktop review. Two optional diagnosis cues can vary the packet within the only potentially advice-influenced cohort (complete, explicitly single-component diagnosis), without disclosing task text, paths, or checks. This is packet-feasibility evidence, **not** a funded efficacy screen: the current rubric maps the two cues to a simple rule that R0 can also implement. The facts are claims, not independent difficulty labels. Before collecting an efficacy corpus or funding a trial, audit packet diversity specifically within the Core-eligible cohort, obtain independent labels and a qualified same-harness profile pair, and freeze a baseline that receives identical facts. Preserve the exact packet projection and privacy scope through that trial.

For initial threshold selection, try everyday-fit thresholds 0.60/0.70/0.80/0.90/0.95/0.99 and unclear ceilings 0.05/0.10/0.20 on development/validation only. Store thresholds as integer parts per million. Use packet errors plus screening task outcomes; choose one policy before confirmation. If none preserves quality, RJ remains shadow-only. Do not relabel difficult cases after seeing Jev.

## Stage 1: 24-task feasibility screen

Create four independent bounded cases in each family below. Use permissively licensed pinned repositories or purpose-built fixtures whose baseline and oracle are independently checked.

| Family | Concrete task patterns and oracle requirements |
|---|---|
| Mechanical | Documentation link relocation, symbol rename, config-key rename, repetitive formatting; assert exact scope and preserved behavior/content. |
| Local defects | Null/default handling, Unicode parsing, numeric boundary, serialization round-trip; hidden edge cases plus no unrelated diff. |
| State/persistence | Save/reload consistency, migration compatibility, cancellation persistence, stale cache invalidation; fresh-process post-state checks. |
| Concurrency | Duplicate completion, lock release on error, overlapping writer, cancellation race; deterministic schedule fixtures. |
| Adapter/protocol | Malformed response, partial stream, wrong capability, pagination boundary; independent contract tests. |
| Cross-component | Settings UI-to-store propagation, API/schema compatibility, dependency-output composition, recovery receipt consistency; integration checks across components. |

Run all four arms once per task: 24 × 4 = 96 task runs, at most 192 worker attempts. Start from a fresh snapshot each time and randomize arm order within task/time block. This is development/screening data, never the confirmatory set.

Success requires runnable profiles, honest accounting, inspectable candidates and no authority violations. Report success counts, disagreements, full cost/latency distributions and failure cases. A positive screen permits further evaluation; it cannot substantiate a two-point quality margin.

Use a separate deterministic stress set for auth/quota errors, failed prerequisites, duplicate/late events, Stop, crashes and resource contention. These should be decided identically without consulting Jev. Do not inflate semantic-routing accuracy by mixing in obvious deterministic cases.

## Stage 2: fixed held-out paired trial

Default bounded trial: **400 unseen tasks from 200 unseen repositories, two tasks per repository**, balanced across the six families (67/67/67/67/66/66). Run R0 and RJ twice independently per task: 1,600 task runs, at most 3,200 admitted worker attempts. Use distinct seeds where the harness exposes them; repeated runs are still stochastic even when seed control is unavailable. Average repetitions within task; they are not 800 independent tasks. Fewer repositories would be a changed protocol with less independent evidence, not equivalent replication.

Before opening this corpus, use the screening results to estimate paired quality discordance and cost variance, then simulate the specified clustered analysis. Record expected power. If it cannot plausibly reach 80% power for the declared useful effect under the funded sample, either register a larger fixed trial with its budget **before any holdout run**, or label the 400-task trial limited-power. Never promise that 400 is sufficient.

For scale only: an independent-pair normal approximation with 10% discordant outcomes, a two-point non-inferiority margin, 80% power and one-sided 5% error needs roughly 1,550 independent tasks even before repository clustering. This is a planning calculation, not this trial's achieved power. Tight quality guarantees can cost far more to establish than the routing inference.

Both arms receive byte-identical initial snapshots and user tasks. Toolchain/dependency caches are prewarmed identically; disable Pytxo route/output reuse. Record provider prompt caching and charge its actual rate. Interleave paired arms across time, with no concurrent tasks sharing a constrained account/GPU. Pin versions for the trial; model/adapter drift invalidates the affected block rather than mixing versions silently.

Freeze task limits in the corpus: default 20 minutes per worker attempt and 45 minutes per task including handoff/checks, with no more than two attempts. Mandatory oracle budget and spend permit are shared across arms. A worker exceeding a cap is a task failure with all consumed cost counted. These limits apply to the bounded benchmark, not every future product task.

## Outcomes and analysis

The offline recorder's analysis version `routing-paired-v2` derives a bounded exposure summary from the Store trace for every complete assignment: local may-send journal rows, completed sends, and the first admitted route. An `applied` decision must be a first-attempt Everyday admission backed by a completed matching send. The paired intention-to-treat result retains all assignments and fallbacks. Report planned opportunity separately from bundle-reported applied admissions and rules-arm strong-default admissions. If the bundle reports no Jev-influenced admission, a favorable numerical comparison is not evidence that Jev beat rules. A direct analyzer input can fabricate its exposure fields; authenticate the private Store traces, recorder ledger, provider/model identity and frozen trial registration before attribution. Even authenticated trace fields cannot prove full Core eligibility at decision time; audit the frozen pretrial labels against Core inputs and compare actual exposure before attributing a result.

An accepted task requires every frozen hidden check, no out-of-scope change, and a blinded reviewer finding that the stated task is satisfied. Normalize candidate presentation and remove arm/model labels. Reviewers record reason codes; a second reviewer adjudicates disputed cases without arm disclosure. Worker-authored tests alone cannot establish acceptance. Any suggested repair after review is a new measured rework item, not retroactive success.

Primary quality: paired difference in accepted-task proportion, RJ minus R0. Primary economics:

    cost_per_accepted_task(arm) = total_cost_of_all_assigned_runs / accepted_runs
    cost_ratio = cost_per_accepted_task(RJ) / cost_per_accepted_task(R0)

Include routing, all attempts (failed/cancelled/startup included), repair transfer, checks and measured rework within the frozen observation window. Record infrastructure separately and include any differing billed infrastructure. If an arm has zero accepted tasks, its metric is infinite, not missing. No success-only cost average.

Pre-register a paired hierarchical bootstrap with 10,000 resamples and fixed analysis seed: resample repositories, then paired tasks within repository; keep both arms and repetitions together. Compute one-sided 95% bounds. If fewer than ten repositories have any discordant paired acceptance result, replace the bootstrap quality bound with a conservative bound: mark each repository with any R0-success/RJ-failure pair, compute its one-sided 95% Clopper-Pearson upper probability U, and use -U as the quality-difference lower bound, ignoring favorable discordance. This bounds harm even with arbitrary within-repository correlation. With zero harmed repositories out of 200, U = 1 - 0.05^(1/200), approximately 1.49%; it is not a zero-width interval. If this conservative bound cannot support the margin, report inconclusive. Publish per-family outcomes and a task-weighted versus repository-weighted sensitivity check.

Promote only when **all** gates pass:

1. Lower one-sided 95% quality-difference bound exceeds -0.02.
2. Upper one-sided 95% cost-ratio bound is below 0.85, including advisor overhead and failures.
3. Upper one-sided 95% bound on end-to-end p95 latency ratio is at most 1.10, with at most two seconds of foreground routing wait.
4. No unauthorized transition, secret disclosure or unaccounted worker launch in the exercised authority cases. The upper one-sided 95% bound on mean additional unplanned human intervention time is at most one minute/task. Report blinded experimental review time separately from product intervention time.

These are conjunctive acceptance gates, not a choice of whichever metric looks best. No optional stopping for success, peeking to retune thresholds or changing margins after holdout results. Stop early only for safety or exhausted authorized budget; publish that as an incomplete trial.

A common external outage may invalidate a paired block only under a preregistered rule applied before seeing quality. Keep the original records, report exclusions, and rerun both arms. Jev outage, abstention, malformed output, profile startup failure or budget exhaustion is treatment/product behavior and counts in intention-to-treat results.

API-cost claims require provider usage/rate evidence. Subscription cohorts report observed quota deltas if available, otherwise strong invocation counts, tokens and time as proxies. Unknown quota cannot become a dollar saving. Do not combine incomparable billing modes into one cost ratio; an API-backed qualified cohort is required for the dollar hypothesis.

## Decision after results

- All gates pass: activate this pinned RJ policy only for the evaluated profile/task cohort. Do not claim general superiority across harnesses.
- Quality preserved, economics inconclusive: keep R0 active; investigate packet utility or experiment power.
- No advantage or worse quality: keep rules-only. Do not build more routing infrastructure to justify the sunk effort.
- Different harness or semantic repair advice: new isolated experiment after portability qualification, not a simultaneous variable in this trial.

The strong-first screen is a reference, not a confirmed quality-equivalence claim against strong-first execution. A separate powered comparison is needed for that claim. All measured and unknown results remain attached to the exact policy/profile/candidate identities.
