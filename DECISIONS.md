# Decisions: Pytxo 2.0

Reversible choices made while building 2.0, newest last. Each says what was
chosen, why, and how to undo it.

1. **`check` is the command; `land` is its alias.** Both run the same engine
   with the same flags. With no branches given, branches and the test command
   are detected. The library's `land()` still never invents a check; the CLI
   fills one in. Undo: split the clap variants again.

2. **Target = the default branch, not the checked-out one.** Order:
   `--into`, then `origin/HEAD`, `init.defaultBranch`, `main`, `master`,
   `trunk`, then the checked-out branch. A hook runs inside an agent's
   worktree, where "the checked-out branch" is the agent branch itself.

3. **Agent branches = patterns plus linked worktrees.** Default list:
   `claude/* codex/* copilot/* cursor/* devin/* jules/* agent/* worktrees`.
   `claude/`, `codex/`, `copilot/`, `cursor/`, `devin/` and `jules/` are the
   prefixes those agents use for the branches and PRs they create; `agent/` is
   Pytxo's own convention. `worktrees` means every branch checked out in a
   linked worktree (Claude Code Desktop, Conductor and Codex use custom
   names there). Override with `git config pytxo.branches "..."` or
   `PYTXO_BRANCHES`; an explicit list replaces the defaults. Remote-tracking
   branches count when no local branch has that name. Branches already in
   the target, or whose merge adds nothing (squash-merged), are skipped.

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
