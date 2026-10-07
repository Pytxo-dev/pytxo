---
title: Beta fleet release plan
slug: beta-fleet-release-plan-2026-10-01
status: active
tags: [release, beta, desktop, demo, fleet]
audience: [human, agent]
layer: project
created: 2026-10-01
updated: 2026-10-01
related: ["[[RELEASE_READINESS]]", "[[pytxo-interface-engineering]]", "[[commit-layer]]"]
---

# Beta fleet release plan

Decisions from Matt (2026-10-01):

- **Story:** spectacle first, trust as payoff. Several vendors' coding CLIs work
  at once on one repository; everything converges into one exact package; an
  unsafe or stale change is refused; one reviewed Apply lands.
- **Beta scope:** real multi-agent Beta. Up to 8 workers, mixed CLIs per task,
  proven natively by the demo run itself.
- **Hero view in the product:** Work becomes a live fleet board (dependency
  diagram plus streaming worker output, ownership and collision locks). Major
  redesign: mockups need approval before implementation.
- **Film:** Remotion hybrid. Real native 4K footage for product moments; motion
  graphics driven by the recorded run ledger so every animated value is real
  and a rerun reproduces it. No tiny text, no generic AI-demo look.
- **Release surfaces:** Desktop, website and CLI TUI each get an audit and
  concrete mockups, approval, then implementation and verification.

## Phase 1 — Backend: multi-CLI fleet (no UI gate)

- Per-task CLI assignment carried in the reviewed plan, its digest and dispatch
  (today one ADE command template applies to every task).
- Desktop Beta blockers move from "Codex, one worker" to "dispatchable,
  permission-mapped CLIs, 1–8 workers, Orbit, PTY". Raise the local cap to 8.
- Per-CLI headless proof: each installed CLI edits files in its isolated
  workspace and exits; failures surface through `agent_failure_hint`.
- Test isolation: the Rust suite must not register temp repositories in the
  developer's real `~/.pytxo/hypervisor.db` (544 of 558 entries observed).

## Phase 2 — Mockups, approval, implementation

- Desktop: fleet board (Work), New work with multi-CLI assignment, Review that
  shows which CLI produced each file.
- Website: hero and walkthrough rebuilt around the film and fleet board.
- TUI: fleet view (one pane per worker, live output, ownership), readable run
  list (titles, not UUIDs), registry filtered to real projects.

## Phase 3 — Reproducible demo and film

- `docs/demo/fleet/`: deterministic fixture repository, mission, and
  `setup.ps1`/`reset.ps1`; capture script for native footage and the run ledger.
- `apps/demo-video`: new composition driven by the captured ledger JSON plus
  native footage; still/contact-sheet inspection and full-decode checks.

## Acceptance

- One native run on the final MSI with six installed CLIs concurrently, combined
  checks passing, stale refusal, exact Apply and History record.
- Desktop/web/TUI checks and e2e green; screenshots at 1440/1920 and 1280×800.
- Film renders from the committed source and captured artifacts; every number
  on screen traces to the ledger.
