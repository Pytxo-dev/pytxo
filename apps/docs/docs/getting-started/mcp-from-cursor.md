---
title: MCP from Cursor
---

# MCP from Cursor

Pytxo exposes orchestration through a **local MCP server** so Cursor (or any MCP-capable IDE) can start runs, inspect status, and tail logs without replacing your editor.

## Build the MCP server

```bash
cargo build -p pytxo-mcp --release
```

## Configure Cursor

Add a local MCP server entry in Cursor settings (`.cursor/mcp.json` or global MCP config):

```json
{
  "mcpServers": {
    "pytxo": {
      "command": "/path/to/target/release/pytxo-mcp",
      "args": [],
      "env": {
        "PYTXO_REPO": "/path/to/your/project"
      }
    }
  }
}
```

Adjust paths for your OS. `PYTXO_REPO` should point at the git root where `pytxo.toml` lives.

## What MCP enables

- Trigger `pytxo run` and preflight from the agent context
- Query run status and agent topology
- Tail structured logs without opening a separate terminal grid

Pytxo remains **local-first**: the MCP process talks to the Rust orchestrator on your machine. No cloud workspace required.

## Troubleshooting

1. Run `pytxo doctor` in the target repo before MCP-driven runs.
2. Ensure the MCP binary matches the CLI version you built from the same commit.
3. Check `.pytxo/data/` is writable.

See [CLI reference](/docs/reference/cli) for flags your MCP tools may wrap.
