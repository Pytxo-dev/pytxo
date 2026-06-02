# v0.1.0

First public control-plane release for [Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).

## Highlights

- **CLI:** `init`, `doctor`, `run`, `status --json`, `logs`, `stop` with `--repo`
- **Git preflight:** `doctor` and `run` require a git repo with `HEAD`
- **Crates:** scheduler DAG (`depends_on`), runner worktrees, SQLite WAL store, log sanitization
- **MCP:** stdio tools for run/status/logs
- **Docs:** Obsidian vault + [repository layout](docs/08-reference/repository-layout.md)

## Reality Deck

The Tauri UI ships from [Pytxo-dev/pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop) and pins this tag for Rust dependencies.

## Install

```bash
cargo install --path crates/pytxo-cli
pytxo doctor
```

## Smoke

```powershell
.\scripts\smoke.ps1
```
