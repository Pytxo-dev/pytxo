---
title: Signal Core
---

# Signal Core

**Signal Core** is Pytxo's context arbitrage layer. The goal: send agents **structural skeletons** of code (signatures, types, imports) instead of full file dumps — targeting up to ~60% lower input tokens on large repos.

## Today

- `signal_core = true` in `pytxo.toml` materializes scaffolded context for task paths
- `signal_fidelity` (`low` | `medium` | `high`) controls how much structure is included per task or globally

## Roadmap

- `tree-sitter` AST extraction on file read
- Checkpoint compression protocol for long sessions

## Configuration

```toml
signal_core = true
signal_fidelity = "low"   # default; per-task override in [[task]]
```

Signal fidelity is a **context tier**, separate from [permission tiers](/docs/reference/permission-tiers).

Back: [Three moats](/docs/concepts/three-moats)
