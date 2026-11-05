# Pytxo for Claude Code

When Claude stops, Pytxo checks the agent branches in your repository together
in the background: it merges them in a throwaway worktree, runs your tests on
the combined result, and tells Claude on its next turn if they break together.
`/pytxo:land` lands the branches that pass together and leaves out the one
that breaks, with a fix prompt for its agent.

It looks at agent branches (`claude/*`, `codex/*`, `copilot/*`, `cursor/*`, ...
or branches in linked worktrees) when there are two or more and one of them,
the target or the test command changed since its last answer.

## Install

In a shell:

```bash
claude plugin marketplace add Pytxo-dev/pytxo --sparse .claude-plugin integrations/claude-code
claude plugin install pytxo@pytxo
```

`--sparse` limits the checkout to the plugin's directories instead of the
whole repository. Inside Claude Code, `/plugin marketplace add Pytxo-dev/pytxo`
and then `/plugin install pytxo@pytxo` work too, without that limit.

The plugin is a `Stop` hook and the `/pytxo:land` skill, both of which run
`npx -y pytxo@2.0.1`, so Node.js must be on your PATH.

## `/pytxo:land`

Type `/pytxo:land` to land the agents' work on your default branch, or on the
branch you name with `--into` or `git config pytxo.into`. Claude runs `npx -y pytxo@2.0.1 land --apply`, which merges the agent
branches in a throwaway worktree and runs your tests on the combined result:

- When they pass together, the target moves to that tested merge.
- When some break together, it lands the largest set that passed as that
  exact set, and leaves the others out, each with why. A branch that has to
  change gets a fix prompt to paste into its agent.
- Nothing lands that no test run checked as that exact set, and when the
  target moves while the tests run, nothing lands at all.

```
LANDED   1 of 2 branches is on main now: claude/rename-price; codex/add-discount is left out.
why      claude/rename-price passed npm test on top of main; codex/add-discount uses price, which claude/rename-price renamed to unitPrice (src/discount.js:1).
next     push main when you are ready. Give codex/add-discount's agent its fix prompt, then run pytxo land --apply again.
```

Name branches, a target or a test command after it, as for `pytxo land`:
`/pytxo:land --into develop --check "npm test"`. It never pushes.

## The check when Claude stops

The hook runs in the background (`"async": true`), so Claude's turn ends right
away and your tests run while you read. When the check is done, its verdict
reaches Claude on its next turn, and Claude tells you what it found; it
changes nothing for it unless you ask. Claude Code shows you nothing of a
background hook by itself, and when the session is idle the verdict waits for
your next message. For example:

```
BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.
why      claude/rename-price renamed price to unitPrice; codex/add-discount still uses price (src/discount.js:1).
next     rebase codex/add-discount onto claude/rename-price and use unitPrice, then Pytxo checks again when an agent stops.
```

When Claude runs subagents in the background, the check waits until the last
one finishes. One check runs at a time per repository, linked worktrees
included: a hook that stops while another one checks waits for that check, up
to its own time limit of 900 seconds, and a check that needs longer says so
and asks you to run `npx pytxo` in a terminal. When the failing output confirms
the cause, the sessions on the two branches behind it each hear it once, from
the same check; otherwise only the session whose hook ran the check does. Only
committed work is checked: when there are two or more agent branches, a
worktree whose changes are all uncommitted gets a hint to commit them. The
check changes nothing in your repository: it runs in a temporary worktree and
only records a receipt under `.git/pytxo/receipts`. Pytxo's own temporary
worktrees never count as agent branches.

## Options

Copy this plugin and change `hooks/hooks.json`:

- Make Claude keep working on a BLOCKED or FLAKY verdict instead of only
  telling you: add `--block` to the command. When the failing output confirms
  the cause, Claude gets a fix prompt: what one branch renamed or removed,
  where the other still uses it, and which branch has to change.
- See the verdict yourself as the turn ends, the way 2.0.0 did: remove
  `"async": true` and `--async`. Claude Code then waits for the check, as long
  as your tests take, before the turn ends; with `--block` too, Claude keeps
  working in the same turn.

And in your repository:

- Pick the branches: `git config pytxo.branches "claude/* feature/*"`.
- Pick the test command: `git config pytxo.check "npm test"`. Without one,
  Pytxo uses the project's test command; a package.json with no test script
  gets its `build`, `typecheck` or `lint` script checked instead, and the
  verdict says that is not a test suite.
- Check against a branch other than the default branch:
  `git config pytxo.into develop`.

Pytxo sends nothing anywhere.
Source: [github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).
