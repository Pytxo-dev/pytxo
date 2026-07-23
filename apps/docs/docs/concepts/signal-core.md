---
title: Signal Core
---

# Signal Core

**Signal Core** compresses context before agents read your code. It sends **structural skeletons** (signatures, types, imports) instead of full file dumps. Savings depend on file shape — tiny fixtures may show single-digit percent; large body-heavy files approach the aspirational ~60% design goal. Measure with `tooling/benchmarks/signal-reduction.ps1` (Phase 73 pin: **4.76%** on `tiny-monorepo/src/a.ts`).

## At a glance

- **tree-sitter** parsing extracts structure on the read path (8+ languages)
- `signal_core = true` in `pytxo.toml` turns scaffolding on for task paths
- `signal_fidelity` (`low` | `medium` | `high`) controls how much structure per task
- Savings are recorded per run; Desktop 2 Focus shows an **arbitrage bar** (`agent_arbitrage`) when samples exist
- Import edges between files show up in Topology Focus during runs

## Configuration

```toml
signal_core = true
signal_fidelity = "low"   # default; per-task override in [[task]]
```

Signal fidelity is a **context tier**, separate from [permission tiers](/docs/reference/permission-tiers).

## MCP reads

Tools `pytxo_read` and `pytxo_read_scaffolded` return skeletons instead of raw files when Signal Core is on. Useful from Cursor without dumping a full checkout.

Back: [Three moats](/docs/concepts/three-moats) · [Pytxo Desktop](/docs/concepts/desktop)
