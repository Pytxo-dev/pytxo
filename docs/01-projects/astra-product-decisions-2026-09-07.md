---
title: ASTRA product and research decisions
slug: astra-product-decisions-2026-09-07
status: active
tags: [project, product, research, economics]
audience: [human, agent]
layer: orchestration
created: 2026-09-07
updated: 2026-09-10
related: [[astra-execution-2026-09-07]], [[beta-competitor-research-2026-09-05]], [[vision]]
---

# Decisions for this increment

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

## Preserved September 10 disposition

Keep the bounded native repairs: consume successful drafts, reset dispatch
scroll, use honest pending-result wording, and suppress background verification
and receipt-query consoles. F968 native execution confirmed the UI fixes and
passed mission/Cancel/Apply/restart with 11 project and 26 independent checks.
Denser video review then identified the remaining startup window as netsh, before
agent-start. Repair the read-only query constructor; preserve query interpretation,
permission/enforcement semantics, opt-in firewall changes and ConPTY. A detached
parent regression demonstrated failure before the flag and success afterward.
All 77 runner tests and strict Clippy passed; the new 891 payload is packaged.

Keep the 94-second f968 film as truthful correctness evidence with that defect
visible and labeled. It has actual 1× footage, disclosed cuts, uninterrupted Apply
and a labeled later restart still. Do not call it a finished smooth demo or reuse
it to certify 891. Physical Escape paused native control before that new payload
launched; explicit resumption is required. No new redesign/platform requirement
is selected. Clean Windows, included-allowance CI and authorized distribution
remain separate gates. See [[astra-evidence-2026-09-07]].

## Preserved September 9 disposition, before native resumption

The historical implementation/native observations below remain tied to their
recorded builds. In particular, “final MSI” in an earlier observation does not
mean the current MSI `26ef95bc…` / packaged EXE `073a1a30…`. That candidate has
359 frozen inputs, passing local package checks and 26 Desktop library tests.
It hides background readiness consoles reproduced on the predecessor fcd build;
intentional vendor sign-in is unchanged. Frontend inputs match the previously
tested menu pass. Native acceptance of the repaired payload is
unexecuted. See [[astra-desktop-menus-2026-09-09]] and
[[astra-evidence-2026-09-07]].

The selected planning, context, task-authority, Windows transport, first-use and
repeat-use repairs have been implemented at their recorded stages; older
**IMPROVE NOW** labels below identify those decisions, not fresh open tasks.
The September 9 pass adds in-window input retention, workspace-scoped commands,
menu/search/focus improvements and connected voice preferences. Deliberate
deferrals and rejected expansions remain unchanged. The remaining substantive
work is exact-build native proof, clean Windows acceptance, confirmed included CI
allowance and authorized distribution. No new platform or redesign is selected.

## Established product direction

Pytxo remains an **agent hypervisor**. The chosen first job is a reproducible
repository bug plus its regression test, using one existing coding-agent CLI.
This is a targeting hypothesis, not demonstrated demand. A second useful job
matters more than getting a user to launch several workers once.

## Competition: coexist with capable native tools

Official sources were read on 2026-09-07. These establish **DOCUMENTED** behavior;
none of these competitors was personally exercised in this research pass.
Absence from a page is not evidence of an absent capability.

| Source and relevant documented behavior | Decision consequence |
| --- | --- |
| [Cursor worktrees and async subagents](https://cursor.com/changelog/04-24-26), [Codex worktree handoff](https://learn.chatgpt.com/docs/environments/git-worktrees) | Background work and worktrees alone do not justify another tool. Compare against their competent native workflow. |
| [Claude Code teams](https://code.claude.com/docs/en/agent-teams): dependencies, task claims, hooks, experimental limits | Coordination and deterministic hooks already overlap. Do not equate task claims with repository-path ownership. |
| [Conductor checks](https://www.conductor.build/docs/reference/checks): review and merge conditions; [BridgeMind changelog](https://www.bridgemind.ai/changelog): remote folders, provider switching, platform-specific availability | Vendor neutrality and review are contested. “Terminal wall” is inadequate competitor framing. |
| [GitHub agent controls](https://docs.github.com/en/copilot/concepts/agents/cloud-agent/risks-and-mitigations), [merge queues](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue) | Remote integration checks and human gates are strong substitutes; local Apply complements protected branch controls. |
| [Factory Missions](https://docs.factory.ai/missions/overview), [Devin](https://docs.devin.ai/get-started/devin-intro) | Structured delegation, bug fixes and validation are already offered. Scriptable checks and clear completion criteria still matter. |

Choose a clear account of Pytxo's own observable contract: scope a job, supervise
instances, inspect the combined candidate and recorded checks, then authorize
Apply. Reject exclusive-verification, comparative speed/cost, and universal
sandbox claims. The strongest objection is that a proficient developer can do
the same job with a native agent, worktree, tests and Git review with less setup.
The smallest falsification test is a competent native comparison followed by the
participant's voluntary choice for their next job.

## Workflow, context and economics

| Area / current evidence | Options and chosen action | Acceptance / uncertainty |
| --- | --- | --- |
| First use: vague templates and optional-CLI ambiguity | **IMPLEMENTED:** concrete path placeholders, clear local Desktop continuation, one ready agent first | Two viewport journeys and the final MSI's real three-task, two-wave Codex mission passed; clean-machine first use remains unverified |
| Repeat use: restored text retained the current repository | **IMPROVE NOW:** restore original workspace/CLI when available, clear prior plan, require an explicit replacement when unavailable | Browser regression reproduces old wrong-workspace behavior; no historical authority reused |
| Native draft clicked before CLI detection finishes | **DEFER** smoother restoration during detection; explicit reselection remains safe | After detection settles, native reuse restores Codex and the original workspace with no ready plan. An early click currently requires reselection; it cannot dispatch stale authority |
| Planning: an inherited API key selected external planning | **IMPROVE NOW:** require explicit cloud opt-in; explicit local mode wins | 15 planner tests pass without network calls; cost remains unknown when unmeasured |
| Context: original-repo scaffolds became stale after dependency composition or retry | **IMPROVE NOW:** read actual worker workspace under original policy; fresh retry directory; enforce fidelity cap and output boundary | Real fixture regressions and Windows link tests; no OS-wide sandbox claim |
| Visual design: established Chroma Aperture, Work/History/Setup | **IMPROVE NOW:** clearer composer hierarchy, readable ownership/checks, compact readiness and reusable drafts | Same-state desktop/narrow screenshots; no navigation/brand redesign |
| Runtime breadth | **MAINTAIN:** existing adapters with capability-specific evidence | Local Flow uses multiple instances of one selected CLI. Mixed-vendor manifests do not establish mixed-vendor Flow support |
| Real native handoff: implementation worker added tests owned by a separate task; package correctly refused | **IMPLEMENTED:** carry explicit reviewed ownership/dependencies/checks into each non-empty task prompt; expose the recorded refusal in Work/History | Actual refused run preserved; both-backend literal-prompt regressions pass. The final native run respected all three scopes and passed eight post-Apply tests; its additional documentation guidance is disclosed |
| Active agent rows finalize after all waves return | **DEFER:** typed incremental terminal-outcome persistence | Pre-existing SQLite timing; no scheduler or Apply-boundary failure observed. Do not infer completion from a worker's verify-ok log |
| Real combined checks passed but native review found stale README prose from an independent task | **MAINTAIN** explicit diff review; use the existing task-prompt editor to clarify the intended combined result. **DEFER** general semantic dependency inference | The original mission was clear; this was a decomposition/context failure, not an ambiguous request. Preserve the ready, unapplied package and disclose the exact prompt override in the new attempt. No claim that Pytxo detects prose contradictions |
| Native Windows CLI shim split the expanded task prompt before model work | **IMPROVE NOW:** documented Codex stdin input with exact UTF-8 and exit/Stop preservation; compact generated Windows guidance | Real CMD-shim regressions: failure reproduced, five controls pass. Other adapters retain their existing transport; no arbitrary-prompt compatibility claim |
| Original permission lookup could confuse configured names with runtime actor IDs after a later grant increase | **IMPROVE NOW:** resolve saved task authority by the same flattened actor order used by execution and persistence; refuse missing or inconsistent receipts | Five actual candidate regressions and two Apply regressions reproduced the old failures; all 95 orchestration tests pass. Orbit/Galaxy Apply remains within one execution domain. Consistently tampered envelopes are not independently attested |
| Accounts and hosted coordination | **MAINTAIN** accountless Core; **DEFER** hosted coordinator | Active setup already excludes Pytxo sign-in. Provider/worker consumption is separate from Pytxo billing; no account/price changes |

Keep native instructions and existing formats. [Agent Skills](https://agentskills.io/specification)
provides progressive disclosure; [ACP](https://agentclientprotocol.com/get-started/introduction)
is an interoperability option when a specific adapter needs it. Neither grants
authority. **REJECT** a proprietary skills router and automatic fan-out for this
increment. **DEFER** generalized production adapters and new protocol plumbing
until a named user problem needs them.

## Evaluation, distribution and second use

Existing native Pytxo and direct-Codex observations both succeeded. Their timings
exclude different preparation/review effort and cannot establish a win. Signal
byte reduction is separate from tokenizer counts, bills and task success.
[Opik](https://www.comet.com/docs/opik/) offers experiment/trace tooling; installing
it would not make black-box CLI cost accounting complete. Reuse local evidence.

**IMPROVE NOW:** truthful first-mission/docs/download copy, final-build native
captures, exact artifact hashes and a short support protocol. **DEFER** broad SEO
comparison pages and campaigns until the supported first-job loop is validated.
CI billing, a clean Windows environment, private security reporting, and public
publication require specific owner actions; source edits cannot close them.

The completion audit implemented missing per-page canonical/share identity and
Evidence sitemap inclusion, corrected a duplicate docs heading, and verified all
45 sitemap routes. Eight bounded local timing observations are recorded in
[[astra-evidence-2026-09-07]]; they do not establish production performance.
**DEFER** further bundle/font changes until target-device evidence justifies them.

**PROPOSED**, requiring recruitment/consent: 3–5 developers, one existing harness,
two real jobs over a week. Record install/harness readiness, plan, dispatch,
candidate result, review, Apply/reject/abandon, and voluntary second use. Capture
intervention time and reasons for abandonment; leave unavailable cost null. Use
local/manual counts without source, paths, prompts or raw logs. This discovers
friction, not market rates or product-market fit. No participant was contacted,
analytics service added, or hosted expenditure authorized.

Back: [[astra-execution-2026-09-07]]
