---
title: Signal Core (context arbitrage)
slug: signal-core
status: active
tags: [orchestration, context, tree-sitter, moat]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-07-30
related: [[adaptive-semantic-scaffolding]], [[product-vision]], [[closed-loop-fidelity]], [[competitive-benchmarks]], [[pytxo-improvement-research]]
---

# Signal Core (context arbitrage)

**Signal Core** is Pytxo’s local **`tree-sitter` AST compiler** for agent context. It intercepts file reads and emits **structural skeletons** — signatures, types, imports, module shape — instead of full source dumps.

## Goal

Cut agent **input token burn** without hiding dependency structure. An **aspirational** target of up to ~60% on large, body-heavy files remains a design goal — not a guaranteed marketing number.

**Measured (Phase 78 real-repo pin):** `tooling/benchmarks/real-repo-signal.ps1`
reported **82.9% weighted scaffold-byte reduction** across 185 tracked
production files in Pytxo (1,253,675→213,847 bytes). The older Phase 73 tiny
fixture remains reproducible at 4.76%, but is no longer the representative
public proof point.

The measurement is a structural-byte proxy. It does **not** establish billable
model-token savings, task success, cost reduction, or fewer retries. Cite
[[competitive-benchmarks]] for the corpus and raw evidence. Desktop 2 Focus
shows per-run `agent_arbitrage` saved-token totals only when real samples exist.

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
- **Shipping:** `tree-sitter` skeletons for **8+** grammars (Rust, TS/JS, Python, Go, Java, C/C++, Ruby), MCP read tools, closed-loop **high-fidelity retry** on agent failure in `pytxo-runner`

Do not add parallel “stripper” utilities outside this module.
