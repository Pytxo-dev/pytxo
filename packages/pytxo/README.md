# pytxo

**Land all your agents' branches in one command.**

Pytxo merges your Claude Code, Codex and Copilot branches in a throwaway
worktree, runs your tests on the combined result, and lands what passes. What breaks stays out, and you see why. Free,
open source, runs on your machine or in CI.

```bash
npx pytxo demo      # watch it catch two branches that pass alone and fail together
npx pytxo           # check the agent branches in your repository
npx pytxo --apply   # and land them if they pass
```

`npx pytxo` finds the agent branches (`claude/*`, `codex/*`, `copilot/*`,
`cursor/*`, ... and every branch in a linked worktree), finds your test command
(`npm test`, `pytest`, `cargo test`, `go test`, ...), and prints a verdict:

```
BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.
why      claude/rename-price renamed price to unitPrice; codex/add-discount still uses price (src/discount.js:1).
next     rebase codex/add-discount onto claude/rename-price and use unitPrice, then run npx pytxo again.
```

Nothing in your repository changes unless you add `--apply`. Exit codes: `0` ready or landed (or
nothing to check), `2` blocked or `--apply` landed only some branches, `3` stale (the target moved),
`1` error, `130` stopped by Ctrl+C.

In CI, one line in a workflow checks each agent pull request together with
the other open ones into its base branch. Add
`.github/workflows/pytxo.yml`:

```yaml
name: Pytxo
on: pull_request
permissions:
  contents: read
  pull-requests: write
concurrency:
  group: pytxo-${{ github.event.pull_request.number }}
  cancel-in-progress: true
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: Pytxo-dev/pytxo/actions/land@v2
```

Your tests run in the job that holds this token, so give the workflow no other
permissions or secrets. If Copilot's coding agent opens pull requests here, add
`actions: read` so the check can include them once someone lets their run
start; with it, code in the checked branches can also read every workflow
run's logs and artifacts. See
[the GitHub Action guide](https://pytxo.com/docs/getting-started/land-in-ci).

Docs: [pytxo.com/docs](https://pytxo.com/docs) · Source:
[github.com/Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo) · Community:
[Discord](https://discord.gg/AUFRPFjSYv)

## How the package works

The first run downloads the same-version prebuilt binary from the public
[pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases) repository and
checks it against the release's `SHA256SUMS.txt`. Package managers that run
install scripts do this during `postinstall` instead; npm 12 and pnpm 10 skip
it by default, and npm's warning about it is safe to ignore. Pytxo sends
nothing anywhere.

Set `PYTXO_SKIP_DOWNLOAD=1` and `PYTXO_BINARY=/path/to/pytxo` for local
development.
