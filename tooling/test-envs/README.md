# Pytxo ADE test environments

Scaffolded git repos for manual and scripted ADE smoke tests. Pytxo is the hypervisor — you bring the terminal agent CLI.

## Quick start

```powershell
# Windows — from repo root
.\tooling\test-envs\scaffold.ps1 -Profile echo
cd $env:PYTXO_TEST_REPO
..\recipes\smoke-echo.ps1
```

```bash
# macOS / Linux
./tooling/test-envs/scaffold.sh echo
cd "$PYTXO_TEST_REPO"
../recipes/smoke-echo.sh
```

## Profiles

| Profile | Config | Purpose |
|---------|--------|---------|
| `echo` | `pytxo.echo.toml` | Scheduling + WAL, no API keys |
| `context` | `pytxo.echo.toml` + stub agent | Signal Core env contract |
| `antigravity` | `pytxo.antigravity.toml` | Antigravity CLI (`agy`) |
| `claude` | `pytxo.claude.toml` | Claude Code |
| `codex` | `pytxo.codex.toml` | OpenAI Codex CLI |
| `cursor` | `pytxo.cursor.toml` | Cursor Agent CLI (`cursor agent`) |
| `opencode` | `pytxo.opencode.toml` | OpenCode CLI |

## Environment variables

| Variable | Description |
|----------|-------------|
| `PYTXO_BIN` | Path to `pytxo` binary — **set automatically by scaffold**; recipes resolve npm global, PATH, or `target/release/pytxo.exe` |
| `PYTXO_TEST_REPO` | Set by scaffold to the temp repo path |

## ADE compatibility

| Tier | Agent | Integration |
|------|-------|-------------|
| A | Antigravity (`agy`) | `pytxo run --cmd "agy …"`, `cli_adapter = "agy"` |
| A | Claude Code | `cli_adapter = "claude_code"` |
| A | OpenAI Codex | `cli_adapter = "codex"` or `/run --ade codex` |
| A | Cursor Agent | `cli_adapter = "cursor"` or `/run --ade cursor` (spawn via `--cmd`) |
| A | OpenCode | `cli_adapter = "opencode"` or `/use opencode` |
| A | Aider / scripts | `cli_adapter = "aider"` or any `--cmd` |
| B | Cursor IDE | MCP (`pytxo-mcp`) — inverse direction; IDE hosts Pytxo tools |

## Recipes

| Script | Description |
|--------|-------------|
| `recipes/smoke-echo.ps1` / `.sh` | Echo agents, dry-run + run + status |
| `recipes/shell-smoke.sh` | CLI control-plane smoke (doctor, dry-run, run, status) |
| `shell-smoke.sh` | CI entry: scaffold echo + `shell-smoke` recipe |
| `recipes/smoke-context.ps1` / `.sh` | Stub agent reads `PYTXO_CONTEXT_DIR` |
| `recipes/live-antigravity.ps1` | Requires `agy` on PATH |

## Recommended test order (agy)

1. `npm i -g pytxo` → `pytxo doctor`
2. `scaffold.ps1 -Profile echo` → `smoke-echo`
3. `scaffold.ps1 -Profile context` → `smoke-context`
4. `agy --version` → `live-antigravity.ps1`
5. `pytxo` TUI to watch runs

See [docs: Testing ADE CLIs](https://pytxo.com/docs/getting-started/testing-ade-clis).
