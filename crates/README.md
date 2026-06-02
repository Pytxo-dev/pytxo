# Pytxo Rust crates

| Crate | Role |
|-------|------|
| **pytxo-core** | Config, types, errors, path helpers |
| **pytxo-scheduler** | Wave / DAG scheduling (path overlap + `depends_on`) |
| **pytxo-runner** | Git worktrees, process spawn, PID registry |
| **pytxo-store** | SQLite WAL (`runs`, `agents`, `events`) |
| **pytxo-sanitize** | Log redaction (Sovereign Shield) |
| **pytxo-orchestrate** | Shared `run` / `stop` / `status` / `doctor` |
| **pytxo-cli** | `pytxo` binary |
| **pytxo-mcp** | stdio MCP server |

Dependency flow: `core` → `scheduler` | `store` | `sanitize` → `runner` → `orchestrate` → `cli` | `mcp`.

See [repository-layout](../../docs/08-reference/repository-layout.md).
