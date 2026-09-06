# CHECKPOINT — Pytxo Beta candidate — 2026-09-06

## Branch / baseline

Beta work is on `codex/beta-candidate-verification`, based on local
`e1807cce5aaa3e02ba36fe4f3cc7ea0370f83f72`. Existing release PR30 uses
`codex/release-1.2.2-integrity`; its remote is one commit behind that local baseline.
The active Beta goal is NOT complete. No release, installer publication or site deploy occurred.
Core commit `f21acfb` is pushed in draft PR31, https://github.com/Pytxo-dev/pytxo/pull/31,
targeting main so protected CI runs. It includes the unpublished release follow-up
e1807cc; review its relationship to PR30 before any merge.

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
- Dependency follow-up: both web lockfiles, demo and tooling full audits clear;
  Desktop production clear, five moderate development UUID findings retained
  with inspected-call-site reasoning. Existing Rust warnings/exceptions unchanged.
  Details: docs/01-projects/beta-dependency-audit-2026-09-06.md and hashed JSON evidence.
- Updated Desktop check/native build PASS; 37 frontend asset hashes unchanged.
  Storybook build PASS and subsequent browser suite 41/41 PASS. An earlier
  premature test invocation hit missing iframe while the build was unfinished;
  it is superseded by the successful test after build completion.
- Web patched pnpm frozen install/lint/build PASS; demo typecheck/compositions PASS.
- Web links PASS and all 17 production browser tests PASS after dependency updates.
- Current demo film: recut from real v3 native evidence with automation-operated
  UI disclosure; no simulated click. Silent 52s/1080p/30fps render and validation
  PASS, source image hashes PASS, transcript/caption check PASS; final contact
  sheet and transition frames inspected. Record: beta-demo-film-2026-09-06.json.
  Local master: apps/demo-video/out/pytxo-demo-silent.mp4. Narrated master still
  requires approved new audio/music certificate; no audio was generated.

## Remaining external gates and scope limits

Clean elevated Windows installation is unverified: this process is not elevated.
The earlier MSI payload started and showed corrected Cursor status; final MSI
comparison includes the later event fix. Fresh PR31 CI run34033572511 failed
before jobs started; annotation101487472432 explicitly says account payments
failed or Actions spending limit must increase. Do not rerun unchanged jobs or
change billing. Final independent CLI review completed after the in-app quota
failure; its production findings and follow-up test-isolation finding are fixed.

No public v1.2.2 release exists; publication and independent download/install
verification remain required for public Beta readiness. No comparative Bench win,
OS-wide sandbox, universal detached-process containment, automatic skills router,
production-effect adapter or external analytics claim is established.

No keys/config/global CLI changed. Isolated Codex is under target/beta-tools.
Keep pre-existing apps/web/captures, apps/web/scripts/.verify, desktop-e2e.log and
 docs/superpowers untouched. Generated tool snapshots are in target/beta-ui-*;
new residual .playwright-cli/output folders contain only this task's captures.

## Continue

Current branch codex/beta-candidate-verification; pushed commits f21acfb, 2fc0f31,
fecebc0 and a4f5828. PR31 is draft and depends on PR30/e1807cc. Final review fixes
and binary/native evidence are committed and pushed; only this documentation
follow-up remains to commit. Inspect Git before acting.

Final independent CLI review completed: target/beta-final-cli-review.log.
It found blocking Desktop refresh IPC, refresh clearing event_persistence_failed,
and candidate approvals using a missing actor while audit errors were swallowed.
Fixed: async/spawn_blocking refresh, reject missing-event runs, use completed
origin actor, persist approval before publishing decision, emit actor-start before
policy gates. Scope: originating Orbit/Galaxy ceiling and one execution domain.

Follow-up review target/beta-review-followup.log found no additional production
bugs; its test-home isolation omission is fixed using the existing shared helper.
Exactly one confirmed test catalog row was removed and archived locally in
 target/beta-review-catalog-cleanup.json. Other catalog rows were untouched.

ALL current build/test/review/native sessions are terminal; no live job to resume.
Final gates after all Rust corrections:
- full workspace PASS target/beta-review-final-workspace-2.log
- full clippy PASS target/beta-review-final-clippy-2.log
- debug CLI/MCP + status JSON PASS target/beta-review-final-{debug,status}.log
- release CLI/MCP PASS target/beta-review-final-cli-mcp.log
- native build PASS target/beta-review-final-native.log
- MSI build/extraction PASS target/beta-review-final-msi.log and beta-review-msi-extract.log
- format and diff checks PASS
The earlier workspace attempt hit Unexpected EOF in billing_link's Windows HTTP
mock. Restore blocking socket reads / consume complete requests; focused and
full reruns passed. The compile error in the first edit is superseded.

Galaxy audit-write failure regression and evidence-gap refresh regression PASS.
Native Orbit run 374bf178-942b-4afd-b589-d22bf2837544 completed; changing only the
fixture package.json caused stale refusal. Refresh ran a deliberate 20s check;
native Maximize returned at 1437ms while verification remained pending, then
refresh produced a new verified v3 package. Candidate effects remain unapplied.
Evidence: tooling/benchmarks/results/beta-review-{native,msi}-2026-09-06.json.
Native captures inspected; test automation operated the UI. Fixture pointer:
target/beta-review-native-fixture-path.txt. Own native process31784 stopped after
verification; snapshots moved to target/beta-review-playwright-residual.
Final MSI extracted to target/beta-review-msi-extracted; only Tauri UNK->MSI
marker differs. This does not establish a clean elevated installation.

Silent recut film is apps/demo-video/out/pytxo-demo-silent.mp4; full render/media
checks and visual inspection passed (beta-demo-film-2026-09-06.json). No need to
rerender for Rust-only fixes; film identifies the original real v3 run and hashes.
No new audio was generated; approved voice/music evidence remains unavailable.

Fresh PR31 CI34038130415 on final code a4f5828 failed before steps, just as the
earlier fecebc0 run did. Do not rerun unchanged jobs or change billing. The
account payment/spending-limit condition prevents hosted execution. Do not
publish/merge/tag/deploy. Record later docs-only CI in the PR body to avoid an
endless commit/checkpoint/rebuild loop.

Remaining public Beta gates: hosted CI, clean elevated Windows installation,
publication and independent download/install verification. Public latest v1.2.1;
no v1.2.2 release. Current code/runtime evidence supports a reviewable local
candidate, not public Beta readiness. Keep goal active until achieved, or mark
blocked only after a real impasse recurs for three consecutive goal turns with
no meaningful work remaining. This turn made material implementation progress.
