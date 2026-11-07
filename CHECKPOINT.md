# CHECKPOINT — Pytxo — 2026-11-07 (2.0.1 candidate in a draft pull request, field test reported, nothing published)

## Branch / commit

- Candidate: draft pull request https://github.com/Pytxo-dev/pytxo/pull/89, branch
  `mbcz/pytxo-2-0-1-6392d6`, worktree
  `C:\pytxo\.claude\worktrees\pytxo-2-0-1-6392d6`, based on `origin/main` at
  `114da5c`. Rust code is final at `ebab582`; the commits after it change only the
  site, docs, evidence and this checkpoint.
- Candidate binary: `target/release/pytxo.exe` in that worktree, built with
  `cargo build -p pytxo-cli --release --no-default-features`, prints `pytxo 2.0.1`,
  SHA-256 `aabd2239b3ab514a195d2a416c2a4638aaa3b1464f3a08b3ef51bfac1c0d8766`.
- Local only: `mbcz/promote-2-0-1` (`295b22c`), the promotion commit for after npm
  serves 2.0.1. It sets `PUBLISHED_VERSION`, updates the changelog and lifts the
  site's deploy hold.
- `main` at `114da5c` holds 2.0.0, which is live on npm, GitHub and pytxo.com.

## Completed

- `check.duration_ms` can't record 0: the clock starts before the spawn and a check
  that ran records at least 1 ms. The assertion at
  `crates/pytxo-orchestrate/tests/candidate_run.rs:285` is unchanged.
- Partial landing (`e51d2e6`) rebased as `e52ff1a`. A read-only reviewer confirmed
  that the ref moves only to a commit recorded as tested and that the receipt says
  what was left out. Of 20 findings, 13 are fixed and 7 are listed below.
- When a branch fails alone, the others run together once more (D65, which replaces
  part of D50). Five branches with one failing now land 4 of 5 in one run, and the
  note without `--apply` names the same set. A second read-only review confirmed the
  target moves only to a commit whose exact set passed every command. Of its 6
  findings, 4 are fixed and 2 are listed below.
- `--help`, the crate description and the 402 messages match the pitch and `/plans`.
- Failure paths run on temp repos; each prints what happened and the next step.
- `/bench` states its build and date: Pybench reran on the candidate. Not deployed.
- Linux glibc floor is 2.34, read from the ELF of the published 2.0.0 binary in WSL.
  2.0.1's `Cargo.lock` changes only Pytxo's own crate versions. The install page says so.
- `apps/web/vercel.json` holds main's site deploy until the promotion commit.

## Not done

- Nothing is merged, tagged, published or deployed.
- CI is the pull request's record, not this file's. Matt said to push on 5 Nov 2026,
  in the session holding this worktree.
- The field test (PYTXO-2) reported on 7 Nov 2026 from a time-out, short of its target:
  - 14 of 33 repos tried, 10 valid pairs, all from llm-d/llm-d-benchmark.
  - 0 real BLOCKED: 0 of 10, 95% Wilson interval 0% to 27.8%.
  - Batches: 2 landed in part, 1 not at all.
  - Fewer than 1 in 20, so Matt's 1 Nov rule drops the catch claim from the lead, and Matt
    decides between the landing pitch, repositioning and parking.
  - Results: `D:\marketing\campaigns\pytxo-foss-study-2026-11\RESULTS.md`.
- Found by the field test, not fixed:
  - When every input of an `--apply` batch conflicts with main, the verdict's
    what/why/next and `left_out` name only the first branch. The receipt's `reasons`
    lists all six. Repro: `runs/Javis603__token-monitor/batch.json` in the study folder.
  - Pytxo also runs the full check on unchanged main when every input conflicts.
- Pull requests run CI on Linux and Windows only, and macOS runs on main and in the
  release gate. So https://github.com/Pytxo-dev/pytxo/pull/90 runs the rust job on
  macOS, on `c3fd73e` plus one commit to `ci.yml`. Never merge it: close it and delete
  its branch once the run has finished.
- Listed review findings, not fixed:
  - A check that rewrites tracked files (D64).
  - `inputs[].status` on a partial landing is documented, values unchanged.
  - `-v`'s ref points at the failed combination.
  - Exit 2 means both blocked and landed some (D52).
  - `--markdown` drops notes.
  - `--into Main` on a case-insensitive filesystem.
  - Gaps in the background hook. It also pays for the run on the others without
    showing its set (D65).
  - When the run on the others passes but its cleanup fails, one branch lands and the
    others read "was not tested together", which is false there. The next step it
    gives is right.
  - No test covers the run on the others when the set doesn't merge, when its cleanup
    fails, or when it can't start.
- When every branch passes alone and no cause is named, `--apply` still lands one and
  the next run takes the others (D50, left as is). 3 branches landed 1, then 1 more; 6
  branches landed 1 first. Those branches get a fix prompt too, though some of them
  may not have to change.
- A batch with a rename pair and a branch that fails alone takes two runs: 1 of 5 lands,
  then 2 of 4, and the last two each get a reason.
- The film has two batch shapes. One branch failing alone lands 4 of 5, so `PLAN.md`'s
  "4 land and 1 sent back" holds for that batch. The rename batch lands 3 of 5, and its
  rename and caller both stay out.
- The site's hero copy changed to "lands what passes. What breaks stays out, with the
  reason." Matt hasn't seen it.
- `public-release-check` exits 1: 21 HIGH, all fake tokens in test code already on
  main, none in this branch's diff. 37 old commits carry an AI co-author trailer;
  rewriting pushed history is Matt's call.
- Carried over: the 2.0.0 notes line, `SECURITY.md` dropping Desktop 1.3.x, deleting
  the two e2e repositories (commands in `docs/checkpoints/2026-11.md`).

## Tests last run

All 2026-11-05, Windows, on this branch; exit 0 unless noted.

| Command | Result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy -p pytxo-runner -p pytxo-land -p pytxo-cli -p pytxo-core --all-targets -- -D warnings` | clean |
| `cargo test -p pytxo-land -p pytxo-cli` | 194 passed, 1 ignored |
| `cargo test -p pytxo-runner --lib`, with `PYTXO_TEST_ATTEMPT_HOST` | 105 passed |
| `cargo test -p pytxo-orchestrate --test candidate_run --test run_apply` | 1 + 15 passed |
| `pytxo demo`, cold, candidate binary | 10.2 s |
| `node packages/pytxo/bin/pytxo.js demo`, with `PYTXO_BINARY` | 11.9 s |
| `npm run lint`, `npm run build`, `playwright test` in `apps/web` | 31 passed |
| `node tooling/scripts/verify-release-version.mjs 2.0.1` and its test | 13 passed |
| `node --test tooling/scripts/verify-release-inventory.test.mjs` | 17 passed |
| `npm test` in `packages/pytxo` | 9 passed, 2 skipped |
| `pytxo bench` from a clean clone | 12/12, 4/4, 0 false refusals |

`a_run_after_a_failed_clean_lands_nothing` is Unix-only and has not run here.
CI's record for this branch is its pull request, and the macOS one for macOS.

## Blockers

- CI has to pass on both pull requests.
- Matt's OK to go past local: review the candidate, then merge, tag and publish.
- The release commands, rollback and go points are below. Matt says go at each step.

## Release packet

After CI is green on the pull request and Matt says go:

```bash
gh pr ready 89 --repo Pytxo-dev/pytxo
```

```bash
gh pr merge 89 --repo Pytxo-dev/pytxo --merge
```

Wait for the CI run on `main` to pass, macOS included.

```bash
git fetch origin main
```

```bash
git tag v2.0.1 origin/main
```

```bash
git push origin v2.0.1
```

The tag starts the Release workflow, which runs CI as its gate and publishes to npm
and GitHub. Check it:

```bash
npm view pytxo version
```

```bash
gh release view v2.0.1 --repo Pytxo-dev/pytxo --json assets --jq '.assets | length'
```

```bash
gh release view v2.0.1 --repo Pytxo-dev/pytxo-releases --json assets --jq '.assets | length'
```

Expect `2.0.1`, `10` and `10`. Then move the Action's major tag:

```bash
git fetch origin tag v2.0.1 && git tag -f v2 "v2.0.1^{commit}" && git push -f origin v2
```

Promote the site only after npm serves 2.0.1, from a fresh `origin/main`:

```bash
git switch -c mbcz/promote-2-0-1-pr origin/main && git cherry-pick 295b22c9 && git push -u origin mbcz/promote-2-0-1-pr
```

```bash
gh pr create --repo Pytxo-dev/pytxo --base main --head mbcz/promote-2-0-1-pr --title "site: 2.0.1 is the published release" --body "For after npm serves 2.0.1."
```

Merge it. A green Vercel check on the pull request proves nothing, so check the live
site afterwards: `/` is styled and says "lands what passes", and `/bench` says
`pytxo 2.0.1`.

Rollback: an npm version can't be republished, so a bad release is fixed by 2.0.2.
Until then:

```bash
npm dist-tag add pytxo@2.0.0 latest
```

```bash
git tag -f v2 "v2.0.0^{commit}" && git push -f origin v2
```

Revert the promotion commit if it merged. Agent hooks pinned to `pytxo@2.0.0` keep
running 2.0.0. Between the merge and npm serving 2.0.1, the repository's hooks and
the plugin pin a version npm doesn't have yet, so keep that window short.

## Next prompt (paste below)

```txt
go PYTXO-3

The goal is in D:\marketing\campaigns\release-ready-2026-11\PLAN.md, under
"PYTXO-3: Pytxo film". Run it after 2.0.1 is public. The batch to film is the one
ranked first in D:\marketing\campaigns\pytxo-foss-study-2026-11\RESULTS.md. Nothing
public without Matt's OK.
```

## Next

<!-- updated 2026-11-07; max 3, ranked -->
1. Publish 2.0.1 — done when: npm `latest` is 2.0.1, both releases have 10 assets,
   CI on main passed on macOS and pytxo.com says 2.0.1 — this file's Release packet —
   blocked: Matt says go.
2. Matt's call on the field test's result — done when: he picks the landing pitch,
   repositioning or parking Pytxo (0 real BLOCKED in 10 valid pairs, under his 1 Nov
   rule) — `D:\marketing\campaigns\pytxo-foss-study-2026-11\RESULTS.md` — blocked:
   Matt decides.
3. PYTXO-3 film — done when: PLAN.md's PYTXO-3 bar is met, on the llm-d batch ranked
   first in RESULTS.md — `C:\pytxo` — blocked: 2.0.1 is public and Matt says
   "go PYTXO-3".

History: docs/checkpoints/
