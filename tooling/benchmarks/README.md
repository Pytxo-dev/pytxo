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
- [`overlay-vs-worktree.sh`](overlay-vs-worktree.sh) / [`.ps1`](overlay-vs-worktree.ps1) — disk + cold-start (Phase 57)
- [`multi-agent-ram.sh`](multi-agent-ram.sh) / [`.ps1`](multi-agent-ram.ps1) — peak RSS during 3-agent run (Phase 57; Windows `.ps1` uses `ProcessStartInfo` so `--cmd` values with spaces are not re-split)
- [`race-blast-collision.ps1`](race-blast-collision.ps1) — same-repo path collision proof: Race Shield wave separation vs naive parallel, Blast Shield CoW gate
- [`cli-cold-start.ps1`](cli-cold-start.ps1) — wall times for `pytxo --help`, `status`, and `doctor --quick`
- [`real-repo-signal.ps1`](real-repo-signal.ps1) — Signal Core weighted reduction across tracked production source in a real repository
- [`real-repo-control-plane.ps1`](real-repo-control-plane.ps1) — real-monorepo Race/Blast scheduler and isolated echo-run overhead, with raw JSON evidence

Desktop snapshot polling (IPC): Desktop 2 uses batched `load_desktop_snapshot` with adaptive intervals (≈1s on active Operations, ≈4s idle, paused when unfocused).

Requires: `git`, built `pytxo` binary (`cargo build -p pytxo-cli`).

**Phase 73 pin:** see [[competitive-benchmarks]] (2026-07-18 Windows host).
