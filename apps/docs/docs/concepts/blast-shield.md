---
title: Blast Shield
---

# Blast Shield

**Blast Shield** isolates agent writes so parallel swarms do not corrupt shared files. Changes only land in your real tree after an explicit merge path.

## At a glance

| Mode | Description |
|------|-------------|
| `worktree` | Git worktree per agent (default, ready for daily use) |
| `overlay` | Copy-layer upper directory; flush copies approved changes into the physical tree |

## Behavior

- Each agent runs in an isolated copy under `.pytxo/worktrees/` (or an overlay upper layer)
- Overlapping `paths` in `pytxo.toml` tasks trigger **separate scheduling waves**
- Copies are removed after the run unless `--keep-worktrees` is set
- On **Galaxy**, merging back into your real tree needs [human approval](/docs/concepts/galaxy-approvals)

## Configuration

```toml
isolation = "worktree"   # default
# isolation = "overlay"  # copy-layer; enable overlay-fuse build on your platform
```

## Approve-to-merge

When an agent finishes in an isolated copy, changes are not applied to your checkout until you approve:

- **CLI / Desktop:** commit workspace / approve merge actions
- **Galaxy tier:** approval queue blocks flush until you confirm

Run `pytxo doctor` and check `overlay_isolation` if you use overlay mode.

Back: [Three moats](/docs/concepts/three-moats)
