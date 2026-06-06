---
title: Blast Shield
---

# Blast Shield

**Blast Shield** isolates agent writes so parallel swarms do not corrupt shared files without an explicit merge path.

## Mechanisms

| Mode | Description |
|------|-------------|
| `worktree` | Git worktrees per agent (default, production-ready) |
| `overlay` | Memory-mapped virtual FS overlay (POC; requires `overlay-fuse` feature) |

## Behavior

- Each agent runs in an isolated worktree under `.pytxo/worktrees/`
- Overlapping `paths` in `pytxo.toml` tasks trigger **separate scheduling waves**
- Worktrees are removed after run unless `--keep-worktrees` is set

## Configuration

```toml
isolation = "worktree"
```

Target: sub-5ms rollback with disk flush only on approve (overlay roadmap).

Back: [Three moats](/docs/concepts/three-moats)
