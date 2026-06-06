# Pytxo v0.1.0

First public release of the **Pytxo** agent hypervisor — local-first PTY orchestration, SQLite telemetry, and an interactive terminal dashboard.

## Highlights

- **CLI** — `init`, `doctor`, `run`, `status`, `logs`, `stop`, `project`, `hitl`, `domains`
- **Default TUI** — run `pytxo` with no subcommand for a live dashboard (doctor, runs, domains, HITL)
- **Three moats** — Signal Core scaffolding, Blast Shield worktrees, Race Shield scheduling
- **MCP** — `pytxo-mcp` for Cursor and IDE integration
- **Reality Deck** — optional Tauri desktop telemetry (`apps/desktop`)

## Install

```bash
npm i -g pytxo
pytxo doctor
```

Or use the install script:

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo/main/tooling/scripts/install.sh | bash
```

Advanced (from source):

```bash
cargo install --path crates/pytxo-cli
```

## Assets

Prebuilt binaries are attached to this release: `pytxo-linux-x64`, `pytxo-linux-arm64`, `pytxo-darwin-arm64`, `pytxo-darwin-x64`, `pytxo-windows-x64.exe`.

Verify with `SHA256SUMS.txt`.
