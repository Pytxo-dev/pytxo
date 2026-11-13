# CHECKPOINT — Pytxo — 2026-11-13 (three QUIC advisories cleared in the lockfile; PYTXO-3 is set up and waits for the claude command-line login)

## Branch / commit

- `main` at `11e02b2e` merges pull request 104, the 60 s hang guard, on top of
  `375769ab` (pull request 103, the clippy 1.100 fix) and `5d96c484` (pull request 102,
  the landing page). 2.0.2 is published (`v2.0.2` on `ae9370f0`, npm `latest`), and the
  Action's `v2` tag points at it.
- This checkpoint travels with `mbcz/quinn-advisories`, cut from `11e02b2e`. When this
  file is on `main`, the lockfile change is too.

## Completed

- pytxo.com replays the 2.0.2 demo, which ends in LANDED (DECISIONS 67). Read live on
  13 Nov after the merge: the replay matched the record's line times and ended on
  16.1 s, the fix prompt opened by keyboard, nothing overflowed at 1440 or 390 px, and
  the share image and its alt were the new ones.
- `is_reparse` in `crates/pytxo-runner/src/git.rs` passes clippy 1.100, which the
  `stable` toolchain in CI now is.
- Two stop-all tests in `crates/pytxo-orchestrate/src/lib.rs` give Stop 60 s before
  they call it hung, up from 5 s. One had timed out on Windows on `main` while no test
  in its binary finished for 12.6 s.
- `Cargo.lock` moves `quinn-proto` to 0.11.19 and `quinn-udp` to 0.5.16. `cargo audit`
  failed `main`'s CI at `11e02b2e` (run 43428584777) on three advisories against them:
  RUSTSEC-2026-0523, RUSTSEC-2026-0524 and RUSTSEC-2026-0538. Only `reqwest` lists
  them, for its HTTP/3 feature, which no package here turns on, so no Pytxo build
  compiled them: `cargo tree -i quinn-proto --workspace --target all --all-features`
  prints nothing.
- PYTXO-3 started on Matt's "Go PYTXO-3" (13 Nov), on the 9 Nov draft in
  `D:\marketing\campaigns\release-ready-2026-11\PLAN.md`. Set up in
  `D:\pytxo-film\take-2`: a clone of sloplint at `365eba9` with no remote, five
  worktrees, five tasks from sloplint's roadmap in `prompts/`, `launch.sh` to run three
  Claude Code and two Codex agents at the same time, and `stamp.mjs` to record each
  session with its timing. `npx pytxo@2.0.2` runs in the clone.

## Not done

- PYTXO-3 has not run an agent. The standalone `claude` command line is not logged in:
  with the desktop app's variables removed, `claude auth status` says
  `"loggedIn": false`. The Claude agents and the ask-Claude-to-merge number wait for
  Matt to run `claude auth login`. The same thing stopped the 8 Nov objection number.
- The 9 Nov draft is led by terminal footage. On 10 Nov Matt called the 8 Nov terminal
  cut "just terminal lines, not interesting" (`pytxo/STORY-SHORT.md` in the campaign
  folder). Show him a 10 s sample before cutting.
- `main`'s CI after this merges is not recorded here. `routed_local_fixture` failed
  once on Windows in pull request 104's CI, in code that pull request could not affect,
  and passed on the re-run; the cause is not found. If either stop-all test times out
  with its 60 s guard, Stop is hanging, not slow: read the log before re-running.
- C: has 8.8 GB free and D: has 20 GB. Film work goes on D:.
- Link previews may show the old share image for a while: its URL did not change, and
  platforms cache by URL.
- Matt's machine has a global `pytxo@0.12.0`, and a bare `npx pytxo` runs that one and
  fails on `demo`. `npm uninstall -g pytxo` before filming, or point
  `npm_config_prefix` at an empty folder.
- Pull request 98 (`mbcz/fix-loop`, `--fix`) is open in another session, is not in
  2.0.2, and needs a rebase onto `main`.
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
| CI on `main` at `11e02b2e` (run 43428584777) | failed: `cargo audit` on Ubuntu found the three advisories; no other job failed |
| `cargo audit` on this branch | passed: no vulnerabilities, the same 16 allowed warnings |
| `cargo metadata --locked` on this branch | passed |
| Vercel and Railway statuses on `11e02b2e` | passed |
| `/health` on link.pytxo.com and cloud.pytxo.com | 200 |
| CI on pull request 104 | the first attempt failed on Windows in `routed_local_fixture`; the re-run passed |

Nothing was compiled locally for the lockfile change, because C: is short of space.
The pull request's CI builds and tests it on Windows and Ubuntu. On 12 Nov, on the
landing page branch, `npm ci`, lint, build and 31 Playwright tests passed and sloplint
scored 0. The 2.0.2 release checks are in `docs/checkpoints/2026-11.md`.

## Blockers

- PYTXO-3: Matt runs `claude auth login`.
- Matt's calls: whether `--fix` (pull request 98) ships before the film is posted, and
  the launch date.

## Rollback

An npm version can't be republished, so a bad 2.0.2 is fixed by 2.0.3. Until then run
`npm dist-tag add pytxo@2.0.1 latest`. To put `v2` back: `git tag -f v2 9a6b2c45`, then
`git push -f origin v2`. The landing page change is one commit, `f8f6b0df`: reverting it
brings back the 2.0.0 replay, whose records are still in `apps/web/public/evidence`.
The clippy fix and the test change alter no behaviour, and the lockfile change touches
two crates that no build compiles.

## Next prompt (paste below)

```txt
Read CHECKPOINT.md in C:\pytxo. PYTXO-3 is set up in D:\pytxo-film\take-2 and waits for
Matt's `claude auth login`. Then run launch.sh, check the branches with
npx pytxo@2.0.2, measure the objection number, and show Matt a 10 s sample before
cutting.
```

## Next

<!-- updated 2026-11-13; max 3, ranked -->
1. PYTXO-3 film — done when: the film bar in the campaign's PLAN.md is met on the
   published 2.0.2 and Matt has the master — `D:\pytxo-film\take-2` and `C:\pytxo` —
   blocked: Matt runs `claude auth login`.
2. `--fix` (pull request 98) — done when: rebased onto `main` with CI green, then merged
   or parked — its own session — blocked: Matt decides whether it ships before the film
   is posted.

History: docs/checkpoints/
