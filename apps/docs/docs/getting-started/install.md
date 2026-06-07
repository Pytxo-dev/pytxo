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

The npm package downloads the matching prebuilt binary from the public [pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases) repository on install.

## Install script

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.ps1 | iex
```

Pin a version:

```bash
PYTXO_VERSION=v0.3.0 curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
```

## Prebuilt binaries

Download platform assets directly from [GitHub Releases](https://github.com/Pytxo-dev/pytxo-releases/releases) (`pytxo-linux-x64`, `pytxo-darwin-arm64`, `pytxo-windows-x64.exe`, …).

## Default TUI

Running `pytxo` with no subcommand opens the **Hypervisor Shell** (trust picker, board, scrollback). Use explicit subcommands in scripts:

```bash
pytxo run --dry-run --config pytxo.toml
PYTXO_NO_TUI=1 pytxo   # print help instead of TUI
```

## Initialize a project

```bash
pytxo trust orbit
pytxo init
pytxo doctor
```

## Next steps

- [Folder trust](/docs/getting-started/folder-trust)
- [First three-agent run](/docs/getting-started/first-three-agent-run)
- [MCP from Cursor](/docs/getting-started/mcp-from-cursor)
