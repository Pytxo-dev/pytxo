# Decisions: Pytxo 2.0 and 2.1

Reversible choices made while building 2.0 and 2.1, newest last. Each says what was
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
   but the headline says FLAKY so nobody blames the agents for it. With
   several checks, one that fails again on the combination speaks over one
   that passed when run again, so a flaky test never hides a real break.

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
    merged ("N with no commit in 14 days not looked at"), and it counts
    branches, not commits. A worktree on a commit the target (or origin's
    copy of it) already has is never old, since its only work can be
    uncommitted: it still gets the hint to commit it. That costs one
    `git rev-list` per run when any worktree is old, plus a `git status` for
    each worktree it exempts.

12. **The lean binary keeps Signal Core.** Measured on Windows x64 release
    builds: full CLI ~30 MB and 233 crates, lean CLI 10.9 MB and 72 crates.
    Most of what is left is the eight tree-sitter grammars behind "same
    function" detection and `--strict`. At 10.9 MB the download is ~1 s at
    10 MB/s and ~5 s at 2 MB/s, inside the 30-second demo budget, so the
    grammars stay. Undo: make `pytxo-signal` optional in `pytxo-land`.

13. **Windows on Arm installs the x64 binary.** Windows 11 on Arm runs x64
    code under emulation; no native Arm build ships yet. The npm package and
    `install.ps1` both do this.

14. **Release tags must be exact versions.** `release.yml` fired on `v*`, so
    the floating `v2` tag for the Action would have started a release. It now
    fires on `vX.Y.Z` and `vX.Y.Z-*` only. The maintainer moves `v2` by hand
   after each final 2.x release (MAINTAINING.md), since the Action pins the
   CLI version it installs.

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
    and remembers the tips so the next Stop stays quiet. An error that
    stops the check, such as a missing `pytxo.into`, is reported once, until
    a tip, the target or the test command changes.
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
    it with the base repository's agent PRs. Nor do pull requests GitHub
    holds until a person approves their workflow run (authored by Copilot,
    `copilot-swe-agent[bot]` or `github-actions[bot]`): another PR's run
    would run their code before anyone approved it. Their own run, once
    approved, checks them with the others. At most 10 PRs at once. Draft PRs
    count. With no other agent PR open into the same base, the run passes
    with a note in the job summary and creates no comment, though it updates
    one an earlier run left. Only the triggering PR gets the comment. Each PR
    is fetched at the commit the run saw (the event's head for the
    triggering PR, the listing's for the others), not at `refs/pull/N/head`,
    which moves with every push: an approved run checks what was approved,
    and a later push gets a run of its own.
    Errors are reported as errors, not as "blocked". The job fails when the
    CLI's last-line verdict tag implicates this PR (it conflicts, fails
    alone, or is in a combination that fails while each passes alone), when
    the result is stale, or on an error. When nothing points at one PR (a
    combined timeout, or a `--strict` collision), every PR in the run is
    implicated and fails. When only other PRs are to blame it
    passes with a notice and the comment names them; FLAKY passes with a
    warning; a CLI without the tag fails on any exit 2. It runs on
    `pull_request`, refuses `pull_request_target` and passes, without a
    checkout, on any other event. A cancelled run posts nothing. The sticky
    comment is the one the token's own account wrote, starting with the
    marker. Code in the checked PRs runs in the job that holds the token and
    can reach it through `$GITHUB_ENV` or the runner, so only the job's
    permissions bound it; the docs say so. A two-job split would isolate it
    and is not built. Held pull requests: superseded by 44, which lets them
    join at a commit a person let run.

18. **Pro licensing is a switched-off scaffold in the Action.** Public
    repositories are free and always will be; local runs are never checked.
    For private repositories in CI: a 14-day trial counted from the first
    commit of the workflow file (no API call, no permission), then a
    `pytxo1.<payload>.<signature>` key passed as `license:` from the
    `PYTXO_LICENSE` secret, verified offline with Ed25519. `LICENSING = false`
    and `LICENSE_PUBLIC_KEY = null` keep it off; switching it on needs a
    signing keypair and a release. The keys: superseded by 46, Dodo Payments
    license keys checked online.

19. **The site leads with the check; Desktop moves to the footer.** The
    header button is "Get started" (the Quickstart), not "Download". The
    homepage drops the Desktop planner, fleet and live-run sections for How
    it works, the one-line Action, a per-branch-CI comparison, Pybench and a
    check FAQ. `/download` stays, reached from the footer as "Pytxo Desktop
    (frozen)", and now leads with `npx pytxo demo`. Same visual system, no new
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
    Captured test output is masked when it is captured: PEM private key
    blocks, the key formats in pytxo-core's `SECRET_PATTERNS` plus npm
    tokens, bearer tokens, URL passwords, and the value of any key whose name,
    lowercased and without `-` and `_`, contains secret, token, password,
    passwd, apikey, accesskey, privatekey, credential or authorization (so
    `privateKey` and an `Authorization:` header too; after `=`, `:`, `=>`
    or `:=`) become
    `[REDACTED]` in receipts, `--json`, hook replies and comments. A name
    ending in Error or Exception is not a key, so `JsonWebTokenError: ...`
    stays readable; the cost is masking harmless values such as
    `max_tokens: 100`. Carriage returns, ANSI escapes and other control
    characters are stripped. It is a prefix scan of about 10 KB; `pytxo-sanitize` would add a 1.5 MB regex
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
    them. `deploy-web.yml`, the Vercel CLI route to production, exits with a
    notice while `PUBLISHED_VERSION` and `CANDIDATE_VERSION` differ. The web
    CI job runs the version check and the inventory test, which asserts both
    holds, so a site-only change cannot drop one. The promotion commit after
    the npm release removes the key, sets `PUBLISHED_VERSION` to 2.0.0 and
    updates the changelog's latest public release. Undo: drop the key.
27. **The changelog's latest public release must match `PUBLISHED_VERSION`.**
    `verify-release-version.mjs` reads it like the README's, so the
    promotion commit cannot leave the docs naming 1.3.6.
28. **A failed binary download does not fail `npm install` or `npx`.**
    Offline machines and proxies are common, and the launcher downloads on
    first run anyway. A binary that fails its SHA-256 or `--version` check
    still fails the install.
29. **Detection reads git config the way git does, and says what it left
    out.** Settings come from `git config --show-scope --get-all`, keeping
    the most specific scope's values, so a repository's `pytxo.check`
    replaces a global one instead of joining it; Git older than 2.26 falls
    back to `--get-all` and keeps the last value, and the demo and Pybench
    name `main` with `symbolic-ref`, since `git init -b` needs 2.28. At most
    the 10 newest branches are checked. Branches that share no history with
    the target (a `gh-pages` worktree) are left out and counted under
    `--verbose`. A worktree whose changes are all uncommitted is named, with
    a hint to commit them. Note lines under the verdict, and `notes` in
    `--json`, say when the target is behind `origin` or branches were left
    out; the no-branches line suggests `git fetch`. `--json` gives each
    branch's `label` when it has one. Undo: drop the notes and the cap.
30. **A squash-merged branch is found by patch id when a merge cannot
    tell.** When the target has edited the same lines again, or Git has no
    `merge-tree --write-tree`, Pytxo squashes the branch with `commit-tree`
    under a fixed identity and date and asks `git cherry` whether the target
    has that patch. The fixed identity makes the object the same on every
    run, so detection writes one object per distinct squash, once. Known
    limit: on old Git a branch squash-merged and later reverted on the
    target is still skipped.
31. **Ctrl+C cleans up; a second Ctrl+C quits at once.** The first Ctrl+C,
    SIGTERM or SIGHUP prints one line, removes the run's scratch worktrees
    and logs, and exits 130; a second one exits 130 at once. The handler is
    installed as the check starts, before branches are found or the demo
    repository is built, which an interrupt removes too. The scratch checkout,
    Pytxo's one long Git command, is stopped like a check: on Windows Ctrl+C
    does not reach the git it starts, which went on writing the worktree being
    removed. Its other Git commands in a scratch worktree are short; an
    interrupt during one can leave files for the next run's sweep. On Unix a
    signal ignored when the binary started (`nohup`, `pytxo &`) stays
    ignored; npx and npm start it with every signal back to its default, so
    this holds only for the binary run directly. The npm launcher ignores
    these signals while the binary runs, so the terminal comes back only after
    the cleanup, and passes the binary's exit code on (128 plus the signal
    number when it was killed). Under `npx` on Windows, npm handles Ctrl+C by
    ending its child shell and returns at once with exit 1 while the binary
    finishes cleaning up; a launcher cannot change that. A run killed
    hard leaves its directory for the next run to remove: scratch worktrees
    are dated by the run start in their tag, and unlocked young ones are
    kept for 24 hours. ctrlc is pinned at 3.4.6, which adds no crates; libc
    is a Unix-only dependency for reading the inherited signal disposition
    and for kill(2), so stopping a check needs no `kill` binary (slim images
    have none).
32. **Checks run in a process group of their own, with nothing to answer
    a prompt.** On Unix a check runs in its own process group, so a timeout
    or Ctrl+C stops the whole tree, and the run's directory is private
    (0700, logs 0600). Scratch checkouts run no Git hooks. Every check gets an
    empty stdin and `GIT_TERMINAL_PROMPT=0`; a check that opens the terminal
    itself (an ssh passphrase, sudo) waits until `--timeout`. `PYTHONPATH`
    gets the checkout first. Submodules are never checked out, even with the
    user's `submodule.recurse` set, and failed verdicts say so when the
    target has `.gitmodules`.
33. **When every branch fails alone, the target runs once.** That run
    (scope `target` in the receipt) tells "the target was already broken"
    from "every branch broke it". It runs once per command and never after a
    timeout, and a target run that timed out is not evidence. After a
    combined run times out, nothing else runs. A branch that conflicts is
    merged onto the target alone before an earlier branch is blamed. Each
    later run in a reused scratch worktree starts clean: `git clean -ffdx`
    keeps only `node_modules` and `target`, whose tools never run stale
    output, so build output a test command runs (a `tsc` outDir) cannot carry
    over from another branch. Cost: on a failure, other ignored caches (a
    `.venv`, Gradle's `build`) are rebuilt for each branch. The clean is best
    effort: a path it cannot remove (a container's root-owned cache, a file a
    leftover process holds open) stays, and the run still reaches a verdict.
    The second combined run that tells a flaky check from a real break starts
    clean too, after the checks before it run again to set it up; the receipt
    keeps their first runs.
34. **Windows checks see the long temp path when it is plain.**
    `C:\Users\RUNNER~1\...` confuses tools that compare paths, so checks run
    under the long spelling, unless it has a space or a non-ASCII letter or
    is a UNC or volume path, where the short spelling is safer for unquoted
    tools and cmd.exe.
35. **The docs keep 1.x material in place and show one setup everywhere.**
    The CLI reference keeps the 1.x commands as H3 sections under "Pytxo 1.x
    commands", so existing anchors work; the 2.0 environment variables come
    first and the old table is "1.x environment variables". Every copy of
    the workflow snippet carries a per-PR concurrency group with
    `cancel-in-progress`. The Claude Code install leads with the shell
    commands and `--sparse` (checked with Claude Code 2.1.299); `/plugin` in
    a session is the second route.
36. **Two results exit 0 without a pass.** `pytxo demo` exits 0 when it
    shows the BLOCKED it was built to show, and 1 when it does not. NO TESTS
    exits 0 (`decision` ready), so a repository without tests does not fail
    CI, and the line says no tests ran. `--apply` refuses it with exit 1 and
    lands nothing: a merge nothing tested is not a result to land.
37. **`npx pytxo` stays the headline command.** npx runs a pytxo already
    installed in the project or globally before it fetches one, so a 1.x
    global install answers instead of 2.0. The install page and release notes
    tell upgraders to run `npm i -g pytxo@latest`, instead of every snippet
    saying `npx pytxo@latest`, which costs a registry check per run.

## 2.1: the check that says why

Spec: [`docs/superpowers/specs/2026-10-29-pytxo-2-1-smart-check-design.md`](docs/superpowers/specs/2026-10-29-pytxo-2-1-smart-check-design.md).

38. **A cause is a definition one branch renamed or removed and another
    branch's new code still uses.** It is looked for only behind a
    "together" verdict, from git and tree-sitter alone: no model, nothing
    sent. The name was defined in the branch's changed files before it and
    is not after, and is no longer code anywhere on that branch (a mention
    in a string, a comment or a doc does not keep it). A use is a line
    another merged branch added where code refers to the name. A string, a
    comment, an object key or a field read `a.name` is none; a call
    `a.name()` is one when the gone definition was a method, and `m.name` is
    one when `m` is a module the file imports. A name the file gives what it
    imports (a default or namespace import, an alias, a name from a `node:`
    module) is the file's own, and a file whose combined version binds the
    name itself (the user's own `const name`, a parameter or a local) has
    none; a call added beside the removed definition, in its own file, is a
    use. A file without a grammar gives none. When the combined change still
    binds the name in any form, there is no cause. It is confirmed only when
    an error, traceback, assert, diagnostic or `FAIL` line of the failing
    output says the name, or a compiler's place or the frame an error was
    thrown at points at a use. An assertion's frame or a caller's does not,
    and a use that is only a type is confirmed only by a line that says the
    name, since types are gone before the code runs. Counts, summaries, test
    titles and paths never confirm. Each failing run's last 40 lines are
    read, and up to ten error lines above them, which the receipt does not
    keep. At most 200 files and 50 names per branch, 100 `git grep` calls
    and 1 MB per file, and never past the deadline: a cap gives no cause,
    never a guess. The cause is a reading of the runs, not a record of them,
    so it stays out of the receipt. Renamed config keys, CSS classes and
    changed signatures are out of scope. To undo, have `cause::find` return
    `None`: the why line is the error line again.
39. **The verdict states a cause only when it is confirmed.** Confirmed, the
    why line says what one branch renamed or removed and where the other
    still uses it, and the next line says which branch to rebase and what to
    use. Not confirmed, the why line stays the error line and a `note` names
    the possible cause. Other branches that use the name too are named in
    the same sentence. The terminal verdict stays three lines; the PR
    comment adds the fix prompt under `<details>`, and `--json` adds
    `verdict.cause` and `verdict.fix_prompt`. Only a confirmed cause has a
    fix prompt.
40. **With three or more branches, the cause's pair is proven before the
    others are cleared.** After a "together" failure with a cause, confirmed
    or not, the failing command runs on the target plus the pair (`pair`)
    and, if that fails, on the target plus every other merged branch
    (`rest`); with one other branch, its run alone already is that run.
    When the pair fails and the rest passes, the pair stays implicated with
    the other branches that use the name and every branch in any other cause
    found, so the Action passes only the rest's pull requests: a second
    branch that removed the name passes in the rest run while it breaks the
    user just the same. When the pair passes, the verdict is 2.0's, with a
    note. At most two extra runs, only of the first failing command, never
    after a timeout, and not when a run would end past the deadline: one run
    of room when the single other branch already ran alone, two otherwise,
    counting each check at its longest run so far. The checks before that
    command run again first, unrecorded, to set the tree up, as for the
    rerun. The pair comes from the cause (confirmed first, then most uses,
    then input order), and a wrong pick passes and leaves everyone blamed. A
    pair or rest row names what it merged in its scope, `pair:1,2` and
    `rest:0,3`, not in a field of its own: 2.0 drops a field it does not
    know when it recomputes a digest, and would read such a receipt as
    edited. A search without a cause would cost log n runs of the suite and
    is not built.
41. **`--block` hands the fix prompt to the agent that has to change.** In
    the using branch's worktree the hook's reason calls it "your branch"; in
    the renaming branch's worktree it says the fix belongs on the other
    branch, or to keep the old name as an alias; anywhere else it is the
    plain prompt. Each of the two agents hears its own prompt once: the
    first to stop runs the check and gets its prompt, and the other gets its
    own at its next stop from the same check, kept by branch in
    `.git/pytxo/hook-kept` for a worktree that still has that branch checked
    out. Each stop answers the way its own hook does, the prompt with
    `--block` and a system message without, so with a confirmed cause both
    sessions see the verdict once, where 2.0 told one session per
    repository. A hook that stops while another hook checks the repository
    waits for that check, up to its own deadline, instead of staying quiet:
    if nothing moved since, it gives the reply that check kept for its
    branch, or nothing if there is none, and if something moved it checks
    itself; it stops waiting for a hook that is gone. The fix runs in the
    agent the user already pays for, so Pytxo still has no model bill.
42. **The why line is the error the check printed, not its count.** In
    order: an error or exception name (`TypeError:`, `error[E…]`, a Rust
    panic, Go's `panic:`); a traceback, an assert, or a diagnostic,
    `<file>.<ext>:<line>: <message>` or tsc's
    `<file>.<ext>(<line>,<col>): <message>`; a `FAIL <file>` line or Go's
    `--- FAIL:`; then any other line with error or fail. Count lines
    (`ℹ fail 1`, `Tests: 1 failed`) and Go's `FAIL <package>` summary never
    are. Error names rank above tracebacks, so pytest's why line is its
    `E   NameError: …`, not the source line under it, and unittest's
    `ImportError: Failed to import test module` ranks with any line that
    says error, below the traceback's own error. It reads the same lines as
    the cause, so one failed package of a Go module, or rustc's error above
    its warnings, still says why.
43. **A branch name means the branch.** The CLI looks up
    `refs/heads/<name>` first, so a tag, a `FETCH_HEAD`-style file in
    `.git` or a short `heads/x` name no longer changes the commit checked.
    The Action checks the same in its checkout: it deletes a tag named like
    a branch it fetched, and leaves out a pull request whose name git still
    reads as something else. A tag pushed with an agent branch's name could
    otherwise have had the check run a commit nobody approved.
44. **Copilot's pull requests join at a commit a person let run.** This
    supersedes the exclusion in 17. With `actions: read`, a held pull
    request joins another's run at the newest of its own commits that has a
    `pull_request` run on this repository whose conclusion is not
    `action_required`. Approving that run, or re-running it, takes a person
    with write access, so its code runs only after someone let it. It is
    fetched by that commit, and left out with a note if a push took the
    commit away. A 403 or 404 from the first runs call that is not a rate
    limit adds the note to add `actions: read`; any other GitHub error
    leaves the held pull requests out with a note, and the run goes on.
45. **`actions: read` is opt-in.** The documented setup stays
    `contents: read` and `pull-requests: write`, and one sentence says to
    add `actions: read` when Copilot's coding agent opens pull requests.
    Agent code in the job can reach the token, and with `actions: read` it
    can read every workflow run's logs and artifacts.
46. **Pro keys are Dodo Payments license keys, checked online and still
    switched off.** This supersedes the Ed25519 keys in 18: a monthly
    subscription needs a key that stops working when it lapses. Only on a
    private repository, after the trial and with `LICENSING` on does the
    Action send anything: the key, to Dodo's public validate endpoint. Only
    a definite answer refuses it: `valid: false`, or a 4xx other than 408
    and 429. No answer within 10 seconds, a network error, 408, 429, 5xx or
    an answer it cannot read runs the check with a warning, so trouble at
    Dodo never blocks a pull request. Keys are not bound to a repository;
    the Action is MIT, so the gate is an honor system until its license or
    Dodo's activations make it binding.
47. **A path into the scratch worktree reads from its root.** Checks run in
    a temporary worktree, so a stack frame, a traceback or a compiler's
    place showed its temp path in the receipt, the why line, the hook's
    reply and the PR comment. Each such path now reads as in the branch's
    own checkout (`discount.js:4:10`, the root itself `.`). The worktree is
    found by its own name, unique to the run, in any spelling: long or
    short, either separator, or a `file:///` URL. A path printed in full
    with a space in the temp directory keeps the part before its last space.
48. **The lean binary grows by 330 KB, over the 200 KB budget.** 2.0's lean
    build is 11,757,568 bytes and 2.1's 12,096,000 (+338,432). All of it is
    first-party code: the cause search, pytxo-signal's name reading, the
    pair runs, and the hook's waiting and kept replies. No crate was added.
    Accepted: about 3% of a binary that `npx pytxo` downloads once.
49. **2.1's work ships as 2.0.0.** 2.0.0 was never published: npm's latest
    is still 1.3.6. So the cause, the pair runs, the fix prompt and the
    waiting hooks ship in 2.0.0, the first release of the check, with one set
    of release notes, instead of 2.0 and then 2.1 days apart (decided 30
    October 2026). 1.0.0 was asked for and is not possible: npm already has
    pytxo 1.0.0, the first public cut (tag `v1.0.0`), and never reuses a
    version, and the check-only CLI breaks 1.3.6's commands, so it takes a
    new major. Entries 38 to 48 keep the 2.1 name they were decided under,
    and the 2.1 spec stays as written. The 2.0 branch and its site promotion
    (`d878f68`) do not ship on their own. The demo records of 28 and 30
    October, which were never published, give way to one of the 2.0.0 build
    that ships.
