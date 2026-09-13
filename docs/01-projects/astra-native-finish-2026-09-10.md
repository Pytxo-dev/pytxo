---
title: Native ownership and background Git repair
slug: astra-native-finish-2026-09-10
status: active
tags: [project, desktop, native, verification]
audience: [human, agent]
layer: orchestration
created: 2026-09-10
updated: 2026-09-10
related: [[reference-led-review-2026-09-10]], [[astra-evidence-2026-09-07]], [[astra-release-proposal-2026-09-07]]
---

# Native ownership and background Git repair

## Observed mission

The review-pilot package (EXE `6acc8a37…`, MSI `7389d07a…`) completed the real
native Work → Review → Cancel → Apply → restart journey. Run
`f3e164cd-928a-43eb-b409-ba726b8c33b7` dispatched three scoped tasks through one
Codex harness in two waves, with at most two workers. Recorded execution lasted
300.140873 seconds; this excludes human preparation and review and is not a
competitive timing claim. Six primary files stayed unchanged through Cancel.
Apply wrote exactly three reviewed files, and 11 project tests plus 26 independent
checks passed. Restart retained the same package and single committed receipt.
The sanitized record is `tooling/benchmarks/results/astra-native-review-2026-09-10.json`.

Native inspection found that the Ownership path panel displayed planned labels
instead of recorded worker identities after task reordering. Prepared file headers
and the Apply receipt already identified the correct actors. Full-resolution video
also confirmed a Git console during startup, approximately 32.833 seconds into the
raw capture. The visible title does not identify its exact spawning call site.

## Bounded repairs

Review now derives a worker label from exactly one agent matching execution domain,
run and task. Missing or ambiguous records display “Worker not recorded.” The
frontend repair changes presentation only; it does not infer authority from a label.

Eight noninteractive Git construction sites now use a shared core helper applying
Windows `CREATE_NO_WINDOW`. Arguments, working directory, environment, captured
output and status handling remain unchanged. Non-Windows platforms retain the
standard constructor. This covers existing local maintenance across permission
profiles and execution domains; the observed acceptance domain uses Orbit. PTYs,
authentication, enforcement policy, verifier deadlines, Stop and Apply are unchanged.

The detached-parent regression actually failed before the flag and passed afterward.
The affected UI suite passed 65 tests; Svelte and CSS checks passed. The bounded Rust
suite passed 193 tests (77 core, 39 orchestrate, 77 runner), and 21 integration checks
passed for worktrees, candidate verification and Apply. Strict all-target Clippy for
core, runner, orchestrate and Desktop passed, as did formatting of the seven Rust
repair files. An earlier default-parallel run failed the existing two-second fast
verifier test once. Its unchanged isolated run and subsequent two-thread suite
passed; contention remains a hypothesis, not a demonstrated cause. Prior 170 browser
and 42 Storybook results cover the earlier pilot, not a newly rerun combined suite.

## Packaged candidate and limits

Artifacts are under `target/astra-native-finish-20260910/`:

- MSI: `9676d38f5749b7ba81a2799c56c423753840fc43e15874de5351a5868a678ea1`.
- Extracted executable: `e0eeb950c53010a1f1834cff3b0c2dae93397e03d062f82d5c3f7499463090b2`.
- 361-input manifest: `282c2c3ea129b154feaad27259b38731670845c585fe2cd423f2efbccbcc8e7a`.

All current and frozen input bytes match. The native build passed with one worker
in 9m04s; frontend build took 6.49s. MSI extraction and import inspection passed.
The standalone/extracted executables differ only in the three expected UNK→MSI
bundle bytes. Both artifacts are unsigned. Extraction is not installation, and
earlier native evidence cannot certify this executable.

During the latest explicit resumption, the older 6acc process closed normally and
the exact e0ee payload launched at 06:01:39 UTC. Its native Work screen restored
the historical f3e164cd run. Physical Escape stopped a Review navigation retry;
the repaired ownership panel remains unverified. No new mission or recording began.
The read-only follow-up found the exact process still running, all six fresh fixture
files unchanged, and an initialized database with zero runs, agents and events.
Records are in `native-run/resumption-20260910-060139.json` and
`native-independent-review/resumption-audit-20260910.json` under the artifact root.
After explicit native resumption, continue the open candidate if present, inspect
Review, select the fresh `approval-risk` fixture and complete the recorded mission,
Cancel, Apply, post-Apply checks and restart. Do not rebuild to repeat this launch.

The 6acc raw video fully decodes and has inspected Review/Apply intervals. An earlier
black-content warning was withdrawn after full-resolution reinspection; small dark
thumbnails had been misread. Neither sparse inspection nor configured recording FPS
proves smooth playback. The game-like mission/crew mockup and Focus/Control toggle
are proposals, excluded from this build and awaiting redesign approval. Clean
Windows, current-source CI, final-build demo and public download remain open.
No Actions dispatch, spending, account changes, commit, push or publication occurred.
