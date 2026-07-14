---
sidebar_position: 2
title: Folder trust
---

# Folder trust

Before Pytxo dispatches agents in a repository, you choose whether to **trust the folder** and which **permission tier** applies. This mirrors VS Code’s “Do you trust this folder?” flow: preflight and planning work without trust; spawning real processes does not.

## First launch

Run `pytxo` in a repo. If the canonical git root is not trusted yet, a full-screen **tier picker** appears:

| Tier | Summary |
|------|---------|
| **DeepSpace** | Air-gapped bubble: minimal host access |
| **Orbit** | Default engineering (recommended) |
| **Galaxy** | Host tools + human-in-the-loop gates |
| **Supernova** | Full host privileges |

Choose **I trust this folder** to persist the tier at `~/.pytxo/trusted-domains.json` and open the Hypervisor Shell.

## What works while untrusted

| Action | Allowed |
|--------|---------|
| `/doctor`, `/dry-run`, `/status`, `/help` | Yes |
| `/models list`, `/models search` | Yes |
| `/run`, `pytxo run` (non–dry-run) | No |
| Natural-language mission dispatch | No |

## CLI

```bash
pytxo trust orbit          # trust cwd with Orbit tier
pytxo trust galaxy --repo /path/to/repo
```

In the shell:

```text
/trust orbit               # re-open or set tier
```

Trusted tier **overrides** the default `permission_profile` in `pytxo.toml` for that repo until you change it.

## Security notes

- Trust is keyed by **canonical repo root**, not the current working directory inside the tree.
- Secrets stay in environment variables. Pytxo never writes API keys to disk.
- See [Permission tiers](/docs/reference/permission-tiers) and [Galaxy approvals](/docs/concepts/galaxy-approvals) for what each tier allows.
