# Tutorial: Pytxo from Cursor via MCP

1. Build the MCP server: `cargo build -p pytxo-mcp --release`.
2. Configure Cursor per [[mcp-cursor-setup]].
3. In your repo, run `pytxo init` once (or let the first `pytxo_run` create `.pytxo/`).
4. Ask the agent to call `pytxo_dry_run` to preview waves.
5. Call `pytxo_run` with `{ "cmd": "echo hello" }`.
6. Use `pytxo_status` and `pytxo_logs` with `agent_id` from the run.

Secrets in command output are redacted in the WAL when `sanitize = true` (default).
