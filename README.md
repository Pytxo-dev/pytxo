# Pytxo

[![CI](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml/badge.svg)](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/pytxo)](https://www.npmjs.com/package/pytxo)
[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](LICENSE)

**Each agent's branch passes. Do they pass together?**

Pytxo merges your Claude Code, Codex and Copilot branches in a throwaway
worktree, runs your tests on the combined result, and blocks the merge if it
breaks. Free, open source, runs on your machine or in CI.

```bash
npx pytxo demo
```

## Try it in 60 seconds

```bash
npx pytxo demo   # a throwaway repo where two agent branches pass alone and fail together
cd your-repo
npx pytxo        # checks your agent branches together
```

No flags, no config, no account. Pytxo finds the agent branches, finds your
test command, merges the branches in a throwaway worktree, runs the tests
there and prints the verdict in three lines: what happened, why, and what to
do next. Nothing in your repository changes.

## The demo

`npx pytxo demo` builds a small repository where one agent renamed a function
and another added a new call to its old name. Each branch passes `npm test` on
its own:

```text
$ npx pytxo demo
Building a throwaway repo where two agents changed the same shop code...
  claude/rename-price  renames price() to unitPrice() in src/cart.js
  codex/add-discount   adds src/discount.js, which calls price()
Checking claude/rename-price and codex/add-discount against main with npm test (from package.json)...
BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.
why      npm test fails on the combined change: FAIL discount.test.js: price is not a function
next     fix one branch so they agree, then run npx pytxo again.
```

The demo needs Node.js and takes a few seconds once the binary is downloaded.

## In CI: one line

Add `.github/workflows/pytxo.yml`:

```yaml
name: Pytxo
on: pull_request
permissions:
  contents: read
  pull-requests: write
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: Pytxo-dev/pytxo/actions/land@v2
```

On every agent pull request, the Action checks it together with the other
open agent pull requests into the same base branch, then posts one comment and
keeps it up to date. A blocked verdict fails the job only when this pull
request is part of the problem; when another pull request is the cause, the
comment names it and the job passes. It runs on `pull_request` only, never
`pull_request_target`. A fork's pull request never joins another pull
request's run: it is checked in its own run, with GitHub's read-only token and
no secrets. Without `pull-requests: write` the verdict goes to the job summary
instead. See [the Action's inputs](actions/land/action.yml).

## When your agents finish

- **Claude Code:** `/plugin marketplace add Pytxo-dev/pytxo`, then
  `/plugin install pytxo@pytxo`. The check runs when Claude stops and stays
  quiet unless two or more agent branches changed.
  [More](integrations/claude-code/README.md)
- **Codex:** copy [integrations/codex/hooks.json](integrations/codex/hooks.json)
  to `~/.codex/hooks.json` and trust it in `/hooks`.
  [More](integrations/codex/README.md)

## Details

**Which branches.** With no arguments Pytxo checks branches named `claude/*`,
`codex/*`, `copilot/*`, `cursor/*`, `devin/*`, `jules/*` and `agent/*`, plus
every branch checked out in a linked worktree, that have a commit in the last
14 days. Name branches yourself (`npx pytxo feature/a feature/b`) or change the
patterns for a repository:

```bash
git config pytxo.branches "bot/* ai/*"
```

**Which target.** With no arguments the branches are checked against the
default branch: `origin/HEAD`, else `main`, `master` or `trunk`. Branches you
name are checked against the branch you have checked out, as in Pytxo 1.x, or
against the default branch if the checked-out branch is one of them or no
branch is checked out. `--into <branch>`, `git config pytxo.into <branch>` or
`PYTXO_INTO` sets the target in both cases.

**Which tests.** Pytxo reads the target branch and picks the usual command
for `package.json` (npm, pnpm, yarn or bun, installing dependencies first when
there are any), Cargo, Go, uv, Poetry, pytest, Deno, Gradle, Maven, .NET, Mix,
RSpec, Rake, Composer or a `Makefile` with a `test` target. Set it yourself
with `-c "npm ci && npm test"` or `git config pytxo.check "..."`.

**How it decides.** Pytxo runs the tests on the combined change first. If they
fail, it runs them on each branch alone to tell a break that only happens
together from a branch that is already broken, then runs the combined change
once more to rule out a flaky test. Merge conflicts block too.

**Exit codes.** `0` ready (or nothing to check), `2` blocked, `3` stale (the
target branch moved during the check), `1` error.

**Output.** `--verbose` adds the receipt, digests and every check's output;
`--json` and `--markdown` are for scripts and comments. `--apply` lands a ready
result on the target branch. Every check records a receipt in
`.git/pytxo/receipts` (`pytxo receipts`).

**Limits.** It catches what your tests catch: a break your tests don't cover
can still land. Conflicts are blocked, not resolved. Pybench (`pytxo bench`)
shows the mechanism on synthetic repositories, not how often it happens in
real projects. Receipts use the in-toto statement format but are not signed.
Pytxo sends nothing anywhere.

**Install it.** `npx pytxo` downloads the native binary for your platform on
first run and checks it against the release's SHA-256 list. To keep it on your
PATH: `npm i -g pytxo`. Binaries for macOS, Linux and Windows are on
[Pytxo releases](https://github.com/Pytxo-dev/pytxo-releases).

## Pytxo 1.x

Pytxo Desktop, missions, fleets and the terminal dashboard are frozen. They
still build from source with the default `full` feature
(`cargo build -p pytxo-cli`); the released CLI ships the check only. Their docs
are under [Advanced](https://pytxo.com/docs) on the site.

## Repository layout

| Path | Purpose |
|------|---------|
| [`crates/pytxo-land`](crates/pytxo-land/) | The check: branch and test detection, the combined merge, verdicts, receipts, Pybench |
| [`crates/pytxo-cli`](crates/pytxo-cli/) | The `pytxo` command |
| [`actions/land`](actions/land/) | The GitHub Action |
| [`integrations/`](integrations/) | The Claude Code plugin and the Codex hook |
| [`packages/pytxo`](packages/pytxo/) | The npm package |
| [`apps/web/`](apps/web/) | [pytxo.com](https://pytxo.com) and the public docs |
| [`apps/desktop/`](apps/desktop/) | Pytxo Desktop (frozen) |
| [`docs/`](docs/) | Internal architecture notes and decision records |

## Build from source

Building needs Rust 1.88 or newer.

```bash
cargo build -p pytxo-cli --no-default-features   # the check, as released
cargo test --workspace
```

- **Site:** [pytxo.com](https://pytxo.com)
- **Docs:** [pytxo.com/docs](https://pytxo.com/docs)
- **Discord:** [discord.gg/AUFRPFjSYv](https://discord.gg/AUFRPFjSYv)

Pytxo 2.0 is a public beta.
