# Decisions: Pytxo 2.0

Reversible choices made while building 2.0, newest last. Each says what was
chosen, why, and how to undo it.

1. **`check` is the command; `land` is its alias.** Both run the same engine
   with the same flags. With no branches given, branches and the test command
   are detected. The library's `land()` still never invents a check; the CLI
   fills one in. Undo: split the clap variants again.

2. **Target = the default branch, or the checked-out one when you name
   branches.** Order: `--into`, then `PYTXO_INTO` or `git config pytxo.into`,
   then, when branches are named, the checked-out branch unless it is one of
   them or HEAD is detached, then `origin/HEAD`, `main`, `master`, `trunk`,
   then the checked-out branch. A bare `pytxo` and the hooks name no
   branches, so they check against the default branch: a hook runs inside an
   agent's worktree, where "the checked-out branch" is the agent branch
   itself. Named branches keep the 1.x rule, so `pytxo a b` on `develop`
   checks against `develop` and `--apply` lands there. A target from
   `--into` or a setting must exist; `pytxo demo` always uses `main`.
   (`init.defaultBranch` was dropped: it names the branch for new
   repositories, not this one's.)

3. **Agent branches = patterns plus linked worktrees.** Default list:
   `claude/* codex/* copilot/* cursor/* devin/* jules/* agent/* worktrees`.
   `claude/`, `codex/`, `copilot/`, `cursor/`, `devin/` and `jules/` are the
   prefixes those agents use for the branches and PRs they create; `agent/` is
   Pytxo's own convention. `worktrees` means every branch checked out in a
   linked worktree (Claude Code Desktop, Conductor and Codex use custom
   names there). Override with `git config pytxo.branches "..."` or
   `PYTXO_BRANCHES`; an explicit list replaces the defaults. Remote-tracking
   branches count when no local branch has that name. Branches already in
   the target or in `origin/<target>`, or whose merge adds nothing
   (squash-merged), are skipped, so a branch merged on GitHub drops out
   before the local target is pulled. Pytxo's own scratch worktrees
   (`pytxo-land-<receipt tag>`, including ones a killed run left behind)
   never count.

4. **The detected test command runs without asking.** That is the product.
   It is printed in the verdict, and it is read from the target branch's
   tree, so an agent branch cannot pick the command. Override with `--check`,
   `git config pytxo.check` or `PYTXO_CHECK`.

5. **Combined first, each branch alone only after a failure.** The common
   case (ready) costs one test run instead of N+1. A failure then runs each
   branch alone and re-runs the combination, so "pass alone, fail together"
   and "flaky" are still proven, never guessed. `pytxo check --per-input`
   keeps the old always-first order.

6. **"Flaky" is its own verdict.** It still exits 2 (blocked), as before,
   but the headline says FLAKY so nobody blames the agents for it.

7. **No agent branches exits 0.** It is not an error and must not fail CI or
   a hook. Errors exit 1; usage errors now exit 1 instead of clap's 2, which
   collided with "blocked".

8. **Lean binary via a cargo feature, not a new crate.** `pytxo-cli` gets a
   default-on `full` feature that holds the frozen commands (shell, missions,
   fleets, tiers, Review/Apply of runs). Release builds pass
   `--no-default-features`, so the npm binary carries only the check.
   Workspace builds and CI smokes keep compiling the frozen code. Undo:
   drop the feature gates.

9. **The interactive shell moves to `pytxo shell` (full build only).**
   `pytxo shell` with no `--eval` opens it. The npm binary has no shell;
   there are no users to migrate, so there is no stub.

10. **Receipts stay; their detail moves behind `--verbose`.** Every run still
    writes one under `.git/pytxo/receipts`. The `refs/pytxo/land/<id>` ref is
    kept only with `--verbose`, the one output that prints it.

11. **Only branches with a commit in the last 14 days count.** Agent branches
    pile up; a check of every branch anyone ever pushed is slow and says
    nothing about today's work. The count left out is printed. Change it with
    `git config pytxo.days N` or `PYTXO_DAYS`; `0` turns the filter off.
    The filter runs before the merge tests, which saves git calls per old
    branch, so the printed count includes old branches that are already
    merged ("N with no commit in 14 days not looked at").

12. **The lean binary keeps Signal Core.** Measured on Windows x64 release
    builds: full CLI ~30 MB and 233 crates, lean CLI 10.9 MB and 72 crates.
    Most of what is left is the eight tree-sitter grammars behind "same
    function" detection and `--strict`. At 10.9 MB the download is ~1 s at
    10 MB/s and ~5 s at 2 MB/s, inside the 30-second demo budget, so the
    grammars stay. Undo: make `pytxo-signal` optional in `pytxo-land`.

13. **Windows on Arm installs the x64 binary.** Windows 11 on Arm runs x64
    code under emulation; no native Arm build ships yet.

14. **Release tags must be exact versions.** `release.yml` fired on `v*`, so
    the floating `v2` tag for the Action would have started a release. It now
    fires on `vX.Y.Z` and `vX.Y.Z-*` only.

15. **Desktop's "install CLI" verifies with `--version`.** It ran
    `pytxo doctor --quick`, which the released CLI no longer has.

16. **The agent hooks run on `Stop` only, and stay quiet by default.**
    `pytxo hook` answers when two or more agent branches exist and something
    changed since its last answer (a fingerprint of the target tip, branch
    tips and test command in `.git/pytxo/hook-last`). In Claude Code it waits
    while `background_tasks` still lists subagents, teammates or workflows,
    so parallel agents get one check, after the last one (that field and
    those types are documented Stop hook input, checked 2026-10-28). One
    hook checks a repository at a time (`.git/pytxo/hook.lock`, shared by
    linked worktrees); the others exit quietly, and a lock older than the
    timeout is taken over. The hook answers before `--timeout` (default 900,
    which both configs pass and declare as the hook's timeout). If the tests
    cannot finish in time it says so, points at `npx pytxo`, never blocks,
    and remembers the tips so the next Stop stays quiet. A setting that
    stops every check, such as a missing `pytxo.into`, is reported once.
    The verdict is a `systemMessage`; `--block` (opt-in) turns BLOCKED and
    FLAKY into `decision: "block"` so the agent keeps working on it.
    `SubagentStop` is not used: it fires once per agent. Hooks pin
    `npx -y pytxo@<version>`.

17. **The Action checks agent PRs only, from the same repository.** It runs
    when the PR that triggered it is an agent PR (branch prefix or agent
    account), and adds the other open agent PRs into the same base, oldest
    first, with the triggering PR last so a conflict names the newest change.
    Pull requests from forks never join another PR's run, where their code
    would see that run's token; a fork PR's own run is read-only and checks
    it with the base repository's agent PRs. At most 10 PRs at once. Draft
    PRs count (Copilot opens drafts). Only the triggering PR gets the comment.
    Errors are reported as errors, not as "blocked". The job fails when the
    CLI's last-line verdict tag implicates this PR (it conflicts, fails
    alone, or is in a combination that fails while each passes alone), when
    the result is stale, or on an error. When only other PRs are to blame it
    passes with a notice and the comment names them; FLAKY passes with a
    warning; a CLI without the tag fails on any exit 2. It runs on
    `pull_request`, refuses `pull_request_target` and passes, without a
    checkout, on any other event. A cancelled run posts nothing. The sticky
    comment is the one the token's own account wrote, starting with the
    marker. Code in the checked PRs runs in the job that holds the token and
    can reach it through `$GITHUB_ENV` or the runner, so only the job's
    permissions bound it; the docs say so. A two-job split would isolate it
    and is not built.

18. **Pro licensing is a switched-off scaffold in the Action.** Public
    repositories are free and always will be; local runs are never checked.
    For private repositories in CI: a 14-day trial counted from the first
    commit of the workflow file (no API call, no permission), then a
    `pytxo1.<payload>.<signature>` key passed as `license:` from the
    `PYTXO_LICENSE` secret, verified offline with Ed25519. `LICENSING = false`
    and `LICENSE_PUBLIC_KEY = null` keep it off; switching it on needs a
    signing keypair and a release.

19. **The site leads with the check; Desktop moves to the footer.** The
    header button is "Get started" (the Quickstart), not "Download". The
    homepage drops the Desktop planner, fleet and live-run sections for How
    it works, the one-line Action, a per-branch-CI comparison, Pybench and a
    check FAQ. `/download` stays, reached from the footer as "Pytxo Desktop
    (1.x)", and now leads with `npx pytxo demo`. Same visual system, no new
    components beyond the CI section.
20. **The hero shows a recorded demo, labelled as a local candidate build.**
    `public/evidence/demo-blocked-2.0.0-2026-10-28.json` is the exact output
    of `npx pytxo demo` from a release build of commit bee65c0, with three
    timed runs (5.5-5.8 s). The demo deletes its repository, so the record
    has no receipt. Re-record from npm after the release if wanted. How it
    works quotes lines of the same record. The share image
    (`public/social/check-1600x1000.png`) is rendered from HTML with
    Playwright from the same lines, with the elided progress lines shown as
    `...`.
21. **The plans page links to no checkout.** It shows Free, Pro ("$29 once ·
    not on sale yet", $5/month or $48/year after the first 500) and Team
    (planned), plus "if development stops, Pro unlocks for everyone" from the
    Impulse plan. `pro.ts`, the checkout route, Dodo config, the account page
    and the legal pages are untouched, so the $12/month product still exists
    behind `/api/billing/checkout-redirect` when live keys are set; only the
    "Get Pro" button on `/plans` is gone.
22. **Docs keep every 1.x page and URL.** The sidebar lists Get started
    (Quickstart, GitHub Action, Claude Code and Codex, Install, the two land
    guides), Look it up (CLI, changelog, receipts) and Evidence, then
    "Advanced: Pytxo 1.x" with the Desktop, mission, tier, MCP and
    troubleshooting pages. No page moved, so no redirect is needed.
23. **ADR-0043 supersedes ADR-0012.** A bare `pytxo` runs the check; the
    shell is `pytxo shell` in a `full` build; `PYTXO_NO_TUI` is gone.
24. **A PR comment treats everything from a branch as untrusted text.**
    Captured test output is masked when it is captured: the key formats in
    pytxo-core's `SECRET_PATTERNS`, bearer tokens and URL passwords become
    `[REDACTED]` in receipts, `--json`, hook replies and comments. It is a
    prefix scan of about 10 KB; `pytxo-sanitize` would add a 1.5 MB regex
    engine to the npm binary. In `--markdown`, test output, branch names,
    paths and reasons appear only as code, the terminal's insights are left
    out, advice names the Action's inputs instead of CLI flags, and the
    comment ends with one `<!-- pytxo-verdict {"label":…,"implicated":[…]} -->`
    line for the Action; the same text anywhere else in the body is broken
    with a zero-width space. The test command itself is not masked.
25. **1.x commands in the npm binary answer in one line.** `pytxo shell`,
    `pytxo doctor --quick` and the other frozen commands say they are not in
    this build and exit 1, unless a branch has that name.
26. **The site does not deploy from `main` until npm has 2.0.0.**
    `apps/web/vercel.json` sets `git.deploymentEnabled.main` to `false`, so
    merging 2.0 does not put 2.0 commands on pytxo.com before npm can serve
    them. The promotion commit after the npm release removes the key and
    sets `PUBLISHED_VERSION` to 2.0.0. Undo: drop the key.
