---
title: Competitive benchmarks narrative
slug: competitive-benchmarks
status: active
tags: [product, positioning]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-09-07
related: [[beyond-the-ade]], [[gtm-open-source-loop]], [[pytxo-improvement-research]], [[mvp-bootstrap]], [[pytxo-vs-github-copilot-app]], [[pytxo-vs-claude-agent-teams]]
---

# Competitive benchmarks narrative

Proof targets for GTM are **ADE terminal walls** and, increasingly, **vendor control centers / OS-sandboxed agents** (Copilot app worktrees, Codex Seatbelt/bwrap). Pytxo must win on measured RAM/isolation/Signal — not on “we also have multi-agent.” Program: [[pytxo-improvement-research]].

## Claims to validate with reproducible benchmarks

| Metric | Target narrative |
|--------|------------------|
| UI frame rate | Desktop 2 structural Focus (not terminal paint); legacy shell WAL poll target ~60 FPS |
| RAM | Presentation + orchestration bounded; compare vs 12-pane webview UIs |
| Swarm size | Core tier: 3 agents; Max: 10+ with cloud offload |
| Signal reduction | Measured on fixture via `signal-reduction` scripts — not an absolute marketing guarantee |

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

## Phase 39 — overlay vs worktree resource repro

Compare isolation backends on the same three-agent fixture (`tests/fixtures/tiny-monorepo`):

| Metric | Worktree | Overlay copy-layer (`overlay-fuse`) |
|--------|----------|-------------------------------------|
| Upper disk (agent write set) | ~180–220 MB (full clone × 3) | ~8–15 MB (changed paths only) |
| Cold start (fixture, est.) | ~2.4 s | ~1.1 s |
| Cloud delta payload | N/A (local PTY) | `delta_from_overlay_upper` file list (~12 files typical) |
| Race Shield contention | See `registry_contention_disjoint_claims` in `pytxo-runner` |

*Estimates from local Windows/Linux runs on `tiny-monorepo` (2026-06); treat as directional. Prefer Phase 73 pinned table below for cited numbers.*

Run overlay integration: `cargo test -p pytxo-runner overlay_layer_not_git_worktree --features overlay-fuse`

Run Race Shield contention probe: `cargo test -p pytxo-runner registry_contention`

## Phase 57 — measured methodology

Repro scripts (repo root, after `cargo build -p pytxo-cli`):

| Script | Measures |
|--------|----------|
| [`overlay-vs-worktree.sh`](../../../tooling/benchmarks/overlay-vs-worktree.sh) / [`.ps1`](../../../tooling/benchmarks/overlay-vs-worktree.ps1) | Upper-layer disk bytes + cold-start ms (worktree: 3× `git worktree add`; overlay: `pytxo run --dry-run` with `isolation = "overlay"`) on `tests/fixtures/tiny-monorepo` |
| [`multi-agent-ram.sh`](../../../tooling/benchmarks/multi-agent-ram.sh) / [`.ps1`](../../../tooling/benchmarks/multi-agent-ram.ps1) | Peak RSS (KB/MB) of `pytxo` parent during a 3-agent swarm |
| [`signal-reduction.ps1`](../../../tooling/benchmarks/signal-reduction.ps1) / [`.sh`](../../../tooling/benchmarks/signal-reduction.sh) | `token_reduction_pct` for `tests/fixtures/tiny-monorepo/src/a.ts` via `scaffold_report` |

**Pinned hardware profile** (Phase 73):

| Field | Value |
|-------|-------|
| Host | 83S0 (AMD Ryzen 7 7735HS with Radeon Graphics) |
| OS | Microsoft Windows 11 Home Single Language |
| Rust / Pytxo | rustc 1.95.0 / workspace `0.5.0` |
| Fixture | `tests/fixtures/tiny-monorepo` |
| Pin date | 2026-07-18 |

**Pinned results** (Phase 73 operator pin):

| Metric | Worktree | Overlay copy-layer | Notes |
|--------|----------|-------------------|-------|
| Cold start (ms) | 570 | 10473 | Worktree = 3× `git worktree add`; overlay = `pytxo init` + `trust` + `run --dry-run` (includes planner CLI overhead) |
| Upper disk | 534 B | 0 B | Overlay dry-run path does **not** materialize an upper layer — upper-disk delta **blocked** on this script path (use a full overlay flush run for write-set MB) |
| Peak RAM (MB) | 19 | — | `multi-agent-ram.ps1`; peak RSS of parent during 3-agent `ping` hold |
| UI frame rate (FPS) | N/A | — | Not measured this pin; Desktop 2 default is structural Focus list (no 60 FPS terminal paint claim) |
| Signal `token_reduction_pct` | 4.76% | — | `signal-reduction.ps1` on `src/a.ts` (21→20 bytes). Tiny fixture; **not** a ~60% guarantee — see [[signal-core]] |

CI runs overlay integration compile-only on Linux (`cargo test -p pytxo-runner overlay_layer`); full disk numbers remain operator-pinned on reference hardware. Do not cite unpinned or blocked cells in marketing.

## Honesty

Marketing copy must match measured results. Absolute “~60% token savings” is an **aspirational** Signal Core target; cite the Phase 73 pin (or re-run `signal-reduction`) for reproducible claims.

**Methodology caveats (Phase 73 pin):** overlay “cold start” includes `pytxo init` / trust / planner CLI overhead — not a pure FS mount timer. Overlay upper-disk shows **0 B** on the dry-run script path (no materialized upper layer); use a full overlay flush run for write-set MB. Do not cite blocked cells.

Category compares: [[pytxo-vs-claude-agent-teams]] (Claude’s own collision + token warnings), [[pytxo-vs-github-copilot-app]] (worktree control center), [[beyond-the-ade]].

## Phase 78 — real-monorepo candidate pin

The Phase 73 tiny fixture remains historical evidence. The release-candidate
benchmark now runs against the tracked production source in the real Pytxo
monorepo at `3acdc77`.

Raw evidence:

- [`signal-real-repo.json`](../../../tooling/benchmarks/results/signal-real-repo.json)
  (the dirty working-state capture used by the table below; the separately dated
  clean Signal record is a different corpus)
- [`2026-07-30-pytxo-control-plane.json`](../../../tooling/benchmarks/results/2026-07-30-pytxo-control-plane.json)

| Measurement | Result | Boundary |
|-------------|--------|----------|
| Signal corpus | 185 tracked production files; 1,253,675 input bytes | Rust, TypeScript, and JavaScript files ≥1 KiB; tests, fixtures, vendor, generated output, and dependencies excluded |
| Signal scaffold output | 213,847 bytes; **82.9% weighted reduction** | Structural-byte reduction, not tokenizer output or billable-token savings |
| Signal median file reduction | **86.3%** | 86.2898% before rounding; median across eligible files; file shape and language mix matter |
| Race preflight | 5 real-path tasks; 1 declared overlap; 2 waves; **0 same-wave path collisions** | Deterministic scheduler workload, not agent coding quality |
| Isolated echo run | **17,578 ms**, 5/5 agents exit 0, primary checkout changes **0** | Local control-plane/workspace overhead; zero model calls |

The real-repo execution initially exposed a Windows copy-layer recursion bug:
concurrent agent destinations under `.pytxo/worktrees` could copy sibling
destinations into one another and terminate with `0xC00000FD`. The candidate
fix excludes the shared workspace subtree during copy preparation and adds
Windows/generic regression coverage. The passing control-plane result above is
from the fixed working tree, so rerun and repin after the fix receives a commit
SHA.

**Not measured yet:** model task success, review quality, paid token cost,
competitor completion time, or voluntary repeat usage. Do not turn this pin
into a “Pytxo beats Codex/Claude” claim.

Related: [[pytxo-improvement-research]] · Phase 73 in [[mvp-bootstrap]].
