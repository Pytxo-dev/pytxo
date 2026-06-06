---
title: Signal Core (context arbitrage)
slug: signal-core
status: active
tags: [orchestration, context, tree-sitter, moat]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[adaptive-semantic-scaffolding]], [[product-vision]], [[closed-loop-fidelity]]
---

# Signal Core (context arbitrage)

**Signal Core** is Pytxo’s local **`tree-sitter` AST compiler** for agent context. It intercepts file reads and emits **structural skeletons** — signatures, types, imports, module shape — instead of full source dumps.

## Goal

Cut agent **input token burn by up to ~60%** without hiding dependency structure.

## Behavior

| Stage | Output |
|-------|--------|
| Parse | Language-aware AST via `tree-sitter` |
| Strip | Remove bodies and noise; keep contracts and topology |
| Tier | Route fidelity by agent role ([[adaptive-semantic-scaffolding]]) |
| Escalate | On compile/test failure, [[closed-loop-fidelity]] raises fidelity for failing nodes only |

## Integration point

```text
Agent read path → Signal Core → PTY / MCP payload
```

Signal Core must run **before** context leaves the machine (local or cloud cache). Never default to “read whole file” when skeleton mode is available.

## Status

- **Designed:** fidelity tiers documented in [[adaptive-semantic-scaffolding]]
- **Shipping:** `tree-sitter` skeletons, MCP read tools, closed-loop **high-fidelity retry** on agent failure in `pytxo-runner`

Do not add parallel “stripper” utilities outside this module.
