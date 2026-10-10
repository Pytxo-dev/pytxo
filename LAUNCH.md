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
| 1 | Hosted CI / candidate MSI / cloud acceptance | **Done** — repo public 2026-10-07, so standard runners are free and unblocked. PR #33: CI green, candidate MSI built, cloud acceptance (install, mixed-agent journey, stale refusal, Apply, 150/175% layout) passed | — |
| 2 | Fresh candidate after Oct 5–6 UI changes | **Done** — candidate run 37651799067 at `15758b7`, acceptance 37652820730 passed | — |
| 3 | Live vendor agents on the final artifact | **Done** — 2026-10-10, the released v1.2.4 MSI's exe (`CA123A4E…`, from an administrative extract of the mirror MSI): Claude Code split the paragraph into 7 owned tasks; Claude Code, Codex and Cursor Agent ran them live, 7/7 completed; 14 files reviewed = 14 applied; stale refusal, refresh, Apply `applied`; fixture tests 22/22 after Apply. Evidence `D:/pytxo-native-acceptance/film-124-live2` | Antigravity: no changes in live runs (headless tool permission auto-denied); OpenCode: its configured default model cannot use tools here. Separate task |
| 4 | Source visibility | **Done: public (MIT)** — gitleaks clean; private vulnerability reporting, secret scanning + push protection, Dependabot alerts and fork-PR approval on | — |
| 5 | Publication | **Done** — v1.2.5 tagged at `03eb9cd` (PR #43); release run 38046963880 green; `pytxo-releases` mirror v1.2.5 (MSI + CLI checksums verified, CLI reports 1.2.5); npm `latest` 1.2.5; native check of the released exe passed. v1.2.4 (`ca214bb`) before it | Left: YouTube upload of film v8, Product Hunt submission, launch posts (Matt writes HN/Reddit) |

## Work log

- 2026-10-10 — v1.2.5 published: New work fits small windows (sticky header and
  bar no longer show text around them under compact density; no title gap,
  plan cards and paths stay in the panel), one page-title size and weight
  across destinations, New work in the sidebar header row, Saved requests in
  the heading, Light theme request text and saved-request titles readable.
  PRs #42 and #43; release run 38046963880; mirror checksums and npm verified;
  native check of the released exe over WebView2 remote debugging passed.
- 2026-10-10 — Gate 3 and the film take. pytxo.com deployed with the 1.2.4
  docs flip (PR #40, `f65a680`); live site serves the v1.2.4 MSI and hero clip.
  Stand-in rehearsal on the released exe passed (7/7 files). Live take 1
  (Claude Code lead + Cursor + Antigravity) passed the journey, but both
  Antigravity tasks exited 0 with no edits: its headless mode auto-denied a
  "command" tool permission, and Desktop showed only "no changes". Codex was
  left off because the journey's click landed on New work's sticky footer.
  Fixed in the journey: scroll until the control itself is under the pointer,
  and never click or type into another app's window (Roblox Studio opened over
  Pytxo during one attempt; that take was stopped). Live take 2 (Claude Code
  lead + Codex + Cursor) passed end to end (see gate 3). The applied app has
  integration bugs the agents' tests did not catch (duplicate theme button, the
  empty-state line shown beside matching tasks, light page under a dark system
  setting): checks passing is not correctness, and the film does not show the
  result app. Film v8 cut from take 2: `apps/demo-video/out/pytxo-live-v8.mp4`.
  Product Hunt thumbnail (PNG + GIF) and 8-image gallery from take-2 stills:
  `D:/marketing/campaigns/pytxo-public-beta-launch/product-hunt/out/`.
- 2026-10-09 — v1.2.4 published: terminal fleet monitor, Review Apply gate,
  background run-finished notification, 1–9 worker keys, store open fix. PR #39
  CI green (incl. Windows Rust), merged as `ca214bb`, tagged, release green,
  mirror checksums and npm verified. Gate 3 (live CLIs on the released MSI)
  still open.
- 2026-10-09 — Terminal fleet redesign (approved: "go all in", "add your own UI
  style"). Desktop: fleet monitor (ASCII scope, activity sparklines, TUI worker
  panes, auto-follow, converge finale), New work fleet preview and compact plan
  panes, terminal status line, ASCII empty state. Site: ASCII aperture hero with
  orbiting agents, hero clip of Desktop playing a sample run, ASCII wordmark
  footer. Native check over WebView2 remote debugging (no pointer takeover)
  found and fixed: the fleet preview clipping at the default 1280 px window; a
  refused Run shown as a stale plan; a store migration race on a newly added
  project ("duplicate column name", "database is locked") plus a repair for
  stores it had already broken. Branch `mbcz/fleet-ascii-redesign`; not merged
  or released. Second pass the same day: Review shows the Apply gate as a
  terminal pipeline; Canvas, History and Setup take the terminal language; a
  background run-finished notification; 1–9 worker jump keys; the site's
  comparison section plays an Apply gate illustration.

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
- 2026-10-07 — Website verified locally: `pnpm lint` clean, `pnpm build` OK,
  web e2e 28/28 (Chrome channel; the pinned Playwright Chromium isn't
  installed). Fixed: Download cards overflowed a 390 px screen by 10 px.
  Guides (`multi-agent-run`, `desktop`) and v1.2.2 notes now describe Split,
  `| files:` ownership and the default team.
- 2026-10-07 — Acceptance journey: with agents pre-ticked it clicked none, so
  the film cut lost its agent beat (`live-cut.mjs` reads that hover). It now
  points at each team agent and ticks only missing ones. Next film take needs
  the final MSI (gates 1–3) and takes over the screen while recording.
- 2026-10-07 — Actions cost: Oct 1–7 used 1,304 Windows + 1,450 Linux min
  ($10 over the free 2,000), almost all PR CI (Rust on Windows ~17 min/run,
  compiled cold — no cache). CI now scopes PR jobs to the files changed,
  caches Rust, folds six jobs that each recompiled the CLI into the Rust job,
  and keeps macOS + the Linux Desktop release build for main/release.
  Acceptance no longer bills Linux while waiting for the Windows build.
- 2026-10-08 — Released v1.2.2. Fixes found on the way: npm audit advisories
  (sharp, source-map-js), CI scope pipe bug, a guided-example button without a
  stable accessible name (broke acceptance layout passes), an expired then
  2FA-blocked `NPM_TOKEN`, and `.vercelignore` hiding the favicon encoder from
  production builds (site deploys use the Vercel CLI; the deploy-web workflow
  has no Vercel secrets).
- Still open: film re-record on the final build (v7 shows the old UI and a dev
  path); root README version (1.2.1) bumps at release; site deploy before the
  releases README sync (its screenshot URL 404s on the live site).
