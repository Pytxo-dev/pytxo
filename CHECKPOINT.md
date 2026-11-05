# CHECKPOINT — Pytxo — 2026-11-05 (2.0.1 candidate verified locally, nothing published)

## Branch / commit

- Candidate: `mbcz/pytxo-2-0-1-6392d6`, worktree
  `C:\pytxo\.claude\worktrees\pytxo-2-0-1-6392d6`, based on `origin/main` at
  `114da5c`. Rust code is final at `61758eb`; `fc964da` and this checkpoint change
  only the site, docs and evidence.
- Candidate binary: `target/release/pytxo.exe` in that worktree, built with
  `cargo build -p pytxo-cli --release --no-default-features`, prints `pytxo 2.0.1`,
  SHA-256 `eeaa4293f10493c911fcda35e717b8428beaf809b3221c54ec8e1abc5795ba65`.
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
- `--help`, the crate description and the 402 messages match the pitch and `/plans`.
- Failure paths run on temp repos; each prints what happened and the next step.
- `/bench` states its build and date: Pybench reran on the candidate. Not deployed.
- Linux glibc floor is 2.34, read from the ELF of the published 2.0.0 binary in WSL.
  2.0.1's `Cargo.lock` changes only Pytxo's own crate versions. The install page says so.
- `apps/web/vercel.json` holds main's site deploy until the promotion commit.

## Not done

- Nothing is merged, tagged, published or deployed.
- The branch is not pushed and no pull request is open. The session's permission
  check refused the push with Matt's OK relayed from the starting session, so CI
  hasn't run on it. The pull request text is in the goal's evidence folder, named in
  `REVIEW.md`.
- The field test (PYTXO-2) has not run: no proof beyond our own fixtures yet.
- macOS has not run on this code. Pull requests run CI on Linux and Windows only;
  macOS runs on main and in the release gate. A temporary macOS-only draft pull
  request was refused the same way.
- Listed review findings, not fixed:
  - A check that rewrites tracked files (D64).
  - `inputs[].status` on a partial landing is documented, values unchanged.
  - `-v`'s ref points at the failed combination.
  - Exit 2 means both blocked and landed some (D52).
  - `--markdown` drops notes.
  - `--into Main` on a case-insensitive filesystem.
  - Gaps in the background hook.
- Without a named cause, `--apply` tries each branch alone plus the rest of a blamed
  pair (D50), so an innocent branch can wait for a second run: 3 branches landed 1
  then 1 more, 6 branches landed 1 first. The film's batch lands 3 of 5, and its
  rename and caller both stay out; `PLAN.md`'s "4 land and 1 sent back" is wrong.
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
| `cargo test -p pytxo-land -p pytxo-cli` | 181 passed, 1 ignored |
| `cargo test -p pytxo-runner --lib`, with `PYTXO_TEST_ATTEMPT_HOST` | 105 passed |
| `cargo test -p pytxo-orchestrate --test candidate_run --test run_apply` | 1 + 15 passed |
| `pytxo demo`, cold, candidate binary | 12.7 s |
| `node packages/pytxo/bin/pytxo.js demo`, with `PYTXO_BINARY` | 7.9 s |
| `npm run lint`, `npm run build`, `playwright test` in `apps/web` | 31 passed |
| `node tooling/scripts/verify-release-version.mjs 2.0.1` and its test | 13 passed |
| `node --test tooling/scripts/verify-release-inventory.test.mjs` | 17 passed |
| `npm test` in `packages/pytxo` | 9 passed, 2 skipped |
| `pytxo bench` from a clean clone | 12/12, 4/4, 0 false refusals |

`a_run_after_a_failed_clean_lands_nothing` is Unix-only and has not run here.
CI's record for this branch is its pull request, if one is open.

## Blockers

- Matt has to say to push, in his own words, in the session holding this worktree.
  Then CI runs on the draft pull request.
- Matt's OK to go past local: review the candidate, then merge, tag and publish.
- The release commands, rollback and go points are below. Matt says go at each step.

## Release packet

After CI is green on the pull request and Matt says go:

```bash
gh pr ready <PR> --repo Pytxo-dev/pytxo
```

```bash
gh pr merge <PR> --repo Pytxo-dev/pytxo --merge
```

Wait for the CI run on `main` to pass. It is the first macOS run on this code.

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
go PYTXO-2

The goal is in D:\marketing\campaigns\release-ready-2026-11\PLAN.md, under
"PYTXO-2: field test". Start after 06:30 +07:00 on 6 Nov 2026. REVIEW.md in that
folder names the 2.0.1 worktree. Nothing public without Matt's OK.
```

## Next

<!-- updated 2026-11-05; max 3, ranked -->
1. PYTXO-2 field test — done when: it has reported from real public repos under
   the rules in `D:\marketing\campaigns\release-ready-2026-11\PLAN.md` —
   `C:\pytxo\.claude\worktrees\pytxo-2-0-1-6392d6` — blocked: not before 06:30 +07:00
   on 6 Nov 2026, and Matt starts it.
2. Publish 2.0.1 — done when: npm `latest` is 2.0.1, both releases have 10 assets,
   CI on main passed on macOS and pytxo.com says 2.0.1 — this file's Release packet —
   blocked: Matt says go.
3. Launch prep: steps 9 to 12 of `pytxo-2.0-report.md` — done when: the repository
   description and topics match the pitch, the plugin is submitted, and real
   repository timing is measured —
   `C:\pytxo\.claude\worktrees\product-review-a0207f` — blocked: Matt edits the
   description, submits the plugin and approves any post.

History: docs/checkpoints/
