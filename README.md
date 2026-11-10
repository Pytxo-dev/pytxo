# Pytxo

[![CI](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml/badge.svg)](https://github.com/Pytxo-dev/pytxo/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/pytxo)](https://www.npmjs.com/package/pytxo)
[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](LICENSE)

**Clear your agent PR pile in one command.**

Pytxo merges your Claude Code, Codex and Copilot branches in a throwaway
worktree, runs your tests on the combined result, and lands the ones that
pass. Every branch it leaves out gets the reason and a fix prompt to hand back
to its agent. Your own tests decide; Pytxo calls no AI model. Free, open
source, runs on your machine or in CI.

```bash
npx pytxo demo
```

On the 10 open agent pull requests of
[llm-d-benchmark](https://github.com/llm-d/llm-d-benchmark) (a 6 Nov 2026
snapshot, in a local clone), `npx pytxo@2.0.1 --apply 'codex/*' 'copilot/*'`
landed 5 in 5 min 41 s and left 5 out: 2 conflict with main, 1 fails the
repository's tests on its own and 2 conflict with branches that landed first.

## Try it

```bash
npx pytxo demo      # a throwaway repo: four agent branches, two land, two get the reason
cd your-repo
git fetch           # get the branches your agents pushed
npx pytxo           # checks your agent branches together
npx pytxo --apply   # lands the ones that pass
```

No flags, no config, no account. Pytxo finds the agent branches, finds your
test command, merges the branches in a throwaway worktree, runs the tests
there and prints the verdict: what happened, why, and what to do next, with a
line for each branch when only some of three or more can land. Nothing in your repository changes unless you add `--apply`. Ctrl+C stops a check and removes
its throwaway worktree; a check that is killed outright leaves the worktree in
`git worktree list` until the next check removes it.

## The demo

`npx pytxo demo` builds a small repository with a pile of four agent branches
and lands it with `--apply`. Each branch passes `npm test` on its own. One
agent renamed a function and another added a new call to its old name, so
those two break the tests together and stay out. The other two land:

```text
$ npx pytxo demo
Building a throwaway repo where four agents each left a branch on a small shop...
  claude/rename-price  renames price() to unitPrice() in src/cart.js
  codex/add-discount   adds src/discount.js, which calls price()
  copilot/add-tax      adds src/tax.js and its test
  cursor/add-guide     adds docs/guide.md
Checking claude/rename-price, codex/add-discount, copilot/add-tax and cursor/add-guide against main with npm test (from package.json)...
  together... failed (0.7s)
  claude/rename-price alone... passed (0.7s)
  codex/add-discount alone... passed (0.7s)
  copilot/add-tax alone... passed (0.8s)
  cursor/add-guide alone... passed (0.7s)
  together, again... failed (0.9s)
  claude/rename-price and codex/add-discount without the others... failed (1.0s)
  copilot/add-tax and cursor/add-guide without the others... passed (0.8s)
LANDED   2 of 4 branches are on main now.
left out claude/rename-price  renamed price to unitPrice, which codex/add-discount still uses
left out codex/add-discount   uses price, which claude/rename-price renamed to unitPrice (src/discount.js:1)
landed   copilot/add-tax
landed   cursor/add-guide
next     push main when you are ready. Give codex/add-discount's agent its fix prompt, then run npx pytxo again.

Fix prompt for codex/add-discount:
Pytxo tested copilot/add-tax and cursor/add-guide on main and landed them. Your branch codex/add-discount is left out: it uses price, which claude/rename-price renamed to unitPrice (src/discount.js:1).
Rebase it onto claude/rename-price, replace price with unitPrice, and run npm test until it passes.
Failing output:
> test
> node run-tests.js
ok   cart.test.js
FAIL discount.test.js: price is not a function
ok   tax.test.js

That repo was temporary and is gone. Now run npx pytxo in a repo where agents work.
```

The demo needs Node.js and takes about 20 seconds once the binary is
downloaded. `npx pytxo demo --keep` keeps the repository so you can look at
what landed.

## In CI: one line

Add `.github/workflows/pytxo.yml`:

```yaml
name: Pytxo
on: pull_request
permissions:
  contents: read
  pull-requests: write
concurrency:
  group: pytxo-${{ github.event.pull_request.number }}
  cancel-in-progress: true
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: Pytxo-dev/pytxo/actions/land@v2
```

Your tests run in the job that holds this token, so give the workflow no other
permissions or secrets. If Copilot's coding agent opens pull requests here, add
`actions: read` so the check can include them once someone lets their run
start; with it, code in the checked branches can also read every workflow
run's logs and artifacts.

A run on an agent pull request checks it together with the other open agent
pull requests into the same base branch and keeps one comment on it, updated in
place. When no other agent pull request is open into that base, the job passes
with a note in the job summary. The job fails when this pull request conflicts,
fails the tests alone, or is in a combination that fails while each passes
alone. With three or more pull requests, when Pytxo proves that two of them
fail together and the others pass without them, only the pull requests behind
the failure fail. When only other pull requests are to blame, the comment names
them and the job passes. When the failing output confirms the cause, the
comment also holds a fix prompt for the branch that has to change. When nothing
in the result points at any pull request, such as a combined run that timed
out, every pull request in the run fails. It runs on `pull_request` only, never
`pull_request_target`. A fork's pull request never joins another pull
request's run: it is checked in its own run, with GitHub's read-only token and
no secrets. A pull request opened by Copilot or `github-actions[bot]`, whose
runs GitHub holds for approval, joins another pull request's run only when the
workflow has `actions: read`, and only at its newest commit whose run someone
let start; the comment names that commit. Its own run, once approved, checks
it with the others. A run checks each pull request at the commit it saw, so a
push after someone approved the run gets a run of its own. Without
`pull-requests: write` the verdict goes to the job summary instead. See
[the Action's inputs](actions/land/action.yml).

## When your agents finish

- **Claude Code:** in a shell,
  `claude plugin marketplace add Pytxo-dev/pytxo --sparse .claude-plugin integrations/claude-code`,
  then `claude plugin install pytxo@pytxo`. The check runs when Claude stops.
  It answers when two or more agent branches exist and one of them, the target
  or the test command changed since its last answer. When the verdict names a
  confirmed cause, the sessions on both branches it names show it once, each
  at its own stop. [More](integrations/claude-code/README.md)
- **Codex:** copy [integrations/codex/hooks.json](integrations/codex/hooks.json)
  to `~/.codex/hooks.json` and trust it in `/hooks`.
  [More](integrations/codex/README.md)

## Details

**Which branches.** With no arguments Pytxo checks branches named `claude/*`,
`codex/*`, `copilot/*`, `cursor/*`, `devin/*`, `jules/*` and `agent/*`, plus
every branch checked out in a linked worktree, that have a commit in the last
14 days, at most 10 at once, newest first. Branches already merged into the
target, squash merges included, are skipped. Only committed work is checked: a
worktree whose changes are all uncommitted gets a hint to commit them. If your
agents push their branches to GitHub, run `git fetch` first. When no branch
matches, Pytxo names the local branches that are not merged yet and prints the
command that checks them. Name branches yourself
(`npx pytxo feature/a feature/b`) or change the patterns for a repository:

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
with `-c "npm ci && npm test"` or `git config pytxo.check "..."`. A compiled
project, such as a Rust workspace, pays a cold build in each throwaway
worktree. Submodules are not checked out there.

**How it decides.** Pytxo runs the tests on the combined change first. If they
fail, it runs them on each branch alone to tell a break that only happens
together from a branch that is already broken, and, if every branch passes
alone, runs the combined change once more to catch a flaky test. If that run
fails too, it looks, with git and tree-sitter, for a definition one branch
renamed or removed that another branch's new code still uses. When the failing
output confirms it, the why line names both branches and the line, and the
next line says which branch to rebase; otherwise a note names the possible
cause. With three or more branches it then tests that pair without the others
and, if they fail, the others without the pair: at most two more runs, so the
verdict can clear the others. If the combined run times out, that is the
answer: the branches do not run alone after it, and it does not run again.
Merge conflicts block too.

**Exit codes.** `0` ready or landed (or nothing to check), `2` blocked or
`--apply` landed only some branches, `3` stale (the target moved), `1` error,
`130` stopped by Ctrl+C.

**Output.** While the tests run, each run is named as it starts and how it
ended (`together... failed (41.3s)`, then `claude/a alone... passed (38.9s)`),
and in a terminal a long run gets a mark each minute. `--verbose` adds what was
found, every test run with its error lines, the receipt's path and a ref to
the combined commit;
`pytxo receipts show <id>` prints the last 40 lines of each run and checks the
digest. `--json` is for scripts and `--markdown` is the GitHub Action's comment.
`--apply` lands only a tested result on the target branch: all of it when it is
ready, and when it is blocked the largest set of branches that passed every test
as that exact set, leaving the others out, each with why. With no test command,
or only a package.json lint or build script, it exits `1` and changes nothing.
Every check records a receipt in `.git/pytxo/receipts` (`pytxo receipts`).

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
