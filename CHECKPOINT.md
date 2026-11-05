# CHECKPOINT — Pytxo — 2026-11-05 (2.0.0 is live with the new pitch)

## Branch / commit

- `main` at `52774af` holds everything shipped: PR #85 (`9a6b2c4`, 2.0.0, site
  v2, the new pitch), PR #86 (`f0d36a7`, the promotion) and PR #87 (`52774af`,
  the clean Vercel build).
- Tags `v2.0.0` and `v2` both point at `9a6b2c4`.
- `mbcz/land-what-works` (`e51d2e6`, partial landing) still sits on
  `mbcz/site-v2` and needs a rebase onto main for 2.0.1.
- Untracked in the worktree: `RELEASE_TODAY.md`. Its ship commands are done.

## Completed

- npm `latest` is 2.0.0. The GitHub release and the `Pytxo-dev/pytxo-releases`
  mirror each have 10 assets.
- pytxo.com serves "Land all your agents' branches in one command." from
  deployment `dpl_BQwEQq9FKh8sER3GbPL7DGhr1ASb`.
- Incident, fixed: the first production build of the promotion restored its
  cache from the 1.3.6 deployment and shipped the old `globals.css`. Production
  had no `[data-site="v2"]` tokens and rendered unstyled from about 23:47 UTC on
  2026-11-04 until the PR #87 deploy; the exact end wasn't recorded.
  `apps/web/vercel.json` now removes `.next/cache` before `next build`.
- Vercel facts behind that incident: the ignored-build step cancels every
  non-main ref, so a green Vercel check on a pull request does not mean a build
  ran. It also cancels a redeploy when nothing under `apps/web` or
  `packages/chroma` changed. A site fix has to reach main as a commit.
- CI now pins `ubuntu-24.04`, pins a copy of the Windows test helper, and holds
  the event loop open in the Action's silent Dodo test.

## Not done

- Main CI is red on `52774af`: `rust (macos-latest)` failed
  `real_candidate_checks_gate_apply_and_refresh_frozen_bytes`,
  `crates/pytxo-orchestrate/tests/candidate_run.rs:285`,
  `assert!(check.duration_ms > 0)`. PR #87 changed only `apps/web/vercel.json`.
  The same assertion failed attempt 1 of the 2.0.0 release gate. The likely cause
  is `crates/pytxo-runner/src/run.rs:1728`, where `started` is set after the
  spawn, so a fast check can record 0 ms. That fix is not made or tested.
- For 2.0.1: the CLI `--help` line and `crates/pytxo-cli/Cargo.toml` still say
  "Do they pass together?", and the 402 messages name `/plans` Pro perks.
- The 2.0.0 release notes keep the repo's standard line "These are prepared
  release notes... not a release announcement." Editing them needs Matt's OK.
- `/bench` still shows its reference run on pytxo 1.3.0.
- Not checked: the home page's demo and lower sections after the CSS fix, the
  Action's job summary text, Linux glibc, timing on a real repository, a real
  hook session, a real `npx pytxo --apply` batch.
- Carried over: Matt's call on `SECURITY.md` dropping Desktop 1.3.x; the
  signed-in account views keep v1 boxes; Arena is neither built nor measured.

## Tests last run

- 2026-11-05, live pytxo.com: the hero is 4 lines with the copy button above the
  fold at 320×700, 390×844, 1280, 1440 and 1920×900, and 1920×1080. `/`, `/plans`,
  `/download` and `/bench` are styled at 375 px and on desktop, with no sideways
  scroll at 375 px and no console errors. `/docs` and the changelog return 200.
- 2026-11-05: PR #87 `ci-ok` and `web` passed; `verify-release-inventory` 17 of 17.
- 2026-11-05: main CI failed on `52774af` (above); it passed on `f0d36a7`.
- 2026-11-04: cold `npx -y pytxo@2.0.0 demo` exited 0 in 14 s. Action e2e against
  `v2.0.0` had 10 PASS and 0 fail, and the demo passed on Ubuntu, macOS and Windows.
- 2026-11-04: release workflow attempt 2 had 20 of 20 jobs succeed.

## Blockers

- None for 2.0.1. Secrets: none in this file; the npm publish worked.
- Matt's cleanup. The e2e repositories are public, and agents don't delete repos:

```bash
gh auth refresh -s delete_repo
```

```bash
gh repo delete mattbaconz/pytxo-action-e2e-20261104-193845 --yes
```

```bash
gh repo delete mbcz-xyz/pytxo-action-e2e-20261104-193845 --yes
```

## Next prompt (paste below)

```txt
Pytxo 2.0.1. Read AGENTS.md and the top of CHECKPOINT.md.

Start with the macOS flake: reproduce why check.duration_ms can be 0 at
crates/pytxo-orchestrate/tests/candidate_run.rs:285, then fix it at the source
in crates/pytxo-runner/src/run.rs, near `started` at line 1728. Declare the
permission profile and execution domain scope. Don't weaken the assertion.

Then change the CLI --help line and the crates/pytxo-cli/Cargo.toml description
to the new pitch, make the 402 messages match /plans, and rebase e51d2e6 from
mbcz/land-what-works onto main. Run cargo test -p pytxo-land -p pytxo-cli and
the affected orchestrate tests.

Don't push, tag, publish, deploy or edit the GitHub repo description without
Matt's OK. No AI attribution trailers.
```

## Next

<!-- updated 2026-11-05; max 3, ranked -->
1. 2.0.1 — done when: `check.duration_ms` can't record 0, main CI passes on macOS,
   the CLI help, Cargo description and 402 messages carry the new wording, `e51d2e6`
   is rebased onto main, and `cargo test -p pytxo-land -p pytxo-cli` passes —
   `C:\pytxo`.
2. Dogfood `npx pytxo --apply` on Matt's next batch of agent branches — done when:
   one real run lands or blocks with a receipt and is kept as launch footage —
   `C:\pytxo` — blocked: needs a real batch of Matt's branches.
3. Launch prep: steps 9 to 12 of `pytxo-2.0-report.md` — done when: the repository
   description and topics match the pitch, the plugin is submitted, and Linux glibc
   and real repository timing are measured —
   `C:\pytxo\.claude\worktrees\product-review-a0207f` — blocked: Matt edits the
   description, submits the plugin and approves any post.

History: docs/checkpoints/
