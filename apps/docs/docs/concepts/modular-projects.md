---
title: Modular projects
---

# Modular projects

A **modular project** lets you run one coordinated swarm across **multiple folders** — for example an API repo, a web app, and a read-only protos package — without merging them into a single git root.

## At a glance

- One project manifest under `~/.pytxo/projects/<id>.toml`
- Each folder is a **root** with a short label (`api`, `web`, `protos`)
- Tasks in `pytxo.toml` can set `root = "web"` to run on that folder
- Read-only roots contribute context but never receive writes
- Each writable root can use its own [permission tier](/docs/reference/permission-tiers)

## Create a project

```bash
pytxo project init my-platform \
  --add ~/dev/api \
  --add ~/dev/web \
  --add ~/dev/protos
```

List roots:

```bash
pytxo project list --id my-platform
```

## Run across roots

```bash
pytxo project run --id my-platform --cmd "echo pytxo" --agents 3
```

One `run_id` spans all writable roots. The scheduler warns when two roots claim overlapping paths in the same wave.

## Reality Deck

When a project is selected in the Deck sidebar, you see:

- A **path panel** listing each root, primary badge, read-only badge, and permission tier
- **Root filters** on agents and topology nodes

## Configuration

In the project manifest:

```toml
[project]
id = "my-platform"
name = "My Platform"

[[roots]]
path = "/home/dev/api"
label = "api"
primary = true

[[roots]]
path = "/home/dev/web"
label = "web"

[[roots]]
path = "/home/dev/protos"
label = "protos"
read_only = true
permission_profile = "deep_space"
```

Per-root `permission_profile` overrides the repo default for agents executing on that root.

## Fleet vs modular project

| | Modular project | Fleet run |
|---|-----------------|-----------|
| Scope | Multiple folders, **one** coordinated run | Multiple **repos**, barrier sync between nodes |
| Command | `pytxo project run` | `pytxo fleet run` |
| Use when | Monorepo or sibling folders in one workflow | Independent repos with explicit `depends_on` |

See [Fleet runs](/docs/concepts/fleet-runs).

Back: [Execution domains](/docs/concepts/execution-domains)
