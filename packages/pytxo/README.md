# pytxo

**Each agent's branch passes. Do they pass together?**

Pytxo merges your Claude Code, Codex and Copilot branches in a throwaway
worktree, runs your tests on the combined result, and blocks the merge if it
breaks. Free, open source, runs on your machine or in CI.

```bash
npx pytxo demo   # watch it catch two branches that pass alone and fail together
npx pytxo        # check the agent branches in your repository
```

`npx pytxo` finds the agent branches (`claude/*`, `codex/*`, `copilot/*`,
`cursor/*`, ... and every branch in a linked worktree), finds your test command
(`npm test`, `pytest`, `cargo test`, `go test`, ...), and prints a verdict:

```
BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.
why      npm test fails on the combined change: FAIL discount.test.js: price is not a function
next     fix one branch so they agree, then run npx pytxo again.
```

Nothing in your repository changes. Exit codes: `0` ready (or nothing to
check), `2` blocked, `3` stale, `1` error.

In CI, one line in a workflow checks every agent pull request:

```yaml
- uses: Pytxo-dev/pytxo/actions/land@v2
```

Docs: [pytxo.com/docs](https://pytxo.com/docs) · Source:
[github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo) · Community:
[Discord](https://discord.gg/AUFRPFjSYv)

## How the package works

The first run downloads the same-version prebuilt binary from the public
[pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases) repository and
checks it against the release's `SHA256SUMS.txt`. Package managers that run
install scripts do this during `postinstall` instead; npm 12 and pnpm 10 skip
it by default, and npm's warning about it is safe to ignore. Pytxo sends no
telemetry.

Set `PYTXO_SKIP_DOWNLOAD=1` and `PYTXO_BINARY=/path/to/pytxo` for local
development.
