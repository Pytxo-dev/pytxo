# Public beta launch — running state

Owner: Claude (session started 2026-10-07). Branch `mbcz/pytxo-public-beta-636343`,
fast-forwarded from main to the beta candidate `4881904`
(`2ntt/pytxo-beta-candidate-20261003`, 105 commits past main). This file is the
short running record; `RELEASE_READINESS.md` keeps the detailed evidence.

## Positioning (kept)

One job → many agents → one verified change. Parallel agents in worktrees are a
commodity (Conductor, Superset, Orca, Emdash, Herdr, vendor apps). Pytxo's
difference is the *end* of the run: one plan split by file ownership (no
branch-per-agent merge), checks Pytxo runs itself on the combined result, and an
exact, stale-safe, journaled Apply. Lead with the review/merge pain the
parallel-agent crowd complains about: "10 agents = 10 branches to read".

## Gates

| # | Gate | State | Unblock |
|---|------|-------|---------|
| 1 | Hosted CI / candidate MSI / cloud acceptance | **Blocked** — GitHub Actions jobs refuse to start: "recent account payments have failed or your spending limit needs to be increased" (run 37344182755, 2026-10-05) | Matt fixes Billing & plans for `Pytxo-dev`, or approves the documented local Windows build fallback |
| 2 | Fresh candidate after Oct 5–6 UI changes | Open — last accepted MSI is `cc0baa8` (Oct 4); onboarding, sidebar and Fleet changed since | Rebuild + rerun acceptance (needs gate 1 or local fallback) |
| 3 | Live vendor agents on the final artifact | Open — Oct 2 run was live; later acceptance used stand-in replays | One native run with real CLIs on the final build |
| 4 | Source visibility | **Decision** — `Pytxo-dev/pytxo` is private (README claims MIT); the public GitHub face is `pytxo-releases` | Matt: open-source for launch (recommended for HN/GitHub trust) or keep private and lead with the releases repo |
| 5 | Publication | Approval-gated — merge, tag, GitHub release, Vercel deploy, posts | Matt approves each exact action |

## Work log

- 2026-10-07 — Audit: candidate branch, release readiness, live site (still
  v1.2.1 copy), film v7, CI state, public repos.
- 2026-10-07 — Biggest usefulness gap: the default planner only splits a job
  that the user already wrote as one line per task naming existing files; a
  normal paragraph became one task or a "name the files" block. Shipped:
  - **Split with Codex / Claude Code** in New work. The lead (or first ready)
    agent reads the project in its read-only mode and returns one task per line
    with `| files:` ownership; Undo, Cancel, elapsed timer, and a warning if the
    project changed while it read. Native: `pytxo_orchestrate::split`,
    `flow_split_request` / `flow_split_cancel`.
  - Planner: a trailing `| files: a, b` list owns exactly those paths, and new
    files in existing directories are allowed (were rejected before).
  - New work defaults to every ready beta agent and that many workers; header
    says "Lead agent"; header stacks below 1100 px (intro used to run under the
    selectors at 860 px).
  - Evidence: planner 37 tests, split 4 unit tests; live Claude split 3 tasks /
    35 s / project unchanged; live Codex split 1 task / 112 s / unchanged;
    svelte-check clean; `split-request.spec.ts` + updated fleet spec pass.
  - Copy: Work map says "Agents on this run" and "Agent says done"; plan task
    headers no longer repeat a default agent that contradicted the select;
    Review header shows `…/parent/project` (full path in tooltip) instead of
    leaking deep dev paths like `.claude/worktrees/…`.
  - Checks: full desktop e2e 443 passed / 8 skipped; `cargo check -p
    pytxo-desktop` OK; `cargo clippy -p pytxo-planner -p pytxo-orchestrate -p
    pytxo-desktop --all-targets -D warnings` OK; rustfmt clean on touched files.
  - Captures regenerated (69 PNGs: desktop-2, docs, web, demo-video).
  - Open: one native split on the final MSI (gate 3).
- 2026-10-07 — Distribution: `pytxo-releases` README rewritten as the public
  front page (why, install, limits, feedback); maintainer notes moved to
  `distribution/pytxo-releases/MAINTAINING.md` (not synced). Website "How it
  works" step 1 mentions Split. Web build untested here (no `node_modules`).
- 2026-10-07 — Launch kit (local drafts, nothing posted):
  `D:/marketing/campaigns/pytxo-public-beta-launch/` — fact sheet with sources
  and do-not-claim list, Product Hunt fields + maker comment, X drafts, HN /
  Reddit research only (Matt writes all HN and Reddit wording).
- Still open: film re-record on the final build (v7 shows the old UI and a dev
  path); root README version (1.2.1) bumps at release; site deploy before the
  releases README sync (its screenshot URL 404s on the live site).
