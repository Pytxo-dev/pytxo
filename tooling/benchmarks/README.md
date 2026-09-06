# Pytxo benchmarks (Phase 1)

## Current Beta evidence

[`beta-single-codex-2026-09-06.json`](results/beta-single-codex-2026-09-06.json)
records one actual natural-language mission using Codex 0.153.4 and the current
uncommitted Beta candidate-verification changes. One worker changed three owned
files, passed task checks, and produced a version 3 package with a separate
passing `npm test` on the frozen combined source. The primary tree remained
unchanged pending reviewed Apply. Start to candidate verification was 177.327
seconds on this host; this includes model execution and is not a benchmark of
Pytxo overhead. A subsequent real Tauri Review/Apply recorded a committed journal;
the three resulting hashes matched the package and four primary-tree tests passed.

The JSON binds the observed CLI binary hash, run ID and package digest. It does
not contain raw prompts, process logs, credentials or absolute repository paths.
Token cost was unavailable and remains null. The `mission-loop/` solo/manual/
Pytxo comparison sheets are unmeasured scaffolds, not comparative results.
Do not use this single observation to claim faster work or a reliability rate.

The [direct-Codex observation](results/beta-solo-codex-2026-09-06.json) completed
the same application change with only the three requested files modified and
four independently rerun tests passing. Its observed process duration was
127.634 seconds. These runs occurred at different times with background builds;
the Pytxo timing ends at candidate verification while the direct timing ends at
CLI exit. Their difference is **not** a measurement of orchestration overhead.

| Observed outcome | Direct Codex | Codex through Pytxo |
|---|---|---|
| Requested three-file change | Completed | Completed |
| Independent primary tests after work/Apply | 4 passed | 4 passed |
| Primary changed by worker before Pytxo review | Yes | No |
| Frozen v3 package with combined-check receipt | Not part of this path | Recorded |
| Native explicit Apply and committed journal | Not part of this path | Recorded |

This is a two-run case study. It supports neither automatic quality improvement
nor a speed win. The additional Pytxo evidence is an inspectable, verified commit
boundary. The final bookkeeping correction was separately exercised by the
deterministic run recorded in `target/beta-final-demo-status.json`, including
persisted verification events and no evidence gaps.

Beta telemetry currently means the local run ledger, process outcomes and
verification/Apply receipts. There is no external-user analytics dataset behind
these results. Any shared diagnostic excerpt should be selected and reviewed
by its owner; this benchmark does not upload telemetry.

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
