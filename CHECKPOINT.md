# CHECKPOINT — Pytxo — 2026-10-31 (2.0.0 and the whole v2 site ready to ship)

## Branch / commit

- `mbcz/pytxo-2-1-smart` in `C:\pytxo\.claude\worktrees\product-review-a0207f`
  is the whole 2.0.0 release (its release content ends at `3b0c168`). Not
  pushed or tagged; npm's latest is 1.3.6. Main (`797ffd6`) is an ancestor.
- `mbcz/site-v2`, same worktree, is that branch plus the site redesign: the
  home page (`dc95ec9`), the inner pages (`2d9da1a`) and the terms price
  (`a09770f`). Not pushed. The site
  goes live only through main (`deploy-web.yml` runs on push to main), so it
  ships with 2.0.0 or after.
- `mbcz/pytxo-2-promote` in `C:\pytxo\.claude\worktrees\pytxo-2-promote` is
  one commit on top of the release: `PUBLISHED_VERSION` 2.0.0, main deploys
  back on, 2.0.0 as the changelog's public release. Push it only once npm
  serves 2.0.0.
- Untracked in the same folder: `pytxo-2.1-report.md` (ship commands),
  `pytxo-2.0-report.md` (decisions, steps 9 to 12), `pytxo-market-map.md`,
  `pytxo-growth-plan.md`, `pytxo-launch-plan.md`, and the site prototypes in
  `site-prototype/` (the approved one is `site-prototype/v2/mark/index.html`).
  The generated `apps/desktop/src-tauri/gen/schemas/*.json` changes stay
  uncommitted.

## Completed

- 2.0.0 is 2.0's check plus the work planned as 2.1 (the cause, the pair
  runs, the fix prompt, the waiting hooks), renumbered to 2.0.0 (DECISIONS
  49). Every version surface reads 2.0.0; `PUBLISHED_VERSION` stays 1.3.6 and
  `apps/web/vercel.json` keeps `"main": false` until npm serves 2.0.0.
- Market map, growth plan and launch plan written (untracked reports above).
- Site v2, after Matt approved the direction ("Approve, build it"): two
  colors (ink `#08080a`, paper `#eceae4`) and one gradient that marks only
  what breaks together; Sora and IBM Plex Mono. The home page is the
  question over a WebGL λ drawn in text, the 2.0.0 demo record replayed at
  its pace, the ledger of the four test runs, where it runs, and the close.
  The home FAQ moved to `/docs/reference/faq`.
- Inner pages in the same system (`2d9da1a`): plans, download, Pybench,
  privacy, terms, refunds, account, account connect, shared receipts and the
  404. No cards, kickers or icons; ruled rows, the home page's type, and the
  content column on the header's (x = 100 at 1440, 20 at 390). The
  gradient marks only "lands a broken main" on Pybench. Prices, claims,
  links and test ids unchanged; `os-icons.tsx` and `plan-roadmap-card.tsx`
  deleted. The shadcn tokens map to ink and paper under `[data-site="v2"]`.

## Not done

- Nothing is pushed, tagged, published or deployed. Matt runs the ship.
- `/terms` now states the `/plans` price (Matt: "Match /plans"; `a09770f`,
  shared `PRO_LAUNCH` in `lib/site.ts`). Still $12/month: `/account`
  (`PRO_PRICE` in `lib/billing/pro.ts`, the Dodo product) and
  `content/docs/getting-started/account.mdx`. The $12 checkout still opens
  where `DODO_ENVIRONMENT=live`. Closing it is billing, Matt's call
  (`pytxo-launch-plan.md` item 3).
- The signed-in account views (`components/account-auth.tsx`: entitlements,
  receipts, Desktop routing) still draw their v1 rounded boxes inside the v2
  page; not captured, since they need a sign-in.
- The receipt page keeps `StateChip` (the epistemic-state colors of
  ADR-0038) next to the verdict, the one place a status color remains.
- `SECURITY.md` says 2.x is supported and 1.3 and older are not, which
  leaves Desktop 1.3.x without security fixes. Matt's call.
- Unverified until npm serves 2.0.0: a cold `npx -y pytxo@2.0.0 demo`, the
  Action on github.com (`tooling/scripts/action-e2e.sh`), Linux glibc, a real
  repository's timing and a real hook session.

## Tests last run

- 2026-10-31, `apps/web` at `2d9da1a`: `next build`; Playwright 31 of 31
  through `PLAYWRIGHT_BASE_URL=http://127.0.0.1:3217`; eslint,
  `tsc --noEmit`, `check:links` (201 links); sloplint 0 on the 13 changed
  files. Captures of all 10 inner pages at 1440×900 and 390×844 reviewed:
  the h1 sits on the header's brand edge, no page scrolls sideways. The
  receipt page was rendered against a local stand-in for Link
  (`LINK_URL=http://127.0.0.1:3218`).
- 2026-10-31, home page at `dc95ec9`: the headline stays four lines and the
  copy button stays above the fold from 320×700 to 1920×1080; the hero
  renders at 144 fps with a 7.1 ms p95 frame on this laptop.
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
Pytxo after the 2.0.0 ship. Read AGENTS.md, the top of CHECKPOINT.md and
pytxo-2.1-report.md in C:\pytxo\.claude\worktrees\product-review-a0207f.
Matt has run the ship steps. Verify, without pushing or publishing anything
yourself: `npm view pytxo@2.0.0 version` prints 2.0.0, a cold
`npx -y pytxo@2.0.0 demo` exits 0, `tooling/scripts/action-e2e.sh` passes
against the v2 Action tag, and pytxo.com serves the v2 home page and inner
pages (check /, /plans, /download, /bench at desktop and phone width).
Report each result with its evidence, then list the remaining steps 9 to
12 of pytxo-2.0-report.md with the exact commands Matt runs.

Don't push, publish to npm, create releases, edit the GitHub repo
description, deploy the site or change billing. Don't touch payments, Dodo
or any live checkout. No AI attribution trailers.
```

## Next

<!-- updated 2026-10-31; max 3, ranked -->
1. Ship 2.0.0 and site v2 — done when: `npm view pytxo@2.0.0 version`
   prints 2.0.0, a cold `npx -y pytxo@2.0.0 demo` exits 0, the Action e2e
   passes, the promotion is merged and pytxo.com serves the v2 site —
   `C:\pytxo\.claude\worktrees\product-review-a0207f` — blocked: Matt runs
   steps 1 to 8 of `pytxo-2.1-report.md`, then merges `mbcz/site-v2`.
2. One Pro, one price everywhere — done when: `/account`, the account docs
   and the CLI's 402 messages name the `/plans` Pro and the $12 checkout
   cannot open — `C:\pytxo\.claude\worktrees\product-review-a0207f` —
   blocked: Matt approves closing the $12 checkout (billing).
3. After the ship, steps 9 to 12 of `pytxo-2.0-report.md` — done when: the
   repository description and topics are set, Linux glibc and a real
   repository are checked, and a real hook session ran before the plugin
   directory submission — `C:\pytxo` — blocked: Matt edits the description
   and submits.

History: docs/checkpoints/
