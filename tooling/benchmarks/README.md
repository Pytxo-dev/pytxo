# Pytxo benchmarks

The offline [Jev routing analysis scaffold](routing/README.md) has synthetic
contract tests only; it is not a completed R0-versus-Jev comparison.

## September 7: three-task observations

The [direct Codex worktree observation](results/astra-direct-codex-2026-09-07.json)
completed the exact mission in [the reproduction fixture](fixtures/astra-first-mission/README.md):
three requested files, eight independently rerun tests, and an unchanged primary
checkout. Its 252.172-second duration ends at CLI exit. The CLI reported 30,811
tokens, without an input/output breakdown or reconciled invoice cost.

Keep the unsuccessful Pytxo attempts alongside later results:

- [Scope refusal](results/astra-refused-run-2026-09-07.json): three workers finished
  and passed task checks, but an implementation worker edited the separately owned
  test file. Package preparation refused it; six primary baseline hashes stayed
  unchanged. This exposed a missing explicit ownership handoff to workers.
- [Windows launch failure](results/astra-transport-failure-2026-09-07.json): the
  revised handoff hit a PowerShell-to-CMD prompt parsing defect. Two workers failed
  before model execution and the dependent task was blocked. No package or Apply
  occurred; six primary baseline hashes stayed unchanged.
- [Review withheld](results/astra-review-withheld-2026-09-07.json): all three
  workers and combined checks passed, but native diff review found a README
  statement contradicted by the correctly changed code. The package remained
  ready and was never applied. This was an agent review decision; Pytxo did not
  automatically detect the prose error. A fresh run uses explicit final-state
  documentation guidance through the existing task-prompt editor.

The [preceding native run](results/astra-native-codex-2026-09-07.json) completed three
tasks in two waves, passed all three combined-candidate commands, and reached
native reviewed Apply. All three applied hashes match the frozen package and
eight independent primary tests pass. Six baseline source hashes were unchanged
before Apply; the source inventory gained or lost no files. The CLI reported
80,975 tokens across the three workers; invoice cost remains unknown.

The unsuccessful attempts above remain separate. Timing endpoints, task-prompt
guidance, instance counts, startup hooks and machine
load differ, so these observations do not establish orchestration overhead, a
speed win, better quality or a reliability rate. The direct run did not receive
the later documentation-task override. A competent direct Git worktree
also preserves the primary checkout. Customer repeat use remains unmeasured.

The [current-build follow-up](results/astra-ci-native-2026-09-07.json) uses the
MSI rebuilt after the CI Stop repair. From the previously applied baseline, it
adds `credentials/` review classification through three tasks and two waves.
All three combined checks passed before exact native Review/Apply. Applied
hashes match; three unrelated files remain unchanged. Eleven repository tests
and 26 separately authored acceptance checks pass; five acceptance checks failed
on the baseline. The committed receipt survived restart. Worker headers report
Astra/xhigh, and the documentation task's explicit guidance is disclosed. This
is different work from the direct comparison above; no comparative inference
is valid. MSI extraction on the development host is not a clean install.

## Historical September 6 Beta evidence

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
