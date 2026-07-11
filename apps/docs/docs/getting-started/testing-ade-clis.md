---
title: Testing ADE CLIs
---

# Testing ADE CLIs with Pytxo

Pytxo orchestrates **headless terminal agents**. You install the agent CLI separately; Pytxo spawns it inside managed PTYs with git worktree isolation and Signal Core context.

## Prerequisites

```bash
npm i -g pytxo
pytxo doctor
```

You need:

- Git 2.20+ with at least one commit in your test repo
- The ADE CLI on `PATH` (for live runs)
- API keys for live LLM-backed agents (BYOK)

## Quick scaffold (recommended)

From the Pytxo repo root:

**Windows (run scaffold and recipe in the same PowerShell session):**

```powershell
.\tooling\test-envs\scaffold.ps1 -Profile antigravity
cd $env:PYTXO_TEST_REPO
& C:\pytxo\tooling\test-envs\recipes\live-antigravity.ps1
```

If `pytxo` is not on PATH, scaffold sets `$env:PYTXO_BIN` automatically (npm global, local `target/release/pytxo.exe`, etc.).

**macOS / Linux:**

```bash
./tooling/test-envs/scaffold.sh echo
cd "$PYTXO_TEST_REPO"
# from Pytxo repo root:
./tooling/test-envs/recipes/smoke-echo.sh
```

Profiles: `echo` (no API keys), `context` (Signal Core stub), `antigravity`, `claude`, `codex`.

## Antigravity CLI (`agy`)

Pytxo recognizes Antigravity via `cli_adapter = "agy"` or `"antigravity"` in `pytxo.toml`.

1. Install [Antigravity CLI](https://antigravity.google/) and authenticate per Google's docs.
2. Scaffold with the antigravity profile:

   ```powershell
   .\tooling\test-envs\scaffold.ps1 -Profile antigravity
   cd $env:PYTXO_TEST_REPO
   .\tooling\test-envs\recipes\live-antigravity.ps1
   ```

3. Dry-run the task graph:

   ```bash
   pytxo run --config pytxo.toml --dry-run
   ```

4. Smoke that `agy` launches in a PTY:

   ```bash
   pytxo run --config pytxo.toml --cmd "agy --help"
   ```

5. Inspect telemetry:

   ```bash
   pytxo status
   pytxo logs --agent <run_id>:agent-0 --tail 50
   ```

Replace `--cmd` with your non-interactive `agy` invocation once the smoke passes.

## Claude Code

```toml
[[agent]]
name = "claude-builder"
paths = ["src/**"]
cli_adapter = "claude_code"
```

```bash
pytxo run --config pytxo.toml --cmd "claude -p \"Summarize src/a.ts\""
```

Requires `ANTHROPIC_API_KEY` (or your configured auth).

## OpenAI Codex CLI

Use the `generic` adapter (default). If PTY quirks appear, try subprocess:

```bash
pytxo run --config pytxo.toml --execution subprocess --cmd "codex --help"
```

## Stub mode (no API keys)

Use the context profile to validate Signal Core without calling an LLM:

```powershell
.\tooling\test-envs\scaffold.ps1 -Profile context
cd $env:PYTXO_TEST_REPO
..\recipes\smoke-context.ps1
```

The stub agent prints `PYTXO_CONTEXT_DIR` and reads `manifest.json`. See the [context launch contract](https://github.com/Pytxo-dev/pytxo/blob/main/docs/02-areas/orchestration/context-launch-contract.md) in the repo.

## Cursor (MCP, not `--cmd`)

Cursor drives Pytxo through **MCP**, not `pytxo run --cmd`. See [MCP from Cursor](/docs/getting-started/mcp-from-cursor).

## ADE compatibility matrix

| Agent | `cli_adapter` | Typical `--cmd` |
|-------|---------------|-----------------|
| Antigravity CLI | `agy` / `antigravity` | `agy …` |
| Claude Code | `claude_code` / `claude` | `claude -p "…"` |
| OpenAI Codex | `generic` | `codex …` |
| Aider / scripts | `generic` | your command |
| Cursor | — (MCP) | `pytxo-mcp` tools |

## Next steps

- [First three-agent run](/docs/getting-started/first-three-agent-run)
- [pytxo.toml reference](/docs/reference/pytxo-toml)
- [CLI reference](/docs/reference/cli)
