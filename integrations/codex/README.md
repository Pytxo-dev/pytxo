# Pytxo for Codex

A Codex `Stop` hook that checks the agent branches in your repository together
when Codex finishes a turn: it merges them in a throwaway worktree, runs your
tests on the combined result, and shows the verdict. It stays quiet unless two
or more agent branches (`codex/*`, `claude/*`, `copilot/*`, ... or branches in
linked worktrees) changed since its last answer.

## Install

1. Copy [`hooks.json`](hooks.json) to `~/.codex/hooks.json` (every repository)
   or to `.codex/hooks.json` in one repository. If the file already exists, add
   the `Stop` entry to its `hooks` object.
2. In Codex, open `/hooks` and trust the Pytxo hook. Codex skips hooks nobody
   has trusted.

The hook runs `npx -y pytxo@2.0.0 hook`, so Node.js must be on your PATH. On
Windows, Codex runs it through `cmd.exe`, which finds `npx` as usual.

One check runs at a time per repository, linked worktrees included: while one
runs, other sessions' hooks in that repository stay quiet. The hook finishes
inside its time limit; if the check needs longer, it says so and asks you to
run `npx pytxo` in a terminal. Pytxo's own temporary worktrees never count as
agent branches.

To make Codex keep working on a BLOCKED verdict instead of only showing it, use
`npx -y pytxo@2.0.0 hook --block`.

Pytxo sends nothing anywhere. Source: [github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).
