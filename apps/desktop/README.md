# Pytxo Reality Deck

Passive telemetry UI for [Pytxo](https://github.com/Pytxo-dev/pytxo): run list, wave agents, xterm log stream, and per-agent git diff.

> **Note:** During the monorepo transition this app lives under `apps/desktop/`. The canonical home is [Pytxo-dev/pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop).

## Prerequisites

- Rust 1.85+ ([rustup](https://rustup.rs))
- Node.js 22+
- [Pytxo CLI](https://github.com/Pytxo-dev/pytxo) built or installed (`cargo install --path crates/pytxo-cli`)

Open the UI from a **git repository** where you run agents (same cwd as `pytxo run`).

## Development

```bash
npm ci
npm run dev          # Vite (terminal 1)
cargo tauri dev      # from src-tauri, or cargo run -p pytxo-desktop from monorepo root
```

Monorepo:

```bash
cd apps/desktop
npm ci && npm run check
cd ../..
cargo build -p pytxo-desktop
```

## Build

```bash
npm run build
cargo build --release -p pytxo-desktop
```

## Architecture

- **Frontend:** Svelte 5 + xterm.js — no direct filesystem access.
- **IPC:** Tauri commands in `src-tauri/src/ipc.rs` call `pytxo-orchestrate` and `pytxo-store` only.

See [repository layout](https://github.com/Pytxo-dev/pytxo/blob/main/docs/08-reference/repository-layout.md).

## Screenshot

![Reality Deck overview](docs/reality-deck.png)

*(Add `docs/reality-deck.png` after capture; GIF optional for launch.)*
