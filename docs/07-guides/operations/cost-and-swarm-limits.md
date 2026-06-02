---
title: Cost and swarm limits
slug: cost-and-swarm-limits
status: active
tags: [guides, operations, tokens]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[tiers-hobbyist-pro-max]], [[sqlite-wal-logging]], [[pytxo-vs-claude-agent-teams]]
---

# Cost and swarm limits

Parallel agents multiply **LLM provider cost**. Community feedback (HN, r/ClaudeCode) consistently flags economics as the barrier to large swarms—not just CPU.

## Pytxo responses

| Mechanism | Effect |
|-----------|--------|
| **Core tier 3-agent cap** | Free OSS adoption without unbounded parallel spend |
| **[[adaptive-semantic-scaffolding]]** | Start low fidelity; expand only on failure |
| **[[token-arbitrage]]** (Pro) | Cache repeated semantic signatures in cloud |
| **[[hybrid-execution]]** (Max) | Offload compile/test; local machine idle |
| **WAL metrics** ([[sqlite-wal-logging]]) | Token usage visible per agent/run for budgeting |

## Operator practices

1. Set max concurrent agents per repo profile.
2. Review WAL token reports before scaling swarm size.
3. Prefer DAG parallelism only where dependencies allow—avoid duplicate exploration agents on the same files.

## Honest limits

Pytxo does **not** reduce per-token list prices. It reduces **waste** (duplicate context, RAM thrash, failed lock retries) and offers **visibility**.

Compare native swarms: [[pytxo-vs-claude-agent-teams]].

Tier reference: [[tiers-hobbyist-pro-max]].
