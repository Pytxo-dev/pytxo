# Claude Code — Pytxo

Read [`AGENTS.md`](AGENTS.md) first for canonical project instructions.

Vault-local context and doc navigation: [`docs/00-meta/CLAUDE.md`](docs/00-meta/CLAUDE.md).

Architecture entry: [`docs/00-meta/MOC-home.md`](docs/00-meta/MOC-home.md).

## PlugDev

This project uses [PlugDev](https://github.com/mattbaconz/plugdev) for the Minecraft plugin test loop.

- Prefer `plug run` over manually starting Paper
- Run: `plug run` (or `plugdev run`) after `npm i -g @plugdev/cli` and `plugdev init --setup --agents --mcp`
- Doctor: `plug doctor` when detection or boot fails
- Console: type server commands in the PlugDev terminal after ready (RCON)
- Clean: `plug clean` / `plug clean --all`
- Modules: `plugdev module list|use` (multi-module reactors)
- Deps: `plugdev deps add|remove|list` (TUI Dependencies screen)
- Headless: `plugdev server start|stop|status|command|logs`
- If MCP is configured (`.mcp.json`), prefer `plugdev_*` tools for headless control
- Optional hotswap (`--hotswap`): method bodies only; falls back to safe reload

Do not use Bukkit `/reload`. On Folia, prefer full restart over safe reload.
