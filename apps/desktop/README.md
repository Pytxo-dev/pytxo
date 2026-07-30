# Pytxo Desktop

Passive telemetry UI for Pytxo: run list, wave agents, xterm log stream, and per-agent git diff.

**Location:** `apps/desktop` in the [pytxo](https://github.com/Pytxo-dev/pytxo) monorepo.

**Requires:** in-tree `pytxo` crates (`pytxo-core`, `pytxo-store`, `pytxo-orchestrate` v0.1.x) via path dependencies.

## Prerequisites

- Node.js 22+
- Rust 1.85+ (repo root workspace)
- Tauri system deps on Linux (see root CI workflow)

Run the UI from the **same git repository** where you execute `pytxo run`.

## Build

From repository root:

```bash
cd apps/desktop
npm ci
npm run check
npm run build:native
```

The self-contained executable is written to the root workspace's
`target/release` directory.

## Architecture

- Tauri IPC calls `pytxo-orchestrate` and `pytxo-store` only.
- Presentation layer has no direct filesystem access (ADR-0001).

Export artifacts and release staging: see [`../desktop-export/README.md`](../desktop-export/README.md).
