---
title: Desktop setup
---

# Desktop setup

First-run guide for **Pytxo Desktop** — the optional control UI for structural telemetry and 3D AST topology (not a multi-terminal IDE).

## Prerequisites

- [Install the CLI](/docs/getting-started/install) (`pytxo doctor` should pass)
- Node.js 18+ and a Rust toolchain (for building from source)
- On Linux: Tauri system dependencies (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

## Build and launch (from source)

From the monorepo root:

```bash
cd apps/desktop
npm ci
npm run check
cd ../..
cargo build -p pytxo-desktop
```

Dev loop (Vite + Tauri):

```bash
cd apps/desktop
npx --yes @tauri-apps/cli@2 dev
```

## First session

1. Trust a folder: `pytxo trust orbit` (or pick a profile in the Desktop trust flow).
2. Initialize config in the repo: `pytxo init`.
3. Open that repo in Desktop — Workspace Home lists projects/domains.
4. Run a dry swarm: `pytxo run --dry-run --config pytxo.toml`, then a real three-agent fixture when ready.

## What you should see

- **Topology** — AST / dependency graph (not a wall of terminals)
- **Diff / logs** — supporting panels for agent writes and WAL events
- **Galaxy approvals** — HITL panel when `permission_profile = galaxy`

## Next steps

- [Folder trust](/docs/getting-started/folder-trust)
- [First three-agent run](/docs/getting-started/first-three-agent-run)
- [Hypervisor shell](/docs/getting-started/hypervisor-shell)
- [MCP from Cursor](/docs/getting-started/mcp-from-cursor)
