# Positioning

**Each agent's branch passes. Do they pass together?** Pytxo merges your
coding agents' branches in a throwaway worktree, runs your tests on the
combined result, and blocks the merge if it breaks.

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
- **Use a merge queue** (GitHub's merge queue, Mergify, Graphite). These do
  test combined changes, but at merge time: after review, with branch
  protection set up first, inside one hosting provider. A break found there
  costs a review cycle.
- **Run a parallel-agent desktop** (Conductor, Emdash and others). They run the
  agents in separate worktrees and leave the combining to you.

## Why Pytxo

- **Your real tests on the combined result.** Not a diff heuristic and not a
  model's opinion: `npm test`, `pytest`, `cargo test`, `go test ./...` or
  whatever the repository uses, run on the merge of all the branches.
- **Before the merge, not after.** Run it the moment the agents finish
  (`npx pytxo`, the Claude Code plugin or the Codex hook), or on every agent
  pull request with one line of GitHub Actions.
- **Local and cross-vendor.** Any agent that leaves a branch counts. It runs on
  your machine with no account and sends nothing anywhere.
- **It says what broke.** When the combination fails, it runs each branch alone
  to tell "breaks only together" from "already broken" and from a flaky test.
  After a combined run times out, the branches do not run alone.
- **Nothing in the repository changes.** The merge happens in a throwaway
  worktree.

## Honest limits

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
  permissions or secrets. A fork's pull request never joins another pull
  request's run: it is checked in its own run, with GitHub's read-only token
  and no secrets.
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

Use: check, together, combined, agent branches, throwaway worktree, READY,
BLOCKED, verdict.

Keep off every page a newcomer sees: control plane, hypervisor, telemetry,
mission, fleet, the Orbit / Galaxy / DeepSpace / Supernova tiers, Signal
Core, Blast Shield, Race Shield, Ultra. They still name real parts of the
frozen 1.x tooling and live in the advanced docs. Lead with the question, not
a category name such as "landing layer".
