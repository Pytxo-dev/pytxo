# Pytxo benchmarks (Phase 1)

## Metrics (target)

| Metric | Description |
|--------|-------------|
| `preflight_conflicts_detected` | Conflicting tasks placed in separate waves |
| `wasted_parallel_edits` | Tasks that would edit same path in same wave (should be 0) |
| `run_wall_time` | End-to-end `pytxo run` duration |

## Scripts

- [`three-agent-preflight.sh`](three-agent-preflight.sh) — bash
- [`three-agent-preflight.ps1`](three-agent-preflight.ps1) — PowerShell

Requires: `git`, built `pytxo` binary (`cargo build -p pytxo-cli`).
