# CHECKPOINT — Pytxo — 2026-11-01 (2.0.0 ships with the new pitch: land the agents' branches)

## Branch / commit

- `mbcz/pytxo-2-land-pitch` in
  `C:\pytxo\.claude\worktrees\pytxo-release-pitch-436e33` is the whole ship:
  2.0.0 (`mbcz/pytxo-2-1-smart`), site v2 (`mbcz/site-v2`), the new pitch
  (`3e55fef`) and this checkpoint. Not pushed or tagged; npm's latest is
  1.3.6. Main (`797ffd6`) is an ancestor. Merging it does not deploy the
  site: `deploy-web.yml` and `apps/web/vercel.json` hold main until
  `PUBLISHED_VERSION` equals the candidate.
- `mbcz/pytxo-2-land-pitch-promote` is one commit on top, the promotion
  `8fbd916` re-applied: `PUBLISHED_VERSION` 2.0.0, main deploys back on,
  2.0.0 as the changelog's public release. Push it only once npm serves
  2.0.0.
- Superseded, not deleted: `mbcz/pytxo-2-1-smart`, `mbcz/site-v2` and
  `mbcz/pytxo-2-promote`; their content is in the two branches above.
  `mbcz/land-what-works` (`e51d2e6`, partial landing) sits on
  `mbcz/site-v2` and needs a rebase onto `mbcz/pytxo-2-land-pitch` for 2.0.1.
- Untracked in this worktree: `RELEASE_TODAY.md`, with the pitch decision
  and the ship commands.

## Completed

- New pitch, approved by Matt on 2026-11-01 ("New pitch, do it"): lead with
  "Land all your agents' branches in one command." and "When two break each
  other, nothing lands." in place of "Each agent's branch passes. Do they
  pass together?" Reason: on Matt's own work, 0 of 21 sibling branch pairs
  failed only together (death match, 2026-10-31, `C:\pytxo\docs\reviews`,
  untracked), so the old lead sold an alarm for a rare event.
  `npx pytxo --apply` already lands a ready result in 2.0.0.
- Changed: the hero, page titles, `PRODUCT_LINE` and `PRODUCT_SUMMARY`, the
  where-it-runs terminal row (`npx pytxo --apply`), the docs intro, README,
  the npm and mirror READMEs, the npm description, the Action description,
  the release notes (one `--apply` clause), `POSITIONING.md` (asking the
  agent to merge as an alternative, no model calls, all-or-nothing landing
  as a limit, new lead rule) and the share card.
- The share card's headline was repainted over the original PNG. The paint
  was first calibrated by redrawing the old headline, which matched the
  original within pixel noise.

## Not done

- Nothing is pushed, tagged, published or deployed. Matt runs the ship.
- The CLI's `--help` about line and `crates/pytxo-cli/Cargo.toml` still
  say "Do they pass together?". That is Rust release code, for 2.0.1.
- The plugin and marketplace descriptions are unchanged: the plugin checks,
  it does not land.
- Carried from 2026-10-31: the CLI's 402 messages name `/plans` Pro perks;
  the signed-in account views keep v1 boxes; Matt's call on `SECURITY.md`
  dropping Desktop 1.3.x.
- Unverified until npm serves 2.0.0: a cold `npx -y pytxo@2.0.0 demo`, the
  Action on github.com, Linux glibc, timing on a real repository, and a real
  hook session.
- Arena ("one task, every agent") is neither built nor measured. The
  rescue-rate pilot is blocked on quota; Claude's weekly limit resets
  2026-11-04.

## Tests last run

- 2026-11-01, `apps/web` on `3e55fef`: eslint, `tsc --noEmit`, `next build`;
  Playwright 31 of 31 through `PLAYWRIGHT_BASE_URL=http://127.0.0.1:3219`;
  `check:links` (201 links); sloplint 0 on the 14 changed files. The hero
  measured 4 lines with the copy button above the fold at 320×700, 390×844,
  1280, 1440 and 1920×900, and 1920×1080.
- 2026-11-01: `node --test tooling/scripts/` 52 of 52; Action 28 of 28; npm
  launcher 7 passed, 4 skipped, 0 failed; `verify-release-version.mjs 2.0.0`
  passes.
- Rust: not rerun, because no Rust changed. The last pass was 2026-10-30 on
  `7a7a2ff` (fmt, clippy `-D warnings`, tests with 0 failed).

## Blockers

- The ship needs Matt, in order: push, PR, merge, tag, npm, Action e2e, the
  `v2` tag, the promotion. The commands are in `RELEASE_TODAY.md`.

## Next prompt (paste below)

```txt
Pytxo after the 2.0.0 ship. Read AGENTS.md, the top of CHECKPOINT.md and
RELEASE_TODAY.md in C:\pytxo\.claude\worktrees\pytxo-release-pitch-436e33.
Matt has run the ship steps. Verify, without pushing or publishing anything
yourself: `npm view pytxo@2.0.0 version` prints 2.0.0, a cold
`npx -y pytxo@2.0.0 demo` exits 0, `tooling/scripts/action-e2e.sh` passes
against the v2 Action tag, and pytxo.com serves the new hero ("Land all
your agents' branches in one command.") and the inner pages (check /,
/plans, /download, /bench at desktop and phone width). Report each result
with its evidence, then list steps 9 to 12 of pytxo-2.0-report.md (in
C:\pytxo\.claude\worktrees\product-review-a0207f) with the exact commands
Matt runs.

Don't push, publish to npm, create releases, edit the GitHub repo
description, deploy the site or change billing. No AI attribution trailers.
```

## Next

<!-- updated 2026-11-01; max 3, ranked -->
1. Ship 2.0.0 with the new pitch — done when: `npm view pytxo@2.0.0
   version` prints 2.0.0, a cold `npx -y pytxo@2.0.0 demo` exits 0, the
   Action e2e passes, the promotion is merged and pytxo.com serves the new
   hero — `C:\pytxo\.claude\worktrees\pytxo-release-pitch-436e33` — blocked:
   Matt runs the ship commands in `RELEASE_TODAY.md`.
2. Dogfood `npx pytxo --apply` on Matt's next batch of agent branches —
   done when: one real run lands or blocks with a receipt and is kept as
   launch footage — `C:\pytxo`.
3. 2.0.1 — done when: `e51d2e6` (partial landing) is rebased onto
   `mbcz/pytxo-2-land-pitch`, the CLI help, Cargo description and 402
   messages carry the new wording, and `cargo test -p pytxo-land -p
   pytxo-cli` passes — `C:\pytxo`.

History: docs/checkpoints/
