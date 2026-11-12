# CHECKPOINT — Pytxo — 2026-11-12 (2.0.2 is published; this checkpoint merges with the site promotion)

## Branch / commit

- `main` at `ae9370f0` is tagged `v2.0.2`. Pull request 100 merged the candidate on
  12 Nov: `70194c2a` sets the versions, `b5aaf98e` updates the site docs and `eb38c985`
  is the checkpoint before this one.
- Release run 43208737324 published it on 12 Nov (20 jobs, CI gate included). npm
  `latest` is 2.0.2, and `Pytxo-dev/pytxo` and `Pytxo-dev/pytxo-releases` each have a
  `v2.0.2` release with 10 assets.
- This checkpoint is on `mbcz/promote-2-0-2-pr` with `7943c3aa` "site: 2.0.2 is the
  published release". That commit sets `PUBLISHED_VERSION`, drops "(candidate)" from
  the changelog row and lifts the site's deploy hold in `apps/web/vercel.json`. When
  this file is on `main`, that commit is too.
- The Action's `v2` tag is still on `v2.0.0` (`9a6b2c45`). See Not done.

## Completed

- 2.0.2 is public. On top of the line per branch, `--days` and the pile lead that
  `main` already had, it adds the hint for branches that match no pattern, the named
  test runs, a demo that lands a pile of four, every conflicting branch named and no
  test run when none merges (pull request 99). Notes:
  `distribution/release-notes/v2.0.2.md`.
- Checked from npm on Windows after the publish: `npx pytxo@2.0.2 --version` prints
  `pytxo 2.0.2`, and `npx pytxo@2.0.2 demo` exits 0 in 20 s with "LANDED   2 of 4
  branches are on main now."
- CI on `main` passed on `ae9370f0`, macOS included, before the tag.
- The merge of pull request 100 changed `Cargo.toml` and `Cargo.lock`, so Railway
  redeployed link.pytxo.com and cloud.pytxo.com. Both deploys succeeded and `/health`
  returned 200 on both.
- A read after that merge showed pytxo.com still on 2.0.1, as the hold intended.

## Not done

- The live site was not checked when this was written, because the promotion merges
  with this file. After the merge, `/docs/reference/cli` should say "This page
  describes 2.0.2" and the quickstart should show the LANDED demo. A green Vercel check
  on the pull request proves nothing.
- The Action's `v2` tag did not move. Matt approved the move on 12 Nov, but the agent
  session was not allowed to force-push the tag, so Matt runs it himself:
  `git fetch origin tag v2.0.2 && git tag -f v2 "v2.0.2^{commit}" && git push -f origin v2`.
  Until then `Pytxo-dev/pytxo/actions/land@v2` installs pytxo 2.0.0.
- The landing page still replays the 2.0.0 recording of the old demo
  (`apps/web/public/evidence/demo-blocked-2.0.0-2026-10-30.json`: BLOCKED, 10 rows,
  8.5 s), and 2.0.2's demo prints 32 lines ending in LANDED. `recorded-demo.tsx`, the
  share image `/social/check-1600x1000.png` and its alt (old headline) are built on the
  old one. The footer says "pytxo 2.0.0", so it stays a true record. Re-recording
  changes the landing page, so Matt sees a proposal first. Not started.
- Pull request 98 (`mbcz/fix-loop`, `--fix`) is open in another session and is not in
  2.0.2. It needs a rebase onto `main`.
- Not run by hand: macOS and Linux terminals, the GitHub Action on a live repository,
  the Claude Code plugin and the Codex hook.
- Not run locally for 2.0.2: a release build, `cargo test --workspace`, the
  default-feature build and the site's lint, build and Playwright tests. CI ran them on
  pull request 100, on `main` and in the release gate.
- Carried over from 7 Nov, not rechecked: the listed review findings,
  `public-release-check`'s 37 old commits with an AI co-author trailer, the 2.0.0 notes
  line, `SECURITY.md` and the two e2e repositories. See `docs/checkpoints/2026-11.md`.
- Field test, unchanged: 0 real BLOCKED in 10 valid pairs
  (`D:\marketing\campaigns\pytxo-foss-study-2026-11\RESULTS.md`).

## Tests last run

All 2026-11-12, on Windows where a machine is involved; exit 0.

| Command | Result |
|---|---|
| CI on `main` at `ae9370f0` (run 43191761265) | passed on Windows, Ubuntu and macOS |
| Release workflow on `v2.0.2` (run 43208737324) | passed, 20 jobs |
| `npm view pytxo version --prefer-online` | 2.0.2 |
| `gh release view v2.0.2 --json assets` in both repositories | 10 and 10 assets |
| `npx -y pytxo@2.0.2 --version` | `pytxo 2.0.2` |
| `npx -y pytxo@2.0.2 demo` | exit 0, 20 s, 32 lines, LANDED 2 of 4 |
| `node tooling/scripts/verify-release-version.mjs` on this branch | candidate 2.0.2, published 2.0.2 |
| `node --test` on the two `verify-release-*` tests, on this branch | 30 passed |

The candidate's local checks of 11 Nov are in `docs/checkpoints/2026-11.md`.

## Blockers

- Matt runs the `v2` tag command above.
- Matt's calls before the launch: whether the landing page's demo is re-recorded before
  the film, whether `--fix` (pull request 98) ships before the film, and the OK on the
  PYTXO-3 draft.

## Rollback

An npm version can't be republished, so a bad 2.0.2 is fixed by 2.0.3. Until then run
`npm dist-tag add pytxo@2.0.1 latest` and revert the promotion commit. If `v2` has
moved, put it back with `git tag -f v2 9a6b2c45 && git push -f origin v2`.

## Next prompt (paste below)

```txt
Read CHECKPOINT.md in C:\pytxo. 2.0.2 is published and the site promotion is merged.
Check that pytxo.com/docs/reference/cli says 2.0.2 and that the v2 tag points at
v2.0.2. Then prepare the proposal for re-recording the landing page's demo on the
published 2.0.2: a run that ends in LANDED and a share image that matches it. Change
nothing on the landing page until Matt has seen the proposal.
```

## Next

<!-- updated 2026-11-12; max 3, ranked -->
1. Re-record the landing page's demo — done when: "The demo, as it ran." replays a 2.0.2
   run that ends in LANDED and the share image matches it — `C:\pytxo` — blocked: Matt
   sees the proposal first.
2. PYTXO-3 film — done when: PLAN.md's PYTXO-3 bar is met, leading with landing and
   filmed on the published 2.0.2 — `C:\pytxo` — blocked: Matt says "go PYTXO-3".
3. `--fix` (pull request 98) — done when: rebased onto `main` with CI green, then merged
   or parked — its own session — blocked: Matt decides whether it ships before the film.

History: docs/checkpoints/
