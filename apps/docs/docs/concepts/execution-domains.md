---
title: Execution domains
---

# Execution domains

An **execution domain** is an isolated orchestration scope — typically one git repository root — with its own:

- SQLite telemetry under `.pytxo/data/`
- Isolated agent copies under `.pytxo/worktrees/`
- [Permission tier](/docs/reference/permission-tiers) (default **Orbit**)

## Multi-project workflows

Run independent swarms on different directories without shared scheduler or log state:

```text
/project-a  →  domain A  →  wave run A
/project-b  →  domain B  →  wave run B
```

Point `--repo` at each root, or set `PYTXO_REPO` for MCP sessions.

List registered domains:

```bash
pytxo domains
```

The hypervisor catalog remembers every domain the Deck or CLI has touched, so you can switch projects from one home screen.

## Modular projects

A single project manifest can span multiple path roots (API repo + frontend + protos) so one swarm coordinates across folders. See [Modular projects](/docs/concepts/modular-projects).

## Fleet runs

When repos are independent git roots with explicit barriers between them, use [Fleet runs](/docs/concepts/fleet-runs) instead of a single project manifest.

Back: [What is Pytxo?](/docs/concepts/what-is-pytxo)
