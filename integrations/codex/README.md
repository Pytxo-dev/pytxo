# Pytxo for Codex

A Codex `Stop` hook that checks the agent branches in your repository together
when Codex finishes a turn: it merges them in a throwaway worktree, runs your
tests on the combined result, and shows the verdict. It answers when two or
more agent branches (`codex/*`, `claude/*`, `copilot/*`, ... or branches in
linked worktrees) exist and one of them, the target or the test command changed
since its last answer.

## Install

1. Copy [`hooks.json`](hooks.json) to `~/.codex/hooks.json` (every repository)
   or to `.codex/hooks.json` in one repository. If the file already exists, add
   the `Stop` entry to its `hooks` object.
2. In Codex, open `/hooks` and trust the Pytxo hook. Codex skips hooks nobody
   has trusted.

The hook runs `npx -y pytxo@2.0.0 hook`, so Node.js must be on your PATH. On
Windows, Codex runs it through `cmd.exe`, which finds `npx` as usual.

Codex waits for the hook, so after a change the turn ends only when the check
does: as long as your tests take, up to the hook's 900-second limit. If the
check needs longer, it says so and asks you to run `npx pytxo` in a terminal.
One check runs at a time per repository, linked worktrees included: while one
runs, other sessions' hooks in that repository stay quiet. Only committed work
is checked: when there are two or more agent branches, a worktree whose changes
are all uncommitted gets a hint to commit them. Pytxo's own temporary worktrees
never count as agent branches.

To make Codex keep working on a BLOCKED or FLAKY verdict instead of only
showing it, use `npx -y pytxo@2.0.0 hook --timeout 900 --block`.

Pytxo sends nothing anywhere. Source: [github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).
