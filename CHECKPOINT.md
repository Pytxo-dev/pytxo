# CHECKPOINT — Pytxo — 2026-11-12 (2.0.2 is published; the landing page replays its demo once this merges)

## Branch / commit

- `main` at `f8be8b4d` has the site promotion (pull request 101, merged 12 Nov) on top
  of `ae9370f0`, which is tagged `v2.0.2`. Release run 43208737324 published it on
  12 Nov. npm `latest` is 2.0.2, and `Pytxo-dev/pytxo` and `Pytxo-dev/pytxo-releases`
  each have a `v2.0.2` release with 10 assets.
- The Action's `v2` tag points at `v2.0.2` (`ae9370f0`). Matt moved it on 12 Nov.
- This checkpoint travels with the landing page change on `mbcz/landing-demo-2-0-2`,
  cut from `f8be8b4d`. When this file is on `main`, that change is too.

## Completed

- 2.0.2 is public. On top of the line per branch, `--days` and the pile lead that
  `main` already had, it adds the hint for branches that match no pattern, the named
  test runs, a demo that lands a pile of four, every conflicting branch named and no
  test run when none merges (pull request 99). Notes:
  `distribution/release-notes/v2.0.2.md`.
- pytxo.com says 2.0.2. Read live on 12 Nov after pull request 101 merged:
  `/docs/reference/cli` says "This page describes 2.0.2", the quickstart shows the
  LANDED demo and the changelog names 2.0.2 as the latest release. CI and Deploy web
  passed on `f8be8b4d`.
- That merge touched only `apps/web`, this file and `docs/checkpoints`, and Railway
  still redeployed link.pytxo.com and cloud.pytxo.com. Both answered `/health` with
  200. Treat every push to `main` as a redeploy of both.
- The landing page replays the 2.0.2 demo (Matt approved the proposal on 12 Nov;
  DECISIONS 67). `apps/web/public/evidence/demo-landed-2.0.2-2026-11-12.json` is
  `npx pytxo demo` from npm: 32 lines ending in LANDED, three timed runs (16.1, 18.5
  and 20.3 s) and the time each line arrived. "The demo, as it ran." replays the rows
  at those times, with the fix prompt's body in a disclosure. The section under it is
  a ledger of the four branches from the same record. The share image and its alt
  show the LANDED lines under the current headline. The evidence page links the new
  record and both 2.0.0 ones.

## Not done

- The landing page change is not merged. Merging deploys the site and redeploys link
  and cloud.
- The record comes from a busy laptop. A recording on a quieter one was planned and
  came out slower (26, 38 and 62 s) because four other sessions were busy, so the
  record is the one the proposal showed, and its limits give the day's range (13.3 to
  62.1 s). To record again on an idle machine: run `npx pytxo demo` three times after
  a warm-up run, in an empty folder, with both streams on one pipe and
  `npm_config_prefix` pointing at an empty directory.
- Matt's machine has a global `pytxo@0.12.0`, and a bare `npx pytxo` runs that one
  and fails on `demo`. `npm uninstall -g pytxo` before filming.
- Pull request 98 (`mbcz/fix-loop`, `--fix`) is open in another session and is not in
  2.0.2. It needs a rebase onto `main`.
- Not run by hand: macOS and Linux terminals, the GitHub Action on a live repository,
  the Claude Code plugin and the Codex hook.
- Not checked for the landing page change: Safari and Firefox, a screen reader on the
  disclosure, and the share image inside a real link preview.
- The homepage's `<title>` still reads "Pytxo: land your coding agents' branches in one
  command", the lead from before decision 66. Left as it is: wording is Matt's call.
- Carried over from 7 Nov, not rechecked: the listed review findings,
  `public-release-check`'s 37 old commits with an AI co-author trailer, the 2.0.0 notes
  line, `SECURITY.md` and the two e2e repositories. See `docs/checkpoints/2026-11.md`.
- Field test, unchanged: 0 real BLOCKED in 10 valid pairs
  (`D:\marketing\campaigns\pytxo-foss-study-2026-11\RESULTS.md`).

## Tests last run

All 2026-11-12 on Windows; exit 0 unless the row says otherwise.

| Command | Result |
|---|---|
| CI and Deploy web on `main` at `f8be8b4d` | passed |
| `npm view pytxo version --prefer-online` | 2.0.2 |
| `npx pytxo demo`, three timed runs for the record | exit 0, 32 lines, LANDED 2 of 4 |
| `npm ci --legacy-peer-deps` and `npm run lint` in `apps/web`, on this branch | passed |
| `npm run build` in `apps/web`, on this branch | passed |
| `npm run e2e` in `apps/web` against that build, on this branch | 31 passed |
| A script on the built page: the replay against the record, the disclosure by keyboard, overflow at 1440 and 390 px | passed |
| `sloplint check` on the changed files | Slop Score 0 |

The built page was also looked at in screenshots at 1440 and 390 px, with the fix
prompt closed and open.

The 2.0.2 release checks are in `docs/checkpoints/2026-11.md` and in the checkpoint
before this one (`784e7d87`).

## Blockers

- Matt's calls: merging the landing page change, whether `--fix` (pull request 98)
  ships before the film, and the OK on the PYTXO-3 draft.

## Rollback

An npm version can't be republished, so a bad 2.0.2 is fixed by 2.0.3. Until then run
`npm dist-tag add pytxo@2.0.1 latest` and revert the promotion commit. To put `v2`
back: `git tag -f v2 9a6b2c45`, then `git push -f origin v2`. The landing page change
is one commit: reverting it brings back the 2.0.0 replay, whose records are still in
`apps/web/public/evidence`.

## Next prompt (paste below)

```txt
Read CHECKPOINT.md in C:\pytxo. 2.0.2 is published and the landing page change
(mbcz/landing-demo-2-0-2) replays its demo. If it is merged, check pytxo.com: "The
demo, as it ran." ends in LANDED, its footer says pytxo 2.0.2 and the share image shows
the LANDED lines. Then wait for Matt's "go PYTXO-3" before the film.
```

## Next

<!-- updated 2026-11-12; max 3, ranked -->
1. Merge the landing page change — done when: pytxo.com's demo ends in LANDED and the
   share image shows it — `C:\pytxo` — blocked: Matt says merge.
2. PYTXO-3 film — done when: PLAN.md's PYTXO-3 bar is met, leading with landing and
   filmed on the published 2.0.2 — `C:\pytxo` — blocked: Matt says "go PYTXO-3".
3. `--fix` (pull request 98) — done when: rebased onto `main` with CI green, then merged
   or parked — its own session — blocked: Matt decides whether it ships before the film.

History: docs/checkpoints/
