---
title: Distribution experiments
slug: distribution-experiments
status: active
tags: [distribution, launch]
audience: [human, agent]
layer: meta
created: 2026-10-19
updated: 2026-10-19
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
