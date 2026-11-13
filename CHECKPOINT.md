# CHECKPOINT — Pytxo — 2026-11-13 (the clippy 1.100 fix is on main; one Windows test timed out there once, and its hang guard is now 60 s)

## Branch / commit

- `main` at `375769ab` merges pull request 103, the clippy 1.100 fix, on top of
  `5d96c484`, the landing page change (pull request 102). 2.0.2 is published
  (`v2.0.2` on `ae9370f0`, npm `latest`), and the Action's `v2` tag points at it.
- This checkpoint travels with `mbcz/stop-gate-test-guard`, cut from `375769ab`. When
  this file is on `main`, the test change is too.

## Completed

- pytxo.com replays the 2.0.2 demo, which ends in LANDED (DECISIONS 67). Read live on
  13 Nov after the merge: the replay matched the record's line times and ended on
  16.1 s, the fix prompt opened by keyboard, nothing overflowed at 1440 or 390 px, and
  the share image and its alt were the new ones.
- `is_reparse` in `crates/pytxo-runner/src/git.rs` passes clippy 1.100, which the
  `stable` toolchain in CI now is. On `main` at `375769ab` clippy passed on Windows,
  Ubuntu and macOS. After that merge the Vercel status and both Railway statuses
  passed, and link and cloud answered `/health` with 200.
- Two tests in `crates/pytxo-orchestrate/src/lib.rs` give Stop 60 s before they call it
  hung, up from 5 s: `active_run_gate_serializes_stop_all_between_threads_until_drop`
  and `stop_all_releases_domain_gate_before_termination_starts`. The first timed out
  on Windows in `main`'s CI at `375769ab` (run 43397312965). In that job no test in the
  binary finished for 12.6 s, and then the rest passed. The same tree had passed in
  pull request 103, and re-running the failed job passed.

## Not done

- `main`'s CI after this merges is not recorded here. If either test times out with
  the 60 s guard, Stop is hanging, not slow: read that job's log before re-running.
- What stalled the runner is unknown. The other test binaries in the failed job took
  about as long as in pull request 103's run.
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
| CI on `main` at `375769ab` (run 43397312965), first attempt | failed: one Windows test timed out, as above; clippy, Ubuntu, macOS, web and both desktop jobs passed |
| The same run with its failed jobs re-run | passed |
| Vercel and Railway statuses on `375769ab` | passed |
| `/health` on link.pytxo.com and cloud.pytxo.com | 200 |
| `cargo fmt --all -- --check` on this branch | passed |

The two changed tests were not run locally: this laptop had no warm build and 2 GB of
free memory. The pull request's CI runs them on Windows and Ubuntu. On 12 Nov, on the
landing page branch, `npm ci`, lint, build and 31 Playwright tests passed and sloplint
scored 0. The 2.0.2 release checks are in `docs/checkpoints/2026-11.md`.

## Blockers

- Matt's calls: whether `--fix` (pull request 98) ships before the film, the OK on the
  PYTXO-3 draft, and the launch date.

## Rollback

An npm version can't be republished, so a bad 2.0.2 is fixed by 2.0.3. Until then run
`npm dist-tag add pytxo@2.0.1 latest`. To put `v2` back: `git tag -f v2 9a6b2c45`, then
`git push -f origin v2`. The landing page change is one commit, `f8f6b0df`: reverting it
brings back the 2.0.0 replay, whose records are still in `apps/web/public/evidence`.
The clippy fix and the test change alter no behaviour.

## Next prompt (paste below)

```txt
Read CHECKPOINT.md in C:\pytxo. 2.0.2 is published and pytxo.com replays its demo.
Wait for Matt's "go PYTXO-3" before the film.
```

## Next

<!-- updated 2026-11-13; max 3, ranked -->
1. PYTXO-3 film — done when: PLAN.md's PYTXO-3 bar is met, leading with landing and
   filmed on the published 2.0.2 — `C:\pytxo` — blocked: Matt says "go PYTXO-3".
2. `--fix` (pull request 98) — done when: rebased onto `main` with CI green, then merged
   or parked — its own session — blocked: Matt decides whether it ships before the film.

History: docs/checkpoints/
