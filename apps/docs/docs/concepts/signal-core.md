---
title: Signal Core
---

# Signal Core

**Signal Core** is Pytxo's context compression layer. It sends agents **structural skeletons** of code — signatures, types, imports — instead of full file dumps, often cutting input tokens sharply on large repos.

## At a glance

- **tree-sitter** parsing extracts structure on the read path
- `signal_core = true` in `pytxo.toml` enables scaffolding for task paths
- `signal_fidelity` (`low` | `medium` | `high`) controls how much structure per task
- Savings are recorded per run and shown in Reality Deck topology stats
- **Structural graph** — import edges between edited files appear in the Deck topology panel during runs

## Configuration

```toml
signal_core = true
signal_fidelity = "low"   # default; per-task override in [[task]]
```

Signal fidelity is a **context tier**, separate from [permission tiers](/docs/reference/permission-tiers).

## MCP reads

Tools `pytxo_read` and `pytxo_read_scaffolded` return skeletons instead of raw files when Signal Core is enabled — useful from Cursor without a full checkout dump.

Back: [Three moats](/docs/concepts/three-moats) · [Reality Deck](/docs/concepts/reality-deck)
