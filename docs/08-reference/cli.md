---
title: CLI reference
slug: cli-reference
status: active
tags: [reference, cli]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-11-08
related: [[pytxo-toml]], [[repository-layout]], [[mission-loop]], [[first-mission]]
---

# CLI reference

Build: `cargo build -p pytxo-cli`  
Binary: `cargo run -p pytxo-cli -- <cmd>`

## Default: the check

`pytxo` with **no subcommand** runs `pytxo check` ([[ADR-0043-check-is-the-default-command]]): it detects the agent branches, the target and the test command, merges the branches in a throwaway worktree and prints a three-line verdict. `pytxo land` is an alias. The released binaries build with `--no-default-features` and carry only `check`, `demo`, `hook`, `receipts` and `bench`.

Target: with no branches named, the default branch (`origin/HEAD`, else `main`, `master`, `trunk`, else the checked-out branch). Named branches are checked against the checked-out branch, as in 1.x, or the default branch when the checked-out branch is one of them or HEAD is detached. `--into <branch>`, `git config pytxo.into` or `PYTXO_INTO` override both.

| `git config` key | Environment variable | Default |
|------|------|------|
| `pytxo.branches` | `PYTXO_BRANCHES` | `claude/* codex/* copilot/* cursor/* devin/* jules/* agent/* worktrees` |
| `pytxo.check` | `PYTXO_CHECK` | found in the target branch |
| `pytxo.days` | `PYTXO_DAYS` | `14` |
| `pytxo.into` | `PYTXO_INTO` | the target above |

`--days <N>` beats `pytxo.days` and `PYTXO_DAYS` for one run, and `--days 0` looks at branches of any age. The verdict says how many branches it left out for age; merged ones are not counted.

With no agent branch to check, the message names the local branches that are open and recent but match no pattern, and a `next` line gives the command that checks them. `--json` lists them as `unmatched`. Branches only a remote has are not named.

While the tests run, each run is named on stderr as it starts and how it ended (`together... failed (41.3s)`), and in a terminal a long run gets a mark each minute. `--json` prints none of it.

A verdict about merge conflicts names every branch that conflicts, grouped by what each conflicts with. When none of the branches merges, the combination is the target itself and no test runs.

`pytxo demo` builds a throwaway repository with four agent branches and runs `--apply` there: two land, and the two that break each other stay out with the cause and a fix prompt. `--keep` keeps the repository.

## Hypervisor Shell (frozen, `full` builds)

`pytxo shell` opens the **Hypervisor Shell** (board + scrollback + operator prompt). Slash commands (`/run`, `/dry-run`, `/doctor`, …) route through `pytxo-shell`. The commands below need the default `full` feature.

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
