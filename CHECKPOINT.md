# CHECKPOINT — Pytxo — 2026-11-13 (pytxo.com replays the 2.0.2 demo; this checkpoint merges with the clippy 1.100 fix)

## Branch / commit

- `main` at `5d96c484` merges pull request 102, the landing page change. 2.0.2 is
  published (`v2.0.2` on `ae9370f0`, npm `latest`), and the Action's `v2` tag points
  at it.
- This checkpoint travels with `mbcz/clippy-1-100`, cut from `5d96c484`. When this file
  is on `main`, the fix is too.

## Completed

- pytxo.com replays the 2.0.2 demo, which ends in LANDED (DECISIONS 67). Read live on
  13 Nov after the merge: the new homepage was served 100 s after it, the replay matched
  the record's line times and ended on 16.1 s, the fix prompt opened by keyboard,
  nothing overflowed at 1440 or 390 px, and the share image and its alt were the new
  ones. Deploy web and the Vercel status passed. Railway redeployed link and cloud, and
  both answered `/health` with 200.
- `is_reparse` in `crates/pytxo-runner/src/git.rs` passes clippy 1.100. CI uses the
  `stable` toolchain, now 1.100, whose `needless_bool` lint failed `main`'s CI on Linux
  and macOS (run 43390237826): off Windows the Windows-only block compiles away and
  leaves `if ... { return true; } false`. The same code passed on `ae9370f0` on 12 Nov.
  It now checks the reparse-point attribute first and returns the symlink check, with
  the same result on every platform.

## Not done

- `main`'s CI after this merges was not watched. The pull request's CI runs the same
  tree on Windows, Ubuntu and macOS.
- Link previews may show the old share image for a while: its URL did not change, and
  platforms cache by URL.
- Matt's machine has a global `pytxo@0.12.0`, and a bare `npx pytxo` runs that one and
  fails on `demo`. `npm uninstall -g pytxo` before filming.
- Pull request 98 (`mbcz/fix-loop`, `--fix`) is open in another session, is not in
  2.0.2, and needs a rebase onto `main`, which also brings it the clippy fix.
- Not checked for the landing page: Safari, Firefox, a screen reader on the disclosure,
  and the share image inside a real link preview.
- The homepage `<title>` still reads "Pytxo: land your coding agents' branches in one
  command", the lead from before decision 66. Wording is Matt's call.
- Not run by hand: macOS and Linux terminals, the GitHub Action on a live repository,
  the Claude Code plugin and the Codex hook.
- Carried over from 7 Nov, not rechecked: the listed review findings,
  `public-release-check`'s 37 old commits with an AI co-author trailer, the 2.0.0 notes
  line, `SECURITY.md` and the two e2e repositories. See `docs/checkpoints/2026-11.md`.
- Field test, unchanged: 0 real BLOCKED in 10 valid pairs
  (`D:\marketing\campaigns\pytxo-foss-study-2026-11\RESULTS.md`).

## Tests last run

| Command | Result |
|---|---|
| CI on `main` at `5d96c484` (run 43390237826) | failed: clippy `needless_bool` on Ubuntu and macOS; Windows, web and both desktop jobs passed |
| Deploy web, and the Vercel and Railway statuses, on `5d96c484` | passed |
| Live checks of pytxo.com after the merge: replay, disclosure, overflow, share image, alt | passed |
| `/health` on link.pytxo.com and cloud.pytxo.com | 200 |
| `cargo fmt --all -- --check` on this branch | passed |

Clippy 1.100 was not run locally: this laptop has Rust 1.98.1, and the lint fires only
off Windows. The pull request's CI runs it. On 12 Nov, on the landing page branch,
`npm ci`, lint, build and 31 Playwright tests passed and sloplint scored 0. The 2.0.2
release checks are in `docs/checkpoints/2026-11.md`.

## Blockers

- Matt's calls: whether `--fix` (pull request 98) ships before the film, the OK on the
  PYTXO-3 draft, and the launch date.

## Rollback

An npm version can't be republished, so a bad 2.0.2 is fixed by 2.0.3. Until then run
`npm dist-tag add pytxo@2.0.1 latest`. To put `v2` back: `git tag -f v2 9a6b2c45`, then
`git push -f origin v2`. The landing page change is one commit, `f8f6b0df`: reverting it
brings back the 2.0.0 replay, whose records are still in `apps/web/public/evidence`.
The clippy fix changes no behaviour.

## Next prompt (paste below)

```txt
Read CHECKPOINT.md in C:\pytxo. 2.0.2 is published and pytxo.com replays its demo.
Check that main's CI is green after the clippy 1.100 fix. Then wait for Matt's
"go PYTXO-3" before the film.
```

## Next

<!-- updated 2026-11-13; max 3, ranked -->
1. PYTXO-3 film — done when: PLAN.md's PYTXO-3 bar is met, leading with landing and
   filmed on the published 2.0.2 — `C:\pytxo` — blocked: Matt says "go PYTXO-3".
2. `--fix` (pull request 98) — done when: rebased onto `main` with CI green, then merged
   or parked — its own session — blocked: Matt decides whether it ships before the film.

History: docs/checkpoints/
