---
title: Distribution experiments
slug: distribution-experiments
status: active
tags: [distribution, launch]
audience: [human, agent]
layer: meta
created: 2026-10-19
updated: 2026-10-20
related: [[MOC-home]]
---

# Distribution experiments

One entry per experiment. Results are measured counts with their source and
window; an entry stays "proposed" until it has run. Target lists that name
third-party repositories live in the private marketing kit, not here.

## 2026-10-19 — Pytxo — E1 "Found it first"

- Status: reviewed (2026-10-20); cold-scan outreach abandoned
- Project and goal: Pytxo 1.3.3; first repositories outside Pytxo running
  `pytxo land` each week.
- Audience: maintainers of public repositories with two or more open agent pull
  requests at once (Claude Code, Copilot coding agent, Codex) that touch the
  same files.
- Hypothesis: a maintainer shown a real combined failure in their own open
  agent pull requests replies, and some add the shadow Action.
- Evidence and limitations: M2 run 2, where two live-agent branches each passed
  `npm test` alone and failed combined, and `pytxo land --per-input` blocked it
  ([first-use evidence](../../apps/web/content/docs/evidence/first-use.mdx)).
  That was a test run on one small repository with tasks we chose. The Action
  writes a job summary and does not comment on pull requests.
- Asset and version/path: one issue per repository where a finding exists, with
  the error lines and a plain-git repro that works without Pytxo; `actions/land@v1.3.3`.
- Placement and rule-check date: the target repository's issue tracker; issues
  were enabled on all seven targets on 2026-10-19; each repository's
  contributing rules are checked before filing.
- Approach and CTA: report the failure first, disclose affiliation, then offer
  the one-file shadow Action ([land in CI](../../apps/web/content/docs/getting-started/land-in-ci.mdx)).
  No finding, no message.
- Measurement, review point, and original decision rule:
  - Measured per target: scan decision, issue filed, maintainer reply within 7
    days, and adoption by GitHub code search for `Pytxo-dev/pytxo/actions/land`
    (baseline: 0 repositories outside Pytxo on 2026-10-19).
  - Review 7 days after the last issue is filed.
  - Repeat if at least one real failure was reported and a maintainer engaged.
  - Modify the channel if failures were reported with no reply.
  - Change targets if none of the seven scans finds a failure.
  - Inconclusive if fewer than four scans complete.
- Result: running. Scans run in a separate public repository on GitHub-hosted
  runners, because untrusted pull-request code must not run on a developer
  machine.
  - Scan 1 of 7 (2026-10-19), nine concurrent Claude Code pull requests:
    blocked. Five conflicted textually; the four that merged passed the
    workspace tests together. No combined test failure, so no outreach.
  - Scan 2 (2026-10-19), three Copilot pull requests: one conflicted textually
    with another; the other two passed together. No outreach.
  - Scan 3 (2026-10-20), two Claude Code pull requests: Pytxo reported each
    passing alone and the combination failing. A plain-git merge of both on a
    fresh runner passed the full suite (0 failures), so the report did not
    reproduce: likely a flaky test or service state shared between check runs.
    No outreach.
  - Interim (3 of 7): no confirmed combined failure. A single failing combined
    run can be a flake; the gate should re-run before blaming the combination.
  - E1b (2026-10-20): merge-only scans (git merges, no pull-request code run) of
    16 repositories, 13 of them not stacked.
    - 7 had no conflicts.
    - 5 had an agent pull request that already conflicts with its base, which
      GitHub shows.
    - 1 had two agent pull requests that GitHub reports as mergeable but that
      conflict with each other, in test index files.
    - This corrects the interim note: inter-PR conflicts were confirmed in 1
      repository, not 2.
- What we learned: one snapshot of public repositories rarely shows a
  reportable combined problem (0 of 3 test scans and 1 of 13 merge scans), and
  maintainers serialize agent merges quickly (one repository merged or closed
  5 of its 9 agent pull requests within a day). Cold-scan outreach is abandoned
  as the primary channel.
  Of 22 filtered candidate repositories, only 7 had concurrent agent pull
  requests sharing files.
- Next experiment: developers who run several agents in parallel on one
  repository and merge the results locally, where the merge pain is not
  serialized by a maintainer. Replies in dated public threads where that pain
  is described, using the first-use evidence run.

## 2026-10-20 — Pytxo — E2 "Show the real run"

- Status: proposed (2026-10-20); nothing posted.
- Project and goal: Pytxo 1.3.3; first developers outside Pytxo who run
  `pytxo land` on their own agent branches.
- Audience: developers who run several coding agents in parallel on one
  repository (worktrees, Conductor-style apps, Claude agent teams, Codex plus
  Claude) and merge the results themselves.
- Hypothesis: when someone describes or designs how they merge parallel-agent
  work, a reply showing the real run gets them to try `pytxo land`. Unlike E1,
  the merge is theirs, so no maintainer's serial merge queue hides the problem.
- Evidence and limitations: M2 run 2 from E1. Both branches edited
  `src/index.js` and git merged them with no conflict; each passed `npm test`
  alone; the combination failed with `ReferenceError: toVal is not defined`, and
  `pytxo land --per-input` blocked it
  ([first-use evidence](../../apps/web/content/docs/evidence/first-use.mdx)).
  One small repository, two tasks chosen to touch the same helper, timings on
  Windows only. It shows the failure exists, not how often it happens.
- Asset and version/path: one reply of about 150 words: the run, its limit,
  affiliation, and one link to the evidence page. Pytxo 1.3.3
  (`npm i -g pytxo`, Node 18 or newer). The draft lives in the private
  marketing kit.
- Placement and rule-check date (2026-10-20):
  - Primary: a maintainer's design thread in GitHub Discussions (opened
    2026-09-28) on how a coordinator fans parallel agents out into worktrees and
    merges their branches. Unlocked; the contributing guide adopts Contributor
    Covenant 2.1 and has no rule against tool mentions or AI-assisted replies;
    GitHub's Acceptable Use Policies forbid spam.
  - Backup: a Hacker News comment (2026-10-17) from a user whose orchestrator
    agent builds merge trains. The HN guidelines forbid generated or AI-edited
    text, so Matt writes any HN reply himself; HN sends no reply notifications.
  - Not selected: a GitHub Discussions comment (2026-10-18) about flagging
    overlapping worktree edits with `git merge-tree`, in a thread about a
    plugin API, where a product link would derail it.
  - Reddit returned 403 from this machine: UNVERIFIED.
- Approach and CTA: answer the thread's own question with the run, say "I build
  Pytxo", one link. CTA: run
  `pytxo land <a> <b> --check "<tests>" --per-input` on the next pair of agent
  branches.
- Measurement, review point, and original decision rule:
  - Measured per reply: date and link; replies within 7 days and whether they
    engage with the failure; any sign the person ran `pytxo land` (they say so,
    ask a usage question, or open an issue); Action adoption by code search
    (baseline 0 on 2026-10-19).
  - Not measurable: pytxo.com has no analytics, and npm downloads cannot be
    attributed to a reply. With one to three replies, results are qualitative.
  - Up to three replies, each in a thread that passes the same checks. Review 7
    days after the last one, and no later than 2026-11-03.
  - Repeat if someone engaged with the failure and at least one person ran
    `pytxo land`.
  - Modify the CTA if people engaged with the failure but nobody ran it.
  - Modify placement if no posted reply gets an answer within 7 days; next
    time use threads under three days old.
  - Abandon the hypothesis if, across three replies with engagement, people say
    serial merges plus CI already cover it.
  - Inconclusive if fewer than two replies are posted by 2026-11-03.
- Result: not run.
- What we learned (research only, 2026-10-20): the failure class is
  recognized in public. A source-control engineer on Hacker News described a
  function changed with its callsites cleaned up while a new call is added
  concurrently, and called it unavoidable at large-repo scale without
  serialized merging. A blog post on parallel agents showed both branches
  passing their own tests. Fresh threads where a person merges parallel-agent
  work by hand were scarce: of the checked candidates, one older launch
  thread was already archived and two posts had no comments.
- Next experiment: decided at review.
