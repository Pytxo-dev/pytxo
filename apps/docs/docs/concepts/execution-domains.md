---
title: Execution domains
---

# Execution domains

An **execution domain** is an isolated orchestration scope — typically one git repository root — with its own:

- SQLite WAL channel under `.pytxo/data/`
- Worktree directory
- [Permission profile](/docs/reference/permission-tiers) (default **Orbit**)

## Multi-project workflows

Run independent swarms on different directories without shared scheduler or log state:

```text
/project-a  →  domain A  →  wave run A
/project-b  →  domain B  →  wave run B
```

Point `--repo` at each root, or set `PYTXO_REPO` for MCP sessions.

## Modular projects (roadmap)

A single Pytxo project manifest may span multiple path roots (API repo + frontend + protos) so one swarm coordinates across folders. See `pytxo.toml` `root` labels on tasks.

Back: [What is Pytxo?](/docs/concepts/what-is-pytxo)
