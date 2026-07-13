---
title: Signal Core
---

# Signal Core

**Signal Core** compresses context before agents read your code. It sends **structural skeletons** (signatures, types, imports) instead of full file dumps. On large repos that often cuts input tokens a lot.

## At a glance

- **tree-sitter** parsing extracts structure on the read path
- `signal_core = true` in `pytxo.toml` turns scaffolding on for task paths
- `signal_fidelity` (`low` | `medium` | `high`) controls how much structure per task
- Savings are recorded per run and shown in Pytxo Desktop topology stats
- Import edges between edited files show up in the Desktop topology panel during runs

## Configuration

```toml
signal_core = true
signal_fidelity = "low"   # default; per-task override in [[task]]
```

Signal fidelity is a **context tier**, separate from [permission tiers](/docs/reference/permission-tiers).

## MCP reads

Tools `pytxo_read` and `pytxo_read_scaffolded` return skeletons instead of raw files when Signal Core is on. Useful from Cursor without dumping a full checkout.

Back: [Three moats](/docs/concepts/three-moats) · [Pytxo Desktop](/docs/concepts/desktop)
