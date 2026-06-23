# Pytxo Rust crates

Control plane for the [agent hypervisor](https://github.com/Pytxo-dev/pytxo/blob/main/docs/06-product/vision.md). UI: [`apps/desktop`](../apps/desktop/).

| Crate | Role | Moat (target / partial) |
|-------|------|-------------------------|
| **pytxo-core** | Config, types, errors, path helpers, moat traits | **target:** `PermissionProfile`, `PermissionEngine` in `moat::permission` |
| **pytxo-scheduler** | Wave / DAG scheduling | Race Shield (planning) |
| **pytxo-runner** | Git worktrees, PTY spawn, PID registry | Blast Shield (worktrees MVP), Race Shield (registry) |
| **pytxo-store** | SQLite WAL telemetry | — |
| **pytxo-signal** | `tree-sitter` AST skeletons on read | Signal Core |
| **pytxo-sanitize** | Log redaction | Sovereign Shield |
| **pytxo-orchestrate** | `run` / `stop` / `status` / `doctor` | Orchestration façade; `HypervisorRegistry`, multi-domain dispatch |
| **pytxo-shell** | Slash command router, `ShellSession` | Hypervisor Shell control plane (TUI, future MCP) |
| **pytxo-planner** | Optional NL mission → `Vec<Task>` | Feature-gated (`PYTXO_PLANNER=1`, `[planner] enabled`) |
| **pytxo-tui** | Default `pytxo` Hypervisor Shell (ratatui) | Board + scrollback + prompt |
| **pytxo-cli** | `pytxo` binary | — |
| **pytxo-mcp** | stdio MCP server | — |

**Planned / partial:** Blast Shield kernel sparse overlay (FUSE/ProjFS) — worktree + copy-layer MVP shipped; Race Shield PTY stdin pump shipped (subprocess mode single-drain opt-in).

Dependency flow: `core` → `signal` | `scheduler` | `store` | `sanitize` → `runner` → `orchestrate` → `cli` | `mcp`.

See [repository-layout](../docs/08-reference/repository-layout.md).
