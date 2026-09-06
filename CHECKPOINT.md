# CHECKPOINT — Pytxo Beta candidate — 2026-09-06

## Branch / baseline

Beta work is being isolated on `codex/beta-candidate-verification`, based on local
`e1807cce5aaa3e02ba36fe4f3cc7ea0370f83f72`. Existing release PR30 uses
`codex/release-1.2.2-integrity`; its remote is one commit behind that local baseline.
The active Beta goal is NOT complete. No release, installer publication or site deploy occurred.

## Completed

- Independent competitor/core/UX/adversarial audits; see Sep5 Beta notes in docs/01-projects.
- One-worker planning preserves all tasks; dispatch respects concurrency; npm/Gradle
  verification inference and configured-task prompts corrected.
- Verifiers are registered, cancellable, bounded and retain cancellation after kill
  failures. Stop-all settles a run between child processes.
- v3 packages bind frozen combined source, included base/candidate inventories and
  the exact executed recipe/enforcement receipt. Combined failure, source mutation
  and input drift block Apply. Refresh preserves frozen effects plus operator files
  and reruns checks; a failed review recovers only after every task succeeded.
- Candidate Git discovery cannot reach the primary repository. Git-history-dependent
  checks fail in this source-only snapshot; excluded dependencies are not attested.
- Live agent rows exist before their first event. Failed event writes are counted,
  recorded as evidence gaps, and block Apply-ready status. SQLite failure injection
  and successful live verification-event persistence are regression-tested.
- Desktop separates process/task checks from candidate evidence, supports login
  recheck and mobile layout, and marks inconclusive Cursor account probes unknown.
- Real Codex0.153.4 single-worker mission completed, produced a v3 package, passed
  native Tauri exact-content Review/Apply, matched all three primary target hashes
  and passed four post-Apply tests. A direct-Codex baseline also changed only the
  requested files and passed four independent tests. These are two observations,
  NOT a speed, quality or reliability-rate claim.
- Final CLI deterministic run044cdc97-7bae-48ba-8ced-51fdd2f69b2e proves persisted
  verification-boundary, command, output and success events with no evidence gap.
- Demo, Bench, first-mission docs, changelog and release-readiness amendment align
  with observed behavior. Public latest is v1.2.1; v1.2.2 release is absent.
- Evidence JSON: tooling/benchmarks/results/beta-*-2026-09-06.json.
  Native captures: docs/_attachments/beta-2026-09-06/.

## Final verification

- Full workspace: PASS target/beta-final-workspace-events-2.log.
- Full clippy warnings denied: PASS target/beta-final-clippy-events.log.
- Debug CLI/MCP: PASS target/beta-final-cli-mcp-events-2.log.
- Release CLI/MCP: PASS target/beta-final-release-cli-mcp.log.
- Desktop check: zero errors/warnings target/beta-v3-desktop-check.log.
- Native build including Cursor fix: PASS target/beta-v3-native-build-final.log.
- Final MSI including event fix: PASS target/beta-final-msi-events.log; extracted
  successfully. Every byte matches the built exe except Tauri's UNK->MSI marker.
- 20 browser checks PASS target/beta-desktop-final-browser.log; six refined
  History-selected desktop/mobile workflows PASS target/beta-v3-history-browser.log.
  Screenshots inspected. Real native receipt/confirmation/journal also inspected.
- Web links152/107files and final production build PASS
  target/beta-v3-web-{links,build}-final.log; lint PASS target/beta-v3-web-lint.log.
- Focused event failure/normal persistence, candidate, dispatch and Stop checks PASS
  target/beta-event-lifecycle-tests.log. Cursor parser five tests PASS.
- cargo fmt --all -- --check and git diff --check PASS.

## Remaining external gates and scope limits

Clean elevated Windows installation is unverified: this process is not elevated.
The earlier MSI payload started and showed corrected Cursor status; final MSI
comparison includes the later event fix. Hosted CI run33827029442 is terminal
failure, with web/native jobs rejected before steps in its last attempt. A fresh
PR check should establish current runner availability; do not infer it from old
failure alone. Original independent review found real bugs now fixed; final
independent retry hit quota and is not counted as completed review.

No public v1.2.2 release exists; publication and independent download/install
verification remain required for public Beta readiness. No comparative Bench win,
OS-wide sandbox, universal detached-process containment, automatic skills router,
production-effect adapter or external analytics claim is established.

No keys/config/global CLI changed. Isolated Codex is under target/beta-tools.
Keep pre-existing apps/web/captures, apps/web/scripts/.verify, desktop-e2e.log and
 docs/superpowers untouched. Generated tool snapshots are in target/beta-ui-*;
new residual .playwright-cli/output folders contain only this task's captures.

## Continue

Inspect current Git/PR state and finish the full goal audit. Do not repeat passed
local runs without a new change/failure. Real v3 fixture path is in
 target/beta-v3-live-mission-path.txt; final deterministic path is in
 target/beta-final-demo-path.txt; final MSI path is in
 target/beta-final-msi-payload-path.txt. No tool runs were pending when this
checkpoint was written. Keep goal active unless complete, or genuinely blocked
for three consecutive goal turns with no meaningful local work remaining.
