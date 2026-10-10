---
title: Positioning
slug: positioning
status: active
tags: [product, positioning, gtm, competitive]
audience: [human, agent]
layer: meta
created: 2026-10-02
updated: 2026-10-10
related: [[product-vision]], [[beta-competitor-research-2026-09-05]], [[competitive-landscape-2026-07]], [[competitive-benchmarks]]
---

# Positioning

**Pytxo is the landing layer for coding agents: it checks what agents
produced as one combined change, shows exactly what will land, refuses if
anything moved, and leaves a receipt.** It does this for a run Pytxo planned
across your agent CLIs and, since 1.3, for branches from any agent tool
(`pytxo land`). Category label in architecture docs: agent hypervisor. Lead with
the outcome; explain the category, never the reverse.

> [!NOTE]
> 2026-10-10: repositioned from "agent hypervisor" to "landing layer" per
> [[pytxo-business-strategy-2026-10-10]] and [[2026-10-10-pytxo-v1-3-design]].
> The H1 "Many agents. One verified change." stays.

## The market on 2026-10-02

Running several coding agents at once is now a commodity. Each source below was
checked 2026-10-02; these are vendor descriptions, not tested comparisons.

| Group | Examples | What they sell |
| --- | --- | --- |
| Parallel-agent workspaces | [Conductor](https://www.conductor.build/docs), [Superset](https://superset.sh), [Orca](https://www.onorca.dev/), [BridgeMind](https://www.bridgemind.ai), CodeAgentSwarm, Vicoa | Many agents side by side, one worktree and branch per task, review and merge each one |
| Vendor apps | Codex app, Claude Code desktop, Cursor, GitHub Copilot app, Zed parallel agents | Parallel sessions for that vendor's agent, usually with worktrees |
| CLI orchestrators | [Bernstein](https://bernstein.run/), Warp orchestration, Claude Code agent teams | A planner fans one job out to workers; merges per task or inside one vendor |

"Every agent at once" therefore no longer differentiates. Orca fans one prompt
to five agents and merges the winner; workspaces leave you to merge one branch
per agent; Bernstein merges each verified task to main.

## Where Pytxo is different

The combination, not any single part:

1. **One job, split by ownership.** Pytxo plans one request into tasks that own
   their files, runs mixed vendors in parallel, and holds a task that shares a
   file until its owner finishes. No branch-per-agent merge afterwards.
2. **Pytxo runs the checks.** Verification commands run on each worker's
   result and on the combined change. An agent saying "done" is not a pass.
3. **One reviewed change, applied exactly.** Review shows the combined files
   and which CLI wrote each. Apply writes those exact bytes and refuses if the
   project changed after review; interrupted Apply is journaled and recoverable.
4. **Local and yours.** Runs on the user's machine with their own CLI accounts;
   no Pytxo account for local work. Windows Desktop first.

Honest weaknesses: smaller agent list than Orca or Vicoa; Windows-only Desktop;
no cloud, mobile or GitHub PR flow; slower than one agent for small jobs.

## Message hierarchy

- **Headline:** the outcome — many agents, one job, one change you trust.
- **Proof first:** Pybench (synthetic: per-branch CI lands 4 broken mains in 12
  scenarios, `pytxo land` refuses all 4, 0 good changes refused) and the 10
  October live run on the released 1.2.4 build (7 tasks, Claude Code + Codex +
  Cursor Agent, 14/14 files applied, stale refusal, 22/22 tests). Quote numbers
  only from their record files.
- **Pillars:** split by ownership · checked by Pytxo · applied exactly · local.
- **Who it is for:** developers who already pay for two or more coding-agent
  CLIs and want them working on the same change without babysitting merges.

## Language

Product UI and the site use plain words: request, plan, task, agent, checks,
review, Apply, changes, project. Keep **Apply** capitalized as the product verb.
Internal terms (candidate, canonical repository, Blast/Race/Signal Shield,
execution domain, enforcement receipt, epistemic state) stay in docs and
technical details, not in headings, buttons or status lines.

## Claims

Allowed with evidence: mixed-CLI runs (v1.2.2), file-ownership scheduling,
Pytxo-run checks, exact Apply with stale refusal, journaled Apply recovery,
local use without a Pytxo account.

Not allowed: faster or cheaper than one agent, better code quality, universal
undo, OS-level sandboxing, enforced network/host isolation, competitor
absence of a feature, testimonials or usage numbers that do not exist.

Back: [[product-vision]] · [[MOC-home]]
