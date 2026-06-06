---
title: What is Pytxo?
---

# What is Pytxo?

Pytxo solves a different problem than BridgeSpace-style **agent development environments** (ADEs) that render walls of terminal panes inside web containers.

## The problem

Cloud ADEs optimize for demos: many parallel terminal grids in browser tabs, heavy RAM and GPU use, proprietary credit models, and vendor lock-in. Developers who want **throughput** need coordination, isolation, and telemetry — not another IDE surface.

## The Pytxo model

Pytxo coordinates **heterogeneous headless terminal agents** in managed background PTYs. It does not replace your IDE; it plugs in via MCP and optional desktop telemetry.

| | Cloud ADE | Pytxo |
|---|-----------|-------|
| Primary UI | Multi-pane terminal grid | Your existing IDE |
| Execution | Remote containers | Local PTYs + worktrees |
| Parallelism | Visual panes | DAG waves + registry |
| Telemetry | Streamed terminal text | Structural logs + optional Reality Deck |
| LLM keys | Often platform-held | BYOK always |

## Productivity layers

1. **Many projects at once** — independent execution domains per repo with separate WAL channels and permission profiles (default **Orbit**).
2. **Modular projects** (roadmap) — one manifest spanning multiple path roots (API + web + protos) in a single swarm.

## Non-goals

- Multi-pane embedded terminal walls in the product UI
- Storing provider API keys in plaintext
- Becoming an LLM vendor (Pytxo Cloud is BYOK-only)

## Next

- [Three moats](/docs/concepts/three-moats) — technical differentiation
- [Reality Deck](/docs/concepts/reality-deck) — optional telemetry UI
