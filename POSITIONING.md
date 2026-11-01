# Positioning

**Land all your agents' branches in one command.** Pytxo merges your coding
agents' branches in a throwaway worktree, runs your tests on the combined
result, and lands them only if they pass. If two branches break each other,
nothing lands and you see why.

This page is the source for the README, the homepage, the npm and Action
descriptions, and the plugin listings. When they disagree, fix them to match
this page or change this page first.

## Who it is for

Developers who run two or more coding agents (Claude Code, Codex, Copilot,
Cursor and the like) on one repository at the same time, each on its own
branch or worktree.

Not for: one agent at a time (there is nothing to combine), or a repository
without tests (Pytxo catches only what the tests catch).

## What they do today

- **Trust each branch's green CI.** CI tests each pull request against the
  base branch, not against the other open pull requests. Two green branches can
  still break main the moment both are merged.
- **Merge by hand.** Merge one branch, rebase the next, run the tests again,
  and find the break after the first branch has already landed.
- **Ask the agent to merge.** Claude Code or Codex can merge the branches and
  run the tests, but that spends the same usage limits as the work, and it may
  merge differently each time.
- **Use a merge queue** (GitHub's merge queue, Mergify, Graphite). These do
  test combined changes, but at merge time: after review, with branch
  protection set up first, inside one hosting provider. A break found there
  costs a review cycle.
- **Run a parallel-agent desktop** (Conductor, Emdash and others). They run the
  agents in separate worktrees and leave the combining to you.

## Why Pytxo

- **One command for the routine job.** Merging three to five agent branches
  by hand means merge, rebase, run the tests, repeat. `npx pytxo --apply` does
  it in one step and moves the target only when the tests pass on the combined
  result, by compare-and-swap, so it refuses if the target moved meanwhile.
  Every run does that work, not only the rare run where branches break each
  other.
- **No model, no quota.** The check never calls a model, so it spends none of
  your agents' usage limits.
- **Your real tests on the combined result.** Not a diff heuristic and not a
  model's opinion: `npm test`, `pytest`, `cargo test`, `go test ./...` or
  whatever the repository uses, run on the merge of all the branches.
- **Before the merge, not after.** Run it the moment the agents finish
  (`npx pytxo`, the Claude Code plugin or the Codex hook), or on every agent
  pull request with one line of GitHub Actions.
- **Local and cross-vendor.** Any agent that leaves a branch counts. It runs on
  your machine with no account and sends nothing anywhere.
- **It says what broke, and which branch has to change.** When the combination
  fails, it runs each branch alone to tell "breaks only together" from
  "already broken" and from a flaky test. When one branch renamed or removed a
  name another still uses, and the failing output confirms it, the verdict
  names both branches and the line, says which branch to rebase, and writes
  that branch's agent a fix prompt, without calling a model. With three or
  more branches it then tests that pair without the others, and when the
  pair fails and the others pass, in CI only the pull requests of the
  branches behind the failure fail. After a combined run times out, the
  branches do not run alone.
- **Nothing changes until it passes.** The merge happens in a throwaway
  worktree. Without `--apply` nothing in the repository changes; with it, only
  the target branch moves.

## Honest limits

- Landing is all or nothing: when the combination fails, nothing lands, even
  the branches that pass on their own.
- It catches what your tests catch. A break your tests don't cover still lands.
- Merge conflicts are reported and blocked, not resolved.
- Nobody has measured how often agents' branches fail only when combined. The
  one large study found concurrent pull requests from different agents in
  about 4.3% of 2,807 repositories
  ([arXiv 2607.04697](https://arxiv.org/abs/2607.04697)). Pybench
  (`pytxo bench`) shows the mechanism on synthetic repositories, not a rate.
- The check is as slow as your test suite, plus installing dependencies, or a
  cold build of a compiled project, in a fresh worktree.
- In CI, your tests run in the same job that posts the comment, so code in
  this repository's agent branches can reach the job's `pull-requests: write`
  token, as in any CI job that runs that code. Give the workflow no other
  permissions or secrets, apart from `actions: read` if Copilot's coding agent
  opens pull requests; with it, that code can also read every workflow run's
  logs and artifacts. A fork's pull request never joins another pull
  request's run: it is checked in its own run, with GitHub's read-only token
  and no secrets. A pull request opened by Copilot or `github-actions[bot]`,
  whose runs GitHub holds for approval, joins another pull request's run only
  with `actions: read`, and only at its newest commit whose run someone let
  start; its own run checks it with the others once someone approves it.
- The cause is named only for a definition one branch renamed or removed and
  another still uses. Other breaks, such as a renamed config key or a changed
  signature, show the error line the tests printed.
- Receipts use the in-toto statement format but are not signed.

## Price

- **Free:** everything on your machine, and the GitHub Action on public
  repositories. Open source under MIT.
- **Pro:** the GitHub Action on private repositories, after a 14-day trial in
  each one. $29 once for the first 500 buyers, then $5 a month or $48 a year.
- **Team:** planned.
- If development stops, Pro unlocks for everyone.

Pro is not switched on yet: until it is, the Action runs free everywhere.

## Words

Use: land, check, together, combined, agent branches, throwaway worktree, READY,
BLOCKED, verdict.

Keep off every page a newcomer sees: control plane, hypervisor, telemetry,
mission, fleet, the Orbit / Galaxy / DeepSpace / Supernova tiers, Signal
Core, Blast Shield, Race Shield, Ultra. They still name real parts of the
frozen 1.x tooling and live in the advanced docs. Lead with the job (landing
the agents' branches), then the guarantee (only if they pass together). Not
with a category name such as "landing layer", and not with the rare failure:
on Pytxo's own work, 0 of 21 sibling branch pairs failed only together
(measured 2026-10-31).
