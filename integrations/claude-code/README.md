# Pytxo for Claude Code

When Claude stops, Pytxo checks the agent branches in your repository together:
it merges them in a throwaway worktree, runs your tests on the combined result,
and tells you if they break together. It answers when two or more agent
branches (`claude/*`, `codex/*`, `copilot/*`, `cursor/*`, ... or branches in
linked worktrees) exist and one of them, the target or the test command changed
since its last answer.

## Install

In a shell:

```bash
claude plugin marketplace add Pytxo-dev/pytxo --sparse .claude-plugin integrations/claude-code
claude plugin install pytxo@pytxo
```

`--sparse` limits the checkout to the plugin's directories instead of the
whole repository. Inside Claude Code, `/plugin marketplace add Pytxo-dev/pytxo`
and then `/plugin install pytxo@pytxo` work too, without that limit.

The plugin is one `Stop` hook that runs `npx -y pytxo@2.0.0 hook`, so Node.js
must be on your PATH. The verdict appears as a message when the turn ends:

```
Pytxo checked the agent branches together.
BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.
why      npm test fails on the combined change: FAIL discount.test.js: price is not a function
next     fix one branch so they agree, then Pytxo checks again when an agent stops.
```

Claude Code waits for the hook, so after a change the turn ends only when the
check does: as long as your tests take, up to the hook's 900-second limit. If
the check needs longer, it says so and asks you to run `npx pytxo` in a
terminal. When Claude runs subagents in the background, the check waits until
the last one finishes. One check runs at a time per repository, linked
worktrees included: while one runs, other sessions' hooks in that repository
stay quiet. Only committed work is checked: a worktree whose changes are all
uncommitted gets a hint to commit them. Nothing in your repository changes: the
check runs in a temporary worktree and only records a receipt under
`.git/pytxo/receipts`. Pytxo's own temporary worktrees never count as agent
branches.

## Options

- Make Claude keep working on a BLOCKED or FLAKY verdict instead of only
  telling you: copy this plugin, change the command to
  `npx -y pytxo@2.0.0 hook --timeout 900 --block`.
- Pick the branches: `git config pytxo.branches "claude/* feature/*"`.
- Pick the test command: `git config pytxo.check "npm test"`.
- Check against a branch other than the default branch:
  `git config pytxo.into develop`.

Pytxo sends nothing anywhere. Source: [github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).
