---
title: ADR-0043 The check is the default command
slug: ADR-0043-check-is-the-default-command
adr_id: ADR-0043
status: accepted
tags: [adr, cli, land, ux, release]
audience: [human, agent]
layer: product
created: 2026-10-28
updated: 2026-10-28
related: [[ADR-0012-hypervisor-shell-default-ux]], [[ADR-0042-landing-gate-and-receipts]]
supersedes: [[ADR-0012-hypervisor-shell-default-ux]]
---

# ADR-0043: The check is the default command

## Status

Accepted for 2.0 (Matt, 2026-10-28: the Impulse plan, "npx pytxo runs the check
instead of opening a shell, with no flags, config or account"). Supersedes
[[ADR-0012-hypervisor-shell-default-ux]].

## Context

ADR-0012 made a bare `pytxo` open the Hypervisor Shell. A newcomer's first
run therefore showed a tier picker and a dashboard, not an answer, and
`pytxo land` needed branch names and a `--check` command before it showed
anything. Pytxo 2.0 narrows the product to one question, "do my agents'
branches pass together?", answered in under 60 seconds with no flags.

## Decision

1. `pytxo` with no subcommand runs `pytxo check`: it detects the agent
   branches (`claude/*`, `codex/*`, `copilot/*`, `cursor/*`, `devin/*`,
   `jules/*`, `agent/*` and linked worktrees, with a commit in the last 14
   days), the target branch and the test command, then prints a three-line
   verdict. `pytxo land` is an alias of `pytxo check`.
2. Detection lives in `pytxo-land` (`detect.rs`); the CLI stays thin.
3. The released binaries build `pytxo-cli` with `--no-default-features`: the
   check, `demo`, `hook`, `receipts` and `bench`. Desktop-era commands
   (`run`, `mission`, `fleet`, `review`, `apply`, `doctor`, `trust`, the MCP
   tooling and the shell) stay behind the default `full` feature and are
   frozen.
4. In a `full` build the shell opens with `pytxo shell`. `PYTXO_NO_TUI` is
   gone.

## Consequences

- First run shows a verdict, or one line that says there is nothing to check
  and points at `pytxo demo`.
- Scripts that relied on a bare `pytxo` opening the shell must call
  `pytxo shell` in a `full` build.
- The release binary drops from about 30 MB to 11 MB on Windows.
- Desktop keeps working with the CLI it installs; it checks the CLI with
  `pytxo --version`.
