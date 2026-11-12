# CHECKPOINT — Pytxo — 2026-11-11 (2.0.2 candidate on a local branch, nothing pushed or published)

## Branch / commit

- Candidate: local branch `mbcz/pytxo-2-0-2` in worktree
  `C:\pytxo\.claude\worktrees\pytxo-onboarding-audit-d1c88b`, based on `origin/main` at
  `31eaedcc`. `70194c2a` sets the versions, `b5aaf98e` updates the site docs, and the
  commit after them is this checkpoint. Not pushed, so CI has not run on it.
- Local only: `mbcz/promote-2-0-2` (`c5bbd89a`), the promotion commit for after npm
  serves 2.0.2. It sets `PUBLISHED_VERSION`, drops "(candidate)" from the changelog row
  and lifts the site's deploy hold.
- `main` at `31eaedcc` holds 2.0.1 plus pull requests 94, 95, 96, 97 and 99. npm
  `latest` is 2.0.1 and both 2.0.1 releases have 10 assets.
- No release binary: the laptop had 0.5 GB free with ten sessions open, so nothing was
  built with `--release`. The checks below ran on a debug build that prints
  `pytxo 2.0.2`.

## Completed

- Pull request 99 merged on 11 Nov: the hint for branches that match no pattern, the
  named test runs, the demo that lands a pile of four, every conflicting branch named,
  no tests when none merges, the protected-branch recipe and the npm page. CI on
  `main` passed, macOS included.
- It fixes both field-test findings of 7 Nov: an all-conflict pile names every branch
  in what and next, and no test runs on unchanged main. Not rechecked: the `left_out`
  that note mentions.
- 2.0.2 candidate: the version everywhere `verify-release-version.mjs` looks, the
  release notes (`distribution/release-notes/v2.0.2.md`), the changelog row and the
  hook pins. `apps/web/vercel.json` holds main's site deploy until the promotion commit.
- The site docs describe 2.0.2: CLI reference, quickstart, docs home, landing guide and
  the docs test's demo line. The demo claim is now "under a minute".
- Pybench's land suite on the 2.0.2 build gives the published 2.0.1 result, so `/bench`
  and its evidence stay as they are.

## Not done

- Nothing is pushed, tagged, published or deployed for 2.0.2.
- The landing page still replays the 2.0.0 recording of the old demo
  (`apps/web/public/evidence/demo-blocked-2.0.0-2026-10-30.json`: BLOCKED, 10 rows,
  8.5 s), and 2.0.2's demo prints 32 lines ending in LANDED. `recorded-demo.tsx`, the
  share image `/social/check-1600x1000.png` and its alt (old headline) are built on the
  old one. The footer says "pytxo 2.0.0", so it stays a true record. Re-recording
  changes the landing page, so Matt sees a proposal first. Not started.
- The Action's `v2` tag is still on `v2.0.0` (`9a6b2c45`) at origin, so
  `Pytxo-dev/pytxo/actions/land@v2` installs pytxo 2.0.0. The 2.0.1 packet's step that
  moves it did not happen. Matt's call.
- Pull request 98 (`mbcz/fix-loop`, `--fix`) is open in another session. If it merges
  first, rebase this branch and add it to the notes and docs.
- The site's lint, build and Playwright tests did not run here (no `node_modules`, no
  memory for a Next build). CI's web job runs them on the pull request.
- Not run by hand: macOS and Linux terminals, the GitHub Action on a live repository,
  the Claude Code plugin and the Codex hook.
- Carried over from 7 Nov, not rechecked: the listed review findings,
  `public-release-check`'s 37 old commits with an AI co-author trailer, the 2.0.0 notes
  line, `SECURITY.md` and the two e2e repositories. See `docs/checkpoints/2026-11.md`.
- Field test, unchanged: 0 real BLOCKED in 10 valid pairs
  (`D:\marketing\campaigns\pytxo-foss-study-2026-11\RESULTS.md`).

## Tests last run

All 2026-11-11, Windows, on `mbcz/pytxo-2-0-2`; exit 0.

| Command | Result |
|---|---|
| `cargo test -p pytxo-land -j 2` | 157 passed, 1 ignored |
| `cargo test -p pytxo-cli --no-default-features -j 2` | 8 + 33 passed |
| `cargo clippy -p pytxo-land -p pytxo-cli --no-default-features --all-targets -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `node tooling/scripts/verify-release-version.mjs 2.0.2` | candidate 2.0.2, published 2.0.1 |
| the same with no argument on `mbcz/promote-2-0-2` | candidate 2.0.2, published 2.0.2 |
| `node --test` on the two `verify-release-*` tests | 30 passed |
| `npm test` in `packages/pytxo` | 9 passed, 2 skipped |
| `pytxo demo`, debug build, run as `npx pytxo`, three runs | 17.9, 18.7 and 19.4 s |
| `pytxo bench --suite land`, debug build | 12/12, 4/4, 0 false refusals |

Not run: the default-feature build, `cargo test --workspace`, a release build, and the
site's checks.

## Blockers

- Matt's OK to push `mbcz/pytxo-2-0-2` and open the pull request. CI has to pass there.
- Matt's OK at each step after it: merge, tag, publish, promote.
- Whether the landing page is re-recorded before the promotion or after it.

## Release packet

After CI is green on the pull request and Matt says go, merge it with
`gh pr merge <number> --repo Pytxo-dev/pytxo --merge`. The merge changes `Cargo.toml`
and `Cargo.lock`, so Railway redeploys link.pytxo.com and cloud.pytxo.com: check
`/health` on both. Wait for the CI run on `main` to pass, macOS included.

```bash
git fetch origin main && git tag v2.0.2 origin/main
```

```bash
git push origin v2.0.2
```

The tag starts the Release workflow, which runs CI as its gate and publishes to npm
and GitHub. Check it. Expect `2.0.2`, `10` and `10`:

```bash
npm view pytxo version
gh release view v2.0.2 --repo Pytxo-dev/pytxo --json assets --jq '.assets | length'
gh release view v2.0.2 --repo Pytxo-dev/pytxo-releases --json assets --jq '.assets | length'
```

Then move the Action's major tag:

```bash
git fetch origin tag v2.0.2 && git tag -f v2 "v2.0.2^{commit}" && git push -f origin v2
```

Promote the site only after npm serves 2.0.2, from a fresh `origin/main`:

```bash
git switch -c mbcz/promote-2-0-2-pr origin/main && git cherry-pick c5bbd89a && git push -u origin mbcz/promote-2-0-2-pr
```

```bash
gh pr create --repo Pytxo-dev/pytxo --base main --head mbcz/promote-2-0-2-pr --title "site: 2.0.2 is the published release" --body "For after npm serves 2.0.2."
```

Merge it. A green Vercel check on the pull request proves nothing, so check the live
site afterwards: `/docs/reference/cli` says "This page describes 2.0.2" and the
quickstart shows the LANDED demo.

Rollback: an npm version can't be republished, so a bad release is fixed by 2.0.3.
Until then run `npm dist-tag add pytxo@2.0.1 latest`, move `v2` back to the commit it
had, and revert the promotion commit if it merged. Between the merge and npm serving
2.0.2, the repository's hooks and the plugin pin a version npm doesn't have yet, so
keep that window short.

## Next prompt (paste below)

```txt
go 2.0.2

The candidate is the local branch mbcz/pytxo-2-0-2 in
C:\pytxo\.claude\worktrees\pytxo-onboarding-audit-d1c88b. Read CHECKPOINT.md there.
Push it, open the pull request and report CI. Stop before the merge: Matt says go at
each step of the release packet.
```

## Next

<!-- updated 2026-11-11; max 3, ranked -->
1. Publish 2.0.2 — done when: npm `latest` is 2.0.2, both releases have 10 assets, CI on
   main passed on macOS and pytxo.com's CLI page says 2.0.2 — this file's Release
   packet — `C:\pytxo\.claude\worktrees\pytxo-onboarding-audit-d1c88b` — blocked: Matt
   says push.
2. Re-record the landing page's demo — done when: "The demo, as it ran." replays a 2.0.2
   run that ends in LANDED and the share image matches it — `C:\pytxo` — blocked: Matt
   sees the proposal first.
3. PYTXO-3 film — done when: PLAN.md's PYTXO-3 bar is met, leading with landing —
   `C:\pytxo` — blocked: Matt says "go PYTXO-3".

History: docs/checkpoints/
