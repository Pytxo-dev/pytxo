---
name: land
description: Land the agent branches that pass together; leave out the one that breaks, with its fix prompt
argument-hint: "[branches...] [--into <branch>] [--check \"<command>\"]"
allowed-tools: Bash(npx -y pytxo@2.0.2 land:*)
disable-model-invocation: true
---

Land the coding agents' work in this repository with Pytxo.

1. In the repository, run:

   ```bash
   npx -y pytxo@2.0.2 land --apply $ARGUMENTS
   ```

   With no arguments it finds the agent branches (`claude/*`, `codex/*`,
   `copilot/*`, `cursor/*`, ... and branches in linked worktrees) and the test
   command. It merges them in a throwaway worktree, runs the tests on the
   combined result, and moves the target only to a commit the tests passed on.
   The target is the branch you name with `--into`, else `git config pytxo.into`,
   else the checked-out branch when you name branches, else the repository's
   default branch. When some branches break together, it lands the largest set
   that passed as that exact set and leaves the others out. It takes as long
   as the test suite: give the command the longest timeout you can, and run
   it in the background if it may take longer.

2. Exit codes: 0 everything landed (or there was nothing to land), 2 blocked
   or only some branches landed, 3 the branch moved while it ran (run it
   again), 1 an error. Exit 2 is a result to report, not a failure to retry.

3. Tell the user what the output says, in its own words: the three lines
   (LANDED or BLOCKED, why, next) and any notes. For every
   "Fix prompt for <branch>:" block, show the branch and the prompt in a
   fenced block, so the user can paste it into that branch's agent.

4. Do not push, do not run it again on your own, and do not change the
   branches that were left out unless the user asks you to.
