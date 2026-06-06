# MCP setup for Cursor

Pytxo ships a stdio MCP server: `pytxo-mcp`.

## Build

```bash
cargo build -p pytxo-mcp --release
```

Binary: `target/release/pytxo-mcp.exe` (Windows) or `target/release/pytxo-mcp` (Unix).

Build from [Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo) or your fork.

## Cursor configuration

Add to Cursor MCP settings (`.cursor/mcp.json` or **Settings → MCP**):

```json
{
  "mcpServers": {
    "pytxo": {
      "command": "C:\\path\\to\\pytxo\\target\\release\\pytxo-mcp.exe",
      "args": [],
      "env": {}
    }
  }
}
```

Run MCP from your repository root so `pytxo.toml` and `.pytxo/data` resolve correctly.

## Tools (v1)

| Tool | Description |
|------|-------------|
| `pytxo_dry_run` | Execution plan JSON |
| `pytxo_run` | Start a run; returns `run_id` |
| `pytxo_status` | Recent runs from WAL |
| `pytxo_logs` | Tail events for `agent_id` |
| `pytxo_read` | Read repo file; uses Signal Core when `signal_core = true` (pass `raw: true` to bypass) |
| `pytxo_read_scaffolded` | Always return AST skeleton (`path`, optional `fidelity`) |

Prefer **`pytxo_read`** for agent context — it respects `pytxo.toml` Signal Core settings.

See also: [cursor-mcp-pytxo](/docs/cursor-mcp-pytxo).
