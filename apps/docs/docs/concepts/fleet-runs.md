---
title: Fleet runs
---

# Fleet runs

A **fleet run** coordinates agent work across **multiple git repositories** with explicit barrier sync — run repo A, wait until it finishes, then run repo B.

This is different from [modular projects](/docs/concepts/modular-projects), which coordinate multiple **folders** inside one project manifest and one `run_id`.

## At a glance

- Fleet manifest at `~/.pytxo/fleets/<id>.toml`
- Nodes are repos with their own `cmd`, `agents`, and optional `depends_on`
- Waves run in topological order; nodes in the same wave can run in parallel
- Progress is recorded in the hypervisor catalog and shown in Reality Deck

## Create a fleet

```bash
pytxo fleet init my-fleet \
  --add ~/dev/backend --cmd "echo backend" \
  --add ~/dev/frontend --cmd "echo frontend"
```

Preview the plan:

```bash
pytxo fleet dry-run --id my-fleet
```

## Run

```bash
pytxo fleet run --id my-fleet
```

Continue later waves even if one node fails:

```bash
pytxo fleet run --id my-fleet --continue-on-error
```

Check history:

```bash
pytxo fleet status --id my-fleet
```

## Reality Deck

The **Fleet runs** panel lists in-flight and recent fleet runs with per-node status across domains.

## MCP from Cursor

The MCP tools `pytxo_fleet_run` and `pytxo_fleet_status` let your IDE start or inspect fleet DAGs without leaving the editor. See [MCP from Cursor](/docs/getting-started/mcp-from-cursor).

## Example manifest

```toml
[fleet]
id = "release-train"
name = "Release train"

[[node]]
id = "backend"
repo = "/home/dev/backend"
cmd = "pytxo run --agents 2 --cmd 'npm test'"
agents = 1

[[node]]
id = "frontend"
repo = "/home/dev/frontend"
cmd = "pytxo run --agents 2 --cmd 'npm test'"
agents = 1
depends_on = ["backend"]
```

Back: [Execution domains](/docs/concepts/execution-domains)
