# CHECKPOINT — Pytxo — 2026-10-31 (2.0.0 and site v2 ready to ship; inner pages next)

## Branch / commit

- `mbcz/pytxo-2-1-smart` in `C:\pytxo\.claude\worktrees\product-review-a0207f`
  is the whole 2.0.0 release (its release content ends at `3b0c168`). Not
  pushed or tagged; npm's latest is 1.3.6. Main (`797ffd6`) is an ancestor.
- `mbcz/site-v2`, same worktree, is that branch plus the site redesign
  (`dc95ec9`). Not pushed. The site goes live only through main
  (`deploy-web.yml` runs on push to main), so it ships with 2.0.0 or after.
- `mbcz/pytxo-2-promote` in `C:\pytxo\.claude\worktrees\pytxo-2-promote` is
  one commit on top of the release: `PUBLISHED_VERSION` 2.0.0, main deploys
  back on, 2.0.0 as the changelog's public release. Push it only once npm
  serves 2.0.0.
- Untracked in the same folder: `pytxo-2.1-report.md` (ship commands),
  `pytxo-2.0-report.md` (decisions, steps 9 to 12), `pytxo-market-map.md`,
  `pytxo-growth-plan.md`, `pytxo-launch-plan.md`, and the site prototypes in
  `site-prototype/` (the approved one is `site-prototype/v2/mark/index.html`).

## Completed

- 2.0.0 is 2.0's check plus the work planned as 2.1 (the cause, the pair
  runs, the fix prompt, the waiting hooks), renumbered to 2.0.0 (DECISIONS
  49). Every version surface reads 2.0.0; `PUBLISHED_VERSION` stays 1.3.6 and
  `apps/web/vercel.json` keeps `"main": false` until npm serves 2.0.0.
- Market map, growth plan and launch plan written (untracked reports above).
- Site v2, after Matt approved the direction ("Approve, build it"): two
  colors (ink `#08080a`, paper `#eceae4`) and one gradient that marks only
  what breaks together; Sora and IBM Plex Mono. The home page is the
  question over a WebGL λ drawn in text (an SVG still without WebGL, one still
  frame under reduced motion), the 2.0.0 demo record replayed at its pace, a
  ledger of the four test runs taken from the same build's pull request
  comment (`apps/web/public/evidence/comment-blocked-2.0.0-2026-10-30.json`,
  rendered locally, never posted), where it runs, and the close. Every
  marketing page has the new header and footer. The home FAQ moved to
  `/docs/reference/faq` ("Questions").

## Not done

- Nothing is pushed, tagged, published or deployed. Matt runs the ship.
- The inner pages (plans, download, bench, privacy, terms, refunds, account
  connect, 404) keep their v1 bodies under the v2 header and footer: cards,
  green status dots and a content column about 50 px right of the header's
  at 1440 px.
- The home page no longer has the founder line or the lukeed/mri run with
  real agents (the approved v2 has neither); that run is still in the docs
  at `/docs/evidence/first-use`.
- `SECURITY.md` says 2.x is supported and 1.3 and older are not, which
  leaves Desktop 1.3.x without security fixes. Matt's call.
- Unverified until npm serves 2.0.0: a cold `npx -y pytxo@2.0.0 demo`, the
  Action on github.com (`tooling/scripts/action-e2e.sh`), Linux glibc, a real
  repository's timing and a real hook session.

## Tests last run

- 2026-10-31, `apps/web` at the site commit: `next build`; Playwright 31 of
  31 through `PLAYWRIGHT_BASE_URL=http://127.0.0.1:3217`; eslint,
  `tsc --noEmit`, `check:links` (201 links); sloplint 0 on the 18 changed
  files. Captures: the headline stays four lines and the copy button stays
  above the fold from 320×700 to 1920×1080; the hero renders at 144 fps with
  a 7.1 ms p95 frame on this laptop.
- 2026-10-30: `verify-release-version.mjs 2.0.0` pass; `node --test
  tooling/scripts/*.test.mjs` 52 of 52; Action tests 28 of 28 and the npm
  launcher's 9 of 9 on `0a2d654`; Rust on `7a7a2ff`: fmt, clippy
  `-D warnings`, tests with 0 failed.

## Blockers

- The ship needs Matt: push, pull request, merge, tag, npm (`NPM_TOKEN`),
  the Action's `v2` tag and the promotion, in that order (report steps 1 to
  8). The site deploys when `mbcz/site-v2` reaches main.
- Secrets: none stored here. `NPM_TOKEN` is a repository secret; rotate it
  if the npm step fails with 401 or 403. The Pro licensing keypair is not
  made yet.

## Next prompt (paste below)

```txt
Pytxo inner pages in the site v2 system. Read AGENTS.md, the top of
CHECKPOINT.md and POSITIONING.md. Build on mbcz/site-v2 (the unpushed
2.0.0 release plus the approved v2 home page): unless your branch already
contains it, run `git switch -c mbcz/site-v2-pages mbcz/site-v2` before
editing. The system is in apps/web/src/app/globals.css (the
[data-site="v2"] tokens) and apps/web/src/components/site/site.module.css;
the home page (apps/web/src/app/(marketing)/page.tsx) is the reference.

Restyle the inner pages to match it: plans, download, bench, privacy, terms,
refunds, account/connect and src/app/not-found.tsx. Two colors and one
gradient that marks only what breaks together; no cards, kickers or extra
status colors; the content column aligned with the header's; type as large
and plain as the home page. Keep every claim, price, link and test id that
the e2e tests and POSITIONING.md rely on: restyle, don't rewrite. Load $taste
(product-ui) and read apps/web/node_modules/next/dist/docs before code.
Respect prefers-reduced-motion. Done when lint, tsc, next build, check:links,
sloplint 0 and Playwright pass (through PLAYWRIGHT_BASE_URL on a private
port, never 3101), desktop and phone screenshots of each page are reviewed,
and the work is committed locally.

Don't push, publish to npm, create releases, edit the GitHub repo
description, deploy the site or change billing. Don't touch payments, Dodo
or any live checkout. No telemetry on by default. No AI attribution
trailers.
```

## Next

<!-- updated 2026-10-31; max 3, ranked -->
1. Inner pages in the v2 system (prompt above) — done when: plans,
   download, bench, the legal pages, account connect and the 404 match the
   home page with their claims unchanged, the checks pass, screenshots are
   reviewed and the work is committed locally on a branch from
   `mbcz/site-v2` — `C:\pytxo`
2. Ship 2.0.0 and site v2 — done when: `npm view pytxo@2.0.0 version`
   prints 2.0.0, a cold `npx -y pytxo@2.0.0 demo` exits 0, the Action e2e
   passes, the promotion is merged and pytxo.com serves the v2 home page —
   `C:\pytxo\.claude\worktrees\product-review-a0207f` — blocked: Matt runs
   steps 1 to 8 of `pytxo-2.1-report.md`, then merges `mbcz/site-v2`.
3. After the ship, steps 9 to 12 of `pytxo-2.0-report.md` — done when: the
   repository description and topics are set, Linux glibc and a real
   repository are checked, and a real hook session ran before the plugin
   directory submission — `C:\pytxo` — blocked: Matt edits the description
   and submits.

History: docs/checkpoints/
