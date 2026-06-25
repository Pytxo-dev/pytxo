---
title: Competitive benchmarks narrative
slug: competitive-benchmarks
status: active
tags: [product, positioning]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-24
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

## Phase 39 — overlay vs worktree resource repro

Compare isolation backends on the same three-agent fixture (`tests/fixtures/tiny-monorepo`):

| Metric | Worktree | Overlay copy-layer (`overlay-fuse`) |
|--------|----------|-------------------------------------|
| Upper disk (agent write set) | ~180–220 MB (full clone × 3) | ~8–15 MB (changed paths only) |
| Cold start (fixture, est.) | ~2.4 s | ~1.1 s |
| Cloud delta payload | N/A (local PTY) | `delta_from_overlay_upper` file list (~12 files typical) |
| Race Shield contention | See `registry_contention_disjoint_claims` in `pytxo-runner` |

*Estimates from local Windows/Linux runs on `tiny-monorepo` (2026-06); treat as directional until pinned in CI hardware profile.*

Run overlay integration: `cargo test -p pytxo-runner overlay_layer_not_git_worktree --features overlay-fuse`

Run Race Shield contention probe: `cargo test -p pytxo-runner registry_contention`

## Phase 57 — measured methodology

Repro scripts (repo root, after `cargo build -p pytxo-cli`):

| Script | Measures |
|--------|----------|
| [`overlay-vs-worktree.sh`](../../../tooling/benchmarks/overlay-vs-worktree.sh) / [`.ps1`](../../../tooling/benchmarks/overlay-vs-worktree.ps1) | Upper-layer disk bytes + cold-start ms (worktree: 3× `git worktree add`; overlay: `pytxo run --dry-run` with `isolation = "overlay"`) on `tests/fixtures/tiny-monorepo` |
| [`multi-agent-ram.sh`](../../../tooling/benchmarks/multi-agent-ram.sh) / [`.ps1`](../../../tooling/benchmarks/multi-agent-ram.ps1) | Peak RSS (KB/MB) of `pytxo` parent during a 3-agent `sleep` swarm |

**Pinned hardware profile** (fill after each release pin):

| Field | Value |
|-------|-------|
| Host | _e.g. AMD Ryzen 7 7840U, 32 GB RAM_ |
| OS | _e.g. Ubuntu 24.04 / Windows 11 26200_ |
| Rust / Pytxo | _e.g. 1.85 / 0.3.3_ |
| Fixture | `tests/fixtures/tiny-monorepo` |
| Date | _YYYY-MM-DD_ |

**Pinned results** (replace `TBD` after local or CI run):

| Metric | Worktree | Overlay copy-layer | Notes |
|--------|----------|-------------------|-------|
| Cold start (ms) | TBD | TBD | 3-agent fixture |
| Upper disk (MB) | TBD | TBD | `.pytxo/worktrees` after run |
| Peak RAM (MB) | TBD | — | 3-agent `multi-agent-ram` |
| UI frame rate (FPS) | TBD | — | Reality Deck terminal pane |

CI runs overlay integration compile-only on Linux (`cargo test -p pytxo-runner --features overlay-fuse overlay_layer`); full disk numbers remain operator-pinned on reference hardware.

## Honesty

Marketing copy must match measured results. See [[pytxo-vs-claude-agent-teams]] for when native agent teams are “good enough” without Pytxo.

Related: [[beyond-the-ade]].
