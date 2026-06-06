---
title: Install
---

# Install

## npm (recommended)

Requires Node.js 18+.

```bash
npm i -g pytxo
pytxo doctor
```

The npm package downloads the matching prebuilt binary from [GitHub Releases](https://github.com/Pytxo-dev/pytxo/releases) on install.

## Install script

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo/main/tooling/scripts/install.sh | bash
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Pytxo-dev/pytxo/main/tooling/scripts/install.ps1 | iex
```

## From source (developers)

```bash
git clone https://github.com/Pytxo-dev/pytxo.git
cd pytxo
cargo install --path crates/pytxo-cli
```

## Default TUI

Running `pytxo` with no subcommand opens the **terminal dashboard** (doctor summary, recent runs, HITL queue). Use explicit subcommands in scripts:

```bash
pytxo run --dry-run --config pytxo.toml
PYTXO_NO_TUI=1 pytxo   # print help instead of TUI
```

## Initialize a project

```bash
pytxo init
pytxo doctor
```

## Next steps

- [First three-agent run](/docs/getting-started/first-three-agent-run)
- [MCP from Cursor](/docs/getting-started/mcp-from-cursor)
