---
title: What is Pytxo?
---

# What is Pytxo?

Pytxo solves a different problem than cloud **agent development environments** (ADEs) that show walls of terminal panes in a browser tab.

## The problem

Cloud ADEs optimize for demos: many parallel terminal grids, heavy RAM and GPU use, proprietary credit models, and vendor lock-in. Developers who want **throughput** need coordination, isolation, and telemetry — not another IDE surface.

## The Pytxo model

Pytxo coordinates **headless terminal agents** in managed background PTYs. It does not replace your IDE; it plugs in via MCP and optional desktop telemetry.

| | Cloud ADE | Pytxo |
|---|-----------|-------|
| Primary UI | Multi-pane terminal grid | Your existing IDE |
| Execution | Remote containers | Local PTYs + isolated copies |
| Parallelism | Visual panes | DAG waves + registry |
| Telemetry | Streamed terminal text | Structural logs + optional Reality Deck |
| LLM keys | Often platform-held | BYOK always |

## Productivity layers

1. **Many projects at once** — independent execution domains per repo with separate logs and permission tiers (default **Orbit**). See [Execution domains](/docs/concepts/execution-domains).
2. **Modular projects** — one manifest spanning multiple folders (API + web + protos) in a single coordinated run. See [Modular projects](/docs/concepts/modular-projects).
3. **Fleet runs** — cross-repo DAGs when barriers must span separate git roots. See [Fleet runs](/docs/concepts/fleet-runs).

## Non-goals

- Multi-pane embedded terminal walls in the product UI
- Storing provider API keys in plaintext
- Becoming an LLM vendor (Pytxo Cloud is BYOK-only)

## Next

- [Three moats](/docs/concepts/three-moats) — Signal, Blast, and Race shields
- [Reality Deck](/docs/concepts/reality-deck) — optional telemetry UI
- [Galaxy approvals](/docs/concepts/galaxy-approvals) — human-in-the-loop for risky actions
