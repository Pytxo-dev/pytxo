# CHECKPOINT — Pytxo — 2026-10-30 (2.0.0 ready to ship; site and market next)

## Branch / commit

- `mbcz/pytxo-2-1-smart` in `C:\pytxo\.claude\worktrees\product-review-a0207f`
  is the whole 2.0.0 release (its release content ends at `3b0c168`). Not
  pushed or tagged; npm's latest is 1.3.6. Main (`797ffd6`) is an ancestor.
- `mbcz/pytxo-2-promote` in `C:\pytxo\.claude\worktrees\pytxo-2-promote` is
  one commit on top: `PUBLISHED_VERSION` 2.0.0, main deploys back on, 2.0.0
  as the changelog's public release. Push it only once npm serves 2.0.0.
- Ship report with every command (untracked):
  `C:\pytxo\.claude\worktrees\product-review-a0207f\pytxo-2.1-report.md`.
  "Decisions for you" and steps 9 to 12 are in `pytxo-2.0-report.md` in the
  same folder, the growth plan in `pytxo-growth-plan.md`.

## Completed

- 2.0.0 is 2.0's check plus the work planned as 2.1: the cause (a renamed or
  removed name another branch still uses, confirmed when the failing output
  mentions it), the pair runs, the fix prompt and the waiting hooks.
- Renumbered from 2.1.0 to 2.0.0 (DECISIONS 49). 1.0.0 is not possible: npm
  has pytxo 1.0.0 (tag `v1.0.0`) and never reuses a version.
- Every version surface reads 2.0.0. `PUBLISHED_VERSION` stays 1.3.6 and
  `apps/web/vercel.json` keeps `"main": false` until npm serves 2.0.0.
- One set of notes (`distribution/release-notes/v2.0.0.md`), one changelog
  row and one demo record
  (`apps/web/public/evidence/demo-blocked-2.0.0-2026-10-30.json`).
- Reviews, each finding checked by a verifier before a fix: the check (two
  rounds), the Action (one), the public surfaces (four) and the renumber
  (one, fixed in `3b0c168`).
- Checkpoint history moved to `docs/checkpoints/` (this file was 567 KB).

## Not done

- Nothing is pushed, tagged, published or deployed. Matt runs the ship.
- `SECURITY.md` says 2.x is supported and 1.3 and older are not, which
  leaves Desktop 1.3.x without security fixes. Matt's call.
- Unverified until npm serves 2.0.0: a cold `npx -y pytxo@2.0.0 demo`, the
  Action on github.com (`tooling/scripts/action-e2e.sh`), Linux glibc, a real
  repository's timing and a real hook session.
- The site, product and market pass Matt asked for on 30 October (next
  prompt below). Not started.

## Tests last run (2026-10-30)

- `node tooling/scripts/verify-release-version.mjs 2.0.0`: pass, published
  1.3.6 here and 2.0.0 on the promote branch.
- `node --test tooling/scripts/*.test.mjs`: 52 of 52 on both branches.
- Action tests 28 of 28 and the npm launcher's 9 of 9 on `0a2d654`; neither
  changed after it.
- `apps/web`: `npm run build` on `3b0c168`, Playwright 34 of 34 on
  `0a2d654`, lint and `tsc --noEmit` before the renumber.
- Rust on `7a7a2ff` (later Rust changes are help text, a comment and the
  version): fmt, clippy `-D warnings` on the workspace and the lean build,
  tests with 0 failed; the hook help test on the full and lean CLI after.

## Blockers

- The ship needs Matt: push, pull request, merge, tag, npm (`NPM_TOKEN`),
  the Action's `v2` tag and the promotion, in that order (report steps 1 to
  8).
- Secrets: none stored here. `NPM_TOKEN` is a repository secret; rotate it
  if the npm step fails with 401 or 403. The Pro licensing keypair is not
  made yet.

## Next prompt (paste below)

```txt
Pytxo site, product and market pass. Read AGENTS.md, the top entry of
CHECKPOINT.md, POSITIONING.md and
C:\pytxo\.claude\worktrees\product-review-a0207f\pytxo-growth-plan.md.
Base all work on mbcz/pytxo-2-1-smart, the unpushed 2.0.0 release (main
does not have it): unless your branch already contains it, run
`git switch -c mbcz/pytxo-site mbcz/pytxo-2-1-smart` before editing.

Matt asked, 30 October 2026: "i want the pytxo website to genuinely feel
professional, clean and like a $100,000 site. ASCII animations and art, and
a lot more. make it insanely convincing and an easier pitch. what will make
an extremely good, releasable product? and how we can market this later
easily. look at every ai agent dev tool, foss projects, and the whole entire
market"

1. Market (a workflow): every AI coding agent and agent dev tool, open
   source and paid (agents, parallel-agent managers, AI review, merge queues,
   stacked pull requests), from primary sources with current pricing: what
   each does, who pays, how it got its first users, and where Pytxo's check
   fits or is already covered. One cited research note.
2. Product: from that, a ranked list of what would make Pytxo an extremely
   good, releasable product, including what people would pay for (Pro is
   scaffolded and switched off; prices are Matt's call), with effort and
   evidence. Recommendations only; changes to the check get their own
   session.
3. Marketing: a launch and distribution plan that is easy to run later: the
   one-line pitch, channels, assets and draft copy. Nothing posted.
4. Website (apps/web): a redesign that feels like a $100,000 site, with
   ASCII art and animations built from real output (the demo record
   apps/web/public/evidence/demo-blocked-2.0.0-2026-10-30.json), never
   invented terminal output, UI or numbers. Major-redesign gate: first show
   Matt a direction with a rendered prototype (the hero and one section,
   desktop and mobile screenshots) and wait for his yes. Load $taste
   (product-ui) and matt-workflow's pytxo-marketing reference, and read the
   relevant guide in apps/web/node_modules/next/dist/docs before writing
   code. Claims match the code and the reviewed wording in POSITIONING.md
   and the docs: the cause is named only for a renamed or removed name that
   the failing output mentions, and the pair runs need three or more
   branches and a cause. Respect prefers-reduced-motion. Keep site.ts's
   versions and vercel.json's "main": false (the release inventory test
   needs it until npm serves 2.0.0). Done when lint, tsc, next build and
   Playwright pass (on a private port through PLAYWRIGHT_BASE_URL, since
   another worktree may hold 3101), the screenshots are reviewed and the
   work is committed locally.

Don't push, publish to npm, create releases, edit the GitHub repo
description, deploy the site or change billing. Don't touch payments, Dodo
or any live checkout. No telemetry on by default. No AI attribution
trailers. Checkpoint before the build if the session gets large.
```

## Next

<!-- updated 2026-10-30; max 3, ranked -->
1. Site, product and market pass (prompt above) — done when: the research
   note and the product and marketing plans are written, Matt approved a
   site direction, and the redesign passes lint, tsc, build and Playwright
   with screenshots reviewed, committed locally — `C:\pytxo` — blocked:
   Matt's yes on the direction before the full build.
2. Ship 2.0.0 — done when: `npm view pytxo@2.0.0 version` prints 2.0.0, a
   cold `npx -y pytxo@2.0.0 demo` exits 0, the Action e2e passes and the
   promotion is merged — `C:\pytxo\.claude\worktrees\product-review-a0207f`
   — blocked: Matt runs steps 1 to 8 of `pytxo-2.1-report.md`.
3. After the ship, steps 9 to 12 of `pytxo-2.0-report.md` — done when: the
   repository description and topics are set, Linux glibc and a real
   repository are checked, and a real hook session ran before the plugin
   directory submission — `C:\pytxo` — blocked: Matt edits the description
   and submits.

History: docs/checkpoints/
