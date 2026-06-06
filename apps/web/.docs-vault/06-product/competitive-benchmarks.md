---
title: Competitive benchmarks narrative
slug: competitive-benchmarks
status: active
tags: [product, positioning]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[beyond-the-ade]], [[gtm-open-source-loop]]
---

# Competitive benchmarks narrative

## Claims to validate with reproducible benchmarks

| Metric | Target narrative |
|--------|------------------|
| UI frame rate | ~60 FPS terminal paint under multi-agent load |
| RAM | Presentation + orchestration bounded; compare vs 12-pane webview UIs |
| Swarm size | Core tier: 3 agents; Max: 10+ with cloud offload |

## Phase 1 repro (implemented)

From repo root after `cargo build -p pytxo-cli`:

**Bash:** [`tooling/benchmarks/three-agent-preflight.sh`](../../../tooling/benchmarks/three-agent-preflight.sh)

**PowerShell:** [`tooling/benchmarks/three-agent-preflight.ps1`](../../../tooling/benchmarks/three-agent-preflight.ps1)

Expected:

1. `--dry-run` shows `task-b` and `task-c` in **separate waves** (both use `package.json`).
2. Full run completes without simultaneous writers on `package.json`.
3. `pytxo status` lists agents with exit code `0`.

## Phase 2 repro

**PowerShell:** [`tooling/benchmarks/phase2-demo.ps1`](../../../tooling/benchmarks/phase2-demo.ps1)

Covers:

1. `pytxo status --json` with optional cost fields after a run.
2. Sanitize unit tests (`cargo test -p pytxo-sanitize`).
3. DAG `depends_on` ordering (`cargo test -p pytxo-scheduler dependency_orders`).
4. Signal Core reduction ([`tooling/benchmarks/signal-reduction.ps1`](../../../tooling/benchmarks/signal-reduction.ps1) / [`.sh`](../../../tooling/benchmarks/signal-reduction.sh)).
5. Desktop compile (`cd apps/desktop && npm ci`, then `cargo build -p pytxo-desktop` from repo root).

Reality Deck terminal pane polls WAL at ~4 Hz; tune toward 60 FPS in Phase 2.1.

## Methodology (future)

- Fixed hardware profile documented per run
- Same repo / task graph for Pytxo vs baseline tool

## Honesty

Marketing copy must match measured results. See [[pytxo-vs-claude-agent-teams]] for when native agent teams are “good enough” without Pytxo.

Related: [[beyond-the-ade]].
