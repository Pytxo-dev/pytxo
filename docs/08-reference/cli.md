---
title: CLI reference
slug: cli-reference
status: active
tags: [reference, cli]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-27
related: [[pytxo-toml]], [[repository-layout]], [[mission-loop]], [[first-mission]]
---

# CLI reference

Build: `cargo build -p pytxo-cli`  
Binary: `cargo run -p pytxo-cli -- <cmd>`

## Hypervisor Shell (default)

`pytxo` with **no subcommand** opens the **Hypervisor Shell** (board + scrollback + operator prompt). Slash commands (`/run`, `/dry-run`, `/doctor`, …) route through `pytxo-shell`. Set `PYTXO_NO_TUI=1` or pass an explicit subcommand for scripts.

## Commands

| Command | Description |
|---------|-------------|
| `pytxo mission "…"` | Plan → approve → isolated run → mission report ([[mission-loop]], [[first-mission]]) |
| `pytxo init` | Create `.pytxo/` dirs and gitignore hint |
| `pytxo doctor` | Verify git, HEAD, worktree support, writable `.pytxo/`, PTY smoke |
| `pytxo run` | Schedule and execute agents in worktrees (`--dry-run` for plan JSON) |
| `pytxo status` | List runs and agents from SQLite |
| `pytxo logs --agent <id>` | Tail stored stdout/stderr events |
| `pytxo stop` | Stop active run or all tracked processes |
| `pytxo trust [tier]` | Trust repo folder (`orbit`, `galaxy`, `deep_space`, `supernova`) |
| `pytxo agents` | List ADE CLIs from the registry (PATH detection) |
| `pytxo shell --eval "…"` | One-shot shell slash command without TUI |
| `pytxo project` | Modular multi-path workspaces (`init`, `list`, `run`, …) |
| `pytxo fleet` | Cross-repo fleet DAG (`init`, `dry-run`, `run`, `status`) |
| `pytxo domains` | List execution domains |
| `pytxo hitl` | Galaxy approval queue (`list`, `approve`, `deny`) |
| `pytxo providers` | Provider registry + whether each API key env is set |
| `pytxo models` | `list`, `search`, or `refresh` model catalog per provider |

### `pytxo mission`

```bash
pytxo mission "Add input validation and update unit tests"
pytxo mission "…" --yes          # skip approve prompt
pytxo mission "…" --json         # plan only
pytxo mission "…" --ade claude
pytxo mission "…" --plan-file plan.json
```

## Shared flags

| Flag | Commands | Description |
|------|----------|-------------|
| `--repo` | most commands | Git repository root (default: cwd) |
| `--json` | `doctor`, `status`, `mission`, … | Machine-readable output |
| `--yes` | `mission` | Approve without interactive prompt |

See also: [[pytxo-toml]], [[first-mission]].

Back: [[MOC-home]]
