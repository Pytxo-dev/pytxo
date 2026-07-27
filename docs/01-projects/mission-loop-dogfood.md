---
title: Mission loop dogfood benchmark
slug: mission-loop-dogfood
status: active
tags: [project, mission, benchmark]
audience: [human, agent]
layer: meta
created: 2026-07-27
updated: 2026-07-27
related: [[mission-loop]], [[pytxo-improvement-research]], [[competitive-benchmarks]]
---

# Mission loop dogfood benchmark

Phase 1 success criterion: a real multi-agent feature without hand-written `[[task]]` rows. Gate Phase 2 on voluntary re-runs within seven days.

## Matrix

Run each mission three ways and record raw logs under `tooling/benchmarks/mission-loop/` (gitignored binaries OK; keep JSON/text logs).

| Mission | Solo strong agent | Manual worktrees | `pytxo mission` |
|---------|-------------------|------------------|-----------------|
| Cross-cutting auth change | | | |
| DB migration + API update | | | |
| Plugin feature + tests + docs | | | |
| Large refactor across modules | | | |
| Three unrelated bug fixes | | | |

## Metrics

| Metric | Why |
|--------|-----|
| Human setup minutes | Orchestration overhead |
| Wall-clock completion | Parallelism value |
| Token / model cost | Swarm waste |
| Human interventions | Autonomy |
| Integration failures | Coordination |
| Final tests passed | Usability |
| Review minutes | Agent burden |
| Successful completion | Outcome |

## How to run a Pytxo cell

```powershell
cd <fixture-or-project>
pytxo trust orbit
pytxo mission "<mission text>" --yes --plan-file .pytxo/mission-plan.json
# after run: capture status
pytxo status --json > .pytxo/mission-status.json
```

Do **not** invent counterfactual “collisions avoided” unless the plan JSON shows `path_claim_overlap` warnings with an explainable stage split.

## Script

`tooling/benchmarks/mission-loop-scaffold.ps1` creates the log directory layout. Fill cells manually from dogfood sessions; publish raw records even when Pytxo loses.

Back: [[mission-loop]] · [[pytxo-improvement-research]]
