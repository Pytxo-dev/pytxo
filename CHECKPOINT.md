# CHECKPOINT — ASTRA product improvement — 2026-09-09

## September 20 — UI refinement built; native motion verified; VM host install needs elevation

User resumed and authorized cleaner task surfaces, less dashboard framing, and
smooth ASCII motion while retaining Chroma Aperture. Removed New work coaching
column; collapsed Saved requests (18 real records preserved); neutralized selected
task/history fills; flattened History evidence; removed redundant lone-task
relationship explanations. Orb now rotates slowly at rest and faster for execution,
with OS/app reduced motion, document visibility and intersection gating.

Independent review fixes: refresh canvas palette when tone changes, and require
painted pixels before comparing animation frames. Final Svelte/CSS PASS and 35
affected browser tests PASS (session32578); scoped diff check PASS. Browser captures
at 1280/1920: target/beta-regression-repair-20260919/refined-*.png.

MSI build49430 exit0; 479 input hashes unchanged before/after packaging.
refinement-installer/pytxo-desktop-windows-x64.msi SHA256
D01DCEA4F699175FAD614FC51377AC4DA2229937EBCF316036BA2540209A0950.
Extracted EXE SHA256
27C073079814DFF4FAF51E1943B96874BFB7DDA15F023C17801A87F6F07DDE95.
Exact inventory and extracted MSVC dependency checks PASS. Unsigned; not installed.
Native window197001898 launched from refinement-extraction payload. Work, New work,
collapsed saved requests, and History visually checked. Saved in-app Reduced motion
was ON, explaining the static glyph; switched OFF per user animation request and
observed different native orb frames. OS preferences unchanged. No workers or Apply
started. Historical worker inspection became unavailable after route remount while
the recorded Apply remained visible; do not imply fresh execution/evidence validation.

Disposable Windows preparation: D:/pytxo-beta-lab. QEMU checksum verified; WHPX
prelaunch/quit smoke passed, but Windows build rejects -tpmdev and cannot provide
the Windows11 TPM setup. Oracle VirtualBox7.2.18 downloaded, published SHA256 and
Oracle Authenticode signature verified. Minimal host installation returned1603;
log explicitly requires administrator rights. User asked asynchronously to install
the verified installer as administrator. Windows11 Enterprise25H2 evaluation ISO
download still running (exec38508 / curlPID5512 at recording), not yet hash verified.
Published expected SHA256 A61ADEAB895EF5A4DB436E0A7011C92A2FF17BB0357F58B13BBC4062E535E7B9.
No VM/clean installation/installed upgrade/updater acceptance yet. New source edits
are not represented by the earlier legacy-retry candidate. No commit/publication.


## September 20 — current acceptance gates consolidated; external prerequisite outstanding

Previous goal turn was progress: native legacy retry and selector verification.
Current audit rechecked all478 candidate inputs and MSI/EXE hashes, compared the
prior retry-state snapshot (only six Desktop UI/preview/test files changed), and
corrected CLEAN_WINDOWS_HANDOFF's false attribution of earlier crash/recovery
proof to the latest payload. Current matrix:
target/beta-regression-repair-20260919/CURRENT_GATES.md.

Live read-only GitHub: latest source CI success is September8 commit eb5f73d,
not candidate dirty HEAD7287970; public latest remains v1.2.1. No fresh claim
about billing/runner allocation is supported. Host Windows11 Home10.0.26200,
registry AppliedDPI120, C:~3GB/D:~374GB; no VBoxManage/vmrun/qemu executable in PATH.
These checks do not establish clean install, multi-DPI, installed upgrade/updater.

No independent build/test repair is presently identified. Need disposable
Windows access and install/upgrade authorization, then approved signed update
feed/artifact. Existing question remains unanswered. First gate-blocked audit
turn after actual native progress; keep goal active, do not mark complete.
If unchanged for three consecutive goal turns and no meaningful safe action
exists, mark blocked. Do not generate documentation churn or replay passing
suites to evade that threshold. No source/runtime edits or publication this turn.


## September 20 — legacy retry and expanded selector verified in rebuilt native candidate

Reviewed MSI build95227 exit0; 478 input hashes unchanged, exact inventory PASS.
Installer legacy-retry-installer/pytxo-desktop-windows-x64.msi SHA256
8C733BCD5B7D66213DA7B7C31776D3A32BC8AC92C8F3A35D0544F60E0A073760.
Extracted legacy-retry-payload/pytxo-desktop.exe SHA256
9D6D56CA876007520ECD76255D9BDB4FD7B9A4F30F9F5435A7B9036A9823A7F5.
Unsigned. Source legacy-retry-reviewed-source.json, build log
legacy-retry-reviewed-msi-build.log. Intermediate legacy builds superseded.

Native original run bb3355c8-330d-466e-8986-46cebc634c5d now shows saved-changes
Work summary and Apply recorded History outcome without false current recovery
warning. Native attempt history still visibly contains both Committed and Rolled
back. Read-only DB query confirms old rollback/error fields remain unchanged.
Eight real run IDs expanded without collapsing the long request heading.
Candidate remains open, window33949802, History attempt section. No workers,
Apply, journal rewrite, source commit or publication performed in this check.
Receipt: target/beta-regression-repair-20260919/native-legacy-retry-receipt.json.

Final 22 affected Work/History tests PASS (session73068), Svelte/CSS PASS
(session13888), scoped diff check PASS; independent review fix integrated.
This verifies extracted-payload behavior on the development host (1282x802),
not clean installation, actual Windows DPI scale or installed updating. Those
and outstanding release gates remain open. NOT VERIFIED BETA.


## September 20 — legacy retry presentation and run selector repaired (browser PASS; native pending)

Older applied run bb3355c8-330d-466e-8986-46cebc634c5d retains confirmed
rollback metadata dated before its successful Apply. Native Work and History
reproduced misleading attention/rollback states. Presentation now distinguishes
strictly older confirmed rollback metadata from current error/recovery evidence;
unresolved attempt journals still take precedence. Work summary, glyph,
repository boundary, and History use that distinction; DB/journals unchanged.
A reviewer found History's old fallback could override a newer error; corrected
and covered by regression. Review then found no further issue.

Native Other runs & IDs with eight IDs collapsed the request heading into a
single-character column. Browser regression reproduced width0; bounded selector
width fixes it. Recovery alert text/action now wraps with explicit spacing.
22 affected Work/History tests PASS (session73068, chunk0dd748); Svelte/CSS PASS
(session13888); scoped diff check PASS. Screenshots inspected: legacy-retry-
partial-false.png, legacy-retry-partial-true.png, work-expanded-runs.png.

Final reviewed MSI build live session95227, log target/beta-regression-repair-
20260919/legacy-retry-reviewed-msi-build.log. Input snapshot legacy-retry-reviewed-
source.json. Previous intermediate builds are superseded and not acceptance
candidates. Do not stage them. Next: poll this build, verify input hashes, stage
MSI/extracted payload, launch through Computer Use, verify original legacy run
in Work/History and expanded Other runs. Old retry-state app was closed through
UI; refresh native window inventory before launching. No DB edit or Apply needed.
Clean install, actual Windows DPI, installed updater and release gates remain
unverified. NOT VERIFIED BETA. No Pytxo source commit/push/publication.


## September 20 — rebuilt candidate recovery retry clears Work warning (native PASS)

MSI build87549 exit0; all recorded retry-state-source.json inputs unchanged
through build. Exact release inventory PASS. Staged candidate:
retry-state-installer/pytxo-desktop-windows-x64.msi SHA256
B1BCDC725CB6842302458979D6FEF6DD8E5AFA0BE2EAF95271889B15FC008EE7.
Extracted retry-state-payload/pytxo-desktop.exe SHA256
CAA5990272BF81EF8A472DF395BAAA693523950CEF6C3A8DEA517BB1941C812D.
Unsigned; payload launch, not installed acceptance.

Native run359bc4ae-86f5-4cb7-b53c-f66f0bde79c3 completed one Codex README task
with recorded combined npm test verification. Fixture baseline c50265d preserves
previous result. Package843ffb280de78c9fe212f03decec8e2064d48a378eb104e18a29e80bd390c727.
CDB matched this candidate PDB; session77744 breakpoint action explicitly logged
CONTROLLED_PRECOMMIT_TERMINATION and stack then .kill/q (exit0). Process/window
absence verified; interrupted journal d955e32a-9492-47a6-986d-2b09ba88dac8
mutating/completed1 and README candidate bytes persisted.

Reopened candidate, observed native partial-mutation warning, clicked Reconcile;
all7 base hashes restored. Retry required exact-package confirmation. Success
recorded2026-09-19T21:20:55.602307100+00:00 with last_apply_error_json NULL and
recovery_state NULL. Review Applied successfully; returning to Work showed
These changes have been saved to your project, no rollback warning. All7
candidate hashes matched; independent npm test5/5 (chunkae94e6). Historical
rolled_back journal retained beside committed e03cc678-dd05-47e0-b86b-f72b3c4d95ad.
Receipt target/beta-regression-repair-20260919/native-retry-state-receipt.json.
No debugger/build/worker remains; candidate window6228318 on Work. Fixture
README is modified by authorized Apply; preserve. No Pytxo source commit.

Scope: real native interruption/reopen/rollback/retry/Work state verified at one
durable precommit checkpoint. Older already-applied stale database rows are not
migrated by the success-transition fix. Clean install, actual DPI, installed
updater, and other outstanding release gates remain open; NOT VERIFIED BETA.


## September 20 — successful retry stale-recovery defect reproduced and repaired

After native recovery/retry run bb3355c8-330d-466e-8986-46cebc634c5d, Review
correctly showed Applied successfully, but returning to Work reproduced an
incorrect rollback alert and attention-required summary. Database retained
last_apply_error and recovery_state=rolled_back alongside applied_at.

Narrow fix in crates/pytxo-store/src/store.rs: finish_run_apply clears current
last_apply_error_json/recovery_state only for successful applied transitions,
in the existing transaction. Journal files and historical attempts unchanged.
New successful_apply_retry_clears_resolved_recovery_error test failed before
fix (chunk4c33ac) and passed with full store26+wallet2 suite (chunk1da1eb).
Clippy all-targets passed (chunk3e61de); formatting passed on retry after one
Windows mapped-file error1224; scoped diff check passed. Independent read-only
review found no actionable issue. Existing already-applied stale database rows
are not migrated by this transition fix; this must remain explicit in acceptance.

Replacement MSI build is live session87549, log
 target/beta-regression-repair-20260919/retry-state-msi-build.log.
Input hashes: retry-state-source.json. Poll the same build handle. Old
background-work candidate remains open and retains the reproduced defect.
Do not claim the fix is native-verified until rebuilt candidate recovery/retry
and Work return have passed. Clean Windows, actual DPI and installed updater
remain open. No Pytxo source commit or publication.


## September 20 — native interrupted Apply, reopen, rollback and retry verified

PASS on exact background-work EXE9CC0F06B41B71B7FC3C272E52851DB59F86F6EFACF5CC32D4BEB0B78ACF6AC4F.
Run bb3355c8-330d-466e-8986-46cebc634c5d used one real Codex worker, README-only
change and recorded npm test verification. Disposable fixture baseline df144e1
preserves the previous verified Apply; no Pytxo source commit was created.
Package 700fd3dfad4f1ac80fd723d23c3c1af3b11a0f5e18f14ccc5b6d8b7e4f39e194.

Matched PDB symbol verify_candidate_poststate provided a breakpoint after all
mutation journal writes and before poststate verification/committed recording.
CDB session69067 hit breakpoint0, then noninteractive EOF exited debugger1 and
terminated Desktop. Process/window absence verified. Journal attempt
440403c4-aa45-49f7-b523-f2a083f69ba6 remained mutating/completed1; README contained
candidate hash722a385f…26d8f8d; database applying, applied_at null. This was a
known checkpoint interruption, although EOF termination occurred before the
planned interactive journal inspection; record that mechanism explicitly.

Reopened same executable. Native Work warned files might be partially modified;
Reconcile recovery state rolled back. All7 base-inventory hashes matched,
independent npm test5/5, database ready/recovery_state rolled_back with
rollback_confirmed true and no successful Apply. Review showed Recovered and
ready to retry. Retry required the same exact-package confirmation; native UI
then Applied successfully. All7 candidate-inventory hashes matched and npm
test5/5. Applied at2026-09-19T20:56:58.938684+00:00. Earlier rollback evidence
remains recorded alongside the successful retry. Fixture README is now modified
by that authorized Apply; preserve it. Reopened Desktop window9504522 remains
on Applied Review. No active worker or debugger session remains.

Evidence: target/beta-regression-repair-20260919/native-recovery-receipt.json,
native-recovery-interrupted-journal.json, debugger-tools/native-apply-breakpoint.log.
PASS scope: actual candidate process termination at one durable precommit
checkpoint, native reopen/reconcile and retry. Not arbitrary midwrite, power
loss, every journal phase, clean installation, actual DPI or installed-updater
proof. Those remaining release gates still prevent VERIFIED BETA/READY status.


## September 20 — native debugger staged and candidate symbols matched

Completed the existing SDK download-only layout (session35148 exit0), then
administratively extracted the debugger MSIs under target/beta-regression-repair-20260919/debugger-tools.
CDB10.0.26100.9169 signature is Valid/Microsoft. No global debugger installation.
CDB noninvasive inspection loaded matching private PDB symbols for running
background-work EXE9CC0F06B…ACF6AC4F, PID36100. Detached with qd; process remained
responsive. Saved matching PDB next to candidate, SHA256
23A8226FF19452AFD56190EB927320513576687449946C9E712A7CAB88EFC235.
The first -pvr/-pd combination was rejected before attaching; documented -pv
succeeded. Explicit reload requires image name pytxo-desktop.exe (hyphen).

Receipt: target/beta-regression-repair-20260919/debugger-readiness-receipt.json.
Apply symbols/disassembly are available; persist_journal is not independently
exposed in this optimized binary. No breakpoint or crash was executed. Next:
establish a defensible durable checkpoint before fresh disposable native
Apply interruption/reopen. Native recovery, actual DPI, clean installation and
installed updater gates remain open. Candidate binary and fixture unchanged.


## September 20 — process-death journal recovery regression verified

Added test-only coverage in crates/pytxo-runner/tests/run_change_set.rs:
interrupted_journal_recovers_after_real_child_process_termination and helper
interrupted_journal_child_holder. A test-owned child loads the immutable package,
holds its repository mutation lease, changes an existing file and creates a
nested file, and reaches the existing durable InterruptAfterRename(2) checkpoint.
Parent verifies actual changed bytes and refusal to recover while the child
holds the lease, then terminates/reaps that exact child. Recovery restores the
original bytes, removes created directories, and is idempotent on a second call.
RAII guard reaps the child even if a parent assertion fails before termination.

PASS: targeted new regression chunk9ff4a1 exit0; full run_change_set suite34/34
chunk1c0e77 exit0, log target/beta-regression-repair-20260919/process-recovery-tests.log;
runner test-target Clippy chunk9af647 exit0; cargo fmt and scoped diff check.
Separate read-only review found no actionable issue and confirmed cleanup order.

Evidence scope: real process death after a controlled durable checkpoint.
Not arbitrary mid-write crashes, power-loss durability, or native Desktop
interrupted Apply/reopen proof. No production source or candidate executable
changed, no MSI rebuild needed for this test-only delta. Latest shipping-input
candidate remains background-work EXE9CC0F06B…ACF6AC4F. Its recorded source
snapshot predates this additional test; preserve both identities explicitly.
Clean-machine, DPI, installed updater and native recovery gates remain open.


## September 20 — native keyboard check; recovery and DPI prerequisites clarified

On background-work EXE 9CC0F06B41B71B7FC3C272E52851DB59F86F6EFACF5CC32D4BEB0B78ACF6AC4F,
Computer Use verified Ctrl+K opens the command palette with search caret;
typing Open latest review filters the recorded run; Enter opens its native
Review and visibly focuses Back to Work; Tab focuses Run details; Space opens
the exact recorded run ID. Applied state remains visible and Apply disabled.
No repository mutation or new worker occurred. Observed window1282x802;
actual Windows scale was not established, so this is not DPI acceptance.

Recovery inspection: runner change_set.rs exposes fault points returning
controlled errors/early results, and orchestrator reconcile_run_recovery takes
a repository mutation lease and persists committed/rolled-back/unprovable state.
No shipping Desktop control for an exact interruption point was found. Existing
backend injection tests do not prove a terminated/reopened native Apply process.
Keep controlled native interruption/recovery acceptance open; do not fabricate
it by editing the existing fixture database or killing an arbitrary process.

Native Settings launch failed: process:C:\Windows\ImmersiveControlPanel\SystemSettings.exe
reported no targetable window. Refreshed list_windows confirmed Settings absent.
No display setting changed. Async question asks user to open System/Display and
report original Scale, enabling reversible150/200-percent native checks.
Earlier disposable-Windows machine question also remains pending.
No build or worker handle is running from this turn. App remains open on applied
Review with Run details expanded. Goal active; this is additional bounded native
keyboard evidence, not complete beta acceptance. Next meaningful work requires
controlled recovery harness/design or available native DPI/clean-machine surface.


## September 20 — native background Work fix verified

Post-fix EXE 9CC0F06B41B71B7FC3C272E52851DB59F86F6EFACF5CC32D4BEB0B78ACF6AC4F
kept the same completed worker, descriptive title and recorded README output
visible after 201 seconds behind Explorer. Snapshot showed Updated36sago,
confirming refresh while unfocused. Observed without reactivating Pytxo first.
The old cancellation candidate lost the same worker after65s. This verifies
the reproduced background evidence-disappearance fix on the new native payload.
Receipt: target/beta-regression-repair-20260919/native-background-work-receipt.json.

App restored to foreground, window9307746, background-work-payload. No new
agent run or Apply during this UI check; earlier Work/Review/Apply remains
bound to cancellation candidate. New payload restored its persisted outcome.
The latest candidate retains only the reviewed UI/polling delta over that binary.
No installation or publication. Remaining gates: controlled native recovery,
100/150/200-percent DPI, clean Windows install and installed updater, and
original intermittent sharing-error investigation. Goal remains active, NOT READY.


## September 20 — background Work evidence disappearance fixed and packaged

Confirmed source cause and native reproduction: an unfocused Work integrity
refresh requested no agents and replaced its displayed snapshot with empty
worker rows. The older native candidate reproduced missing worker evidence
after 65 seconds behind Explorer; focus restoration recovered the records.

DesktopShell now includes agents whenever Work is displayed; existing background
polling throttle remains. PreviewBackend now honors includeAgents=false, matching
native IPC. New background-work regression failed before production fix, passed
after it, then was strengthened to assert refresh-age reset and known output text.
Affected browser suite29/29 passed (2953); strengthened1/1 passed (1059);
Svelte/CSS checks passed (80222). Separate read-only review found no production
correctness issue. No permission, execution-domain, or Apply logic changed.

New MSI build96835 exit0. All478 input hashes stable. Exact inventory passes.
MSI: target/beta-regression-repair-20260919/background-work-installer/pytxo-desktop-windows-x64.msi
SHA256: 0E24344C02EE4C8C405778CAC812FC3CC1C4B310986ABB7A9507CA916349AA3C.
EXE: target/beta-regression-repair-20260919/background-work-payload/pytxo-desktop.exe
SHA256: 9CC0F06B41B71B7FC3C272E52851DB59F86F6EFACF5CC32D4BEB0B78ACF6AC4F.
Snapshot/log: background-work-source.json / background-work-msi-build.log.
New native app window9307746 restored the completed/applied fixture run and
worker evidence. Background post-fix native observation currently pending.
CLEAN_WINDOWS_HANDOFF.md now identifies this candidate; MSI remains unsigned.
See background-work-fix.md for reproduction and test details. NOT READY.


## September 20 — cancellation candidate native success and Apply verified

Exact extracted MSI payload 3BD1D86152C4756A326CE182EBF3993CEAE2A2BE3DE5D768902F37D5A9C503EE
completed real Codex run 832d82a7-1e8f-4c9c-b1aa-4e5c46970f82 at
2026-09-19T19:43:58.523421200Z. Worker exit0, per-task npm test 5/5;
combined candidate check passed at 19:43:58.494884100Z. Orbit, one README task.
Native Work showed Completed, Review showed exact before/after contents and
combined checks passed. Reviewed blob preserves the native acceptance marker.

Native exact-digest confirmation applied candidate
04f7a097c8b73092d444bd8a86c39fafca56720694fae918d1efeeadc1b38271
at 2026-09-19T19:52:30.394979500Z. UI shows Applied successfully / Apply recorded.
All seven destination inventory hashes independently match; destination npm test
passes 5/5 (exec chunk39e1dd exit0). README now hashes to
F0BF36CF5886C88B534B47D2E9332D0FCB07A4F5CFE4DD88717C6495CF74FCDD.
Fixture HEAD remains e5f7dee; README is the authorized local applied change,
not a new source commit. Preserve it and all earlier runs/receipts.
Evidence: target/beta-regression-repair-20260919/native-cancellation-candidate-apply-receipt.json.

This supersedes the running status in the preceding entry. No worker remains
active for either native acceptance run. Stop classification and normal
Work/Review/Apply are now verified on this candidate within the exercised scope.
The original sharing error did not recur, but its root cause is still unproven.
Do not equate these results with clean install, installed updater, controlled
Apply interruption/recovery, or 100/150/200-percent DPI acceptance. NOT READY.
Native app remains open on the applied Review, window7735126, cancellation-payload.


## September 20 — cancellation candidate built and native Stop verified

Built MSI with npm run build:msi (session79450 exit0). All 477 input hashes
match the pre-build snapshot; exact release inventory passes.
MSI SHA256: 6F3DDE12996FA174104AD6146DEE86A9790935F677336D4B80A98B809CAD8083.
Extracted EXE SHA256: 3BD1D86152C4756A326CE182EBF3993CEAE2A2BE3DE5D768902F37D5A9C503EE.
Artifacts: target/beta-regression-repair-20260919/cancellation-installer,
cancellation-payload, cancellation-source.json, cancellation-msi-build.log.

On that exact native payload, run dfd7e516-400c-4ba6-9156-b45747de1d54
started real Codex worker PID33088 (child19980). Native Stop confirmation
completed at 2026-09-19T19:37:27.686618900Z. Run and agent both persist cancelled;
worker actual exit1 retained. Native heading, task tile and inspector all show
Stopped. Both observed PIDs are gone; registry has no entries for this run.
No prepared digest or Apply. README SHA256 remains
E3991AAD561D3FE0B101A3C2047DFE11F91D497DD917BBD3DF2C6954EB2B63BD.
Receipt: target/beta-regression-repair-20260919/native-cancellation-fixed-receipt.json.
This closes the reproduced native Stop-status defect for this exercised path.
It does not prove complete descendant-tree auditing or every cancellation race.

Fresh normal run 832d82a7-1e8f-4c9c-b1aa-4e5c46970f82 is running on the same
payload; do not start a duplicate. One README task, one worker, npm test.
Clean Windows handoff: target/beta-regression-repair-20260919/CLEAN_WINDOWS_HANDOFF.md.
MSI is unsigned. Disposable-machine environment question remains pending.
Original sharing failure, recovery, DPI, clean install and installed updater
remain open. No installation, publication or Pytxo source commit performed.


## September 20 — active-process cancellation classification fixed in source

Reproduced Stop-during-execution incorrectly yielding ProcessFailed on the
subprocess backend. Added SingleResult.cancelled, captured under the process
registry lock when exit settles. PTY settles before output drain via callback;
subprocess settles before pipe joins. Stop durable before settlement wins;
later Stop cannot reclassify an already captured result. This does not claim
Stop caused an exit that raced cancellation, and cloud execution is excluded.

Cancelled workers retain output and actual exit evidence. Retry, verification
and successful-workspace cleanup are suppressed; AgentRunOutcome is Cancelled.
Permission profiles and per-repository execution domain unchanged.

PASS: full cargo test -p pytxo-runner --locked -j 2 (session44720 exit0),
workspace all-target Clippy (48361 exit0), formatting and scoped diff checks.
Strengthened both-backends regression with Signal Core retry enabled and real
README context; rerun session31111 exit0 confirms no retry or verify events.
Read-only review found no new correctness issue in the delta.
Logs: cancellation-runner-tests.log, cancellation-final-clippy.log,
cancellation-retry-enabled-test.log under target/beta-regression-repair-20260919.

Next required: capture updated source identity, rebuild MSI, native Stop rerun
and fresh successful verification on new payload. Current open native app is
still publication-hardening A3921702…A888948B; it lacks these cancellation changes.
Do not mark the native status defect verified fixed until rebuilt UI evidence.
Original sharing error, recovery, DPI, clean installation/updater gates remain.

## September 20 — typed cancellation outcome implemented (partial Stop fix)

Added PytxoError::Cancelled and AgentRunOutcome::Cancelled. Explicit durable
Stop errors at registration and verification now record cancelled / agent-cancelled
instead of generic failed. Permission profiles and per-domain scope unchanged.
Genuine process/verification failures retain their prior statuses. No UI relabeling
and no rewriting historical failed rows.

Review caught output/accounting loss in an initial early-return approach; fixed
before completion. Cancelled verification retains worker stdout/stderr, actual
worker exit code and workspace, with normal cleanup. Test asserts output marker,
exit0 (worker succeeded before verification Stop), cancelled outcome and workspace
retention even with keep_worktrees=false.

PASS: dependency_outcomes 9/9 after production fix; affected retention test rerun
1/1 after strengthening keep_worktrees=false; verification_boundary_tests 13/13;
workspace all-target Clippy before output-retention correction and runner all-target
Clippy after correction; fmt/scoped diff checks. Read-only review found no further
correctness issue in this bounded delta.

REMAINING: PTY/subprocess killed during execution can return a nonzero SingleResult
without typed cancellation and still map to ProcessFailed. Must cover this with a
controlled Stop-during-execution test and preserve real failure/Stop race semantics.
Do not call the native Stop-status defect fully fixed. New source is not in the
currently open publication-hardening MSI; rebuild/native revalidation pending.
Files: core/src/error.rs, runner/src/run.rs, runner/tests/dependency_outcomes.rs.
Other beta gates remain open; NOT READY.

## September 20 — native Stop verified; task-status defect found

New publication-hardening payload A3921702…A888948B launched real Codex run
6c5e37c1-fe57-4959-8143-28298c0e97ed (one README task, Orbit). Observed worker
PID 22216 and child 34032 before native Stop confirmation. Both were absent
after Stop; run is cancelled at 2026-09-19T19:03:24.548455400+00:00. Registry
entries empty and cancelled_runs retains this run. Destination README hash
remains E3991AAD561D3FE0B101A3C2047DFE11F91D497DD917BBD3DF2C6954EB2B63BD.
No prepared candidate, verifier completion or Apply. Stop test PASS within
this scope; not proof of successful completion or full descendant-tree audit.
Receipt: target/beta-regression-repair-20260919/native-stop-receipt.json.

Native task-status defect: run says Stopped but worker tile/inspector says
Failed. Stored agent status is failed with null exit; lifecycle error explicitly
says cancelled by Stop. Source: failed_agent_result in runner/run.rs maps all
lifecycle errors to ProcessFailed; orchestrator persists outcome.ledger_status().
UI correctly renders that stored status and already supports cancelled/stopped.
Fix cancellation at the typed outcome boundary, preserving genuine failures;
do not relabel all failed agents merely because the parent run was cancelled.
Original sharing failure, recovery, DPI, clean install and updater remain open.

## September 20 — publication-hardening MSI built and opened

Build session 60145 completed exit 0: npm run build:msi, voice-whisper and
explicit distribution static CRT config; release compile 2m25s.
Staged installer: target/beta-regression-repair-20260919/registry-publication-installer/pytxo-desktop-windows-x64.msi
MSI SHA256: 4B3928B89C25545CBEF5DD53412667939111748914D77A8340CD4C1B2D5B0FDF.
Extracted executable: target/beta-regression-repair-20260919/registry-publication-payload/pytxo-desktop.exe
EXE SHA256: A3921702891BBEE2D9F7E7B05E73000F8CEFE2EC290580009D6F55CFA888948B.
Version resource: 1.2.2. Exact inventory/checksums PASS. All 477 captured inputs
match after build; only registry source differs from diagnostic input snapshot.
Snapshots/log: registry-publication-source.json / registry-publication-msi-build.log
under target/beta-regression-repair-20260919.

Closed the previous diagnostic app through its titlebar; Computer Use launched
and selected the exact new executable (window 395260). Native Work restored the
completed README run; Review restored candidate d371b6e0fe212ac1a381360a9a3ac59fc5df97229c0a6ae16d819e4d70dd1a85
and exact before/after content, including preserved destination marker.
No Apply or new verifier execution occurred on this new binary yet.
This is extracted-payload launch and persisted-review evidence, not clean install,
installed updater, crash recovery, DPI, or resolution of the original error 32.
Next: affected native execution/Stop verification on the new payload, controlled
recovery, DPI and disposable-machine install/update acceptance. NOT READY.

## September 20 — registry publication hardening (source only)

Removed the Windows delete-before-rename step from process registry publication.
std::fs::rename supports replacement; the primary now remains in place until
publication. Permission profiles and per-repository execution-domain scope are
unchanged. No retry or timeout increase was added.

A compatible held-reader test passes before and after the change, so that
hypothesis did NOT reproduce the original native error. A deny-delete handle
fails publication with Windows error 5 and preserves the original bytes. The
former delete-first path reported error 32. Both are accepted diagnostic codes;
operation context and exact byte preservation remain asserted.

Verification: cargo test -p pytxo-runner --locked -j 2 PASS (exit 0);
cargo clippy -p pytxo-runner --locked --all-targets -j 2 -- -D warnings PASS;
cargo fmt --all -- --check PASS; scoped git diff --check PASS.
Source SHA256: 431062EFEA2DE974531DC43B3B30FEF7AAFC8CB877DAEF5DBFB89DC102FBDCBA.
Original intermittent native sharing violation remains unresolved. Existing
MSI and extracted-payload evidence predates this functional source change;
rebuild and affected native acceptance are required before candidate promotion.

## September 20 — native stale refusal and refresh PASS

On diagnostic extracted executable A16D5FE4…06A694D9, run
556be41c-637b-4613-80a8-bf1ae97b4b34 completed README-only work and checks.
Changing the destination after opening Apply confirmation caused source_drift
at 2026-09-19T18:30:47.690485800+00:00. Native UI disabled Apply and offered
Refresh review. Destination SHA256 stayed E3991AAD561D3FE0B101A3C2047DFE11F91D497DD917BBD3DF2C6954EB2B63BD;
no applied_at receipt was created. Fixture-only commit e5f7dee preserves the
acceptance marker; no Pytxo source commit was made.

Native Refresh produced digest d371b6e0fe212ac1a381360a9a3ac59fc5df97229c0a6ae16d819e4d70dd1a85
from prior 8c6725a905c703c9cdc62ede58f8ab1dec7d0bda3ffbe9cd54188d2f49e39f78.
The refreshed comparison includes the marker in Before and the frozen agent
result in After. A newly opened confirmation names the new digest; cancelled
without Apply to preserve the marker. Evidence:
target/beta-regression-repair-20260919/native-stale-refresh-receipt.json.
This proves native refusal/refresh/fresh confirmation, not a native two-client
old-digest replay. That authorization property has a separate backend test.

Release remains NOT READY: intermittent sharing violation, controlled native
recovery, native DPI, clean installation, and installed updater remain open.
The latest generated mockups are proposals, not implemented or runtime evidence.

## September 20 — diagnostic native rerun

UPDATE: run 2c585a7f-3e5f-4ba7-9521-3ca48045b8ab is TERMINAL completed.
Pytxo's verifier passed npm test (5/5); combined candidate check passed.
Native Review inspected exactly one test file, Apply confirmation named digest
f1afc77f28f436b3d81a9cdcfdcb031c95635b0771b13cd4fa1d5ca7a401d3b1,
and native UI reported Applied successfully. DB records applied at
2026-09-19T18:14:48.530187900+00:00. All seven candidate inventory hashes
match the destination; independent destination node --test passes 5/5.
Receipt: target/beta-regression-repair-20260919/native-diagnostic-apply-receipt.json.
477 diagnostic snapshot inputs still match. Full runner suite 45770 TERMINAL
exit 0, 151 tests PASS. No build/test processes from this pass remain live.

Prior sharing violation did NOT recur and is NOT proven fixed. Diagnostics only
changed error context; preserve the original failure as an open reliability
finding. Next acceptance: native stale/refreshed review and controlled recovery,
native DPI, clean install and installed upgrade. This successful Apply used the
extracted diagnostic MSI payload, not an installed package. The disposable
example now intentionally has test/risk-policy.test.mjs modified by Apply;
do not reset or erase this evidence.

Registry polling hypothesis did not reproduce the native error: new concurrent
reader/register/remove test PASS. Added operation/path context to registry I/O
errors; no permission or verification-policy changes. Controlled Windows held-
handle test proves error 32 is attributed to replacement and old state survives.
All 4 registry tests and 13 verifier lifecycle tests PASS.

Diagnostic MSI build 64664 TERMINAL exit 0, source snapshot differs from prior
477-file candidate only at crates/pytxo-runner/src/process_registry_file.rs.
MSI target/beta-regression-repair-20260919/registry-diagnostic.msi SHA256
B1E2608F8F2BCA3F130904B0E3B0D0CA14D4518DA7F5DE1FAF92E5109B05FE52.
Extracted diagnostic payload SHA256
A16D5FE4EFBE0975CDD88BB754D089E5F7E7021ABF6E9C6A67F778D706A694D9.
Native window 17302172 runs registry-diagnostic-payload/pytxo-desktop.exe;
old final-payload process closed via UI. No installation or publication.

One Codex task launched as run 2c585a7f-3e5f-4ba7-9521-3ca48045b8ab in the
same Approval risk demo, one owned test file, default npm test verification.
Observed live cmd PID 33000, started 01:10:23 local. Re-poll exact run/process;
do not launch a duplicate. Full runner suite process 45770 is also live; log
registry-diagnostic-runner-tests.log in the evidence directory. Next: read
operation-specific verification outcome, then repair only the proven cause.

## September 20 — native control restored; acceptance in progress

Native run 2aa33320-721a-4687-8f97-9715b9d9a28f completed as verification FAILED.
Codex produced the requested three additional tests in its isolated upper tree;
independent `node --test` there passes 5/5. Original repository tracked diff
remains empty. Agent's own sandboxed node test reported spawn EPERM; separate
Pytxo `verify-failed` database event reports Windows sharing violation os error
32 after `verify: npm test`. Do not conflate these failures. No candidate was
prepared and native Review remained disabled. Investigate verifier lifecycle
registry I/O (`run.rs` VerificationLifecycle and process_registry_file.rs);
exact failing filesystem operation is not yet established. No source fix yet.

Computer Use via node_repl and @oai/sky now works. Opened the exact final
MSI-extracted payload (F70A21A8…1FDB4BC4), not the installed 0.9.0 executable.
Setup > General visibly reports Desktop v1.2.2. Manual update check transitions
through Checking to No newer version; download/install/restart remain untested.
Native New work detects Codex ready and builds one task/one worker under Orbit.
Started a bounded regression-test task in the existing disposable Approval risk
demo; baseline HEAD 142d4e0e63b9d32a57b54eeebb397584a9fd4418, 2 tests PASS,
tracked files clean before run. Native run outcome/Review/Apply still pending.
The previous unavailable-native-control blocker is superseded, not a current
reason to stop. No installation, publication or feed mutation performed.

Observed UX follow-ups: project selector has many indistinguishable `repo`
entries from persisted catalog; generated plan path list also includes paths
the request says to keep unchanged. Neither has been repaired in this pass.

## September 19 — final package ready for native acceptance; blocked handoff

Final build session 58021 TERMINAL exit 0. Final staged MSI, payload hashes,
version/scope inspection, inventory verification and zero-drift source receipt
are in `target/beta-regression-repair-20260919/`; RELEASE_READINESS has identities.
NATIVE_ACCEPTANCE.md updated to final staged MSI (10CEECD4…479EE547), not the
archived pre-format build. No build/test processes remain active from this work.

Automated/source/browser checks and local packaging complete. Native final-artifact
install, real Codex Work/Review/Apply, stale/recovery, DPI and installed update
proof remain missing. Same unavailable-native-control condition recurred over
three continuations; question for access/manual observations remains pending.
Goal blocked, not achieved. Resume when native controls or actual manual evidence
are available; use the exact staged artifact and acceptance checklist. Do not
publish, assume installation, or convert fixture captures into runtime proof.

## September 19 — workspace gates passed; final MSI build live

Full locked workspace tests session 91088 TERMINAL exit 0. Fixed the four rustfmt
wrapping blocks; final format check PASS. All-target workspace Clippy with
warnings denied session 92974 TERMINAL exit 0 (1m09s).

Final 477-file snapshot: target/beta-regression-repair-20260919/candidate-final-source-before.json.
Final `npm run build:msi` live session 58021, log candidate-final-msi-build.log
in that folder. Poll that exact session; do not restart from quiet output.
Previous installer archived as candidate-before-format.msi; existing hashes
refer to it, not the pending rebuilt output. When complete, compare source,
hash MSI and extracted payload, and update NATIVE_ACCEPTANCE.md identity.
Native-access/manual-acceptance question remains pending. Goal unverified.

## September 19 — captures current; workspace gate live

Capture run 38157 TERMINAL 25/25 PASS; earlier 4033 failed on obsolete Agents
heading (fixed). Fresh captures mirrored to docs/site/demo; parity and links
PASS. Focus check 88266 TERMINAL 4/4 PASS: viewport confirms no sticky-header
occlusion. Final affected website run 21866 TERMINAL 15/15 PASS after captures.

`cargo test --workspace --locked -j 2` is live in session 91088, log
`target/beta-regression-repair-20260919/workspace-tests.log`. Do not restart while
live. Format check failed only line wrapping in ipc_voice.rs (three guards) and
store.rs:1109; repair after test finishes and preserve updated source identity.
No runtime source edits yet since MSI. Snapshot drift is captures/capture test.

Native control unavailable; async question for native access/manual acceptance
is pending. Concrete exact-artifact checklist in evidence/NATIVE_ACCEPTANCE.md.
Independent full workspace verification continues; goal not complete or blocked.

## September 19 — MSI complete; website final browser rerun live

MSI session 18232 TERMINAL exit 0. Website builds 96219 and 60053 TERMINAL exit 0.
MSI hash/version/scope and extracted payload identity are in RELEASE_READINESS
and `target/beta-regression-repair-20260919/`. Post-build 477-input hash comparison
shows zero drift. MSI is unsigned, per-machine, not installed. Packaged payload
differs from loose EXE only at MSI/UNK bundle marker; use the packaged identity
for acceptance. Comparison session 27945 TERMINAL exit 0.

Website initial browser run 44966 TERMINAL: 19 PASS / 4 FAIL. Fixed actual tab
keyboard origin bug and updated obsolete presentation assertions. Rebuilt site
successfully. Final full browser run live in session 10740; log
`target/beta-regression-repair-20260919/website-browser-final.log`.
Final run 10740 is now TERMINAL: 23/23 PASS (1.4m). Receipt and inspected
desktop/mobile walkthrough captures saved. Embedded Desktop images are older
than current task-first UI despite parity passing: refresh marketing captures
before launch. Desktop element capture includes sticky-header overlap; inspect
actual viewport/focus before claiming a product defect. Next: fresh product
captures, then exact packaged Windows real-job/Apply/recovery/DPI/upgrade proof.

## September 19 — package build live; update documentation aligned

MSI session 18232 remains live, compiling Desktop after the frontend succeeded.
Do not restart it based on unchanged log text. Interim check of all 477 source
hashes found zero changed build inputs; repeat after packaging completes.

Desktop setup docs now describe the actual movable inspection layout and Review
access from Work. Added candidate-specific updater activity checks, stage failure
recovery, manual-download version caveat and post-reopen version confirmation;
Troubleshooting links to that section. Link check 192 PASS, MDX generation PASS,
asset parity and docs-source checks PASS. No published-version promotion.

Fresh website `npx next build` is live in session 96219, log
`target/beta-regression-repair-20260919/website-build.log`. Read-only asset checks
passed before building; avoided prebuild's asset-copy/delete side effects.
After completion, run the website browser suite against this build and inspect
desktop/mobile captures. Native/installed acceptance remains open.

## September 19 — combined browser repair complete; Rust gate started

The full browser gate subsequently completed with 307 PASS (3.3m), superseding
the 21 failures below. MissionDock scroll anchoring and pane-width responsive
layout now preserve Stop/Review visibility after moving, resizing and reloading
docks. Tests follow current disclosures without weakening authority/visibility
assertions. Focused 24 PASS; Svelte/CSS clean; version consistency PASS.
Evidence: `target/beta-regression-repair-20260919/`, including copied combined
runner receipt, inspected screenshot and source hashes.

Combined locked Cargo tests for core/store/orchestrate/Desktop completed exit 0;
log `target/beta-regression-repair-20260919/combined-rust-tests.log`. Release
verifier tests passed 22/22. Source manifest hashes 477 Git-known build inputs.
Windows `npm run build:msi` started with two Cargo jobs (session 18232), log
`target/beta-regression-repair-20260919/candidate-msi-build.log`.
Inspect the live process/result before restarting. Next is source-bound packaging
and native real-job/Apply/recovery/upgrade acceptance. Goal remains unverified.
No source commit, installation, deployment or publication. Latest generated
desktop mockups are illustrative proposals, not a new implementation mandate.

## September 19 — combined browser gate exposed 21 regressions

Legacy `dispatch_run_cmd` now refuses non-debug Desktop builds before dispatch;
normal and standalone New work use scoped Flow. Existing review/recovery and
external CLI capabilities remain. Four public-doc pages align with unpublished
beta scope and setup recovery; 191-link check and MDX generation PASS, Desktop
library clippy PASS. No publication, installation or commit.

Full production-preview suite TERMINAL: 286 PASS / 21 FAIL, 307 total, four
workers, 6.2m. Session 32231 finished; do not poll/restart it as a live wait.
Failure contexts and runner receipt saved in `target/beta-combined-browser-20260919/`.
RELEASE_READINESS lists the failure groups. Most need current disclosure/label
journeys, but long-mission bottom-dock Stop visibility is a product-layout concern.
Do not weaken the 100% action visibility, identity, evidence or scroll assertions.

Next: resolve this combined regression gate, then source-bound packaging and
native real-job/Apply/recovery/upgrade acceptance. Goal remains active, unverified.

## September 19 — Desktop Flow beta admission implemented

Desktop Flow preview/dispatch now use scoped orchestration entry points for
Codex, one worker, Orbit at domain/worker level and local PTY. Checks precede
dispatch claims and use the same config snapshot as execution. General CLI
Flow/configurations remain unchanged; additional agents may be inspected but
cannot launch from this beta Flow journey. This supersedes the earlier statement
that supported combinations are presentation-only for Flow, not all launch paths.

Flow integration 16 PASS; final beta subset 3 PASS; profile/adapter unit test PASS;
Desktop clippy PASS; browser workflow/menu/admission 26 PASS and capture rerun
1 PASS; Svelte/CSS clean. Blocker screenshot inspected; evidence in
`target/beta-admission-20260919/`, details in RELEASE_READINESS.

Next: non-Flow Desktop launch exposure and beta docs, then combined packaging
and native real-job/Apply/recovery/upgrade proof. Do not label the whole candidate
verified. No product commit, installer invocation, deployment or publication.

## September 19 — onboarding stale-readiness repair

Reproduced and fixed retention of connected/ready state after an agent detection
failure. Rechecks now withdraw old observations, in-flight actions disable
progression, and no-ready users can explicitly defer agent setup. Codex is first
in onboarding. Preview recovery coverage includes absent/signed-out/unknown
agents, detection failure and failed sign-in launch; launch alone never implies
readiness. Combined browser set 10 PASS, final recovery set 5 PASS, Svelte/CSS
clean. Error/compact screenshots inspected; evidence in
`target/agent-setup-recovery-20260919/` and RELEASE_READINESS.

Native vendor authentication and actual Windows lifecycle remain unverified.
Continue supported-runtime boundary review and candidate packaging/acceptance;
do not mistake controlled browser recovery for beta completion. No product
commit, installer invocation or publication. Preserve inherited changes.

## September 19 — beta first-use presentation verified

Codex leads Setup, with installation and vendor-account status separated.
Additional agents/integrations and alternate default permissions expand on
demand; existing defaults and explicit agent choices remain intact. New work
groups the beta starting point separately. This does not enforce a narrower
runtime support boundary.

Affected browser set 23 PASS, vendor-session shell regression 1 PASS, Svelte/CSS
clean. Fresh 1280/860 screenshots inspected and retained with source hashes in
`target/beta-first-use-20260919/`; exact commands/limits in RELEASE_READINESS.
Next: remaining first-use failure/auth states and supported-runtime boundary,
then combined candidate packaging and real native lifecycle/upgrade acceptance.
The beta goal remains active and unverified. Preserve inherited dirty work;
no product commit, installer invocation or publication.

## September 19 — exclusive updater handoff implemented

`pytxo-core::UpgradeGuard` coordinates current-source work and upgrades through
shared/exclusive OS locks for the same catalog home. Runs (including detached
supervisors), fleets, Flow dispatch, Apply/refresh/discard/recovery and legacy
commit hold shared ownership. Desktop native handoff holds exclusive ownership
from post-download preflight through install/relaunch, with token-bound release.
Terminal creation and voice start/resume/finish also participate. All profiles
retain existing permissions; this is upgrade admission, not sandbox authority.

Cross-process contention, abrupt-exit release, run/Flow/mutation refusal and
detached ownership tests PASS; dispatch/fleet/Flow regressions PASS; Desktop 44
tests and final clippy PASS; updater 9 tests and Svelte/CSS PASS. Exact scope and
older-binary/other-catalog limitations are recorded in RELEASE_READINESS.

Next: finish remaining beta UI/first-use alignment and candidate verification,
then current-source packaging, actual MSI/UAC and real Work/Review/Apply/upgrade
acceptance. Exclusive source admission does not close these native/release gates.
No product commit, installer invocation or publication. Preserve inherited work.

## September 19 — updater preflight and launch confirmation

Added native `update_preflight` in `src-tauri/src/update_safety.rs`, wired through
IPC to the shared updater. It observes all catalog domains using read-only
existing stores, unbounded active/preparing/applying/recovery counts, active
fleets, open workspace terminals and voice work. Errors fail closed. Added
local version-request receipt before install and next-launch comparison against
`getVersion`, independent of feed availability. Receipt failure prevents install.
Native updater 2.10.1 source confirms failed installation retains its downloaded
bytes: retries reuse them, with another preflight, instead of leaking resources
through repeated downloads. This supersedes the earlier fresh-download retry note.

Verification: 8 updater tests, all 25 store tests and all 44 Desktop library
tests PASS; Svelte/CSS clean; Desktop library clippy with warnings denied PASS.
See RELEASE_READINESS for exact commands and limitations. No native installer
was invoked. Remaining: exclusive/race-free updater handoff, real upgrade and
receipt acceptance, native UI/error/progress acceptance, and full beta scope.
Do not mistake preflight observations or version-string comparison for those
remaining gates. No commit/publication. Preserve inherited work.

## September 19 — updater lifecycle consolidated

Banner/General Settings now share `update-controller.ts`, `desktop-updater.ts`
and `UpdateControls.svelte`. Error suppression and hidden install errors removed;
download progress, bounded network timeouts, resource cleanup and stage-specific
retry added. The running binary remains the version authority. 19 updater/menu
checks PASS; final padding-only change rechecked with four updater tests and
zero-warning Svelte/CSS checks. See RELEASE_READINESS for proof limits.

Next updater work: authoritative active-work guard and requested-version receipt
on next launch, then real installed-app upgrade proof. Do not use the bounded
history snapshot as an all-runs safety guarantee. Backend IPC `load_desktop_snapshot`
is in `src-tauri/src/ipc.rs`; run reservation/recovery is in
`crates/pytxo-orchestrate/src/lib.rs`. Preserve inherited changes; beta remains
unverified and no publication is authorized.

## September 19 — approval drawer verified in browser

Implementation resumed under the existing beta objective. Legacy workspace
flush requests cannot be approved through the Desktop generic button or shortcut;
they route to separate candidate Review without resolving the request. Denial
stops the request without claiming to discard files. Approvals now use a compact
right-side drawer with pinned actions. No backend Apply authority was changed.

Final combined 24-case approval/layout/DPI/glyph batch PASS; Svelte and CSS checks
PASS. Wide and compact browser screenshots inspected. See RELEASE_READINESS and
`target/approval-drawer-20260919/`. Preserve all inherited work. The candidate is
still unverified: shared updater lifecycle/error handling, remaining beta journeys,
current-source packaging and native/real-job/upgrade acceptance remain outstanding.

## September 17 — visible spatial Review correction

The latest user instruction supersedes the default-collapsed map below. Review now defaults to visible candidate relationships at 900px+ available content width, including dock occupancy. Real prepared files converge on the exact candidate; selection highlights its connection and opens the existing comparison. Three file nodes plus Browse all files bound diagram density without losing the full inventory. Recorded verification opens existing checks; destination details expose the actual read-only run target. Focus on code persists in local storage; narrow content defaults to the compact summary and can explicitly open an accessible list layout. Header spacing and empty inspection-toolbar space were reduced. No successful integration animation was introduced.

Evidence in `target/review-spatial-correction-20260917/`: before-wide and after ready/missing/stale/focused/narrow PNGs, static check, regression logs and receipt. Static PASS (0 errors/warnings, CSS lint); 58 distinct affected tests PASS across the final batch and two locator-only reruns (56 + 2). Tests cover map actions, preference across reload, dock occupancy, keyboard, reduced motion, exact content, stale digest, recovery and existing confirmation behavior. Impeccable layout detector returned no findings. Apply handlers, loadReview and selectPreparedFile are unchanged; protected Core/Tauri/IPC hashes unchanged. Browser fixture captures are not native or release acceptance. Native laptop/DPI and packaged acceptance remain NOT RUN. No publication, commits, website edits or backend changes in this correction.

## September 17 — Review finishing pass

Kept the focused comparison composition. Consolidated verification into one compact summary and moved eligibility into the Apply decision area; removed the repeated verification pipeline. Added an explicit Show candidate map disclosure with recorded files, exact candidate, checks and destination. Increased code/file-label readability, exposed the full Apply destination, retained one contextual Back to Work action and corrected command pluralization. History remains available through main navigation. Exact-content panes remain neutral; no inferred line-change highlighting.

Evidence: `target/spatial-workbench-20260917/finishing/`. Static checks PASS (0 errors/warnings; CSS lint). Focused regression batch 64 PASS; final affected visual/capture batch 11 PASS; final expanded-map/long-content confirmation 2 PASS. These overlap, not 77 distinct tests. Coverage includes stale identity, refresh, recovery, keyboard focus, reduced motion, narrow/light layouts and text enlargement. New substantial-file browser fixture and synthetic long destination stress are explicitly not native execution evidence. Product-asset parity PASS; Review/Apply captures refreshed locally. Impeccable detector completed without findings on the changed Review component.

Core/Tauri/IPC protected hashes and the reviewed Apply request/handler are unchanged. Only the browser preview gained an opt-in substantial-content fixture. No inherited files removed, no staged changes, commits or publication. Native laptop readability, Windows DPI, packaged execution and genuine mission footage remain NOT RUN. This is UI acceptance, not release acceptance.

## September 17 — spatial Work → focused Review refinement

The approved follow-up refines the existing hypervisor direction. Work task
selection highlights only recorded prerequisites/dependents and opens recorded
worker output. Waiting details distinguish recorded state from an unknown
scheduler reason. Candidate selection opens Review. In-memory inspection context
is scoped by run/domain and survives the round-trip; it conveys no authority.

Review now uses the recorded mission title when available, selectable filename
rows, neutral exact-content panes and an expandable candidate/evidence overview.
The destination inventory and four-column status strip are removed. The aperture,
identity disclosure, destination, refusal reason and Apply live together in the
persistent decision area. Inspection tools are collapsed initially in Review.
Existing authorization, refresh, journal and bounded content-read behavior remain.

See RELEASE_READINESS for current checks and
`target/spatial-workbench-20260917/` for the before-state hashes and source receipt.
Native/package proof and a genuine accepted mission video remain unavailable;
website imagery is refreshed browser fixture UI, explicitly labeled as such.

## September 17 — hypervisor UI locally implemented

Approved standalone mockups in `docs/design/mockups/README.md` are translated into
the real Desktop and website. Branch/HEAD remain
`codex/beta-candidate-verification` / `72879702f90f2b74eece888bb117df59608f9b56`;
no staging, commit, push, package or publication. Preserve inherited work.

New presentation primitives: `ExecutionMap.svelte`, `RepositoryBoundary.svelte`
and the pure `execution-topology.ts` projection. Small recorded dependency graphs
are spatial; dense/narrow graphs retain every task in a list. Unknown worker or
check evidence stays unknown. Review separates speculative content from destination,
retains exact-content reading and pinned Apply authorization, and keeps decisions
reachable. Composer, History and local website now share the approved direction.

See RELEASE_READINESS for final browser checks and the source receipt at
`target/hypervisor-ui-20260916/source-checkpoint.json`. That receipt identifies
local source/captures, not a release candidate or reproducible build manifest.
Website images are explicitly browser fixtures, never native execution proof.
The requested visual slice is implemented; native/package/clean-Windows and the
broader Beta release gates remain open. No new architecture work is implied.

## September 16 — Public Beta trust path (local implementation, not release)

Branch remains `codex/beta-candidate-verification`, HEAD
`72879702f90f2b74eece888bb117df59608f9b56`; index empty. Started with 115 modified
tracked and 125 untracked files. No inherited changes were reset or staged.

Phase 1A's stale-review sequence was reproduced in the existing Core fixture
before repair. The required reviewed digest now travels from pinned confirmation
through typed backend/IPC into Apply, checked under the mutation lease before a
new claim. Missing/wrong/stale identity fails closed; B stays ready. Two clients,
changed evidence with identical targets, concurrent Apply and notification-loss
browser paths have regression coverage. Recovery of a previously authorized
journal remains separate. Legacy live-diff Apply is refused. Unknown Review is
neutral rather than a green shield.

Phase 1B is partial: run-specific additional checks use fresh backend preview;
save-reviewed-plan rejects modified checks; preview persists worker authority,
dispatch retains it even after a larger configured limit, and legacy previews
without it require regeneration. Desktop requests one worker without rewriting
configuration. Check inputs survive same-window draft navigation; restoring a
historical request clears unrelated additional checks. Stale plans no longer
claim Ready. Codex is preferred only when no explicit/saved agent is selected.

See RELEASE_READINESS for tests and RELEASE_PLAN for remaining approved work.
The native/package acceptance gate is BLOCKED: native control is unavailable in
this session. No current-source MSI, installed-app journey, clean Windows,
large Desktop/website redesign, pilot, launch media or Product Hunt publication
has been completed. Do not promote this checkpoint into release evidence.

## September 15 — Cline identity correction

- **Source state:** branch `codex/beta-candidate-verification`, HEAD remains
  `72879702f90f2b74eece888bb117df59608f9b56`. This narrow correction is
  intentionally dirty and uncommitted beside the inherited and generated work
  already classified below. No reset, clean, stash, staging, commit, remote
  mutation, publication, upload, release, or deployment occurred.
- **Correction:** the Cline CLI row incorrectly used Cline's older purple
  favicon. `cline.svg` now contains the current transparent robot mark from the
  official `cline/cline` repository at commit
  `94980446c99f24040e9ed7a03e7726be4aea9198`. Pytxo renders the unchanged path
  geometry as a current-color mask: white in Void and dark in Light. The
  adjacent `Cline CLI` label remains the accessible identity, so the full
  wordmark is not duplicated and no opaque tile was introduced. Exact source,
  upstream/local hashes, license, and the trailing-newline normalization are in
  `apps/desktop/public/ade/PROVENANCE.md`.
- **Regression contract:** the focused harness/scroll matrix passed **5/5** at
  its existing 1920x1020, 1280x800, and 860x560 coverage. It now asserts that
  Cline renders through the transparent mask, contains the official robot
  viewBox, and cannot regress to the old `#863bff` bolt. `npm run check` passed
  with zero Svelte/type/style errors, the static SVG safety scan found no
  script, `javascript:`, `foreignObject`, or external reference, the frontend
  production build passed, and scoped diff checks passed.
- **Candidate:**
  `target/ui-audit-2026-09-15/pytxo-desktop-cline-logo-final.exe`, 37,699,072
  bytes, SHA-256
  `F164E774308C8EB1D8CD8086DE1C3D26A64C8453895D29E304366D437E818532`.
  The isolated one-job native release/custom-protocol build passed in 6m19s.
  Exact source-manifest digest
  `7c2ede2070bbcd231de09b246463f5eda5241ccb079a95d2c76c56d1e3d7a9b1`
  and the dirty-source inventory are in
  `target/ui-audit-2026-09-15/source-receipt.json`.
- **Native boundary:** the older spacing candidate was closed through its
  visible window control, then the exact process-qualified candidate above was
  launched and inspected at 1282x802. In the live expanded catalog, Cline's
  white robot mark rendered cleanly beside `Cline CLI` in Void; Light rendered
  the same silhouette in dark ink. No background tile, clipping, stale bolt, or
  layout/scroll regression was visible. Void was restored afterward. No
  authentication, permission, agent-run, verification, Review, or Apply action
  was initiated. Alternate Windows DPI remains untested.
- **Evidence:** supplied before, matched browser after, and exact native after
  captures are under `target/ui-audit-2026-09-15/screenshots/` as
  `user-before-cline-logo.png`, `cline-logo-after-{dark,light}.png`, and
  `native-cline-logo-{dark,light}.jpg`.

## September 15 — Setup support-card spacing follow-up

- **Source state:** branch `codex/beta-candidate-verification`, HEAD remains
  `72879702f90f2b74eece888bb117df59608f9b56`. This narrow UI follow-up is
  intentionally dirty and uncommitted beside the inherited and generated work
  already classified below. No reset, clean, stash, staging, commit, remote
  mutation, publication, upload, release, or deployment occurred.
- **Spacing correction:** the Agent harnesses catalog, MCP/Cloud support card,
  and Permission profile panel previously touched edge-to-edge. The support card
  also inherited outer panel padding around its own padding and placed
  `npx pytxo-mcp` on a wasteful second row. `AgentsScreen.svelte` now gives the
  support card a 12px preceding gap, removes the duplicate outer padding, and
  keeps the command alongside the MCP explanation; `SettingsScreen.svelte`
  gives the following settings group a 14px gap. The narrow layout still stacks
  the integrations and command when required.
- **Regression contract:** the focused browser test now asserts that the support
  card is no taller than 112px at 1280x800 and that both adjacent gaps remain at
  least 10px. The final harness/scroll matrix passed **5/5** at its existing
  1920x1020, 1280x800, and 860x560 coverage. `npm run check` passed with zero
  Svelte/type/style errors, the explicit style check passed, the frontend
  production build passed, and scoped diff checks passed.
- **Candidate:**
  `target/ui-audit-2026-09-15/pytxo-desktop-spacing-final.exe`, 37,699,072
  bytes, SHA-256
  `0D04ED2654FF8D2E5AF278B274E6C2BF691ECA84B424F8E54CEEE3F6EEEC9677`.
  The isolated one-job native release/custom-protocol build passed in 6m54s.
  Exact source-manifest digest
  `3c04b4a9aacdaca5fd2f58bf94ca8fa0bcbef5938b5ce0f7a14d5cc084f04332`
  and the dirty-source inventory are in
  `target/ui-audit-2026-09-15/source-receipt.json`.
- **Native boundary:** the older logo candidate was closed through its window,
  then the exact process-qualified spacing candidate was launched and inspected
  at 1282x802. Its live 6-found / 14-recognized catalog was expanded and scrolled
  through Qwen to the same state as the user report. The support card rendered
  as a compact single row with distinct spacing above and below; the far-right
  scrollbar and fixed navigation rails remained correct. No authentication,
  permission, agent-run, verification, Review, or Apply action was initiated.
- **Evidence:** the supplied before capture and matched browser after capture are
  `target/ui-audit-2026-09-15/screenshots/user-before-integration-spacing.png`
  and `target/ui-audit-2026-09-15/screenshots/integration-spacing-after.png`.
  Native build logs and the exact receipt live beside them. Alternate Windows
  DPI remains untested.

## September 15 — complete local harness identity set

- **Source state:** branch `codex/beta-candidate-verification`, HEAD remains the
  local checkpoint `72879702f90f2b74eece888bb117df59608f9b56`. This bounded
  follow-up remains intentionally dirty and uncommitted beside the inherited,
  separately scoped, and generated files already classified below. No reset,
  clean, stash, broad staging, commit, remote mutation, publication, upload,
  release, or deployment occurred.
- **Three remaining identities:** Antigravity now uses Google's exact transparent
  full-color PNG at 24px; Aider uses the exact transparent 32px pixel mark from
  its official repository at 20px; Grok Build switches between the byte-exact
  white and black SVG logomarks supplied in SpaceXAI's official brand archive.
  The Desktop catalog therefore has transparent identity marks for all **14/14**
  recognized harnesses. No speculative redraw, opaque tile, `.ico`, CSS
  recoloring, crop, mask, or decorative logo background was introduced.
- **Publication boundary:** the Antigravity mark is included only in this local
  review candidate. Google's current product-icon guidance requires compatibility
  use approval; a public candidate must record that approval and required
  attribution or omit the mark. The asset's presence also does not prove runtime
  integration or resolve the separate Antigravity service-terms question. Exact
  source URLs, byte hashes, licenses, and decisions are in
  `apps/desktop/public/ade/PROVENANCE.md`, `THIRD_PARTY_NOTICES.md`, and
  `docs/01-projects/pytxo-final-three-harness-logo-research-2026-09-15.md`.
- **Validation:** the exact asset hashes match the recorded upstream files and a
  static SVG safety scan found no scripts, `javascript:` URLs,
  `foreignObject`, or external references. `npm run check` passed with zero
  Svelte/type/style errors. The final harness-identity and scroll-owner matrix
  passed **5/5**; the focused identity suite then passed **2/2** after adding
  matched middle-catalog evidence. The frontend production build passed. An
  isolated one-job native release/custom-protocol build passed in 5m32s.
- **Candidate:**
  `target/ui-audit-2026-09-15/pytxo-desktop-all-harness-logos-final.exe`,
  37,699,072 bytes, SHA-256
  `FFC0A339446E956EB9D71368C4AC4882C3769BE1821B8427A566EF568D2A31EE`.
  Exact source-manifest digest
  `181d41eb841bdfbfc100c261c35757da30dec6c2e7e415cb73967c409762b15a`
  and the dirty-source inventory are recorded in
  `target/ui-audit-2026-09-15/source-receipt.json`.
- **Native boundary:** the process-qualified exact candidate above was launched
  at 1282x802 and traversed in its real Setup surface. The live catalog reported
  6 found / 14 recognized. Antigravity, Aider, and Grok rendered cleanly without
  tiles in the dark theme; the app was switched to Light and the same three were
  inspected again, including Grok's supplied black theme variant. The Setup
  scrollbar remained flush with the far-right window edge throughout real wheel
  scrolling while both navigation rails stayed fixed. The original Void theme
  was restored afterward. No authentication, agent run, permission change,
  verification, Review, or Apply action was initiated. Alternate Windows DPI
  and runtime mission proof for newly added harnesses remain untested.
- **Evidence:** matched browser captures for the top, middle, and bottom of the
  expanded catalog in both themes are under
  `target/ui-audit-2026-09-15/screenshots/`; the native build log and exact
  receipt live beside them. This follow-up closes the three missing-identity gap,
  not the broader demo, Apply, multi-root, signing, installer, or public-release
  gates.

## September 14 — Impeccable Desktop scroll, identity, and harness follow-up

- **Source state:** branch `codex/beta-candidate-verification`, HEAD remains local
  checkpoint `72879702f90f2b74eece888bb117df59608f9b56`. This follow-up is
  intentionally dirty and uncommitted alongside the inherited/separately scoped
  files already classified below. No reset, clean, stash, broad staging, commit,
  remote mutation, publication, upload, release, or deployment occurred.
- **Setup scroll and scale:** Setup owns one bounded scroll region while its
  navigation stays fixed. The native thumb is now a restrained 4px visual inside
  a 10px hit region on a transparent track; the content gutter is reduced. A
  user-observed fullscreen follow-up found that the scroll owner itself still
  inherited the 960px reading-width cap, leaving the thumb floating inside the
  pane. The cap now belongs to an inner `.settings-content` wrapper while the
  scroll viewport spans the full pane, placing the scrollbar at the far-right
  window edge without stretching the content. Main History controls and
  explanatory text retain the readability increases from this audit. See
  `docs/01-projects/pytxo-ui-audit-2026-09-14.md`.
- **Agent identities:** removed the hardcoded white logo tile and replaced opaque
  favicon treatment with 11 transparent marks across the 14 recognized
  harnesses: Claude Code, OpenAI Codex, Cursor Agent, OpenCode, Gemini CLI,
  GitHub Copilot CLI, Factory Droid, Cline, Goose, Kimi Code CLI, and Qwen Code.
  OpenAI and Cursor use theme-specific official assets; the remaining marks use
  reviewed upstream artwork or exact extracted foreground geometry. No `.ico`
  file is rendered. Antigravity, Aider, and Grok Build remain intentionally
  text-only: no suitable distinct transparent first-party Antigravity mark was
  found, Aider's usable official icon is an opaque tile, and Grok's official
  asset archive was unavailable behind its CDN challenge. Pytxo does not replace
  these with speculative community redraws. Exact source, license, commit/hash,
  and adaptation records are in `apps/desktop/public/ade/PROVENANCE.md` and
  `THIRD_PARTY_NOTICES.md`; the legacy opaque Cursor, Gemini, and OpenCode files
  are preserved but unused.
- **Harness catalog:** 14 harnesses are recognized. Grok Build, Factory Droid,
  Cline CLI, Goose, and Kimi Code CLI add reviewed headless command wiring.
  Antigravity is corrected to `agy -p`. Qwen Code is detection-only and is
  synchronously blocked from Flow dispatch and CLI/shell defaults until its
  approval model is mapped. Amp is deliberately deferred (documented Windows
  support is WSL-only); Grok Bot is a cloud product, not a local CLI. This is
  catalog/detection evidence, not proof that the new CLIs are installed or have
  completed missions. Source review: `docs/01-projects/pytxo-harness-catalog-research-2026-09-14.md`.
  The existing website compatibility table and FAQ now mirror the same 14-entry
  registry, corrected commands, and visible Qwen detection-only caveat.
- **Candidate:**
  `target/ui-audit-2026-09-14/pytxo-desktop-transparent-harness-marks-final.exe`,
  37,625,344 bytes, SHA-256
  `7EC20B974E442A062B2E557704339DC92EA73F382EB006C3AF66B7C952A043B5`.
  The transparent-mark frontend production build passed, followed by an
  isolated release/custom-protocol build with one compiler job. Exact source
  manifest digest `9e16efdc8bf04dfa2ebdba3b3c9dbded980a20f5fbb6fce26b5ea5ae82daac6f`
  and current dirty inventory are in
  `target/ui-audit-2026-09-14/source-receipt.json`.
- **Validation:** Svelte/type/style checks passed; formatting and diff checks
  passed; registry tests **3/3**, Desktop auth/status tests **9/9**, and the shell
  detection-only guard **1/1** passed. The first selected browser run was 83/85;
  both failures were stale assertions (collapsed technical detail and the prior
  outer scroll owner), and the focused rerun passed **17/17**. The final combined
  matrix passed **85/85**, zero skipped/unexpected/flaky. A broader orchestrate test-binary
  build hit a Rust compiler memory-allocation failure; the changed orchestration
  source subsequently compiled successfully in the one-job release candidate.
  `cargo check -p pytxo-cli` passed with one build job. The website production
  build passed (62 static pages); affected ESLint passed and the final homepage
  matrix passed **15/15** at desktop/mobile coverage. After the fullscreen
  correction, Svelte/style checks and the focused Setup matrix passed **3/3** at
  1920×1020, 1280×800, and 860×560. The final harness-identity and scroll-owner
  matrix passed **5/5** across 1920×1020, 1280×800, and 860×560, including dark
  and light themes, all 13 local SVG responses, transparent wrappers, theme
  switching, and zero rendered `.ico` references. `npm run check` and the
  frontend production build passed; the broader 85-check matrix was not repeated
  because the final product changes were limited to Setup identity and layout.
- **Native boundary:** the exact transparent-mark candidate named above was
  launched through its process-qualified path and maximized to a 1536×816
  capture on September 14. Setup reached its live 6-found / 14-recognized
  catalog. The scrollbar was observed flush with the far-right window edge, and
  a real pointer-wheel scroll moved harness content while the primary and
  settings navigation rails remained fixed. The collapsed and expanded catalog
  was traversed from top to bottom; all 11 mark-bearing identities rendered on a
  transparent background, and the three deliberate text-only identities were
  observed without placeholder tiles. This closes the final-candidate relaunch,
  fullscreen scrollbar, and harness-identity gaps. The remaining native gaps are
  alternate Windows DPI and mission/runtime execution for newly added harnesses.
  No agent run, authentication interaction, permission change, verification,
  Review, or Apply was initiated.
- **Evidence:** user-supplied before screenshot and final browser captures are
  under `target/ui-audit-2026-09-14/screenshots/`; dark/light expanded-catalog
  captures, independent audit reports, the native build log, and the exact
  receipt live beside them. Pointer docking, another Windows DPI, runtime proof
  for newly added harnesses, real non-empty Apply, and a complete demo run remain
  outside this follow-up's proven boundary.

## September 14 — native handoff resumed; 4K Remotion capture preparation

- **Saved take / rendered proof:** user replied **stopped**. Recordly Editor
  displayed the actual 7:49 recording. Retained raw MP4, cursor sidecar and
  diagnostics under `target/presentation-pass-2026-09-14/media/raw/` without
  removing originals. Source: 1920×1020, 469.07805s, nominal 60fps / 48.563fps
  average, no audio. Actual app rectangle inspected at 1602×1002; unused canvas
  cropped (not a claim of an independently tested Windows DPI configuration).
- Rendered `media/out/capture-proof-4k.mp4`: **3840×2160, 30fps, 10.000s, 300
  frames**, 1,944,656 bytes; SHA-256
  `B586CD5F4F051DFA7F030EA8E169549545337976A3D4C2F73FDDE23B8FA1C097`.
  Remotion renders real source seconds 30–40 with one camera zoom. Upscaled
  detail is explicitly labeled. Baked-in pointer/highlight retained, no duplicate
  cursor, invented product states or demo completion claim. Editable source and
  capture receipt in `media/edit/`; raw SHA is recorded there.
- TypeScript and full cropped/final video decode passed; contact sheets inspected
  under `media/qa/cropped-proof` and `media/qa/remotion-4k-proof`. Normal-speed
  playback, full-source privacy review, true mission capture and final film remain
  pending. Recordly Ctrl+S did not produce a saved `.recordly` file; the editable
  Remotion project is retained. Native recorder start/stop still needs manual help.
- Capture continuation: user replied **recording**. Recordly's native HUD
  reported REC (00:50, then 01:37); Pytxo's genuine New run → composer
  transition was exercised. Stop failed with unavailable geometry, then a
  refreshed attempt failed the Recordly-versus-WebView hit-test. Automated input
  stopped; user asked to click **Stop** and reply **stopped**, leaving the editor
  open. Recording start is observed, but saved footage/privacy/cursor alignment
  and render remain unverified. No agent job or Apply was initiated.
- The user explicitly said **desktop ready** and requested **4K**, using Remotion.
  This supersedes the earlier “later” pause below. The exact candidate
  `target/presentation-pass-2026-09-14/pytxo-desktop.exe` (SHA-256 `604330B13B45B013439007FA0C81BA563ADC8D8F5CBD2658AF23592F68AC8984`)
  was launched through `launch-candidate.ps1 -DesktopReady` after closing the old
  idle candidate through its UI. The native tool identified the new executable.
- **Observed native:** clean-profile onboarding, existing Claude Code/Codex ready
  states, selection of only `C:\pytxo-disposable-demo-2026-09-14-taskboard`, and the
  empty Work surface at 1282×802. No auth interaction, new agent job, verification,
  Apply, ledger seeding, permission change, or fixture source modification occurred.
  This is limited launch/onboarding evidence, not full native acceptance.
- The active AMD display reports **1920×1080**. Delivery is **3840×2160**; unless
  a genuine higher-resolution source becomes available, captured UI will be
  upscaled. Actual capture dimensions/fps and Windows DPI remain unproven.
- Recordly 1.4.0 launched and **Pytxo Desktop only** was selected, microphone and
  webcam off. Its Record action was rejected twice because the target point
  `(1070, 930)` was over `msedgewebview2.exe`, not Recordly, including after
  reactivation. No bypass/repeated blind clicks. Asked the user to click Record
  once and reply **recording**; native input is paused for that handoff.
- Remotion skill used: isolated editable 4K capture-proof composition under
  `target/presentation-pass-2026-09-14/media/edit/`, using existing 4.0.502 modules
  through a local junction. TypeScript passed. The render guard correctly refuses
  missing genuine footage/receipt. **No MP4 rendered and no recording claimed.**
  Historical `apps/demo-video` source/assets remain untouched. No new package,
  paid service, source commit, upload, publication, or remote operation.
- Updated `docs/demo/README.md` for 4K delivery, source-resolution disclosure,
  Remotion commands and the specific recorder handoff. Prior 234 Desktop and 19
  website checks below remain prior-session evidence; no product source changed
  in this continuation. Pointer docking, non-empty Apply, smaller/DPI acceptance,
  complete raw run and final edit still await testing.

## September 14 — presentation source pass; native/demo handoff deferred

**Current boundary:** the user answered **later** to the native handoff request.
Do not interact with the desktop or launch the new candidate until a fresh
explicit handoff. A native observation returned mismatched visual/window context;
no native input was issued. This is a tested local source pass, not completed
native acceptance, a real Apply demonstration, a finished video, or release approval.

### Preserved source and implementation

- Branch remains `codex/beta-candidate-verification`, HEAD
  `72879702f90f2b74eece888bb117df59608f9b56`. No Pytxo staging/commit or remote
  operation occurred. The prior intended polish and inherited unrelated work
  remain uncommitted. No reset, clean, stash, or blanket source replacement.
- Recoverable starting inventory: `target/presentation-pass-2026-09-14/`
  contains `starting-state.json` (135 incoming dirty/untracked files),
  `inherited.patch`, and 94 source/document backups under `before/`.
  `source-receipt.json` classifies this pass against that inventory; its
  `unexpected` list must stay empty. `current-working-tree.patch` includes
  inherited work and is **not** a stage-all instruction.
- Desktop: `MissionDock.svelte` now mounts destinations before sparse-pointer
  hit-testing and evaluates the release position. A one-move browser regression
  failed before this repair and passes afterward. This is not yet the confirmed
  cause/fix for the previous native relocation failure.
- Desktop: the new `evidence-motion.ts` action uses Motion's free DOM mini
  runtime for interruptible evidence reveal, immediate hiding, cleanup, and
  OS/in-app reduced motion. It excludes native child previews and suspends while
  dragging/resizing. No animation controls Apply or retains outgoing run evidence.
- Review: verification remains visible; exact package identity is one disclosure
  away. Identical blocker text is rendered once while preserving the disabled
  action's accessible description. No backend safety guard changed.
- Website: clearer hero language; explicit Describe → Work → Verify → Review →
  Apply; a local Kokonut-inspired, keyboard-operable Work/Review/History capture
  walkthrough with Motion; truthful modular-scope copy. A measured 21.25px mobile
  tab-change frame shift was corrected and regression-tested. Existing imagery
  is still explicitly labeled browser preview evidence, not new native footage.
- Dependency changes: pinned `motion@13.2.0` in Desktop and website; lockfiles
  updated, unrelated Fumadocs peer/version drift removed. No React component
  port into Svelte. Runtime notices are in `THIRD_PARTY_NOTICES.md` and beside the
  local candidate. Standalone mini entry: 12,060 minified / 4,997 gzip bytes, no
  React rendering inputs; this is not a net application-bundle delta.
- Durable workflow: `docs/08-reference/pytxo-interface-engineering.md`; existing
  machine-local `pytxo-interface/SKILL.md` now routes to it, with a before copy in
  the evidence directory. Its YAML/link validation passed using the existing
  Node YAML package; the Python skill validator lacked PyYAML. No global MCP or
  paid Motion+ setup. Bklit and Anime.js deliberately deferred.
- Source scope: Desktop dock/review/action + related tests/manifests; website
  hero/product/home + walkthrough/tests/manifests; notice/contract/demo/acceptance
  docs and checkpoint. Existing Setup/composer/active-work layouts, backend,
  published download targets, historical video source/assets and unrelated docs
  remain unchanged by this pass. Use the receipt for the exact file list.

### Validation and candidate identity

- `npm run check`: Svelte **0 errors / 0 warnings**, style lint passed.
- Final Desktop production-preview matrix: **234/234**, zero skipped/flaky/
  failed; `desktop-final-tests.json`. Earlier focused runs overlap and are not
  additional unique tests. Two digest-visibility assertions now open the
  disclosure and verify the same exact value; no safety assertion was removed.
- Native release build passed in 11m06s with `custom-protocol`, one build job,
  and the supported main `dataDirectory` configuration for
  `presentation-demo-2026-09-14`. No backend source changed; broad Rust suites
  were not rerun for this presentation-only pass.
- New candidate: `target/presentation-pass-2026-09-14/pytxo-desktop.exe`,
  **37,583,360 bytes**, SHA-256
  `604330B13B45B013439007FA0C81BA563ADC8D8F5CBD2658AF23592F68AC8984`.
  Native-source manifest digest:
  `12928ed070924e13c02ea521b46c2914f6a5b18a70db886b93bf34cf2555274c`.
  See `source-receipt.json`, `tauri-demo-config.json`, and `native-build.log`.
  Reused the inactive `target/local-ux-checkpoint-staged/native` build cache;
  its prior FBBD9238 executable is preserved as `previous-checkpoint-build.exe`.
  The running 82234B6D candidate was not altered or closed.
- Website final production build passed: Next 16.2.12/Turbopack, TypeScript,
  62 static pages, one static-generation worker. Affected ESLint passed.
  Final browser matrix: **19/19**, including mobile frame stability, keyboard,
  reduced motion, evidence links, scope and availability copy. See
  `website-build.log` and `website-final-tests.json`.
- Existing asset checks passed: 27 Desktop source captures, 27 documentation
  references, 10 marketing captures; 45 public MDX sources. This proves asset
  parity, not new native acceptance. No new screenshot or recording was uploaded.

### Demo preparation and resume

- Versioned setup/reset/runbook: `docs/demo/README.md`, `setup.ps1`, `reset.ps1`,
  and `fixture/`. Prepared **only** the disposable
  `C:\pytxo-disposable-demo-2026-09-14-taskboard` repository. Its separate local
  fixture baseline is `6121266e636c0ac733150f024df6a572d6873c22`, tag `demo-initial`;
  it has no remote. This is not a Pytxo source commit.
- Fixture baseline tests **2/2**; CLI dry-run accepts two scoped tasks in two
  dependency stages, no conflicts. Actual browser preflight passed sample tasks,
  add/reload, no JS errors and `.git` refusal. Reset target/sentinel validation
  passed; destructive restore was not invoked. No real agents or Apply ran.
- Recordly, FFmpeg and FFprobe are available. The 10-second capture proof,
  complete genuine source run, editable recording and MP4 are still blocked.
  Prepared ignored `media/raw`, `media/edit`, `media/out`, `media/qa` locations;
  did not create a fake replacement film or alter historical video assets.
- Evidence: `site-before/` and `site-after/` at 1440/390; `website-frame-before.json`
  and `website-frame-after.jsonl`; `evidence-settled-browser.png`;
  `review-after-browser-1280x800.png`; `review-support-before-browser.png`;
  `taskboard-before-browser.png`. Review images differ in scroll state; they are
  not a matched native before/after pair.
- Exact gates and implementation map:
  `docs/01-projects/pytxo-presentation-acceptance-2026-09-14.md`.
  Desktop native readiness, pointer relocation, non-empty Apply and video remain
  BLOCKED by the deferred handoff. Website local preview passes, but final native
  imagery is still needed. Public release gates are NOT TESTED / not ready.

**Next action when the user says “desktop ready”:** resume with this checkpoint
and the demo runbook. Close the old Pytxo window through its UI, then use the
hash-guarded `target/presentation-pass-2026-09-14/launch-candidate.ps1` with its
explicit handoff flag. Verify the new process/profile, native docking both ways,
keyboard/cancel/resize/scoping/reduced motion, and the actual fixture-only
Verify → Review → Apply path. Capture the genuine run, then replace the website's
preview imagery and prepare the local edit. Never Apply to C:\pytxo. Do not
publish, push, deploy, upload, release, purchase, or modify remote state.

## September 14 — final native interface acceptance (local)

- Relaunched the exact final isolated candidate by filesystem path:
  `target/pytxo-interface-polish-2026-09-13/native/release/pytxo-desktop.exe`,
  37,578,752 bytes, SHA-256
  `82234B6D46A0A281D7B398463BEAA8DDE5743EE6D67F6F0B455447EC496D6BA5`.
  Native process inventory reported that same executable path; the installed
  Pytxo copy was not used.
- Completed native visual and interaction acceptance at 1282x802:
  - New run presents the outcome prompt, plan-before-start helper, ready Codex
    agent, and disabled `Build plan` action without clipping or stale docks.
  - History rendered 17 real runs with request-first titles where matched,
    truthful legacy fallbacks, shortened secondary IDs, and the Commit boundary
    summary. Typing `risk summaries` into native search filtered 17 rows to the
    expected 3 and clearing it restored all 17. Accessibility `set_value` was
    unsupported by this WebView field, but ordinary click/type/clear input
    worked; this is a controller-path limitation, not an observed product bug.
  - Review rendered `Nothing to Apply`, `Combined checks: not verified`, and a
    disabled `Apply reviewed changes` action whose accessible reason is `The
    prepared package contains no file changes.` No Apply or Discard action was
    invoked.
  - `Run details` disclosed the full run UUID and closed again. `Files` opened
    in one interaction with its prepared-source context. Returning to History
    hid the unpinned Files view; reopening Review kept it hidden while the
    explicitly pinned cross-project Local preview remained visible and labeled
    `Other project`.
  - The right dock's keyboard-accessible divider resized in native Desktop and
    accepted cancellation. Three bounded pointer-drag attempts (toolbar view and
    open tab to the bottom zone) all produced the application's honest `stayed
    in place` status rather than a relocation. Pickup/cancellation feedback is
    therefore observed, but native destination docking remains unproven; the
    passing browser pointer matrix is not substituted for that missing native
    proof.
- Post-change native captures:
  - `target/pytxo-interface-polish-2026-09-14/after-new-run-1282x802.jpg`
  - `target/pytxo-interface-polish-2026-09-14/after-history-1282x802.jpg`
  - `target/pytxo-interface-polish-2026-09-14/after-history-filtered-1282x802.jpg`
  - `target/pytxo-interface-polish-2026-09-14/after-review-1282x802.jpg`
  - `target/pytxo-interface-polish-2026-09-14/after-review-files-open-1282x802.jpg`
  These ignored files are generated acceptance evidence, not source.
- A safe border-drag attempt did not resize the application window, so a smaller
  native viewport and a second Windows DPI configuration remain unverified. No
  OS scaling setting was changed. The candidate remains open for manual use.
- This acceptance pass changed no Desktop source and did not require another
  source build or browser test run; it exercised the already hash-identified
  candidate whose final source validation is recorded below. Only this receipt
  was updated. No push, publish, release, upload, deployment, Apply, Discard, or
  remote-state change occurred.

## September 13 — interface workflow and demo-critical UX polish (local)

- Continued from the validated local-only checkpoint on branch
  `codex/beta-candidate-verification`: HEAD
  `72879702f90f2b74eece888bb117df59608f9b56` (`checkpoint: preserve
  validated Desktop structural UX`). The branch remains one local commit ahead
  of `origin/codex/beta-candidate-verification`. No inherited dirty file was
  reset, cleaned, staged, committed, or absorbed into this work.
- Added a private reusable Pytxo Desktop interface workflow at
  `C:/Users/mattbaconz/.codex/skills/pytxo-interface/` and routed future Pytxo
  Desktop UI work to it from
  `C:/Users/mattbaconz/.codex/skills/matt-workflow/SKILL.md`. The skill records
  the actual Chroma Aperture language and the Work -> Review -> Apply hierarchy,
  density and scaling rules, dock/drag behavior, state semantics, accessibility,
  reduced motion, native evidence requirements, and the decision ladder of no
  animation -> CSS -> Motion JavaScript -> Anime.js. It explicitly rejects
  decorative motion, generic AI-SaaS cards/effects, and React ports into Svelte.
- Verified current first-party instructions before recommending any interaction
  tool. The evidence note is
  `docs/01-projects/pytxo-interface-upstream-research-2026-09-13.md`, linked from
  `docs/00-meta/MOC-home.md`. Motion's framework-neutral JavaScript API remains
  the preferred candidate only when CSS becomes brittle; Anime.js is reserved
  for exceptional timeline/SVG choreography; Kokonut UI and Bklit UI are
  references rather than source/dependency feeds. No package, Motion AI tool,
  plugin, skill installer, MCP server, or editor configuration was installed.
  The repository's skill validator could not start because PyYAML is absent in
  both available Python runtimes. A deterministic manual validation confirmed
  valid frontmatter, the expected skill name, and all referenced files; PyYAML
  was not installed merely to make the validator runnable.
- Implemented the following bounded Desktop polish without changing the native
  review/apply contract:
  - New run's solo composer can use 920px rather than stopping at 720px, reducing
    the unexplained empty right column and giving agent controls room.
  - History now leads with the saved request title when a draft can be matched
    strictly by dispatched run and execution domain. Project and a shortened run
    ID are secondary; search is labeled for work history and includes request,
    project, and ID. Legacy runs retain a neutral fallback rather than inventing
    titles.
  - Review is named `Review changes`, returns to `History`, leads with project
    context, and places the raw UUID under a closed `Run details` disclosure.
  - Entering or re-entering Review hides persisted unpinned contextual docks by
    default, while preserving their saved placement. Evidence remains one click
    away and explicitly pinned views remain visible. Work restores the saved
    arrangement. The dock shelf is labeled `Inspect`.
  - Pointer dragging now has CSS-only pickup/source elevation, stronger valid
    targets, spatial main-layout response, and live-region success/cancellation
    announcements. A first grid-settle transition was removed after focused
    tests showed it could race an immediate second drag; repeated docking remains
    immediate and interruptible. No animation dependency was added.
- Fresh validation of the final source state:
  - `npm run check`: passed with zero Svelte errors and warnings; stylelint
    passed.
  - Full non-capture production-preview gate:
    `npx playwright test --config playwright.config.ts --grep-invert
    @marketing-capture --reporter=list`: 229/229 passed in 2.9 minutes. This
    includes multiple viewports, reduced motion, 200% text zoom, user zoom, and
    device-scale simulations at 1.0, 1.25, and 1.5. These are browser/WebView
    simulations, not proof of a second Windows DPI configuration.
  - The first full run was 226/229. Two failures asserted the already-obsolete
    pre-checkpoint phrase `One ready CLI is enough`; checkpoint source already
    said `One coding agent is enough`. The third used the Review identity selector
    replaced by the new disclosure. Only those stale/expected assertions were
    aligned. Final review then caught a saved-but-hidden Review view that moved
    without revealing itself when dragged. The move path now explicitly reveals
    the view; the strengthened 24-test Review/docking matrix passed 24/24 before
    the final full 229-test run.
  - Final isolated `npm run build:native`: passed. Exact executable
    `target/pytxo-interface-polish-2026-09-13/native/release/pytxo-desktop.exe`,
    37,578,752 bytes, SHA-256
    `82234B6D46A0A281D7B398463BEAA8DDE5743EE6D67F6F0B455447EC496D6BA5`.
  - Final `git diff --check` and source-scope review passed. No Critical, High,
    or Medium issue remains in this bounded UI slice.
- Native evidence boundary:
  - The previous exact native build
    `target/release/pytxo-desktop.exe` (SHA-256
    `EDD4E5B9C2C5AF34AF26D2E1F3C811D8F805899FFD820C722E646C9AC590B0BE`)
    was visually inspected at 1282x802 before the source changes. Captures:
    `target/pytxo-interface-polish-2026-09-13/before-new-run-1282x802.png`,
    `before-history-1282x802.png`, and `before-review-1282x802.png`.
  - The installed Pytxo copy was rejected when the generic native launcher chose
    it. An intermediate isolated candidate (SHA-256
    `964A55A83430A1DD8DC6BC099C31E79386211534BB5C6C62055A154F924084E0`)
    was then launched by exact filesystem path; the live process inventory
    reported that exact path and the native accessibility tree exposed the
    updated `Inspect` label and New run content at 1282x802. Final diff review
    subsequently produced the narrow saved-view reveal fix above. The final
    hash-identified candidate is the `82234B...` build recorded in validation;
    it was not relaunched after the lock-screen boundary.
  - Post-change visual acceptance could not be completed: the screenshot surface
    switched to the Windows lock screen while the Pytxo accessibility tree
    remained available. Interaction stopped immediately. The file
    `target/pytxo-interface-polish-2026-09-13/native-candidate-interrupted-lock-screen.png`
    records that interruption and is not product-UI evidence. Candidate visuals,
    native pointer dragging, a smaller native window, and a second Windows DPI
    configuration therefore remain unverified; no after screenshot is claimed.
- Deliberately unchanged: Work / History / Setup navigation, Chroma Aperture
  identity, Setup's current three-rail structure, the real agent-progress model,
  all verified-candidate/Nothing-to-Apply/staleness/isolation/Apply semantics,
  and the documented modular-project boundary. Pytxo still does not claim
  end-to-end modular projects without a real multi-root mission demonstrating
  frozen authority, isolated execution, combined verification, drift handling,
  verified Apply, and recovery.
- This slice remains intentionally uncommitted. Its repository-owned paths are
  the five changed Desktop source/style files under
  `apps/desktop/src/components/desktop2/`, the nine affected specs under
  `apps/desktop/e2e/`, the upstream research note, the documentation-home link,
  and this receipt. The private skill has five new/updated files outside the
  repository. All previously classified release, web, demo-video, historical,
  benchmark, capture, and build-output dirt remains intentionally uncommitted as
  documented in the prior section. The new native build and captures are ignored
  generated evidence under `target/`.
- No push, publish, release, upload, deployment, account mutation, or other
  remote-state change was performed.


## September 13 — validated Desktop source checkpoint committed locally

- Created local-only commit
  `72879702f90f2b74eece888bb117df59608f9b56`
  (`checkpoint: preserve validated Desktop structural UX`) on
  `codex/beta-candidate-verification`. Its parent is
  `eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78`; its exact tree is
  `3d1e5878c9d3eaf962c7816052835d6ba29d68f3`.
- The commit contains 119 validated paths: Desktop source, native support,
  production UI assets, Desktop tests, the supporting core/planner/runner/
  orchestrate changes, and the directly applicable interaction/modular-safety
  notes. It does not contain release, marketing-site, demo-video, capture,
  benchmark-result, build-output, or handoff-receipt files.
- Final index audit before commit: zero unstaged overlays on candidate paths,
  zero forbidden staged paths, `git diff --cached --check` passed, and
  `cargo fmt --all -- --check` passed.
- Minimum post-staging validation passed:
  - `npm run check`: zero Svelte errors and warnings; stylelint passed.
  - Focused Desktop journeys: 46/46 passed across first use, mission docks,
    review hierarchy, and review state.
  - Affected Rust libraries: 248/248 passed (`pytxo-core`, `pytxo-desktop`,
    `pytxo-orchestrate`, `pytxo-planner`, and `pytxo-runner`).
  - Orchestration flow integration: 10/10 passed.
  - Focused multi-root run identity: 1/1 passed.
  - Affected-package Clippy with `-D warnings`: passed.
  - `npm run build:native`: the first attempt reached Cargo but could not
    replace the running, locked `target/release/pytxo-desktop.exe` (Windows
    error 5). The same command then passed in an isolated target directory;
    the resulting executable SHA-256 is
    `FBBD92387D82B398F6CD164DECB119FBB1DC3BDE5AC5C55A17126701F445FE62`.
- The following 119 working-tree entries remain intentionally uncommitted.
  They are inherited or separately scoped work, except for this post-commit
  receipt and the explicitly identified generated evidence:
  - post-commit receipt: `CHECKPOINT.md` (it also retains its inherited
    handoff history);
  - root/release/control work: `DEMO.md`, `README.md`, `RELEASE_PLAN.md`,
    `RELEASE_READINESS.md`, and `PYTXO_ASTRA_MASTER_GOAL.md`;
  - demo-video work: every remaining modified or untracked path under
    `apps/demo-video/` (README/package/validation/composition sources,
    Aperture sources and props, and its product image);
  - marketing site and public docs: every remaining modified or untracked
    path under `apps/web/`, excluding the generated captures listed below;
  - older design/release notes:
    `docs/01-projects/astra-evidence-2026-09-07.md`,
    `docs/01-projects/astra-execution-2026-09-07.md`,
    `docs/01-projects/astra-product-decisions-2026-09-07.md`,
    `docs/01-projects/astra-release-proposal-2026-09-07.md`, and every
    remaining path under `docs/superpowers/`;
  - release-verification tooling:
    `tooling/scripts/verify-release-version.mjs` and
    `tooling/scripts/verify-release-version.test.mjs`;
  - generated evidence: `desktop-e2e.log`, every PNG under
    `apps/web/captures/site/`, every PNG under `apps/web/scripts/.verify/`,
    and every current `astra-*.json` under `tooling/benchmarks/results/`;
  - ignored build/test output: `apps/desktop/dist/`, the existing `target/`
    outputs, and `target/local-ux-checkpoint-staged/` (including the isolated
    native build and focused Playwright output).
- No push, publish, release, upload, deploy, or other remote-state mutation was
  performed.


## September 13 — structural Desktop correction verified locally

- Started from the stabilized inherited baseline below; no inherited source was
  reset, cleaned, staged, or discarded. Current identity remains branch
  `codex/beta-candidate-verification`, HEAD
  `eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78`. The dirty tree now has 190
  porcelain entries because this pass added three documentation/evidence paths.
- Wrote the severity-ordered interaction diagnosis in
  `docs/01-projects/desktop-interaction-audit-2026-09-13.md`. The highest-risk
  defect was epistemic: Review advertised **Ready to Apply** while combined
  candidate checks were unverified. Wrong-context task docks were the next
  structural defect. Raw-ID History, mixed vocabulary, draft density, and
  advanced controls remain follow-up redesign work rather than being hidden by
  superficial polish.
- Review now has distinct `verification_required` and `nothing_to_apply`
  presentation states. A non-empty unverified package offers **Verify
  candidate** and cannot open Apply; only a verified non-empty candidate may
  Apply. Empty packages say **Nothing to Apply** and keep Apply disabled. The
  native backend's existing authorization and freshness checks remain intact.
- Task docks are contextual. Unpinned saved views render only for their matching
  run, while explicitly pinned views retain their source identity. New run,
  History, and Setup render neither the task-view shelf nor stale dock panes.
  Saved layout references are preserved. New run also omits unavailable voice
  controls and uses **Project** for the core selector.
- Exact browser core matrix: 140/140 passed in 2.6 minutes. Focused structural
  contracts: 15/15 passed. Secondary Review/UI/polish matrix initially exposed
  four obsolete visual assertions (optional chip detail and an old planned agent
  alias); the assertions were aligned with the recorded-agent contract, and the
  final combined secondary matrix passes 32/32 in 1.0 minute. Full evidence is
  recorded in `target/flat-02-plain-work/STRUCTURAL-VALIDATION.md`.
- `npm run check` passes with 0 errors / 0 warnings and CSS lint passes.
  `git diff --check` passes. The existing two-root orchestration primitive test
  passes 1/1, but it only proves shared run ID plus distinct root/worktree IDs.
- Exact final native artifact: `target/release/pytxo-desktop.exe`, 37,578,752
  bytes, SHA256
  `edd4e5b9c2c5af34af26d2e1f3c811d8f805899ffd820c722e646c9ac590b0be`.
  The exact process was inspected at 1282×802 against its real local ledger:
  New run had the direct outcome composer with no unavailable voice controls;
  Setup and History had no task dock; Work retained scoped views; the real empty
  prepared package said **Nothing to Apply** and its Apply action was disabled.
  Fresh-profile native onboarding, native DPI variation, and a real non-empty
  native candidate remain untested; browser fixtures are not substituted for
  those claims.
- Defined the required multi-root mission/candidate/verification/Apply/recovery
  contract in `docs/01-projects/modular-project-safety-contract-2026-09-13.md`
  and corrected `docs/06-product/modular-projects.md`: Pytxo currently has
  multi-root execution primitives, not end-to-end modular projects. No multi-root
  enablement was added to Desktop and no such capability is claimed.
- No commit, push, upload, publication, release, deployment, account change, or
  production mutation was performed.


## September 13 — inherited handoff stabilized; native UX audit next

- Verified checkout identity before product changes: branch
  `codex/beta-candidate-verification`, HEAD
  `eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78`, 187 porcelain entries
  (119 tracked modifications, 68 untracked entries), zero staged. All 14 recorded
  source hashes matched. The handoff JSON's first status string incorrectly lacks
  the unstaged leading space for `CHECKPOINT.md`; its prose and the live index both
  confirm no staged changes, and the remaining counts/paths match.
- Reproduced the inherited four failures with a temporary copy of the obsolete
  onboarding assertions: vendor login/recheck and missing-Git recovery at 1440px
  and 390px all time out on the removed `Continue with Desktop` interstitial.
  Classified all four as stale tests. The temporary spec was removed. No product
  behavior was changed to satisfy obsolete assertions.
- The exact current 139-test handoff matrix passes 139/139 in 2.5 minutes. Current
  `npm run check` passes with 0 errors / 0 warnings and CSS lint passes. Modular
  manifest tests pass 3/3; Flow authorization/freshness/dispatch integration tests
  pass 10/10.
- `git diff --check` exposed four whitespace-only lines in `WorkActive.svelte`.
  They were removed without behavioral change; final diff check passes. This is
  the only source change made while stabilizing the handoff.
- Exact post-repair native build passes. Artifact:
  `target/release/pytxo-desktop.exe`, 37,578,752 bytes, SHA256
  `49904602792edcc9b9e4feb0092f8d5db3330907d08a50658b5c2fefc5804694`.
- Full evidence and limits: `target/flat-02-plain-work/PHASE1-BASELINE.md`.
  This is a trustworthy local baseline, not release, installer, clean-Windows,
  real multi-root mission, or publication evidence. Phase 2 must inspect this
  exact running native artifact before any structural Desktop change.


## September 13 - STOPPED: plain-language task workspace handoff

**Latest instruction:** Matt explicitly stopped implementation and requested this
handoff. No further UI edits, builds, new test runs, desktop input or publishing
were performed after that request. The already-running browser suite was allowed
to finish and its output was collected. Resume implementation only when the next
session is asked to resume. Do not treat this checkpoint as automatic permission
to continue after the stop.

### Repository and candidate identity

- Actual checkout: `C:\pytxo`; branch `codex/beta-candidate-verification`.
- HEAD `eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78`, subject
  `fix: remove Windows runtime prerequisite and verify final candidate`.
- 187 porcelain entries at handoff: 119 tracked modifications and 68 untracked
  entries; no staged changes. These include extensive earlier Desktop, Rust,
  website, docs and demo work. **This session did not create all that work.**
  Preserve it all; do not reset, clean, stage everything, or replace with HEAD.
- Exact dirty inventory, worktree listing and current slice source hashes:
  `target/flat-02-plain-work/handoff-state.json`. Hashes identify the stopped
  source, not a verified build. `before/` contains pre-slice component copies and
  pre-edit copies of 14 affected e2e files; it is not a complete checkout backup.
- Other registered worktrees: `C:\pytxo-v1-1`, `C:\pytxo-v1-1-1`, and the stale
  prunable `C:\Users\mattbaconz\.config\superpowers\worktrees\pytxo\pytxo-desktop-2`.
  Do not prune or modify these as part of resuming Desktop work.
- No new specialists were spawned. Last team listing showed only root running
  and `preview_boundary_review` completed. There was no other observed active
  source writer in this team. External writers cannot be ruled out; recheck.
- Current HEAD matches the recorded PR #31 baseline. GitHub was NOT reread in
  this handoff. The September 12 record says PR31 open/draft with 12 successful
  jobs in run `34181828835`; it supersedes the old universal CI-billing blocker.
  Do not assert current GitHub/allowance/public status from that historical read.
  No hosted Actions were triggered here, and current dirty source has no CI run.
- Current source is newer than `apps/desktop/dist` (last build at local
  2026-09-13 16:32:55) and newer than the last native executable. **No final
  combined verification, new native build, frozen RC, MSI or public release was
  produced for the plain-language slice.** No commits, pushes, spending,
  dependency installation, account/security/global changes or publication.

### Implemented in the interrupted plain-language session (local source)

1. `WorkActive.svelte`: actual saved request title is the dominant heading, with
   project context below. Full request is an expandable disclosure. Added a
   plain-language progress/next-action summary. Removed the front-door technical
   stage strip, visible run IDs, wave counters and isolation machinery.
2. Retained precise Starting/Running/Completed status chips after the initial
   test caught Starting being flattened into Working. Starting/pending and
   failed startup have distinct summaries. Unknown states do not claim agents
   have reported back. Existing partial-Apply, preparation/recovery errors and
   pending decisions remain visible. Stop confirmation and domain targeting stay.
3. Run IDs and switching live inside `Other runs & IDs`. The focus shortcut now
   focuses the task heading rather than a tab hidden inside a closed disclosure.
   Review is consistently named `Review changes` in Work and BoundaryPanel/History.
   No Apply authorization or freshness logic was removed.
4. Work has a one-click `View details` action. Dock toolbar/new view title is
   `Checks & details` (internal kind remains `evidence`). The technical receipt,
   candidate verification, IDs, ownership, paths and recovery remain available.
   Existing user-saved view titles are not rewritten. `Return to mission` became
   `Return to task`. Read-only output source help uses the new details label.
5. Main boundary copy is concise and profile-aware. The **last unverified edit**
   requires both workspace and Apply surfaces to report `enforced` before saying
   changes stay separate until Apply. Host-direct/non-flushable/unknown cases do
   not borrow that assurance. The **last unverified edit** also requires manifest
   v3, candidate receipt v1 and a nonempty all-passing check set before claiming
   saved checks passed; failed checks have a visible negative summary.
6. `RunLedger.svelte`: `What the agents are doing`; Task / Agent / Status columns;
   scheduling groups shown as Step, with reported-back counts. Exact stored
   launcher display name, or neutral Agent N fallback. Paths and exit details
   moved to the existing row inspector. Row targeting, receipts and J/K navigation
   retained. Narrow columns reflow without the old fixed width.
7. Onboarding now explains coding agents -> project -> reviewed changes without
   requiring the banned internal vocabulary. Stages read Welcome / Agent /
   Project / Ready. Welcome and Ready copy are direct; agent step describes using
   the existing account; project step says Choose your project. No duplicate
   logo/icons were introduced; anchored actions and necessary scrolling retained.
8. `FlowScreen.svelte`: plain request introduction, `What should Pytxo do?` input,
   Project & coding agent label, Step groups, `Use as new request` for old drafts.
   Permissions/execution/isolation and verification commands are disclosed under
   `Permissions and technical details`. Warning messages and blockers remain
   outside it; warning codes have their own disclosure. No missing-check,
   stale-plan, dispatch or permission gate was weakened. **Last unverified edit:**
   padding/focus styling for the technical disclosure.
9. `DesktopShell.svelte` and `lib/mission-selection.ts`: remember explicit run
   selection under `pytxo-selected-work-v1` as domainId/runId only. Reopen resolves
   both IDs against the fresh snapshot and respects the project-picker startup
   preference. No processes, terminal input permissions, plans or review authority
   are persisted. Project switching records that project's selection. This
   addresses the earlier native mismatch of empty central project vs saved dock.
   Browser reload regression passed; native restart has NOT been inspected.
10. Updated accessible-name assertions across the 14 e2e files recorded in
    `before/e2e/`. Added `e2e/first-use.spec.ts`: jargon-free welcome and task front
    door at 1280x800 and 860x560, dominant request heading, one-click actual scoped
    checks, technical disclosure/stale-plan blocking, exact two-key restoration
    and reload without execution authority. Existing security assertions remain.
    Legacy beta onboarding tests were adjusted to the already-existing four-stage
    journey before the stop, but these latest test edits have not run.

### Exact verification results and their limits

All browser commands below ran from `C:\pytxo\apps\desktop`, production-preview
backend, two workers. These are browser fixtures, not real agent execution.
Do not add overlapping matrices together or call `final-tests/` a final pass.

- First `npm run check`: exit 0, Svelte 0 errors / 8 unused CSS warnings; CSS lint
  passed. Warnings were then removed. Second and third `npm run check`: exit 0,
  Svelte **0 errors / 0 warnings**, CSS lint passed. Most recent check session
  `59662` predates the final Work summary/receipt safeguards and Flow styling.
  Those last edits are **not checked** at the stopped source.
- Initial browser command:
  `npx --no-install playwright test e2e/shell.spec.ts e2e/mission-dock.spec.ts e2e/epistemic-state.spec.ts e2e/flat-onboarding.spec.ts e2e/review-hierarchy.spec.ts e2e/menu-workflow.spec.ts e2e/workspace-fit.spec.ts e2e/agent-identity.spec.ts --workers=2 --reporter=line --output=../../target/flat-02-plain-work/tests`
  Session `75085`: **111 passed, 13 failed, 124 total, 3.5m, exit 1**.
  Failures: Starting and Running label regressions (2), run switch newly behind
  disclosure (1), old onboarding heading/credential-copy expectations (4), and
  six old ledger-heading capture expectations. State distinction was restored;
  label/disclosure assertions were updated. The follow-up covered these repairs.
- Follow-up browser command:
  `npx --no-install playwright test e2e/first-use.spec.ts e2e/shell.spec.ts e2e/mission-dock.spec.ts e2e/epistemic-state.spec.ts e2e/flat-onboarding.spec.ts e2e/review-hierarchy.spec.ts e2e/menu-workflow.spec.ts e2e/workspace-fit.spec.ts e2e/agent-identity.spec.ts e2e/beta-workflow.spec.ts --workers=2 --reporter=line --output=../../target/flat-02-plain-work/final-tests`
  Session `66006`: **135 passed, 4 failed, 139 total, 4.8m, exit 1**. Finished
  after the stop request; result collection only, no new suite was launched.
  Four failures are beta-workflow's two onboarding tests at 1440 and 390: vendor
  login/recheck and missing-Git recovery. All time out on the obsolete
  `Continue with Desktop` step. It had already been removed from the current
  onboarding before this slice. Tests were changed before the stop to follow
  Agent -> Project -> Ready and updated copy; **rerun still required**. The
  in-flight report may show current line text around an older loaded test; the
  locator error and captured error-context record the actual failed operation.
- The seven new first-use tests passed in that 139-test execution, including
  selected-task restoration. Dock/keyboard/freshness, review hierarchy, agent
  identity and responsive History/workspace checks also passed there.
- No new Rust test/build, native EXE/MSI, real ADE dispatch, native screenshot,
  clean Windows acceptance, final demo/Bench export or current-source CI here.
- Two screenshots were visually inspected: 1280 active-task and 860 welcome
  listed below. Other generated captures were not all manually inspected.
- A timed first-time-human comprehension test has NOT been conducted. Tests only
  validate the language/hierarchy contract; do not claim ten-second usability
  has been empirically proven.

### Remaining UX defects / review points

- Finish the stopped verification before calling this slice done. Latest source
  differs from preview/native artifacts. Four amended beta tests remain red until
  rerun, and other accessible-name tests outside the selected matrices need their
  affected checks; do not weaken assertions to get green.
- At 860x560 the welcome's third explanatory row is below the anchored footer;
  necessary body scrolling exists, but first-view completeness needs review.
  At 1280x800 with the long request, the waiting-decision strip is near/below the
  fold (the AppBar Approvals action stays visible). Prioritize actual decisions
  without making proof panels dominate the task again.
- Long saved titles can end mid-word because stored titles are truncated; full
  request is reachable. Old runs lacking a linked Flow draft show Your coding
  task rather than a meaningful request. Do not invent a title from agent IDs.
- Other runs & IDs, task IDs such as plan/ui/tests, and remaining CLI/project
  terminology still warrant polish. Technical details intentionally retain their
  exact vocabulary. Do not hide failure reasons to satisfy a text-only jargon test.
- Review `RunLedger` reported-back counting: it still derives settlement from
  state tone; arbitrary unknown states must not be promoted to successful results.
- Task restore needs native restart, missing/deleted project and project-picker
  acceptance. It restores references only, not live sessions. Older saved docks
  may intentionally point elsewhere; preserve their scope rather than relabeling.
- The no-onInspect fallback still renders BoundaryPanel directly to preserve its
  warning behavior; the simplified one-click composition is the native dock path.
- Full approved 46-interface inventory, coordinator/control-session integration,
  and coherent multi-folder delegation remain unfinished. Do not declare the
  whole-product goal complete based on this visual/front-door slice.

### Exact multi-folder / modular-project state (source read at handoff)

- Desktop can create a modular-project manifest by adding a second folder to a
  single-root workspace, list roots, attach additional folders and detach
  non-primary roots from WorkspaceSettingsPanel through native IPC. This is
  metadata grouping; it does not trust a folder or alter permission profiles.
- `workspace_project.rs` rejects missing/file/duplicate/nested/overlapping paths
  and conflicting folder names, checks collisions against effective custom root
  labels, and uses create_new to avoid overwriting an existing manifest. Tests
  exist for preservation/no grant, invalid roots and custom-label collision;
  none were rerun in this plain-language slice.
- **Desktop coordinated multi-folder execution is NOT wired.** Flow passes
  `project_id: null` with one selected domain, labels project entries primary
  folder, and refuses restoring a project draft as ordinary single-domain work.
  `flow.rs::validate_task_claims` explicitly rejects task.root: Flow previews are
  limited to one execution domain. Do not remove this guard to make the UI appear
  supported.
- The separate CLI `project_run` path exists: it reads a ProjectManifest and runs
  the plan on the primary domain with root-label context. This code path is not
  evidence that Desktop dispatches linked per-folder jobs or that every root has
  reviewed atomic Apply. Current reviewed Apply remains one repository/domain.
- Next product work is explicit linked-folder dispatch/status/selection with each
  folder retaining its own preview, permissions, verification, review/Apply,
  affected-path freshness and recovery. Cross-root atomic Apply is deferred.
  No native end-to-end multi-folder acceptance was obtained here.

### Native status, blockers and evidence lineage

- Last returned native window (read-only list succeeded): ID `12912166`, title
  `Pytxo Desktop - Agent inspection validation` (actual title uses an em dash),
  executable `C:\pytxo\target\flat-02-identity\pytxo-agent-inspection-final.exe`.
  SHA256 `89408caedf63b10a0a3fb6cf4883eca31569e3f388d1b4e64ee7ca5fd7e76730`.
  Profile identifier `com.pytxo.flat02.menus`. Refresh window inventory before
  later input; do not assume a saved HWND remains valid.
- That EXE predates every plain-language edit above. Last verified native capture:
  `target/flat-02-identity/native-final-recorded-workspace.jpg`; shows actual
  historical custom-agent record, unknown launcher and full recorded workspace.
  Known ADE identity has prior browser/Rust evidence, not a fresh native ADE run.
- Earlier native fit captures:
  `target/flat-02-supervision/native-workspaces.jpg`, `native-history-long.jpg`,
  `native-history-short.jpg`, `native-agent-output.jpg`, `native-agent-activity.jpg`.
  These belong to EXE D228FD91 and real persisted historical test-run records.
  The older text saying native capture was stopped is superseded by the later
  resumed acceptance entry/README. Preserve chronological build distinctions.
- There is no current observed Computer Use outage: list_windows succeeded this
  turn. Prior captures sometimes showed another foreground app, stale UIA/geometry,
  or fragmented glyphs. Reobserve/activate the uniquely returned Pytxo window;
  bounded retry only. Prior physical Escape stopped control and later user
  explicitly resumed it; the latest stop now pauses all further testing.
- Use only Computer skill/@oai/sky through node_repl for native control. No OS
  security/settings automation or browser-policy bypass. Actual Windows 125/150%
  scale and minimum native size remain unverified; browser viewport/text zoom is
  not a substitute. Exact clean-Windows installer acceptance remains open.

Browser fixture evidence for this slice (before last source safeguards/styling):

- `target/flat-02-plain-work/final-tests/first-use-the-requested-ch-042aa--are-one-click-away-at-1280/active-task.png`
- Same directory: `task-with-checks.png`.
- `target/flat-02-plain-work/final-tests/first-use-the-requested-ch-274ef-s-are-one-click-away-at-860/active-task.png`
- Same directory: `task-with-checks.png`.
- `target/flat-02-plain-work/final-tests/first-use-first-use-explai-aad4d-internal-vocabulary-at-1280/welcome.png`
- `target/flat-02-plain-work/final-tests/first-use-first-use-explai-82f9a--internal-vocabulary-at-860/welcome.png`
- Both test output directories retain failure `error-context.md` files. Evidence
  paths named final-tests are historical output names, not an acceptance verdict.
- Existing approved design packet: `target/pytxo-flat-desktop-02/PROPOSAL.md`;
  illustrative menu board `target/flat-02-menus/mockup-board.png`. Neither is
  running-app evidence. Do not regenerate or re-request the same design approval.

### Files to start from next session

All paths below are relative to `C:\pytxo`:

- Layout and composition: `apps/desktop/src/components/desktop2/DesktopShell.svelte`,
  `WorkActive.svelte`, `RunLedger.svelte`, `MissionDock.svelte`,
  `desktop2-shared.css`; `apps/desktop/src/lib/dock-layout.ts`,
  `mission-selection.ts`; `apps/desktop/src/app.css`, `deck-scroll.css`.
- Proof/output/preview/terminal: desktop2 `DockInspection.svelte`,
  `AgentInspector.svelte`, `BoundaryPanel.svelte`, `PreparedDockFile.svelte`,
  `RunReviewScreen.svelte`, `LocalPreview.svelte`, `WorkspaceTerminal.svelte`;
  `lib/recorded-output.ts`, `local-preview.ts`, `workspace-terminal.ts`.
- Projects/workflow: desktop2 `WorkspaceSettingsPanel.svelte`,
  `WorkspacesScreen.svelte`, `FlowScreen.svelte`, `HistoryScreen.svelte`,
  `AppBar.svelte`; `lib/ipc.ts`, `desktop-backend.ts`,
  `desktop-backend.preview.ts`, `types.ts`, `composer-draft.ts`;
  `apps/desktop/src-tauri/src/ipc.rs`, `workspace_project.rs`,
  `crates/pytxo-orchestrate/src/project.rs`, `flow.rs`, `lib.rs`.
- Onboarding: `apps/desktop/src/components/setup/SetupWizard.svelte`,
  `SetupShell.svelte`, `SetupStepFrame.svelte`, and SetupStep*.svelte.
- Regression entry points: `apps/desktop/e2e/first-use.spec.ts`,
  `mission-dock.spec.ts`, `workspace-fit.spec.ts`, `flat-onboarding.spec.ts`,
  `beta-workflow.spec.ts`, `shell.spec.ts`, `epistemic-state.spec.ts`,
  `review-hierarchy.spec.ts`, `menu-workflow.spec.ts`, `agent-identity.spec.ts`.
- Planning: `PYTXO_ASTRA_MASTER_GOAL.md`, `RELEASE_PLAN.md`,
  `RELEASE_READINESS.md`, `docs/01-projects/astra-flat-desktop-02.md` and
  `astra-ui-feedback-2026-09-13.md`. Their prior-completion claims have dates;
  this section is the current handoff, not a reason to restart broad research.

### Must not regress / next action

Preserve Chroma Aperture, mission/task as the central object, contextual real-data
proof within one click, approved movable/resizable/hideable right/bottom docks,
keyboard controls, named layouts/reset and per-domain/run/agent identity. Preserve
read-only agent output vs explicitly created/enabled interactive user terminal,
terminal-edit freshness invalidation, scoped Stop with confirmation, visible
failures/partial rollback/recovery, exact immutable review bytes and Apply guards.
No fake live identity, completion percentages, provider support, authentication,
combined verification, model calls or demo proof. No discarded user layouts/drafts,
folder data, integrations or unrelated edits. No trust grant from grouping.

Keep interactive agent TUIs, arbitrary nested splits, floating windows, marketplace,
full-app-termination session survival, replacement IDE and cross-root atomic Apply
out of this scope. Existing layout approvals stand; spending, substantial new
libraries, security/account changes, commits/pushes/merges and publication retain
applicable gates. Do not repeatedly ask about the same approved design.

**Single next action after a resume request:** verify the stopped plain-language
slice (latest `npm run check`, amended beta tests, affected Work/Flow/docking and
selection tests), resolve remaining failures without weakening contracts, then
build and inspect the exact native source. No new approval decision is pending.
After that continue linked-folder/control-session work, integrated frozen RC,
clean Windows exact-installer acceptance, matching actual MP4/Bench/site/docs,
and explicit approval before publication/download verification.

## September 13 — native fit accepted; recorded launcher/workspace slice

Resumed authorized native inspection of D228FD91: actual workspace catalog fits
beside its right dock; wide History shows 17 stored runs without an outer page
scrollbar, and filtering to one run removes the list scrollbar while retaining
needed receipt scrolling. Exact historical agent output/empty Activity inspected.
These are persisted historical test-run records, not a fresh live ADE success.
Native captures and lineage: target/flat-02-supervision/README.md.

Implemented the next bounded supervision slice: AgentDto exports optional exact
registry launcher identity from its saved command, never raw argv or an inferred
task alias, plus recorded workspace path. Dock and Task & receipt show these
facts; custom commands remain Not identified. Paths wrap and the project-root
row uses full width. Native review corrected Execution folder wording to Recorded
workspace because persisted paths can be review sources, not live process cwd.
No runner/orchestrate, permission, review/Apply or freshness behavior changed.

Final: 18 browser tests, 1 Rust identity contract test, Svelte (zero errors and
warnings), CSS lint and voice-enabled native release build passed. All relevant
checks rerun after the wording/field correction; matrices overlap. Final native
EXE target/flat-02-identity/pytxo-agent-inspection-final.exe, SHA256
89408caedf63b10a0a3fb6cf4883eca31569e3f388d1b4e64ee7ca5fd7e76730.
Final native screenshot verifies unknown launcher and full recorded workspace
against exact historical agent. Source hashes match, app left open. No installer,
frozen RC, release or publication. Known launcher display is Rust/browser-tested;
a native real ADE/live session has not been dispatched or claimed.

Next bounded product slice: bind coordinator/control/session records and route
modular-project selection into linked folder runs, each retaining its own domain
review, Apply, freshness and recovery boundary. No cross-root atomic Apply or
interactive agent TUI. Existing adapter, Windows scaling/minimum-window, exact
clean-installer, final MP4/Bench/site/docs and explicit publication gates remain.
No human approval blocks continued local work; no extra design approval needed.
Existing dirty work preserved; no spending, dependencies, global/security changes,
commits, pushes or hosted Actions. Evidence: target/flat-02-identity/README.md.


## September 13 — workspace fit and recorded agent activity

Implemented the next bounded local slice: workspace catalogs reflow against their
available width into labeled rows, with wrapped paths and accessible actions.
History uses the available content height, with no artificial bottom-scroll area;
short lists do not scroll, overflowing wide lists scroll internally, and narrow
layouts use one scrolling list/detail region. Added agent Activity from exact
stored domain/run/agent events, separate from worker stdout/stderr, with recorded
status. Scrolling up pauses output following; Follow output returns to the end.
No execution, permission, Stop, review, Apply or freshness contract changed.

Final source checks: Svelte zero errors/warnings and CSS lint passed. Combined
suite: 75 passed, one assertion needed the new compact Approvals label; final
seven affected checks passed after retaining the exact zero-count assertion with
its label. Initial follow-position test was corrected to await smooth keyboard
scroll completion. Test matrices overlap. Browser evidence is fixture evidence,
not native runtime proof. Source snapshots, slice diff and hashes are in
`target/flat-02-supervision/`; see README.md for commands and evidence.

Voice-enabled native build passed (3m10s). Copied executable:
`target/flat-02-supervision/pytxo-workspace.exe`, SHA256
D228FD91473F5115FC8B60D85D509C5310723DB5AB938B4A26AAEBDBAF4D008D.
Launched window 5375742, title Pytxo Desktop — Workspace validation, using the
existing isolated com.pytxo.flat02.menus profile. First state capture was stopped
by physical Escape. Desktop control stopped immediately; no native screenshots
or runtime acceptance are claimed for this build. No MSI/frozen RC/publication.

Next action: resume native inspection of workspace fit, History and stored agent
Activity when Matt is ready. Then continue runtime identity/coordinator records
and linked-folder execution with independent per-domain review/Apply/freshness.
Adapter, Windows scaling, exact installer, MP4/Bench/materials and publication
gates remain. No new design approval is required; no spending/push/publication,
new dependencies or global/security settings changed. Existing dirty work kept.


## September 13 — onboarding and menus implemented from the new mockup

Matt requested a fresh onboarding/menus mockup and implementation with mature
hierarchy and resizing. This work is authorized; no repeated layout approval.
Created `target/flat-02-menus/mockup-board.png` as an illustrative six-interface
reference. Implemented horizontal onboarding stages/shared Back footer, responsive
Setup rail/section picker, fixed menu headers/actions with internal list scroll,
compact profile tiles, grouped Layout/saved views, Escape dismissal and a compact
single empty approvals state. Approval metadata no longer compresses/clips.
Existing dirty work, runtime data, Stop/review/Apply and freshness are preserved.

Initial affected browser suite: 86 passed. Final follow-up: 21 affected dock,
approval and resize checks passed after screenshot/native repairs. Final Svelte
check: zero errors/warnings; CSS lint and voice-enabled native build passed.
Matrices overlap. Evidence, isolated slice diff, source hashes and failed-test
history are in `target/flat-02-menus/README.md` and `identity.json`.

Final native validation: `target/flat-02-menus/pytxo-menus-final.exe`, SHA256
99CB1D323DE2D5E19D44055C70AF9C169B2C59BE4ACDC94DA89A24A5347E87B0.
Final native captures verify Appearance, compact empty inbox and layout Escape.
Earlier same-slice captures verify onboarding, actual discovery, workspace menus,
command scrolling and maximize/restore; they belong to the intermediate EXE.
Native minimum size, Windows 125/150% scaling and populated native approvals
remain unverified. Source changes are local; no MSI, frozen RC or publication.
No dependency/global/security changes, model calls, hosted Actions or Git pushes.

Next authorized product slice remains real mission supervision/ADE panels and
linked folder runs with separate domain review/Apply/freshness receipts. Then
adapter verification, final native/scaling acceptance, frozen candidate, exact
clean-Windows installer, matching MP4/Bench/materials and explicit publication.
The full 46-interface/whole-product goal is not complete. No new design approval
is needed for its approved scope. Final validation app is open; no human action
blocks the next authorized implementation slice.


## September 13 — Flat Desktop 02 approved; first implementation slice verified

Matt explicitly approved Flat Desktop 02: "yes, I approve, let's implement."
No new approval is needed for its bounded sequence. Implemented four visible
onboarding stages, optional terminal tools/display, content-sized short stages,
anchored actions, independently scrolling recent folders, workspace retention on
Back, focus/pressed-state accessibility, flat profile vectors and three sourced
ADE marks. Existing app/Rust/web changes are preserved; no reliability contract
was changed. Details: [[astra-flat-desktop-02]].

59 affected browser tests passed. After native inspection found recent-list
body overflow, the final three onboarding regressions passed, including the
five-folder case. Final Svelte/CSS and voice-enabled native build passed. Do not
sum overlapping test matrices. No project dependencies were changed.

Final isolated validation EXE: target/flat-02-native/pytxo-flat02-final.exe
SHA256 6FEF84136F6965023D8FD37C42A1B5630D4E36DAB043D8A95F03EC6E35C3437C.
Computer Use observed final Welcome, actual CLI discovery, Workspace/scroll,
Ready and 110% app zoom. Native screenshots and source identity are in
`target/flat-02-native/`. This is not an MSI, frozen RC or published release.
A native edge-drag resize attempt did not change dimensions; minimum-size
acceptance is browser-only. Actual Windows 125/150% scaling remains untested.

Next authorized slice: real mission supervision/ADE panels and linked folder runs
with separate domain review/Apply/freshness receipts, then Grok Build-first
adapter verification. The 46-interface packet is not fully implemented. Existing
clean-Windows, final MP4/Bench/materials and publication gates remain open.
No additional design approval is required. No sign-in/trust/paid run/Apply/install
was performed through native testing. Original debug app/session is preserved.

## Historical — September 13 Flat Desktop 02 proposal before approval

Matt rejected the glossy generated permission emblems and requested flat 2D art,
removal of both repeated onboarding logos, cleaner/non-scrolling short interfaces,
mockups for the Desktop, visible ADE panels, and broader CLI support. He explicitly
said to await approval before implementation. That decision was subsequently resolved by Matt’s explicit approval recorded
above; this section preserves the design-packet history.

Prepared `target/pytxo-flat-desktop-02/PROPOSAL.md`, `ALL-INTERFACES.pdf` (46 interface
mockups + 3 image studies), `index.html`, `screens.json`, `ADAPTERS.md` and assets.
All are design/example content, not native evidence. Detailed mockups/brief govern;
generated image details (invented P logo/model names) are not specifications.
The prior approved private docking proposal remains intact and its approval still
stands. The new packet adds a conversational mission/coordinator design, linked
per-folder runs with separate existing Apply receipts, and Grok Build-first adapter
work. Interactive agent TUIs, cross-repo atomic Apply, Grok Bot cloud control and
existing consequential-action gates remain excluded/deferred.

Official docs confirm Grok Build headless/streaming/ACP interfaces and describe
Grok Bot as a cloud-computer application; found Grok Bot CLI is third-party. Factory
Droid and Goose are bounded candidates, not verified Pytxo integrations. Some
official brand assets were unavailable; use clearly provisional text badges rather
than invented vendor logos until vetted assets are sourced.

No application code, current art assets or native settings changed. All 15 recorded
application-source hashes from the last slice still match. HTML JS syntax, PDF
page/inventory and representative rendered mockups checked; no browser workaround
or native test. Next action: one explicit approval/revision of Flat Desktop 02.
The previous implementation evidence and all release gates below remain historical
context; nothing in this design packet closes them.

## September 13 — screenshot feedback implemented; native acceptance pending

This supersedes the unchanged-blocker note below: Matt provided six concrete UI
defects and authorized local repairs. Implemented responsive review flow without
the tall evidence column; direct pointer dragging to hidden/visible right/bottom
docks; dismissible workspace feedback; generated permission emblems; compact
voice/discovery/integration controls; real first-attachment project creation and
secondary-folder add/remove from workspace settings. Trust and Apply semantics
remain unchanged. Existing files and earlier hardening are preserved.

Folder grouping is real; multi-folder Flow dispatch is still not implemented.
The UI explains primary-folder execution and keeps the existing project CLI's
coordinated workflow available. Next product slice is to reconcile multi-root
mission planning with reviewed Apply, not merely pass a project ID through.

Fresh checks: 38 Desktop Rust tests, clippy with warnings denied, Svelte/CSS and
frontend build passed. The affected workflow matrix passed 49 tests; a separate
14-test feedback/DPI matrix passed (100/125/150% simulated DPI × 90/100/110% app
zoom). All six final feedback regressions passed, including cross-dock preservation
and notice dismissal. Matrices overlap; do not sum them as distinct tests.
These browser tests use preview/IPC fixtures, not native run proof.

Built and launched a separate voice-enabled native validation executable with
isolated app ID/main browser profile, preserving the running debug app/session.
SHA256 D54CE7FBEF71AEC9FB3FB1AE8CE212EACE2BBF1F18A550731B879238A43E88D4.
Its native accessibility tree reached onboarding, but the screenshot displayed
Roblox Studio over Pytxo. No native drag, folder attachment, or visual acceptance
is claimed; desktop input stopped. Next human action: pause competing desktop
automation and foreground the UI feedback validation window for native testing.

Branch/HEAD remain codex/beta-candidate-verification / eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78
plus preserved local changes. The older MSI 322489A0… predates these changes.
The configured validation EXE is not an installer or release candidate. No new
MSI, clean Windows acceptance, matching MP4, CI run, push, publication or deployment.
Details: docs/01-projects/astra-ui-feedback-2026-09-13.md;
evidence: target/astra-ui-feedback-20260913/. Existing five-stage release sequence
and all consequential-action approvals remain in force.

## September 13 — resumed blocked audit: shared desktop remains unavailable

Previous turn made concrete progress: reproduced/fixed cross-run review identity,
30 affected tests passed, corrected MSI built and payload verified. This continuation
read the current evidence and made one read-only Computer Use observation using a
fresh returned Pytxo window. Screenshot instead showed Roblox Studio in play mode.
The app task inventory also listed Build Roblox Workbench and Obby Universe active;
this establishes concurrent desktop work, not proof of which actor caused earlier
window changes. No input, activation retry, mission dispatch or Apply this turn.
Do not treat the Roblox image as Pytxo evidence or interfere with that task.

The same native-access prerequisite has persisted across the user-triggered resume
and two continuations; independent verification/packaging and the concrete review
repair are complete. Mark goal blocked, not complete, until Pytxo has a stable
exclusive desktop handoff. Current candidate remains package322489A0…/payloadF84BF980…,
not installed or native accepted. Do not rebuild or rerun unchanged tests merely to
continue. Next human action: pause other desktop automation and foreground Pytxo.
Then resume native acceptance of the corrected build and prepared dependency mission;
keep all remaining clean-Windows/media/publication gates and existing approvals.


## September 13 — cross-run review identity fixed and packaged

A concrete mismatch observed during native navigation was reproduced in the browser
fixture: opening a pinned run's prepared review while another review remained open
changed the header to run-8f2c but retained pkg-71ad-immutable and the old evidence.
MissionsScreen now keys RunReviewScreen by both execution domain and run ID, so
package, agents, file selection and confirmation state reload together. Same-run
snapshot updates retain their component. Backend Apply/freshness contracts unchanged.

New regression failed before the fix at the package identity assertion. It now
checks old/new package and exact task identities through the real pinned-dock path.
Final review-depth + mission-dock: 30 passed (49.1s). Svelte: zero errors/warnings;
CSS passed. Read-only reviewer found no mandatory changes. Initial combined run had
29 passes and a new overbroad assertion matching signal-core.md; it was replaced
with positive exact task-label checks before the final successful rerun. Logs,
review and browser screenshot: target/astra-review-identity-20260913/. The screenshot
uses existing preview fixtures and is not native confirmation or release footage.

Fresh default-config, voice-whisper/static-CRT MSI build exited 0. All 457 input
hashes stayed unchanged. Administrative extraction and packaged PE imports passed;
full byte comparison proves only the three-byte Tauri bundle marker differs from
the standalone EXE. Current test candidate identities:
MSI 322489A0F8858E4FAEEC2C20E09875F24D5B85920EC8D363402E4D8445CA57B3
Packaged EXE F84BF980AED4482A515452745A9824CF155EE37D2CF3EE07A197BA6FE24FC284
Standalone EXE E5ED40A55AB1F5C47838E7AFE6E2092BE004AB7000DA93207C5039B740A75806
Paths and evidence: target/astra-review-identity-20260913/package/candidate.json.
Prior AE6D704B MSI remains preserved and predates this fix. This current package is
not installed, launched, native-accepted, frozen or published. Older full-workspace
checks are historical for unaffected Rust/web source; these 30 checks cover the
final affected frontend slice. No Actions runs or source commits/pushes occurred.

Native control is still awaiting the prior manual restore handoff. No additional
native input or new mission was attempted this continuation. Next: inspect this
corrected build natively, including pinned review switching, then execute existing
dependency-acceptance/MISSION.txt; maintain terminal/scaling, exact-installer/clean
Windows, final matching media and publication gates. All existing approvals and
user work preserved. This turn made implementation and verification progress.


## September 13 — native access briefly restored; window-control handoff blocked

User resumed native testing, then explicitly answered Ready—take desktop control.
Initial activation and native screenshot succeeded on existing PID24508, debug
SHA3693D25BE5B5D7C14D845AA5616F2007D50E39137D28AC672960FEBF0F94EFCC. New run
opened. All twenty dependency fixture baseline hashes still matched; fixture HEAD
da8ad03e093d69cdf8d37158af17c62ee163bdd1 and Pytxo HEAD eb5f73d9… are unchanged.
No mission text was entered, plan dispatched, new run started or Apply performed.

Subsequent input was rejected as stale window state, user input detected, and
window bounds changed. After the explicit handoff, fresh returned window2887590
remained minimized. Requested activation/get_window/state recovery was attempted;
activation returned user input detected and a separately refreshed state still
reported minimized. Do not attribute these rejections conclusively to user activity;
a tool fault is possible. No alternate UI automation or process restart was used.
The successful initial Pytxo screenshot is actual native observation, not mission
acceptance. A later capture of foreground Codex is not Pytxo evidence.

Native input stopped after bounded recovery. Exact resume prerequisite: manually
restore Pytxo Desktop so its workspace is visible, then leave it open for control.
Continue the unchanged dependency-acceptance/MISSION.txt and offline recipe once
accessible. This is the first turn of the fresh resumed blocked audit; no goal
complete/blocked update is justified this turn. Existing installer/package evidence,
layout approval and publication/security/spending gates remain unchanged.


## September 13 — blocked audit after independent packaging work

Previous continuation made progress by verifying the MSI payload. This turn checked
the remaining RELEASE_PLAN steps and corrected DEMO.md's stale September 10
"current" capture target/proposal status. It now records the actual unlaunched
packaged identity, historical debug acceptance and exact final-footage prerequisite.
No film was regenerated and no additional native input or approval probe occurred.

The same unavailable native desktop has persisted through the user-triggered resume
and two automatic continuations. Safe integrated checks, packaging and payload
inspection are finished. The next required acceptance slice needs desktop access;
clean Windows needs permitted owner sign-in/installation, final media needs accepted
runtime footage, and push/release/deployment/download promotion needs explicit
approval. Repeating tests, rebuilding unchanged source, or checking the lock screen
again would not advance those gates. Mark the goal blocked, not complete, until
Windows is unlocked and Pytxo is foregrounded. Resume dependency-acceptance/MISSION.txt
then; preserve all approvals, source and candidate identities. Full objective remains.


## September 13 — packaged MSI payload identity verified

The previous turn completed packaging and identified the locked desktop. This
continuation completed the safe independent administrative extraction using
`tooling/scripts/verify-windows-msi.ps1`; exit 0, one packaged EXE, no forbidden
versioned MSVC runtime imports. No installation, app launch or native input.

The initial exact-hash comparison correctly failed: standalone EXE 0039E600… and
MSI payload D0B2E4772FF965794F31ED1205CD26B232E215A8E627D1A67C81676FBBCC463C
differ. Full byte comparison then proved the only difference is Tauri's three-byte
`UNK` to `MSI` bundle marker, with all other 37,324,288 bytes accounted for. The
original MSI SHA AE6D704B… remains unchanged. Do not use the standalone executable
hash to identify installed footage. Full evidence and rerunnable byte comparison:
target/astra-integrated-20260913/package/msi-extraction/{result,runtime-imports,
payload-identity}.json and package/verify-payload-identity.py. candidate.json now
records both identities. Extracted administrative MSI is not a distributable copy.

This closes payload inspection only. The local candidate is not frozen, installed,
clean-Windows accepted or published. Native dependency-bearing mission and pending
terminal/scaling acceptance still need restored desktop access; final media needs
the accepted build. No code, approvals, global/security settings or external state
changed. Existing next action remains unlock Windows and foreground Pytxo; no
activation retry or repeated permission request was made in this continuation.


## September 13 — integrated checks and MSI completed; native resume blocked by locked desktop

Source remains C:\pytxo, codex/beta-candidate-verification,
eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78, with existing dirty/untracked work
preserved. No source commit, push, hosted Actions run or publication. The approved
mission workspace/docks remain implemented locally; this is verification progress,
not a new layout proposal or release acceptance.

Integrated checks completed: full cargo workspace tests, clippy with warnings denied,
format check, Desktop Svelte/CSS check (zero errors/warnings), 197 production-preview
Desktop tests, one development test, six reference-capture tests, web lint/build and
23 web tests, demo typecheck/privacy contract checks, and 44 release-tooling tests.
These are recorded in target/astra-integrated-20260913/checks.json and adjacent logs.
Browser captures use fixture data and do not establish native acceptance. Earlier
native Apply/Stop/restart evidence remains historical, bound to debug EXE 3693D25B….

Integrated repairs: rustfmt on six already dirty/new Rust files; obsolete mission
content selectors and compact-sidebar expectations corrected without dropping
behavioral assertions; release-version verifier now separately checks candidate
1.2.2 versus declared public 1.2.1, rejects candidate download aliases, and preserves
artifact consistency checks. Initial UI run was cancelled after repeated obsolete
locator failures; final complete rerun passed. Release verifier regressions and
bounded read-only review passed. Full details: release-version-review.md in the
integrated evidence folder. No online public-availability verification was performed.

Local npm run build:msi finished exit 0 with default Tauri configuration,
voice-whisper and static MSVC CRT. All 457 recorded source inputs were unchanged
across the build. Prior MSI preserved. Candidate copies and full hashes are in
 target/astra-integrated-20260913/package/candidate.json:
MSI AE6D704B3E366EF2745867CBAD4DA8C56676463F9CB5355BDE5835E9009F5C41
EXE 0039E600ACB7415844282BB58CAC1CEED41CE301CA92CB12BB4865E7897C1B46.
Static PE import check passed; dynamic/runtime dependencies remain unverified.
This MSI is not installed, clean-Windows accepted, frozen for release or published.

On the user's native-resume request, fresh selection found existing Pytxo PID24508,
window2887590. Activation again returned `failed to activate captured window`.
A read-only capture showed the Windows lock screen while accessibility text still
reported Pytxo's stopped run af6825ea…46fd and Sessions0. The image is NOT native
Pytxo evidence. No further input, restart, mission dispatch or Apply was attempted.
This now identifies a concrete desktop-access prerequisite instead of an unexplained
activation error. Do not interact with the lock screen or repeat activation retries.

Next required human action: unlock Windows and bring Pytxo Desktop to the foreground.
Then resume the existing dependency-acceptance/MISSION.txt with its explicit offline
setup recipe; preserve the twenty-input fixture and previous accepted runs. Pending
terminal-input handoff, actual Windows scaling, dependency-bearing native journey,
default-profile/exact-installer and clean-Windows acceptance, matching final media,
and approval-gated publication/download remain open. Reuse RELEASE_PLAN's existing
five stages; no additional design approval is needed.


## September 13 — dependency-bearing acceptance prepared; native activation blocked

Previous turn made progress (native Stop/restart accepted). This continuation
prepared the required dependency-bearing acceptance project rather than repeating
the classifier mission. Existing trusted disposable fixture now contains
acceptance-app: TypeScript 5.9.3 build, Zod 4.4.3 validation, module exports,
project aggregation, CLI, malformed-input checks and fifteen tests. Original
classifier source/docs/tests remain unchanged. Root test recipe runs its fourteen
tests plus the reporting app's fifteen. Baseline: all twenty-nine pass.

Offline npm initially failed ENOTCACHED for zod-4.4.3.tgz. No network fallback was
used. Local archives were packed from already-installed packages with scripts
disabled, then a lockfile was resolved and npm ci --offline succeeded. Vendor
archives are fixture-only inputs, not new Pytxo dependencies or product artifacts.
The fixture's local commit da8ad03e093d69cdf8d37158af17c62ee163bdd1 includes the
new app and root test/config recipe; source checkout HEAD remains unchanged.

Existing [[task]] verify supports an ordered setup/test recipe, reused here:
`npm --prefix acceptance-app ci --offline --ignore-scripts --no-audit --no-fund`
then `npm test`, for each of three tasks with explicit dependencies. Native plan
must show those commands before dispatch. Mission fixes completion-rate semantics
in the reporting package, adds regression/CLI tests, then updates result docs.
Setup is explicit: current local planner does not infer dependency installation,
and isolation excludes node_modules. Do not claim automatic setup or native
combined-candidate acceptance from successful baseline npm commands.

Computer Use returned `failed to activate captured window` on New run, then again
after fresh selection/activation of the unique Pytxo window (id 2887590). Input
stopped after that bounded recovery; no UI action/dispatch success is assumed.
Process 24508 is still live, exact accepted debug build; latest durable run is
still af6825ea…46fd cancelled, no active marker, all twenty fixture inputs match
baseline hashes. Do not close/restart a live app merely because activation failed.

Evidence under target/astra-native-dock-20260913/dependency-acceptance/: baseline.json,
baseline-tests.log, ready.json, MISSION.txt, fixture-source/ (portable tracked
snapshot including local archives). No dependency mission has been dispatched.
Next human action (stated once): bring Pytxo Desktop to the foreground and leave
it open. On restored desktop access, use this exact MISSION.txt and recipe for
native plan/run/review; independently verify candidate and Apply bytes before
marking that acceptance complete. Preserve previous applied/stopped evidence.
If input remains blocked, do not repeat the same activation attempt or handoff;
progress independent integrated checks under the existing plan instead.

No source code change, tool installation, network package download, security/global
setting change, Actions run, installer, MP4, push or publication this continuation.
Whole-product goal remains active; this is first occurrence of this activation
blocker in the current continuation, not grounds to mark the goal blocked/complete.

## September 13 — native Stop settlement and restart verified

Previous turn was progress (Apply and restart accepted); this continuation adds
actual cancellation evidence on the same 3693D25BE5B5D7C14D845AA5616F2007D50E39137D28AC672960FEBF0F94EFCC
native debug build. Run af6825ea-749a-421b-a63f-b602987e46fd used the existing
trusted disposable fixture, Orbit, PTY, three dependency stages and the installed
OpenAI Codex CLI. The fixture's previous accepted Apply bytes were preserved in
local fixture-only commit c7ad093640146a2ab0e2d82ccf0b44217e502c91; none of the
six primary file bytes changed when preparing this baseline. No Pytxo source commit.

Native Stop confirmation was clicked while agent-0 was live. Seven observed
process identities (root and descendants, PID plus creation time) all exited.
The durable run is cancelled, active marker removed, process registry empty;
one task is stored failed without exit, two blocked_by_dependency. No candidate
was prepared, no Apply attempt exists, and all six primary files match baseline.
Normal close/restart retained these facts and the preceding run's applied digest.
Native UI shows Stopped, three of three waves settled, disabled Stop and Review.
Screenshots 48–51 are actual native captures; 51 is after restart. Task-row labels
still say Failed / Dependency failed according to stored states, a presentation
limitation rather than evidence of successful task work or a fresh code fault.

Fresh local command: cargo test -p pytxo-orchestrate --test stop_exact, exit 0,
four tests passed, including Windows descendant termination and durable cancellation.
Recorded final source hashes remain unchanged. Evidence:
target/astra-native-dock-20260913/stop-acceptance/{baseline,settled,restarted,
running-process-tree,process-exit-verification,restart}.json and stop-exact-tests.log.
App left open, PID 24508, Sessions 0, copied validation profile; no live mission.
No product code changes, CI runs, dependencies, security/global setting changes,
publication, installer or MP4 in this continuation.

Next bounded action: the existing RELEASE_PLAN stage-2 representative dependency-
bearing repository mission acceptance, then integrated/default-profile candidate
verification. Do not rerun this tiny-fixture Stop check just to restore context.
Windows OS scaling, pending manual terminal input, exact installer/clean Windows,
matching media and approval-gated publication/download remain open. This Stop
check does not make the whole product or release complete. Do not repeat the
pending terminal handoff or ask again for the approved layout.

## September 13 — native mission Apply and restart accepted on the corrected build

Supersedes the in-progress mission result below. Native EXE SHA256
3693D25BE5B5D7C14D845AA5616F2007D50E39137D28AC672960FEBF0F94EFCC ran the corrected
three-stage mission dc873e30-4b14-4780-a1ee-b45ec1c8dad6: code, tests, then result
documentation. All three agents completed with exit 0. The original eleven tests
were preserved; fourteen project tests and twenty-seven independent assertions
passed on both reconstructed candidate and applied primary files. README was
read and matched the combined behavior. Captures 40–47 are actual native UI.

Native Cancel preserved all six primary files with zero Apply attempts. A guarded
README drift probe then caused backend Apply refusal with zero attempts (44).
Restoring only those probe bytes and Refresh review reran combined verification.
Refresh changed timestamps and package digest, not prepared file bytes or other
manifest evidence: a4f78eef… became
a55406a475bc98a1bdd073aee7b1dd21e41940099bd0664db8a614790fc83fc0.
The native confirmation displayed this refreshed identity (45). Apply wrote the
three exact reviewed after-images; the other three files remained unchanged.
Exactly one committed receipt exists: 523ce985-fe79-4021-9287-535689d7d381 (46).
Normal close/restart of the same EXE retained all bytes and the identical receipt;
Apply remained disabled and Applied was visible (47). Restart PID 28296, copied
validation profile verified by child process. App remains open, Sessions 0.
Owned loopback preview server session 46754 was stopped.

Implemented repairs in this continuation: checkout preflight before Flow readiness
and dispatch; readable startup failure; New run exits dock focus and menus without
losing saved views; result-documentation dependencies in the local Signal fallback;
Review adapts to actual center width. Independent review findings were repaired.
Final affected checks: 38 UI, 16 planner, 15 orchestration tests; Svelte 0 errors/
warnings, CSS and strict library Clippy passed; native build passed. Recorded source
hashes still match after acceptance. Earlier preview runtime checks retain their
own build identities; they are not all rerun on this final debug executable.

Evidence: target/astra-native-dock-20260913/combined-review-record.md and
fresh-mission/{refresh-identity,applied-byte-verification,restart-verification}.json,
phase snapshots, applied-tests.log and applied-independent.log. Branch remains
codex/beta-candidate-verification, HEAD eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78;
161 default status entries / 203 with every untracked file expanded were preserved.
No source commit, push, merge, Actions run, installer, publication or new MP4.
The local disposable-fixture baseline commit below did not commit Pytxo source.
PR #31/CI/public release observations below remain dated reports, not a new check.

Remaining: actual Stop settlement on this build, Windows OS scaling, pending manual
terminal-input acceptance, final/default-profile RC and exact installer, clean
Windows, matching demo/site/docs and approval-gated publication/download. External
filesystem drift refusal is verified; automatic live drift detection and terminal
UI typing are not established by this probe. This is native development acceptance,
not clean-install or release certification. Next bounded action: native Stop
settlement in an isolated fixture. Do not repeat the pending terminal handoff or
request this already-approved layout again.

## September 13 — native mission acceptance uncovered and repaired workflow defects

Current continuation in progress; do not replay the old run. Native run
be362e5b-8c06-4da7-a340-321feef858e0 failed startup on prior uncommitted fixture
Apply results. Six primary files stayed unchanged, zero tasks/contracts. Fixed
Flow checkout preflight before plan readiness and before dispatch claim, retaining
executor rechecks and profile exemptions. Startup status is readable. New run
leaves focused/compact inspection while preserving saved views. Independent review
findings were fixed; 37 UI + 15 orchestration tests passed, Svelte/CSS and Clippy
passed. Native EXE 2413CE26 confirmed the blocker and focus recovery (33–36).

Only the disposable PytxoMenus20260910B/fixture accepted edits were committed as
f0c8453d6b4004bae5e2b9b0a62332f56f5bed33; all six file bytes remained unchanged.
Pytxo source branch/HEAD are unchanged. Fresh native run
0c7e2acf-c126-4fd2-80c3-89794536be65 completed all three tasks, prepared package
9eaa78d47f4366188fbe96b4fda7328def761e6b91e6c28f9df3cd12d81f4b71, and passed
three combined commands plus 27 independent code assertions. But README task ran
alongside implementation and wrote a stale implementation-missing statement.
This is a rejected complete-mission outcome, not a successful demo. Native Apply
Cancel left six files unchanged and zero attempts. Package retained for evidence.
See target/astra-native-dock-20260913/fresh-mission/REVIEW-0c7e2acf.md.

Next repair is implemented: Signal fallback result-documentation waits for earlier
code/test tasks; unrelated typo edits and explicit manifest plans keep their
behavior. Native Review also crushed filenames beside the right dock; container
queries now use the actual remaining width. Final checks/build use combined-reviewed-*
logs; native reinspection/repeated mission pending at this checkpoint insertion.
The previous native app was closed only after the run settled and Cancel verified.

Preview forward Tab and offline-server Retry succeeded earlier in this continuation
on 1740B643 (captures 27–29); old server session 12227 stopped, replacement 46754.
No installer/frozen candidate, final MP4, clean-Windows acceptance, CI minutes or
publication. Existing manual terminal input handoff remains pending; do not repeat.

## September 13 — native preview recovery verified

This continuation made verified progress. Current validation EXE SHA256:
1740B643FC405E1BC83572CCDAC607B0A2FBB4C8C7ECCEAB25F90E4B969FD2E9.
PID 34680; preview-recovery-reviewed-launch.json records start identity. Actual
copied main browser profile was checked in preview-recovery-reviewed-profile.json.
It remains a debug build with explicit validation-only profile configuration.

Native rejected-redirect testing exposed ineffective Resume after failure, then
a late old-renderer sync error after the first Retry repair. Fixed with explicit
Retry (close acknowledgement before replacement), renderer-identity guards, and
serialized explicit Close. An independent read-only reviewer found related stale
close/hide races; those are fixed and covered by delayed-completion regressions.
Re-review found no mandatory remaining issue. No Rust/security/dependency change
in this continuation. Source edits: LocalPreview.svelte and local-preview.spec.ts;
README and existing planning/evidence records updated.

Final checks: 14 preview/dock tests passed (46.2s), Svelte 0 errors/0 warnings,
CSS pass, native build passed (51.08s). Logs prefix preview-recovery-reviewed-.
Earlier 35 native tests/Clippy and 78 integrated UI tests were not rerun this turn;
they remain preceding evidence, not final-build runtime certification.

Actual native captures 24/25 show rejected redirect -> Retry -> working page,
counter clicked to 1, no stale error. Process evidence shows old preview browser
33900 exited, replacement 24932, main browser 32764 unchanged, exactly one child
preview browser after recovery. Alternate server 19419 received no requests.
Capture 26 proves backward Tab exit from the first page control, Tab into Pytxo,
then Enter paused the view. The app is left open/paused, Sessions 0; owned fixture
server on 19418/19419 remains live (exec session 12227). Earlier capture 21 is
1AB48645; 22/23 are the reproduced interim 2148344B defect, not final evidence.

Bounded download probe on 1AB48645 requested /download, opened no save UI and
created no named file in default Downloads; do not equate that with exhaustive
download acceptance. Full forward Tab traversal, renderer-crash/offline-server
cases, actual Windows scaling and fresh run/Stop/drift/review/Apply remain open.
No new MSI/RC, MP4, CI minutes, push, merge, publication or approval request.
Branch/HEAD unchanged; 158 dirty/untracked entries preserved; root was the only
writer. Existing manual terminal handoff remains pending and must not be repeated.
Next bounded action: finish native forward-keyboard/offline-preview acceptance,
then exercise fresh one-harness work through review/Apply using the existing plan.

## September 13 — final native preview slice checkpoint

The compact-header repair is implemented and inspected in the actual native app
at 1282 x 802. Final validation EXE SHA256:
1AB48645874466E26DA2251F119F28224522D3F9F1ADE1FF1F0418D0E2CB26A7.
PID 37208; preview-density-launch.json records its start identity. The actual
copied main profile was verified in preview-density-profile-verified.json. This
uses an explicit validation-only profile configuration, not a release installer.

Final affected checks: 13 preview/docking tests passed (43.6s), Svelte 0 errors/
0 warnings and CSS pass; final native build passed. The preceding integrated
78 UI tests and 35 native tests/strict library Clippy remain relevant; only the
preview header changed after those checks. Logs and screenshots are under
target/astra-native-dock-20260913. Capture 18 shows the repaired bottom viewport;
19 shows the interactive fixture counter at 1 and keyboard focus on Pause after
F6 then Tab. Enter actually paused the native page (capture 20). UI Automation's
focused-element text stayed at RootWebArea; the visible focus ring and successful
keyboard activation provide the stronger evidence.

The existing historical mission remains completed/Applied; no fresh harness run
or Apply was performed. The native app is left open with preview paused and zero
user terminal sessions. The owned loopback fixture server remains available on
19418/19419 (exec session 12227). Full Tab traversal, download/redirect/failure
paths and actual Windows scaling remain unaccepted. Earlier popup/frame/boundary
probes below belong to DD1EF49B; final Rust code is unchanged, but they are not
mislabelled as final-EXE interaction. No general network-sandbox claim is made.

Branch/HEAD remain codex/beta-candidate-verification / eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78;
158 dirty/untracked entries are preserved. No CI minutes, MSI, new MP4, push,
merge, release or publication. The existing terminal handoff remains pending;
do not repeat it. Next bounded action: complete remaining native preview
keyboard/failure acceptance, then fresh run -> review -> Apply acceptance under
the existing release plan. No additional layout approval is needed.

## September 13 — raw local-preview renderer and dock integrated (preceding checkpoint)

Native follow-up: DD1EF49B validation EXE actually used the copied profile, verified
by process metadata. Its raw child used a separate unique isolated-previews data
directory. Capture 15 shows the real loopback fixture running; custom IPC/alternate
server fetches rejected, no alternate server request, popup produced no window or
server request, same-server frame loaded while alternate-server frame did not.
The injected inert ipc.postMessage returned without a result; this alone is NOT
denial proof. Capture 16 shows native page hidden for View options. Capture 17
exposed an over-tall header after moving to bottom. A compact header repair and
final dock regression/rebuild are in progress. All these are validation EXEs and
fixture probes, not fresh harness/Apply, final installer or public-release proof.

The approved preview is implemented in the dirty local tree. Raw Wry child,
actual private-storage verification before navigation, no Pytxo IPC/protocol
connection, denied popup/download/permission requests, exact-server navigation
and resource restrictions, F6/Tab focus handoff and visible load/block reporting.
Right/bottom integration restores references only. Revisioned visibility, leases,
awaited dialog/menu barriers, hide-failure close recovery and gesture suspension
address independent review findings. Stop/review/Apply authority is unchanged.

Fresh checks: 78 integrated UI tests passed (1.7m); four preview tests passed
after adding delayed-hide and dead-preview recovery assertions (27.2s). Svelte
0 errors/0 warnings and CSS pass. Final Desktop native tests: 35 passed, including
existing PTY ownership/replay and review/recovery checks; library strict Clippy
passed. Logs: target/astra-native-dock-20260913/preview-*.log.

Actual native capture 14 shows expected refusal when the existing test process
forces shared WebView2 storage (intermediate EXE 8b77c28d). The first attempted
relative-profile validation build, ba39bb3d, unexpectedly used the default profile;
process-path inspection caught this and it was stopped before further interaction.
Locked tauri-runtime 2.11.1 drops WindowConfig.data_directory. main_profile.rs now
works around that omission only for explicitly configured safe relative main
profiles; default startup, capabilities and CSP remain unchanged. Independent
read-only review found no additional mandatory issue in that fix. Positive native
preview acceptance is still being verified; do not treat build/test success as it.

Original test profile preserved; its stopped 34.5MB browser data was copied to
LOCALAPPDATA/main/pytxo-preview-validation-20260913 for a validation-only build.
The final executable identity and actual process paths must be recorded before
opening a test page. No MSI, CI, push, merge, publication, settings/account change
or new dependency version. Terminal typing handoff remains pending, not repeated.
Fresh run/Stop/Apply, scaling, frozen candidate/installer/demo/publication remain
open. Current branch/HEAD unchanged: codex/beta-candidate-verification / eb5f73d9.

## September 13 — local-preview boundary decided and policy tested

The previous goal turn was verified implementation progress. This turn continued
independent preview work while the existing terminal handoff remains pending.
No new layout decision or terminal response was inferred.

Inspected the exact approved PROPOSAL.md and locked Tauri 2.11.2/Wry 0.55.1 source.
One bounded read-only adversarial reviewer examined the boundary; no research
swarm, model switch, dependency installation or shared write ownership. Tauri
managed children inherit native plumbing; whole-window capabilities, development
origin equivalence and an internal channel-fetch ACL exception make an ACL-only
preview unsuitable for the approved isolation claim. This is source evidence,
not a demonstrated exploit or a native isolation pass.

Chosen implementation path: raw Wry child using the already-built locked version,
no Pytxo command callback/custom protocols, separate verified storage context,
explicit navigation/popup/download policies and native geometry/lifecycle checks.
The review clarified that this reuse is not inherently a substantial new dependency
and fits the approved scope without another layout approval. Any actual capability,
CSP, authentication or isolation expansion retains the explicit security gate.
The initial suggestion of a new dependency approval is superseded by that review.

Implemented local_preview_policy.rs (exported library module only): local HTTP(S)
target validation, exact-origin navigation and main-webview caller rules. It adds
no IPC command, renderer or navigation. Review caught a blob: same-origin bypass;
preview-policy-blob-repro.log reproduces it and the fixed scheme check has a
regression. Fresh final four tests pass; Desktop library clippy -D warnings passes.
Logs and independent review are under target/astra-native-dock-20260913; design
decision: docs/01-projects/astra-local-preview-boundary-2026-09-13.md. The exact
dependency patch passes git apply --check but has NOT been applied. Cargo manifests,
main CSP and capabilities are unchanged. The running 38825c1f native EXE is still
the preceding output build; no new native screenshot or release acceptance claim.

Next authorized implementation slice: integrate the raw child renderer and preview
dock using this policy, then demonstrate the native isolation/lifecycle boundary.
Do not stop at the policy module or substitute an iframe/floating browser. Preserve
the current native process/user sessions during subsequent builds. Terminal typing,
fresh lifecycle/Apply, final candidate/installer/demo and publication gates remain
open. No push, merge, CI, spending or publication occurred; goal is incomplete.

## September 13 — readable recorded output, native verified

The preceding native turn was progress, not a wait: it changed source and produced
new native evidence. The manual terminal handoff remains unanswered; independent
authorized work continued without repeating it or opening another approval gate.

Recorded agent output now defaults to plain text with terminal formatting removed.
A small per-observer/per-stream reader handles escape sequences across event/page
boundaries, keeps stdout/stderr independent, normalizes CRLF and preserves Unicode.
It does not replay cursor redraws or interpret links/clipboard commands. Source
details offers raw event text and explains the presentation difference. Stored
payloads, event identity/cursor, 600-event retention, read-only claims status,
verification and Apply authority remain unchanged. Formatting-only events have an
explicit empty state. No dependencies or native authority changes.

Fresh final evidence under target/astra-native-dock-20260913:
- readable-final-tests.log: 13/13 passed (36.7s), including split sequences,
  control strings/cancellation, stream separation, Unicode/CRLF and UI raw/plain
  switching alongside mission/dock/layout/terminal contracts.
- readable-final-check.log: Svelte 0 errors/0 warnings; CSS lint passed.
- readable-native-build.log: native custom-protocol build passed (51.05s).
- Running EXE SHA256 38825c1fa9b1691dc80d15ddda2709fb54ce900037a21f5ef93f5c50537c4c05;
  readable-native-launch.json identifies PID/profile/start time. This supersedes
  the 51f458a1 executable below; neither is an installer/frozen candidate.
- Native screenshots 12-native-plain-output.jpg and 13-native-raw-output.jpg show
  the real historical f3e164cd run in Focus view, with matching raw text available.
  Native toggling verified; no fresh run/Apply or Windows scaling claim.

Touched: recorded-output.ts, DockInspection.svelte, browser fixture, two scoped
test files, Desktop README and checkpoints. Existing work preserved; no CI,
installation, push, merge or publication. Whole-product goal remains incomplete.
Next: record the already-requested manual terminal result, then continue native
lifecycle/freshness acceptance and isolated preview work in RELEASE_PLAN.md.
Do not repeat the same terminal handoff or reopen the approved layout decision.

## September 13 — native testing resumed; docking density fixes verified

Matt explicitly resumed native testing, superseding the earlier desktop-control
pause below. One root writer; C:/pytxo remains on codex/beta-candidate-verification,
HEAD eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78, with 148 dirty/untracked status entries
preserved. No design approval is pending. No CI run, install, global setting,
dependency, account, push, merge or publication action was performed.

Rebuilt and inspected the actual Tauri custom-protocol app using Computer Use and
the existing isolated acceptance profile. Real recorded run f3e164cd-928a-43eb-b409-
ba726b8c33b7 and package 1cfe11d5230f07188808f8b102a9fc5f7b2a789940bcce573ece73d5d0c51bfd
were inspected: agent output, movement to bottom, resizing/hiding, three candidate
checks, dependency waves, immutable before/after file bytes and matching full
Review. Apply was correctly disabled for this previously applied package. This is
fresh native display/interaction evidence over historical real records, not a fresh
dispatch/Apply acceptance run.

Native findings fixed locally: long mission briefs now collapse with complete text
available; source metadata is expandable while read-only/claims status remains
visible; bottom output uses available space; truncated generated titles show an
ellipsis; terminal/layout dialogs are centered; saved tall bottom docks reserve
enough mission height for full Stop/Review controls alongside a right dock.
Apply authority and source-freshness checks were not changed.

Fresh evidence: target/astra-native-dock-20260913/README.md maps captures to builds.
39 affected browser tests passed; the final 9 docking tests passed after the last
native-discovered fix, including maximum-height two-dock layout after reload,
keyboard disclosure/resizing, terminal ownership UI and centered End dialog.
Final Svelte/CSS checks: 0 errors/0 warnings. Final native build passed (58.41s).
Running debug EXE SHA256:
51f458a1a341a54cf4ecdd1b63900432ebabd1de4addffe0d93c5599ce27ddde.
Final native captures 10 and 11 verify laptop-sized controls and centered terminal
dialog; native pointer resizing and keyboard Source disclosure also exercised.
Earlier 104 UI / 29 native / freshness tests remain historical, not rerun today.

Remaining: fresh full dispatch/Stop/Cancel/drift/Apply/restart journey, actual typed
workspace-terminal lifecycle, actual Windows display scaling, isolated local URL
preview, integrated candidate freeze, exact-installer clean-Windows acceptance,
matching demo/website/docs and explicitly approved publication/download checks.
Some recorded output still contains literal ANSI sequences. No new MSI or MP4;
the old MSI and public release cannot be attributed to this debug executable.

Next bounded acceptance action: Matt manually creates a workspace terminal in the
open disposable fixture and types Write-Output 'PYTXO_NATIVE_OK', confirming its
output. Computer Use guidance prohibits terminal commands and terminal automation;
no bypass was used. The Create dialog was inspected and cancelled, Sessions is 0.
Then continue native lifecycle/freshness acceptance and the existing finite plan.
The master goal is incomplete; the earlier paused-control blocker is superseded.

## September 12 — goal blocked on resuming native acceptance

The desktop-control pause after physical Escape persists for a third consecutive
goal turn. Background follow-up has been completed and verified; this turn's
read-only check found all five recorded UI evidence inputs unchanged and confirmed
the saved 104-test passing result, branch and HEAD. No test rerun or desktop input
was performed. This is not a completion claim or a fresh native test.

The remaining critical path needs resumed native inspection and isolation testing:
rebuild the latest UI, accept its actual mission/docking and shell lifecycle,
complete the isolated local-preview integration, then freeze the candidate before
installer acceptance and matching demo/release materials. The preview is still
unimplemented; the whole-product goal and release gates remain incomplete.

Mark the goal blocked to stop automatic retries on the same pause. Required human
action: explicitly resume native testing when desktop control is available. Retain
the existing layout approval and all publication/security/spending gates.


## September 12 — background docking accessibility and metadata follow-up

Progress continued with desktop control paused after the user's physical Escape.
Only the root implementation worker is active. C:/pytxo remains on
codex/beta-candidate-verification at eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78;
existing dirty work was preserved. No native interaction or app restart occurred.

Reproduced and fixed: hiding Focus view reopened its original dock; CSS text zoom
left the sidebar expanded; compact inspection clipped the full Stop control;
compact/focused resize controls did not change actual panel height; and Evidence
showed stale snapshot package metadata after a newer exact review arrived.
Sidebar collapse now uses available content width without overwriting the user's
saved preference. Compact panes preserve mission/action space; keyboard sizes and
pointer drag scale match actual geometry, including 200 percent text zoom. Explicit
null package metadata from a newer review clears older snapshot metadata. Apply
eligibility, permission profiles and source-freshness enforcement were not changed.

Fresh final checks (target/astra-mission-dock-20260912):
- dock-integrated-final.log: 104/104 passed (2.3m): mission-dock, dpi, shell,
  menu-workflow and epistemic-state. Includes full Stop visibility, Focus/hide,
  compact keyboard/pointer resizing, saved layouts and metadata consistency.
- check-accessibility-final.log: Svelte 0 errors/0 warnings; CSS lint passed.
- desktop-clippy-final.log: current native Rust passes clippy with warnings denied.
- Browser screenshots inspected: browser-wide.png, browser-laptop.png and
  browser-200-percent-inspection.png. These use explicit preview data, not native
  or Windows display-scaling evidence. Source hashes: accessibility-evidence.json.
- Earlier focused failures remain as regression evidence; earlier 83- and 29-test
  passes are superseded for affected UI coverage by the final combined 104 tests.

The running native executable remains the preceding bbb25acd build and does NOT
contain this follow-up. Do not associate its hash or the preceding slice screenshot
with these newer source changes. Full native journey/scaling, exact-installer clean
Windows acceptance, final demo and publication remain incomplete. Local URL preview
integration is still absent; RELEASE_PLAN.md records its required native isolation
checks. Main CSP, capabilities, dependencies, global settings and release state were
not changed. No CI run, installation, push, merge or publication was performed.

Next action: Matt resumes native testing when ready for desktop control. Rebuild
these final sources, inspect the actual mission/dock journey, then complete isolated
preview integration and candidate acceptance. No layout approval is pending.


## September 12 — approved docking implemented; final native inspection interrupted

Matt explicitly approved the existing mission-workspace and right/bottom docking
scope. No further approval of that layout is pending. Local implementation now
includes central mission/actions; exact run/domain readonly output and evidence;
frozen before/after file inspection; saved-plan dependencies; movable, resizable,
pinned and hideable views; keyboard controls; named layouts/reset; separately saved
wide/narrow arrangements; and explicit user-owned PTYs with session listing,
input enablement, independent replay, resize/end and full-exit handling.
User terminals are separate from agents and reviewed Apply. No process authority,
terminal input or output is saved in layouts. Existing hardening was preserved.

Fresh evidence in target/astra-mission-dock-20260912/:
- affected-browser-recheck.log: 128/128 affected browser tests passed before the
  final narrow-arrangement change. The earlier failing log remains diagnostic.
- dock-narrow-tests.log: final six docking tests passed, including independent
  narrow order/height/visibility across reload and restoration of wide placement.
- check-final.log: Svelte 0 errors/0 warnings and CSS lint passed.
- native-contract-final.log: 29/29 native Desktop library tests passed, including
  actual ConPTY file creation, scope denial, resize, replay and confirmed end.
- freshness-test.log: direct workspace edits after review reject outdated Apply.
- desktop-clippy.log: Desktop clippy with warnings denied passed before the final
  one-line fail-closed live-session-count correction; library tests cover final Rust.
- native-final-build.log: final custom-protocol debug build passed (51.64s).
- native-build-identity.json: exact executable hash and launch PID, when available.

Computer Use was stopped by the user's physical Escape key while locating the
rebuilt native window. No further desktop interaction followed. Final native
screenshots/inspection are NOT complete. The earlier native-slice1-evidence.jpg
belongs to the preceding slice and historical run data, not the final executable.
Browser screenshots use explicit preview fixtures. Real shell execution was tested
through Rust; Computer Use policy prohibits entering shell commands through UI.
No native terminal-input, quit-dialog or full workflow acceptance is claimed.

Remaining approved-scope gap: isolated, explicitly selected local web preview;
the built-in dependency view is implemented. Do not loosen CSP/IPC or substitute
an unsafe embedded page. Final native laptop/scaling/state inspection, exact-build
Cancel/Apply/restart acceptance and integrated candidate freeze remain open.
No new MSI, CI dispatch, dependency installation, push, merge or publication occurred.
The older MSI/public release records do not describe these new source changes.

Next action: resume native inspection when Matt is ready for desktop control;
inspect the exact recorded build, then finish isolated preview and integration
acceptance before freezing a release candidate. No repeat layout approval.


## September 12 — integrated docking/session slice verified locally

The affected browser suite passed **128/128** after resolving the new navigation
selectors and restoring the central unresolved-rollback warning. All **29 native
Desktop library tests** passed, including real ConPTY input/file creation, resize,
independent replay, scope denial and confirmed shell end. The final docking-control
suite then passed **5/5**, including named layouts, keyboard Escape cancellation,
focus/narrow return, immutable file content and simulated terminal input routing.
Logs are under `target/astra-mission-dock-20260912/`. The earlier 126-test run failed
21 checks; keep that log as superseded diagnostic evidence, not current acceptance.

Local implementation now includes user-owned PTYs, a Sessions list, explicit input
enablement, end/quit handling, saved layouts, per-workspace geometry, pin/move/hide/
resize/focus and reset, scoped readonly events, prepared before/after bytes and saved
plan dependency views. Existing Stop/Apply contracts remain in place. Terminal
output and input authority are never stored in layouts; restored missing sessions
cannot respawn. The PTY library and xterm were already present; Cargo gained only
the Desktop crate's direct reference to the existing locked portable-pty dependency.
No package installation or new library version was introduced.

Computer Use policy prohibits entering shell commands through Windows UI. Actual
shell execution above was tested directly in Rust; browser terminal controls use
an explicitly isolated IPC test fixture. Neither is a native terminal-screen claim.
Final native rebuild/inspection and targeted checks after lazy-loading terminal UI
are still in progress. No new MSI or public release has been made.

## September 12 — mission dock slice 1 verification; terminal slice in progress

Approved implementation is underway, not waiting for another layout decision.
Mission/right-bottom observer integration, exact run/domain output reads and scoped
review navigation are local. Two new browser tests passed (identity/pin/restore and
move/resize/hide/reload/reset); the native independent-cursor/membership test passed.
The initial native build succeeded: debug custom-protocol EXE SHA256
`1e3e65f0b00cea60f8735c1c19faed7840e4bac869b18e3e1305234ef01575a9`.
Computer Use inspected that exact executable, empty Work and real historical
approval-risk-demo records with the right Evidence dock. Missing review contracts
were visible as errors, not fabricated successful verification. Screenshot:
`target/astra-mission-dock-20260912/native-slice1-evidence.jpg`.
The installed older app was accidentally launched by the Computer launch resolver;
its process path was detected, that task-launched process was closed, and the exact
new EXE launched directly. Its old screen is excluded from implementation evidence.

Inspection found duplicate mission copy, now removed in source. Workspace-specific
geometry/focus controls and the separate user-shell registry are the next local
slice; these later edits are not covered by the above EXE or initial tests yet.
No installer, CI run, public release, dependency download or publication was made.

## September 12 — explicit docking approval; implementation resumed

Matt explicitly approved the existing mission-workspace and right/bottom docking
mockups and summarized scope in this task. This resolves the design gate below;
do not ask again for the same layout. Use the unchanged private PROPOSAL.md.
One implementation owner; preserve all prior dirty work and reliability hardening.

Slice order: real mission/right-dock integration and independent output cursors;
move/resize/hide/pin, keyboard, persistence/reset and narrow layout; then the
separate user-created workspace terminal and bounded built-in views. Native
inspection is authorized. Agent TUIs, arbitrary splits/floating windows,
marketplace and survival after full application termination stay deferred.
Spending, security changes, substantial dependencies and publication remain gated.

Implementation and fresh validation are in progress; no new completion claim yet.

## Current September 12 — state restored; docking decision pending

This entry supersedes older current-state wording below; retain the historical
evidence with its named build. This turn changed records only, not application code.

- Checkout: `C:/pytxo`, `codex/beta-candidate-verification`, HEAD
  `eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78`. Fresh GitHub inspection finds PR31
  still open/draft at that exact head; no successor was found. Before these record
  edits: 94 tracked modified files, 77 individual untracked files, no staged diff.
  Preserve the entire dirty tree; no reset, stage, commit, push or branch switch.
- Only this root is active in the collaboration tree. The known same-checkout
  `Audit Pytxo product strategy` task completed read-only on September 10; no other
  active Pytxo writer was visible in the task inventory. This is an observation,
  not a lock against an external editor. No native/build process was found in the
  bounded process query; no native screen inspection occurred.
- Fresh remote evidence: CI run `34181828835` has 12 successful jobs for PR31's
  committed head. The old pre-start billing rejection is historical. It does not
  explain a current failed run. The September usage API reports zero net Actions
  charges; the Actions budget remains $0 with stopping enabled. Remaining included
  allowance is absent from that response. No workflow was dispatched. Public mirror
  Latest remains v1.2.1; the local candidate is not public.
- The September 10 MSI `9676d38f...` and extracted EXE `e0eeb950...` still match
  their recorded hashes. All 361 frozen inputs match the manifest; 359 current
  inputs match byte-for-byte. `apps/desktop/README.md` and
  `apps/desktop/src/components/setup/SetupStepWorkspace.svelte` differ only in
  CRLF/LF: normalized text is identical. Do not claim raw source identity or undo
  those changes. No new build or runtime acceptance was performed.
- The exact `pytxo-dock-proposal-01` packet under the private September 10
  visualization directory is intact: all nine manifest entries match. Reuse its
  `PROPOSAL.md`, three PNGs and `proposal.html`; the active-agent PNG was inspected
  again. These are illustrative designs. The prior HTML browser-policy rejection
  remains respected; no browser preview or workaround was attempted.
- No subsequent explicit approval of that exact docking proposal was found in
  this task or the checked coordinating/audit history. Earlier approval covers the
  bounded Work/Review hierarchy pilot, not docking, a user shell or agent TUIs.
  Desktop-control permission does not approve redesign. Stop at this decision.

`RELEASE_PLAN.md` now carries the finite five-checkpoint completion sequence;
`RELEASE_READINESS.md` distinguishes the committed, local, packaged and public
states. Historical 6acc Cancel/Apply/restart and 11+26 checks remain valid for 6acc;
the e0ee launch is recorded, but its complete native acceptance is not. Older
release-proposal wording saying e0ee never launched is superseded by this entry
and the existing September 10 resumption record. Final-source CI, clean Windows,
matching final-build demo/materials and approved public delivery remain unfinished.

**Single next action:** Matt approves or declines the existing mission-workspace
and right/bottom-dock MVP. Do not repeat polling or resume the broad audit while
that decision is unresolved. No new agents, tools, model changes or global settings.

## Current September 10 — ownership/Git repairs packaged; native paused

Read [[astra-native-finish-2026-09-10]] for current source scope and acceptance.
The preceding 6acc review-pilot payload completed a real native mission, Cancel,
Apply and restart. Run `f3e164cd-928a-43eb-b409-ba726b8c33b7` used one Codex
harness for three scoped tasks in two waves (maximum two workers), completing in
300.140873 seconds. Six primary files stayed unchanged through Cancel. Apply
wrote exactly three reviewed files; 11 project tests and 26 independent checks
passed. Restart retained the same package and one committed receipt. The sanitized
record is `tooling/benchmarks/results/astra-native-review-2026-09-10.json`.

Native inspection exposed two bounded defects: Review Ownership path used planned
agent labels after dispatch reordered tasks; startup briefly opened a Git console.
Prepared file attribution and actual Apply ownership were already correct. Review
now uses unique recorded domain/run/task actors, otherwise “Worker not recorded.”
Eight noninteractive Git sites share a Windows CREATE_NO_WINDOW constructor.
Arguments, cwd, environment, PTYs, permission policy, verifier deadlines, Stop and
Apply authority are preserved. Existing profiles/domains retain their contracts;
the observed disposable execution domain uses Orbit.

Current package under `target/astra-native-finish-20260910/`:

- MSI `native-build/Pytxo Desktop_1.2.2_x64_en-US.msi`: SHA256 `9676d38f5749b7ba81a2799c56c423753840fc43e15874de5351a5868a678ea1`, 10,854,400 bytes.
- Extracted EXE `msi-extraction/payload/PFiles/Pytxo Desktop/pytxo-desktop.exe`: SHA256 `e0eeb950c53010a1f1834cff3b0c2dae93397e03d062f82d5c3f7499463090b2`, 36,733,952 bytes.
- 361-input manifest: SHA256 `282c2c3ea129b154feaad27259b38731670845c585fe2cd423f2efbccbcc8e7a`.

All current/frozen inputs match; eleven inputs differ from 6acc. MSI build passed
with one worker: 9m04s native compilation, 6.49s frontend, 587.802s full command.
Extraction/runtime-import inspection passed. Both artifacts remain unsigned;
standalone/extracted EXEs differ only in three UNK-to-MSI bundle bytes. Independent
package review passed 21 checks. Source reviews and verification addendum remain
under `target/astra-review-20260910/native-independent-review/`.

Affected verification: 65 UI tests, Svelte zero errors/warnings and CSS lint,
193 Rust library tests (77/39/77), 21 worktree/candidate/Apply integration checks,
strict all-target Clippy for core/runner/orchestrate/Desktop and formatting passed.
The detached-console regression has actual red/green evidence. Initial default-
parallel libraries failed the existing two-second fast-verifier test once; unchanged
isolated and bounded two-thread runs passed. Cause is unproven; failed evidence is
retained. Prior 170 browser/42 Storybook results remain earlier pilot evidence,
not a rerun after these repairs.

The new Windows packet is complete under `windows-validation/`: ZIP
`f614e293ab196ee02d1017723367490060a9d02abd0db9f306f6541458441556`, ISO
`fead94437a2ddde31812e0b7ca6c91cd6375069782c7200d1928ab61db462137`.
All 21 entries match, and fourteen challenge/history files and older packets are
preserved. This is preparation, not clean-Windows installation or guest acceptance.

Native control is paused after physical Escape during the latest explicit
resumption. The older 6acc process closed normally. The exact e0ee payload launched
at 06:01:39 UTC and its Work screen restored the historical f3e164cd run. PID 3364
remained live in the subsequent read-only audit. A Review click left Work visible;
the coordinate retry was stopped, so the ownership repair is not natively accepted.
No fresh mission or recording started. The new fresh fixture is
`C:/Users/mattbaconz/AppData/Local/Temp/PytxoNativeFinish20260910/approval-risk`,
commit `59c10833efec1320ce10d3fc0281745ebf63cb58`. Its six baseline files are
unchanged; eight project tests pass, and independent baseline checks show 21 pass
and five expected failures. Launch initialized a database with zero runs, agents
and events. Capture and screenshot directories remain empty. The launch and
resumption records are in the new artifact directory's `native-run/`; the independent
resumption audit is under `native-independent-review/`. Resume input or recording
only after explicit native resumption.

The 6acc raw video fully decodes and has inspected Review/Apply intervals, but
contains a confirmed Git startup interruption and private background during
restart. An earlier black-content report was withdrawn after full-resolution
inspection: small dark thumbnails had been misread. Historical footage cannot
certify e0ee or the final smooth demo. See DEMO.md for the current edit record.

The requested game-like mission/crew mockup is a visual proposal only. Its
Focus/Control toggle and structural redesign are excluded from this build and
still require approval. Preserve current Work/History/Setup and the approved
hierarchy pilot. No account changes, spending, Actions dispatch, commit, push or
publication occurred. PR31 CI still covers eb5f73d, not the dirty candidate;
remaining included Actions allowance is unresolved. Clean Windows requires safe
RAM headroom and direct owner sign-in. Public-download and final-demo gates remain
open. Normal Git index remains unchanged.

Next native action: explicit “resume native testing”; inventory current windows
and continue the exact extracted e0ee payload if it remains open. Launch only if
absent. Reobserve Review, inspect runtime ownership, then select the fresh fixture
and exercise startup, Cancel, Apply, post-Apply tests and restart.
Record the actual app and separately witness startup consoles; avoid unrelated
desktop footage. Do not rebuild merely to capture this already identified package.

## Earlier September 10 — 891 package ready; native control paused

The user pressed physical Escape again at approximately 23:12 UTC September 9.
The native tool stopped during window inventory, before the new payload launched.
No further native tools or new capture followed. Automatic goal continuation
does not resume native control. Explicit user resumption is the next native gate.
The preceding turn made progress: f968 completed Apply/restart, a real 94-second
video exposed a netsh startup popup, and the bounded repair was tested and built.

Current package under `target/astra-ux-20260909/native-build-netsh/`:

- MSI `7d30141b1d7c7be15fb8d44e09970508306c5e2c89efa390da38bb222897344a`, 10,854,400 bytes.
- Packaged EXE `8910d153d73cff2973892c15a9bba58a797617e1ecd3af865c21991f39e2593d`, 36,733,952 bytes, under `msi-extraction-netsh/payload/PFiles/Pytxo Desktop/`.
- Standalone EXE `aaeec206c297c0097cb6c230f5730d804ea02c9cc42dabb64488bd4609ef4ddd`; only the three UNK→MSI bundle bytes differ.
- 359-input manifest `019fa099b4e12b29cddde5f3aeee3f6b5982d44c9b138b7058b636c64ceb2ac5`; all current/frozen copies match.

MSI build completed successfully in 4m37s native compilation; frontend built in
6.91s with unchanged `index-DnFef0wh.js` / `index-0MkAvHdn.css`. Build session 16833
is terminal, exit 0. Packaged dependency inspection passed; MSI/EXE are unsigned.
The independent build review passed 16/16 checks. Only network_isolation.rs
differs from f968. Its read-only netsh receipt query now suppresses console
creation, preserving arguments, result interpretation, profiles, firewall-install
logic and ConPTY. Scope: existing local receipt construction in one execution
domain, observed with Orbit. A dedicated detached-parent test actually failed
before the flag (exit 101, console created) and passed afterward. All 77 runner
library tests, strict runner Clippy and formatting passed. The unchanged frontend
retains 157 browser tests and zero Svelte errors/warnings plus CSS lint. The 42
earlier story checks were not rerun for this follow-up. No native acceptance is
claimed for 891; do not reuse f968 proof as acceptance of these bytes.

`launch-netsh-native.ps1` is prepared but unexecuted. `native-run-netsh/` contains
the fresh `PytxoMenus20260910B/fixture`, six exact baseline files, eight passing
project tests and five expected independent failures. No mission or recording
started there. Its read-only snapshot helper requires an explicit run UUID.
After resumption: inventory windows, close predecessor normally, launch 891,
record startup/verification, then run Review → Cancel → Apply → tests → restart.
The previous f968 process 29936 was still present in a read-only process query;
no current screen state was inferred from that query. Preserve all prior fixtures.

The f968 host result remains valid for its named bytes: run
`b6446140-6586-4f9f-a2ab-58282100298b`, three tasks/two waves, 225.51 seconds.
Six primary files stayed unchanged through Cancel; Apply wrote exactly three
reviewed files, 11 project tests + 26 independent checks passed, and restart retained
the same package and single committed receipt. Candidate/final reviews passed
18 + 29 checks. `tooling/benchmarks/results/astra-native-menus-2026-09-10.json`
binds those facts to MSI 0c2d70e0… / EXE f968d665… / source 351aaa8f….

The private 94-second Remotion walkthrough is
`native-video-followup/out/pytxo-native-apply-f968.mp4`, SHA256
`fb0e00a3eb476e9b644218a89ee9086a556323c87c1afc2a71851a03da6ecd3d`.
It contains 88 seconds of actual 1× video, a 6-second labeled later restart still,
and uninterrupted Apply at output 67–88s. Startup netsh interruption is retained
and labeled; verification frames showed no interruption in the stated sampled
intervals. Decode/typecheck/frame QA passed; normal-speed human playback is
unverified. The sanitized film record is `astra-native-menus-video-2026-09-10.json`.
This is correctness evidence, not a finished console-free demo. Old 073/c77 films
and raw/failed takes are preserved; unrelated desktop-tail footage stays private.

Read-only post-stop audit confirmed PR #31 remains draft/open at eb5f73d with 12
green jobs from 34181828835. Those checks do not cover the dirty source. Public
release is still v1.2.1. Current September usage reports 616 Linux, 4 ARM, 419 Windows,
135 macOS minutes and zero net Actions charges; the $0 stop budget remains enabled.
The API does not establish remaining included allowance. No runs were dispatched.
Raw responses are under `post-stop-audit/`. Preserve native-first consolidation
and the existing PR #31 authorization; no quota probe, budget change or publication.

Clean-Windows validation also remains open. Existing VM/install work is authorized,
but the guest needs safe RAM headroom and direct owner sign-in. Automated password
entry was rejected twice by tool approval review as “blocked by policy”; do not
retry through another input path. Host execution is not clean-install evidence.
The new private validation packet is complete under `windows-validation-netsh/`:
ZIP `588c4407619ae732312eea6e9168d6ba13aac69e93064739d55ce78e7fa571c6`,
ISO `2adf651bebba55a724e062463cd4e6c5eac35ef34368310307585b05750d2b05`.
All 21 packet, ZIP and extracted ISO entries match; 14 challenge/history files
remain unchanged. Every native/guest result for 891 is unexecuted. Prior f968
ZIP/ISO and guest disk/controller remain unchanged. Root owns source,
runtime and final integration; specialists own only their bounded private artifacts.
No normal-index staging, commit, push, spending, account change or publication.

`native-netsh-verification.json` consolidates the tested build and prepared packet
with explicit evidence limits. The selective source handoff is prepared under
`source-handoff-native-reviewed/` using a separate Git index; its manifest binds
the archive tree and file hashes. Git normalization applies to that archive;
the 359-input native manifest separately identifies the actual compiled bytes.

## Preserved September 10 — 073 acceptance and repair preparation

Matt explicitly resumed native testing. Packaged EXE `073a1a3063…` completed
run `0738e1eb-5db8-486a-b9dd-24822e993fad`: three tasks, two waves, 226.25 seconds.
The six primary fixture files stayed unchanged before Apply and after Cancel.
Apply committed exactly the three reviewed files; 11 project tests and 26
independent checks passed. Restart retained Completed / Applied / Committed.
The sanitized record is `tooling/benchmarks/results/astra-native-menus-2026-09-09.json`.
This is native host evidence, not a clean Windows installation or hosted CI pass.

Desktop Duplication captured actual motion after window-title capture proved
stale. A private 42-second Remotion verification excerpt is at
`target/astra-ux-20260909/native-video/out/native-verification-excerpt.mp4`.
Its cuts and later restart still are labeled. Recording ended before Cancel and
Apply, and a visible verification terminal interrupts the source; this is not
the final full demo. Raw footage, screenshots, hashes and reviews are preserved.

Testing reproduced three bounded follow-ups: a verification console flash,
retained sidebar draft after dispatch, and live-view scroll inherited from the
plan. Root is repairing these and changing task wording to “Awaiting result”
where the backend has not recorded a terminal result. The runner change applies
to existing local verification in one execution domain, preserving permission
profiles, cancellation and Apply authority. Runner tests: 76/76; strict runner
Clippy passed. Final combined UI checks passed 157/157, Svelte zero errors/warnings,
CSS lint and formatting passed; 64 screenshots are archived in `browser-followup`.
The new MSI built in 4m12s native compilation and passed packaged runtime inspection.
All 359 current/frozen inputs matched; manifest `351aaa8f002ab293d9bcd74bd47251fba14259610e92fb172117f36fdccffa53`.
MSI SHA256 `0c2d70e0dc098f506aa5b2d7b76f209872d924f96aa5d7ad873aadc110b72d1f`;
packaged EXE `f968d665ca939c6635e6350f423218f5a5a74145769b3aca6273710bdcfd71a5`.
Both are unsigned. These bytes do not inherit the 073 native acceptance.

Native control remains authorized; no new physical Escape occurred. Fresh window
inventory after the overnight gap found Pytxo closed. The new f968 payload is now
running (launch record 20260909-222910, PID 26436) against the fresh disposable
`PytxoMenus20260910/fixture`, preserving the earlier fixture and profile. Native
dispatch cleared Continue draft and returned the live overview to the top.
Run `b6446140-6586-4f9f-a2ab-58282100298b` is executing; do not duplicate it.
Capture `native-run-followup/capture/native-followup-full-01.mp4` started
22:30:52 UTC and auto-ends at 22:45:52. Next agent action: inspect the candidate,
record Cancel/Apply while capture is live, verify exact bytes/tests, then restart.
No human action is needed for this local pass. Clean Windows access, included CI allowance and publication
remain separate gates. No Actions, spending, account changes or publication.

## Preserved September 9 — native readiness repair before resumption

Matt's explicit Computer invocation resumed local native testing. The preserved
fcd packaged build completed fresh onboarding, selected the disposable fixture,
reported existing Claude/Codex sessions ready, and opened Work and New run.
Readiness checks visibly opened unwanted console windows. Background probes now
use Windows CREATE_NO_WINDOW; the explicit vendor sign-in launcher is unchanged.
The physical Escape key stopped Computer Use again before draft entry. No native
input or new capture occurred after that stop; automatic goal continuation does
not resume it. The old completed run and both test profiles remain preserved.

The final repair passed **26/26 Desktop library tests**, including the real
Windows cmd wrapper, console absence, stdout/stderr and exit status. Strict
Desktop Clippy and formatting passed; independent review found no remaining
issue in the bounded change. An earlier test-fixture quoting failure is retained.
Only `apps/desktop/src-tauri/src/ipc_meta.rs` changed among the 359 native build
inputs; the preceding 154 browser tests / 42 stories cover the unchanged frontend.

Current candidate in `target/astra-ux-20260909/native-build-probes/`:

- MSI `26ef95bc5a87d721bcece8e9df6d8a92ff52dc3d63f7aef03483a2937551999b`, 10,854,400 bytes.
- Extracted payload `073a1a3063d9d41b95a38bdb6cc80f1b987080beae4638a5f17a172730028b35`, 36,733,952 bytes, under `msi-extraction-probes/`.
- Standalone EXE `bea5693316db74727c6f8a930107b75ed3534b64f2fdb4bc4802b14e9bdb8adb`; its only difference from the payload is Tauri's three bundle bytes.
- 359-input manifest `8d42c8b55dcc401d7b9f39a7f6342eb4e8396e24a4dc11bd7b1c0fc87d47a125`; every current/frozen input matched after building.

MSI build and packaged runtime-dependency inspection passed. The frontend rebuilt
in 8.55s with the same JS/CSS assets; native compilation took 4m05s. MSI and payload
are unsigned. This repaired payload has **not** been launched or installed.
`native-probe-verification.json` holds the evidence. The refreshed private packet
is `windows-validation-probes/pytxo-windows-local-validation.zip`, 21 entries,
SHA256 `b198756ba53d8ede67c38869e52f1fd70d530778d23e7afa31922f2ea7236320`.
All entry sizes/hashes match; 14 fixture/verifier/history files and the retained
installation/mission protocol are unchanged. Native result fields stay unexecuted.

Recordly 1.4's floating HUD failed target validation twice, including one recovery
retry. The fallback produced readable Work framing, but all twelve sampled tiles
of its 80-second raw clip show the same screen and pointer. Action timing was not
synchronized; it proves neither capture failure nor working motion capture.
It is diagnostic footage, not a demo. No mission was dispatched or Apply attempted;
all six primary fixture hashes still match baseline. The fresh baseline passed
eight tests and the independent baseline produced 21 passes/five expected failures.
Details: `native-run/rehearsal-result.json`. Historical c77 footage remains separate.

Next human action: explicitly resume native testing. Then close the predecessor
normally, launch the new exact payload, recheck background readiness and prove a
timestamped concurrent UI/capture transition before the full mission recording.
Clean Windows access, confirmed included CI allowance and publication decisions
remain separate gates. No Actions run, spending, sign-in/password change, normal
staging, commit, push or publication occurred during this follow-up.

## Preserved September 9 — deeper local menu and workflow pass

Matt explicitly reopened the implementation scope for a much better sidebar,
every menu and local UI testing. That work supersedes the earlier conclusion
that no selected product edits remained. See
`docs/01-projects/astra-desktop-menus-2026-09-09.md` for the complete before/after
record. Work / History / Setup and the reviewed Apply boundary remain intact.

The current source adds retained mission inputs with original workspace/agent,
workspace-scoped command actions and Stop confirmation, grouped/searchable
settings, keyboard-operable workspace and consent dialogs, clearer responsive
agent actions, consistent light-theme controls, filtered History detail and
working voice capture preferences. The adversarial reviewer found and then
cleared draft-scope and keyboard-recording lifecycle bugs. Source ownership is
back with root; no parallel writer remains.

Final combined-source validation passed: **154/154 production browser tests**
(3.1m), **42/42 rebuilt Storybook checks** (32.4s), Svelte zero errors/warnings,
CSS lint and the Windows MSI build. The browser run saved **64 screenshots**.
All nine Setup sections, workspace/command/consent menus, draft recovery,
workspace-scoped Stop confirmation, History, responsive layouts, custom scrolling
and the existing review controls are covered. These tests use preview fixtures.

The same 47 Tabler icons now use 84 public direct imports across 16 files.
Production build: 264 modules / 8.32s, versus about 66s immediately before.
Storybook: 272 modules / 1m39s, versus the preceding 6,377 / 6m46s. These are
local observations, not CI guarantees; Storybook's large-chunk advisory remains.
The final missing-workspace story reproduced and then verified repair of a
misleading restoration notice. Independent review cleared the scoped changes.

Current preserved candidate in `target/astra-ux-20260909/native-build-accepted/`:

- MSI `3bd609e083fa61897f87db904f0f99e8c71bd738b6b2048ebfad0e6049f9ae4f`, 10,854,400 bytes.
- Extracted packaged EXE `fcd039e122a2c46c05e9c0e97ff9a036e6f4fc9e876cd005ae6e7960b9692b05`, 36,733,952 bytes.
- Standalone EXE `a4e790ff52f58257aa80d7b726ac968d2d1ae8bcac6571d213825ab00876116d`; use the packaged identity for native acceptance.
- 359-input manifest `312223caa0f75b5f321e6dc37eeb90d56cb22177f1ed4faa3de4457f9dae9342`; current and frozen copies match after building.

MSI extraction and packaged PE runtime-dependency inspection passed. The MSI is
unsigned; it has not been installed or launched for acceptance. Complete logs,
hashes and screenshot inventory: `target/astra-ux-20260909/verification.json`.
Earlier failed runs and superseded local builds remain preserved. Neither the
e775/7abe candidate nor c77 film validates these new menus.

Local review: `http://127.0.0.1:5174/#/work` (production UI, fixture data).
Next human action for this increment: try New run → enter an outcome → Setup →
Continue draft, then inspect both Appearance themes and Ctrl+K. The broader goal
still needs native control resumption for exact-build testing/recording, clean
Windows access, confirmed included CI allowance and publication decisions.
No native input, VM boot, sign-in, password change, normal-index staging, commit,
push, Actions run, paid action or publication occurred. Existing approvals persist.
The initial 71-path menu handoff remains preserved under
`target/astra-ux-20260909/source-handoff/`. The subsequent documentation/capture
handoff is `target/astra-ux-20260909/source-handoff-followup/`, with 72 intended
paths, current hashes and an archive constructed through a separate Git index.
The normal index and branch are preserved; raw recordings, private notes and
unrelated work are excluded. Native source inputs and the tested MSI are unchanged.

Continuation audit: PR31 is still open/draft at `eb5f73d`, with the same twelve
green jobs; no current-source run was dispatched. The execution and product
decision notes and demo README now distinguish the current 3bd/fcd candidate
from historical native observations. DEMO's current 60-second capture plan leads
with real native pointer/hover/click/scroll interaction and includes a ten-second
recorder check before the full take. No new footage was claimed or generated.
Read-only preflight found Recordly 1.4.0.0 and media tools installed, but native
Computer Use remains unavailable in this runtime and the previous Escape stop
has no explicit resumption. The next human action for the remaining native stage
is to enable native control and confirm resuming local Pytxo testing/recording.
No recorder preference, account, security, VM or spending changes were made.

## Preserved earlier September 9 — reviewed source and remaining gates

The selected UI, website, film/privacy and packet work is complete locally at the
artifact identities below. Independent scope review found no additional selected
product implementation to add while native acceptance is pending. Execution,
evidence and release notes now lead with the polished candidate instead of older
checkpoints. `target/astra-polish-20260908/goal-audit/source-handoff.md` identifies
30 modified and 24 new intended files; unrelated work, raw clips and the private
master brief remain excluded. No staging, commit, push or Actions dispatch occurred.

Existing authorization covers further PR31 CI within the included allowance;
do not ask again for that same authorization. September billing API evidence is
saved in `goal-audit/actions-september-usage.json`; the live Actions budget in
`goal-audit/budgets-current.json` is $0 with further paid usage blocked. Reported
minute quantities are Linux 616, ARM 4, Windows 419 and macOS 135, all attributed
to private `pytxo`; all reported Actions net amounts are zero. Historical 1/2/10
weighting would imply 2,808 used of 3,000 and 192 remaining. These are conditional
calculations, not a verified live allowance. Current official docs omit those
factors, while still referring to multipliers. A comparable run models 201 with
per-job rounding, so do not dispatch a whole run as a quota probe. Verify the
owner dashboard when native control can resume; preserve all required checks and
the $0 cap. See `goal-audit/release-scope.md` and the release proposal for sources.

The unanswered native-resumption request has recurred across at least three goal
turns. No input was issued after Escape. Remaining outcomes need that response
(native workflow, polished-build film, owner billing view), guest access for the
clean-Windows protocol, or later explicit publication/redesign decisions. The
next human action is permission to resume Computer Use for testing and recording
the exact e775 MSI / 7abe payload. The goal is not complete; prior c77 native proof
does not close this candidate's acceptance gates.

## Preserved September 9 — redacted film and Windows packet ready locally

The polished Desktop remains frozen at MSI `e7752cf4…`, packaged EXE `7abe2097…`.
All 356 current inputs and their frozen copies were rechecked at 17:15:50 UTC
September 8; zero drift. Native testing/recording of this exact build remains
pending. The prior Computer Use session was stopped with Escape. The request to
resume is unanswered; no native input, VM boot, sign-in or password change occurred
during this continuation. The pending request remains unchanged while independent
authorized film/site work progressed. Do not mark the goal complete.

The 58-second checkpoint film is now visibly redacted and reviewed:
`apps/demo-video/out/pytxo-aperture-native.mp4`, 7,696,333 bytes, SHA256
`925ddfcf60d94f83c75d3ec86755cdc765036d3899b08410a5dfce6a734400e2`.
This records the preceding c77 build, not the new polish. Three raw MP4s remain
unchanged/private and are explicitly ignored; original master c1d8f262… and props
are preserved under `target/astra-polish-20260908/film-privacy/`.
Labeled opaque masks bind exact source hashes and follow the Root row's movement
without obscuring the modal digest. All 1,920 source frames decoded, 29 original
capture hashes checked; all 26 selected encoded master frames visually inspected
(root 8 critical boundaries, independent reviewer 18). Master and all 26 image
hashes independently verified. Typecheck, 14 rejected invalid edits, native asset
consistency and 58s/60fps/H264/BT709 format/black scan passed. Nine holds match the
reading/captured states. Complete 1× machine playback: 3,480 frames, 10 dropped,
zero corrupt, no media errors, one 332ms startup wait; no continuous human watch.
No native 60fps claim. Twenty-two scoped film input files are frozen separately;
manifest SHA256 `a016cf33d30c87582f8ddb31b1ea94077e9fb79eca10b0d97c105586a340a621`.
Canonical film record: `tooling/benchmarks/results/astra-aperture-demo-2026-09-09.json`.
Final independent reconciliation found no remaining discrepancy:
`target/astra-polish-20260908/film-privacy/final-reconciliation.md`.
New fractional trims/source/camera edits require fresh timing and coverage review.

New external Windows packet:
`%TEMP%/PytxoPolishedWindowsPacket20260908/pytxo-polished-windows-validation-packet.zip`,
21 entries, 10,636,189 bytes, SHA256
`e8d05cd1ad545382d107dda3dd4743ceac8188cfcab465a343226f3d49dc6ffd`.
Independent entry inventory and root ZIP hash pass. Six baseline fixture files,
verifier and mission texts retain their bytes. The guide preserves validation and
recovery commands/requirements with updated candidate/history references; its
original and old c77 evidence are retained as history. Includes e775 MSI, 7abe payload identity and the 356-input
manifest. Staging is not installation or execution; existing guest history stays
explicit. No clean-Windows gate was closed.

Website review found and fixed stale “current candidate” wording, unlabeled mobile
corpus values and unsupported open-source wording in the footer. It now identifies
the recorded checkpoint separately from the newer unverified native build, gives
each mobile metric a visible semantic label and uses neutral “Rust core” copy.
Web build/lint and 3 focused tests pass (`web-build.log`, `web-lint.log`,
`web-evidence-tests.log`). Two screenshot tests reran after correcting only their
scroll position beneath the fixed header (`web-viewport-tests.log`); 2/2 pass.
Four final 1440/390 viewport images independently accepted, root inspected mobile.
Evidence is under `target/astra-polish-20260908/web-viewport-results/` and `web-review/`.

No Actions, pushes or publication. Prior 12 green checks still belong to eb5f73d.
CI handoff recommends one consolidated PR-only push after native proof and the
applicable authorization; main merge separately deploys the website. Focus/Control
redesign approval and publishing/security/spending gates remain unchanged. Exact
next human action: allow Computer Use to resume for this polished build's native
test and recording. Preview remains at 127.0.0.1:5174, CUA IAB tab4 marked deliverable;
film server on 9419 serves only the MP4. Isolated QA browsers are closed.

## Preserved September 8 Desktop polish — source changed after native checkpoint

Matt requested Cursor/Codex-level polish, custom scrollbars and a much cleaner
layout. Root implemented bounded Work/History/Setup polish; see
`docs/01-projects/astra-desktop-polish-2026-09-08.md` for the complete before/after
table. Focus/Control proposal approval remains pending; no implementation of that
larger navigation change. Existing publishing/security/spending gates remain.

Production browser suite 141/141 passed (`target/astra-polish-20260908/release-tests-02.log`),
legacy development test 1/1 passed (`dev-tests.log`), Svelte0errors/0warnings and
CSSlint passed (`check-final.log`). This includes the repaired startup bug that
overwrote saved Light with Void. Independent source review found no remaining
issues after authority-copy, destructive-contrast and CSS-cascade fixes.
The final keyboard/switch styling passed7/7 focused checks against the exact MSI
frontend (`packaged-frontend-tests.log`, screenshots`packaged-frontend-results/`).
Storybook built in4m34s and all41stories passed (`storybook-tests-final.log`).
Fresh component loading/empty/error/offline screenshots and reduced-motion check
are in`state-captures/`; these have intrinsic Storybook host height and are not
native-window viewport proof. CSSlint passed after the last switch-hover repair.
Independent visual review found no material issue in Light1280/960Work and
Apply/Discard dialogs plus Appearance110%; root inspected dark/Light screenshots.

New frozen356inputs SHA256`58534b17878177ac4340d8dd8ea85e1986c29e59a3acf46332aee2d88ff460be`;
source and frozen copies still match after the successful2worker native build.
MSI10,850,304bytes SHA256`e7752cf4f704110319efd26711c264f516fbc3dc818f354f2f335cfa3166f1af`.
Extracted packaged EXE SHA256`7abe2097b8c886aaccbd27cb35882bbf0f28f81e3d9b435975082c6ff6183947`.
Standalone build EXE528038… differs only in3ASCIIbytes`UNK`→`MSI` at31171048,
matching Tauri's recorded bundle-type patch. Use the extracted7abe executable for
future native proof. MSI dependency inspection passed; this is not installation
or launch evidence. Exact outputs, source copies and records are under
`target/astra-polish-20260908/native-build/`. Old475MSI preserved there separately.

Built preview server95668 serves127.0.0.1:5174. CUA IAB browser1/tab4 is marked
deliverable and has completed preview-only onboarding through UI; Work is visible
with sample data. No tool installation, agent authentication, folder selection or
notification permission was performed in that preview. No native input after the
previous Escape stop. Native new-build interaction/capture is the next stage;
clean-Windows, exact-current-source CI and publication gates remain open.
No new Actions, push or publication.

The previous355source inputs are preserved byte-for-byte in
`target/astra-motion-20260908/native-build/frozen-source/` with a copy record.
The c77 executable / 475 MSI / prior native mission below are historical checkpoint
evidence, not this newer source. Guest password unchanged and clean-install gate
still open. Native Computer Use was stopped with Escape during the prior turn;
no subsequent native input has occurred. Isolated browser previews only this turn.

The original58sRemotion film finished and is now archived as
`target/astra-polish-20260908/film-privacy/original-unredacted-master.mp4`,
7,678,376bytes, SHA256`c1d8f262d9e74b80f9ecb19610c6af0de7860d5494ac7a88e40388e4e5db3a5d`.
3480frames,1920×1080,H264/BT709,60fpsencoded,silent. Format/black scan passed.
Sample review covered22frames/all7scenes/transitions; continuous1×machine playback
ended without errors (3480totalframes,12dropped,0corrupted,one267msstartupwait).
Not a human full-film watch or proof of native60fps. Host-profile path visible in
Apply/Result samples: crop or visibly redact before any public export. Raw proof
stays private. New polished native footage still owed. Website build/lint and
2evidence tests at1440/390 passed for the c77 checkpoint JSON; no deployment.

External Windows packet ZIP SHA256`89e6a6ffe9b3271c221605f49262ffddf90108f6b4d0878d689ae58d3080522b`
has16entries/10,607,961bytes and passed independent inventory/hash audit including
all6baselinefixture files. `target/astra-polish-20260908/packet-audit.json` labels it
checkpoint-only; it contains the475MSI, not a future polished build.

## Preserved earlier September 8 checkpoint — native proof and film preparation

This section supersedes the temporary state below. Matt requested smooth, distinctive
Remotion MP4s with actual native interaction proof and an optional simplified/customizable
layout. The goal is active. Existing publishing, spending and major-redesign gates remain.
No new Actions run or publication occurred in this continuation.

Six bounded native source changes are frozen in the 355-input manifest
`target/astra-motion-20260908/native-build/source-inputs-sha256.json` (SHA256
`c75246603cca1c2947a80f3d405a3ba932bf6961c58d23c019a7e98f0c52fa90`).
Main min-height fixes real wheel reachability; the narrow review grid now stacks without
clipped Inspect controls; app/OS reduced motion stops actual animations and pseudo-elements;
scale/density choices expose selected state. Desktop check:0errors/0warnings, production
build passed. Focused review/motion browser suite:22/22 passed at1280×800 and960×640.
Actual WorkLoading animation red/green evidence is retained. Prior81/82 run exposed the
narrow-grid issue before its repair; preparing-SVG and initial wrong-route results are
explicitly not evidence of the loading-animation defect.

New MSI built with two workers; all355 inputs rechecked and packaged PE dependency check
passed. MSI SHA256 `475f7f9125c84f2f3fd2ca092533005f915a080bc3f4e73676d32df977795f5c`;
EXE `c77b8418cf0d2a5f3303f10220ac484fd804909cb5dd314cffb06b559bc88d43`.
Old cf8MSI is preserved as `native-build/previous-cf8-msi.msi`; old proof remains historical.
This extracted-payload host rehearsal is not a clean install, signing or current CI proof.

Fresh native run `16dab199-72fa-4596-b6fd-610e7c0ebb4c` used existing authorized Codex0.153.4,
Orbit,3tasks/2waves/max2 in `%TEMP%/PytxoAperture20260908/fixture`. Native Computer Use
exercised onboarding, Windows folder picker, task editing, dispatch, real wheel access,
all3exact diffs, Cancel, Apply and reopened receipt after process restart. No temporary CSS.
The code task precedes Tests; Docs is independent and feeds the combined candidate.
All3combined checks passed. All6primary paths and contents stayed unchanged before Apply
and after Cancel (complete inventory excludes only.git/.pytxo). Package
`ceee5c50c2add38a852bca21569773ca08073dba1ae38624b0a89f6804cda1d0` applied as attempt
`adef4a39-9515-41d5-b1ce-7d6da31426d3`, committed. Post-Apply:11repository+26independent
tests passed,3changed hashes match package,3other hashes match baseline; receipt survives
restart. Raw private events contain private global instructions; do not publish them.
Sanitized evidence: `tooling/benchmarks/results/astra-aperture-native-2026-09-08.json` and
`astra-aperture-plan-2026-09-08.json`. Local Bench now references this proof; web build/QA
is still owed after its latest edit.

Native app currently PID28068, started15:13:28.3896082Z, window210830640, CDP9327,
profile `target/astra-motion-20260908/native-run/home`; observe exact ownership before
closing or input. Actual renderer is1600×1000 pixels for1280×800 CSS atDPR1.25.
Three real MP4clips under `apps/demo-video/public/product/aperture/`: Review13s
(source0–9 plus50–54,41s cut at9); Apply11s continuous(source13–24); result8s afterrestart.
Raw frames, valid timestamp checks, per-frame SHA256 and cut provenance remain under
`target/astra-motion-20260908/`. Renderer capture omits OS chrome/cursor.60fps is encoded
cadence, not measured native performance. Diagnostic old captures are not final footage.

New58s/60fps Remotion composition integrated alongside original52s film. Imported JSON
is strictly validated; assets gate verifies exact planner graph, video hashes/metadata and
Bench claims. Combined film typecheck and asset validation passed. Render session55573
(`film-render.log`) uses2workers. Pending: render completion, fullresolution/transition
inspection, continuous playback, metadata/blackscan/hash record, website build/affected
browser tests/screenshots, final reviewer fixes. Do not report a completed new film yet.

Focus/Control interactive proposal at `%TEMP%/PytxoApertureLayoutProposal20260908/`
passed10checks, desktop/mobile screenshot inspection and keyboard/motion checks. Server
session19507 serves only this proposal at127.0.0.1:9418; root IABtab2 is open. Explicit
async approval question for implementing this larger layout is pending. No answer is not
approval. No product customization-as-shipped claim. All3specialists are read-only/idle
except final reviewer; root owns checkout/native app. Storybook server41964 uses6006.

Computer Use works through @oai/sky in Node REPL. Earlier exec Windows-sign-in commands
were rejected by automatic review (“blocked by policy”); do not circumvent. Guest password
is unchanged. The guest showed Windows updates7%, then normal guest shutdown14:12:58;
sessiona747441a… stopped, disk preserved. No QEMU currently running. Easier guest password,
clean-Windows final-byte validation and public-download gates remain open. The latest MSI
has not run in that clean guest. The old12-job CI belongs to prior source, not these changes.


## Current state — final CI passed; final MSI installed; visible guest awaits owner sign-in

**Explicit sign-in retry after 13:44 UTC:** Matt said **"sign in for me"** while
the third resumed blocked-audit turn was active. Root took back VM input, observed
the lock screen, pressed Enter, and inspected the masked password field. With
this explicit authorization, root retried the same ordinary combined sign-in
command through execution-tool review. It was again rejected before execution,
with only **"blocked by policy"**. No password was read/typed and no temporary
credential input was created by either rejected command. Authorization is clear;
the remaining barrier is the execution-tool restriction. Do not ask Matt to
authorize the same automated action again or retry through alternate indirection.

The supported remaining path is direct owner input into the already-visible
QEMU console using the existing local guest account file. `Ctrl+Alt+G` releases
input grab, as indicated by the actual window title. Hand guest input back to Matt
until he confirms Windows desktop readiness; do not capture or type during that
handoff. The same manual-sign-in/tool blocker has persisted across the three
resumed turns; mark the goal blocked again. Preserve the current monitored VM and
all evidence. No additional CI, build, account settings or publication changed.

**13:24 UTC blocked audit:** the prior turn was a verified wait on live launcher
83838 and its QEMU child, not new product/guest acceptance. The same direct-owner
sign-in requirement now persists across the console-preparation turn and two
automatic continuations. No owner confirmation arrived. Launcher83838/QEMU17140
remain live; current resource sample is3,898MiB free/20,508MiB unused commit with
zero pressure samples. Guest input remains with Matt; no keys or screen captures
were sent during the handoff. Mark the goal blocked, not complete. The visible
console remains available under its existing memory guard. Resume after Matt
confirms Windows sign-in, first checking this exact session/process state; do not
blindly launch a replacement or retry automated password entry. No additional
Actions/build/publication occurred.

**Latest continuation:** memory recovered at 13:01 UTC. Root raised only the
launch floor to 5,120 MiB; eleven boundary assertions and independent review passed.
Guest stays 2 GiB/two CPUs, with three samples spanning 15 seconds, post-ISO-hash
check and unchanged 1 GiB/4 GiB runtime floors/two-sample shutdown guard. Prior
policy/results are retained under `resource-resume-20260908-1302/` in the VM root.
The 13:04 guest reached Windows sign-in without memory pressure.

Automatic approval review rejected the combined shell command for reading the
task-local disposable password, writing temporary input, typing it and removing
the temporary file. The only reason returned was **"blocked by policy"**. None
of that command executed; no password was read/typed or temporary input written.
Do not retry the password through another tool or repurpose the OAuth broker.
Independent review found no existing private Windows-password input path.

Root prepared a supported **visible SDL console for direct human sign-in**.
Only ignored VM-controller code changed: explicit `--interactive` is online-only,
selects SDL and uses `windowsHide:!interactive`; default launches stay hidden.
The first SDL attempt retained the hidden flag and had no visible main window;
that was corrected and independently reviewed. Both preceding sessions `df0630c2…`
and `cde1e4d0…` shut down normally (`guest:true`, `guest-shutdown`, QEMU exit 0),
with no resource abort. All disk/data remain retained; contents preservation is
not a new verified claim. No Pytxo build/source, CI, account setting or publication
changed.

**Current live guest:** launcher **83838**, session
`a747441a-d59b-49ef-8ce4-81bfb2158a58`, manager **12692**, QEMU **17140**,
started **13:15:51 UTC**. Three launch samples were 6,572 / 6,563 / 6,558 MiB;
the post-hash check passed. Actual main window:
`QEMU (Pytxo clean Windows 10 validation-0)`, handle **1638974**, inspected through
process metadata. The guest rendered its normal Windows lock screen. Enter was
sent once; no password input followed. Host free RAM was 4,207 MiB at 13:17:28.
Latest safe guest capture: `screens/console-visible-1319-password-ready.png`
(shows the lock screen, not a password-entry success).
Preparation evidence: `interactive-console-visible-20260908/console-result.json`.

**Input ownership is handed to Matt.** Async question asks him to sign in directly
using the saved local guest account file and confirm the Windows desktop is ready.
Do not send guest keys or take screenshots during his sign-in. Do not ask him to
paste the password into chat. Normal guest input/captures may resume after that
confirmation; independently inspect the desktop then. Status/resource monitoring
may continue without reading his input. The saved local account is in ignored
`target/astra-clean-vm-20260907/guest-local-account.json`; no credential contents
belong in reports or tool outputs.

After sign-in: recover the allowlisted installer log/result and packet-copy record
with prepared `command-export-installed-1304.txt`; then normal Start-menu launch,
optional CLI path, V4 prerequisite/fixture checks, serial-baseline adjustment and
fresh plan. Final MSI/EXE remain `cf8db2e…` / `d75be6…`. No guest vendor auth,
mission or Apply has run. Ordinary OAuth's separate InPrivate/private transport/
clean shutdown/fresh-manager contract remains unchanged. The goal stays active
on this first continuation that reached the new owner-input boundary.

**12:06 UTC blocked audit complete:** the previous continuation made no progress;
it measured 3,444 MiB free RAM and confirmed no guest process. This third resumed
turn now measures 2,888 MiB, with no QEMU process and unchanged worktree inventory.
The same physical-memory blocker persists and no user app may be selected for
closure without authorization. The selected-scope gap review found no independent
implementation remaining. Mark the goal blocked again, not complete. Next action:
free enough unused application memory to sustain at least 5 GiB available before
the monitored 2 GiB guest retry, or identify an unused app the assistant may close.
No CI/build/publication rerun occurred. All artifacts and the guest disk remain.

**12:05 UTC resumption:** the app reports the goal active again (updated around
12:01), starting a fresh blocked audit. No new app-closing/publication approval
arrived. Minecraft/Javaw and QEMU are now absent; the old Minecraft closure request
is obsolete. The task's expired Brave sign-in tab is also absent. Browser inventory
shows the remaining tabs belong to the owner's other work plus the existing org
settings tab; none were closed. No task-owned native app/guest remains running.
Host free RAM measured 3,542 MiB at 12:01 and 3,540 MiB at 12:05, below even the
unchanged 3.5 GiB configured launch floor and below the proposed 5 GiB retry gate.
Unused commit is ample but does not establish the needed physical headroom.
Current selected work, MSI identities and uncommitted files are unchanged.
This resumption yields no completed product/guest work. Next human action: free
roughly 2 GiB from unused applications, or identify a specific unused app/window
the assistant may close normally, then continue. Do not close other active work,
lower the resource guard, rerun CI/builds or publish to manufacture progress.

**09:19 UTC blocked audit:** the previous continuation revalidated the blocker
without completing additional product/guest work. Current free host RAM is
5,014 MiB, still below the proposed 5 GiB resume threshold; Minecraft remains
open and no QEMU process exists. Permission to close the client is unanswered.
The same resource/permission blocker has now persisted across the resumed turn
and two automatic continuations. No independent authorized implementation remains
outside the guest/publication gates. Mark the goal blocked, not complete; resume
after actual headroom improves or the owner authorizes normal Minecraft closure.
Do not re-run CI/builds, change host settings, publish, or repeat failed guest
attempts solely to keep the goal active. All work and guest disk remain retained.

**09:18 UTC revalidation:** launcher 42225 is terminal and no QEMU process remains.
Minecraft's window is no longer labelled multiplayer, but five bounded host-memory
samples (5,366 / 4,793 / 4,481 / 5,143 / 5,122 MiB) did not establish three
consecutive samples above the proposed 5 GiB resume threshold. Probe 55626 exited
0 with `ready:false`; no guest boot or active resource-policy change occurred.
Evidence: `target/astra-clean-vm-20260907/resource-readiness-probe-0916.json`.
The independent selected-scope gap check found no remaining local implementation
outside guest acceptance and publication. Permission to close Minecraft remains
unanswered. The previous turn made installation/evidence progress; this continuation
revalidated the same memory/permission blocker and leaves the goal active.

**Latest at 09:10 UTC:** final-installed inventory was exported and hash-verified:
`guest-evidence/c41a57d8-0a29-48f0-a946-11d1bdfc36a7-inventory-final-installed.json`
under `target/astra-clean-vm-20260907/`. Installed EXE matches final payload
`d75be6b767164e681b278c717551b84f62baf6987467751f2e98a39538c72db5`, version 1.2.2.
All six checked VC++ runtime DLLs remain absent. The final MSI installation
returned exit 0; after removal of the preceding same-version candidate, the local
Pytxo data directory remained present. Its contents were not compared. This is not
a previous-version upgrade test.

The installed Start-menu shortcut was clicked at 09:04:27 UTC. At 09:04:37 an
application window was visible but still white; no rendered onboarding is claimed.
Host free memory measured 919 then 955 MiB, below the revised 1,024 MiB floor.
At 09:04:40 the guard requested ACPI shutdown. Windows did **not** complete it
within 15 seconds: QMP records `SHUTDOWN` with `guest:false` and
`reason:host-qmp-quit`. QEMU exited 0 at 09:04:56; launcher 42225 exited 1 for
resource abort. No forced process kill was observed. This is **not clean Windows
shutdown evidence**. The guest disk remains preserved and stopped.

Immutable continuation record binds twelve evidence files:
`guest-evidence/c41a57d8-0a29-48f0-a946-11d1bdfc36a7-final-install-continuation-v2.json`.
Independent review verified all twelve file hashes and narrowed the directory
existence claim; the original continuation record is retained as superseded.
The MSI log/result and packet-copy record still need allowlisted export from the
guest. No fixture helper, OAuth callback or guest mission ran in this session.
The one-attempt policy forbids another resource-abort retry until host demand
changes. Root requested permission to close the visible Minecraft client normally
(about 1.2 GiB); **unanswered**, so do not close it. No host app/settings changed.
After actual headroom improvement, resume with fresh boot, recover these files,
normal native launch/optional CLI, prerequisite/fixture checks, then reassess
resources before ordinary guest OAuth and a serialized real-harness mission.
The reviewed OAuth privacy latch requires clean shutdown and a fresh manager
before screen capture/login-status verification. It was never armed here.

No further Actions, source build or publication was run. All twelve source CI
jobs remain green. Publication and public-download acceptance remain gated.

Read-only runtime review confirmed supported serialization: first run the
unchanged V4 Initialize-Fixture helper (six original hashes, eight fixture passes,
21 independent passes/five expected failures); then change only the guest fixture's
`max_agents = 2` to `1`, commit locally and record a separate six-file baseline.
Keep packet files/manifests immutable. Build a **fresh** Desktop plan, inspect
three tasks/three single-worker stages, retain folder-scoped Orbit, and apply the
supplied documentation prompt. Ensure `pytxo.toml` is unchanged from this serial
baseline before and after Apply. This changes scheduling/fixture identity, not
the MSI. It does not prove 2 GiB capacity. No current worker-count UI exists.
Deterministic model-free recovery cases need the optional CLI/native dispatch API;
normal Desktop Flow cannot start arbitrary commands. Do not add a fake vendor
shim or call those guest cases passed from source/host tests.

### Earlier in the installed guest session

**At 08:57 UTC:** guest `c41a57d8-0a29-48f0-a946-11d1bdfc36a7` remained
running under launcher 42225, manager 32860, QEMU 39824. Slower local-password
entry succeeded and the desktop was inspected. No input typing process remains
active. Final-before inventory from the earlier session was recovered and
hash-verified (`06d18374…`), confirming old EXE `44106e…` and all six checked VC++
runtime DLLs absent. All sixteen final packet files copied and matched at 08:44:17.
Normal Programs and Features removal was approved in the guest; exported
`inventory-final-removed.json` confirms the EXE and program registration absent,
Pytxo local data directory present (contents not compared), and the six runtime DLLs still absent.

The final MSI `cf8db2e…` installed through the ordinary wizard. Automatic launch
was unchecked; Finish returned installer exit **0** at 08:55:02 UTC. Guest log:
`C:\PytxoValidation\evidence\install-20260908-085240.log`, SHA256
`4552a3b832a59bc765e576611c4b2ff341c092694335658a59c1e8d43050f273`.
Its `.result.json` also exists in the guest. `Inventory final-installed` ran and
rendered JSON; export that inventory, installer log/result and packet-copy record
next, using the allowlisted helpers. Then verify installed EXE hash `d75be6…`,
normal Start-menu launch, optional CLI path, prerequisite capture and fixture
checks. Guest native launch, authentication and mission are not yet verified.
The latest inspected screenshot is `screens/bounded-installed-inventory-result-0857.png`.
Get a fresh prompt before typing. The one-attempt resource constraints below apply.

### Earlier September 8 resumption and policy history

User resumed at 07:40 UTC, asking for alternatives and another attempt. The
goal is active again; prior blocked audits below are historical. Memory rose to
4,985–5,026 MiB across three samples, and the post-ISO-hash check passed without
changing the resource policy or closing user apps. Launcher session 63411 started
online guest `d6cf4ed6-9b78-4187-b18f-ecc28441d49a` at 07:48:22 UTC, QEMU PID 6476,
manager 18888. Existing disk, final packet and V4 helper media are mounted.
Local guest sign-in succeeded and the desktop/PowerShell prompt were inspected.
The V4 `Inventory -Stage final-before` command was entered and inspected before
execution, but its completion was not observed. At 07:59:47 UTC two low-memory
samples (last 420 MiB) triggered the owned guest's shutdown sequence. QEMU exited
0 at 08:00:02 UTC; the launcher exited 1 to report resource abort. No forced kill
was observed, and the disk and all user apps remain preserved. No final MSI
installation or new guest mission is claimed. Free host memory afterward was
about 2.6 GiB, below the unchanged 4 GiB launch gate.
Continue by checking/exporting any completed final-before inventory, final packet copy, normal removal/install
of the preceding same-version candidate, installed hash/Start launch, and local
fixture checks. Final MSI identity remains `cf8db2e…`, payload `d75be6…`.

The bounded auth review found ordinary browser OAuth does not require enabling
device-code login. Exact Codex 0.153.4 source validates state and redeems with the
guest's retained PKCE verifier. Manual delivery of its one-time callback is a
source-informed alternative, not documented/tested support; official docs name
SSH forwarding. Existing text-command files must not store that sensitive URL.
Runtime specialist completed the broker and opt-in VM-manager integration under
ignored `target/astra-clean-vm-20260907/`; ownership returned to root. Independent
review found and closed missing denial handling and capture release based only on
keyboard acknowledgments. The final privacy latch now lasts for the entire
manager lifetime; cleanup reports `cleaned_but_quarantined` and requires a normal
guest shutdown/fresh boot before screenshots or independent Codex status checks.
Use a freshly observed InPrivate guest browser for callback entry. There is no
acknowledgment-only reset. Root reran 29 synthetic tests, 11 resource assertions,
seven lifecycle checks and both syntax checks, all passing. Records and frozen
helper hashes: `target/astra-clean-vm-20260907/auth-transport-verification-20260908/`.
The callback must avoid tool logs, command files and screenshots; ordinary browser
history is a separate persistence surface, so do not claim no persistence anywhere.
No real callback listener, account setting or credential-cache transfer occurred.
Brave's old auth-tab selection failed; fresh task tab `160932152` proved browser
access and was closed after the check. Prior guest inventory confirms full Edge
152 is already installed; no browser download is needed. Root reset only this
task's unused Node kernel; the 08:26:33 resource check found 3,730 MiB free and
23,201 MiB unused commit, still below the retained 4,096 MiB launch threshold.
Independent review supported one bounded installation/local-check attempt with a
3,584 MiB launch floor and a stricter 1,024 MiB runtime abort floor. Replaying the
previous session would trigger about 16 seconds earlier under that runtime guard.
The task-local policy was revised, reviewed, and passed eleven boundary assertions
plus seven lifecycle checks; all old policy files/results are preserved under
`target/astra-clean-vm-20260907/resource-policy-revision-20260908/`. Commit floors,
two-sample pressure rule, post-hash recheck and shutdown deadlines are unchanged.
One-attempt launcher session 42225 passed three samples (4,026, 3,984, 3,941 MiB)
spanning at least 15 seconds and the post-hash recheck. Guest session
`c41a57d8-0a29-48f0-a946-11d1bdfc36a7` started at 08:33:55 UTC, manager 32860,
QEMU 39824. It reached the Windows lock/sign-in screen. The first 39-character
saved local-password entry was rejected; no password value was displayed.
Explicitly focused slower re-entry is in progress in exec session 87302.
**Wait for that input process to finish before sending any other guest keys.**
Inspect `vm-active.json` and current screenshot before continuing. Latest sampled
free memory was 1,798 MiB while running, above the revised 1,024 MiB abort floor.
If pressure aborts this attempt, stop retries until host demand changes. Limit
the attempt to final-MSI installation and local fixture checks; separately assess
settled memory before authentication or workers. No user app or host setting changed.

**Goal blocked, September 8 at 03:38 UTC.** Third consecutive post-CI blocked
audit: 1,884 MiB available RAM against the 4 GiB guest launch gate, no live owned
guest/waiter/CI-watch process, and no response to the temporary device-login
enable/sign-in/restore approval. The previous turn was no progress, not a live
wait. Source remains `eb5f73d`; all twelve CI jobs passed. No further required
work can proceed without resources or owner approval. Goal scope is unchanged;
it is not complete. Resume when memory is sufficient and the owner answers the
pending account-setting request. Publication still needs its separate approval.

Consolidated commit `eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78` was pushed to
PR31's existing branch at 02:57:15 UTC. All twelve jobs in CI run `34181828835`
passed; the final Windows job completed at 03:29:16 UTC:
https://github.com/Pytxo-dev/pytxo/actions/runs/34181828835 . Watch session 15986
completed 0. No retry, tag, merge, release or deployment was started. The selective
44-file commit excludes all six pre-existing untracked path groups. Post-CI
checkpoint/ledger updates remain local to avoid another Actions run for status.

Actual logs under `target/astra-final-native/ci-*-final.log` confirm 130 release
Desktop browser tests plus one development test, 41 Storybook checks, and 23
website browser tests passed. Workspace test result totals: Windows 469,
Linux 452, macOS 452; zero failed/ignored in those steps. Cross-platform Clippy,
audit and applicable feature checks passed. Cargo audit retained 19 allowed
warnings. `ci-final-run.json` binds every job to the exact source commit.
The post-run billing API reports all usage discounted and zero net amount:
Linux 616 minutes, Windows 419, macOS 135, Linux ARM 4 cumulative this month.
These are SKU quantities, not an invented remaining-included-minutes value.

Guest memory-wait session 12774 was identity-checked and stopped before any
guest boot at 03:32:48 UTC. Its final free-memory sample was 1,958 MiB, below
the retained 4 GiB launch gate. `target/astra-final-native/guest-wait-stopped.json`
records the cleanup. The initial cleanup guard refused a local/UTC comparison;
the actual UTC start identity was checked before the corrected stop. No user app,
guest disk or Windows setting changed. No watcher or test server remains active.

Remaining: sufficient host memory, the unanswered temporary Codex device-login
enable/sign-in/restore approval, final guest acceptance, then explicit staged
publication approval and public-download validation. This is the first post-CI
blocked checkpoint after substantial new implementation/evidence; the goal is
active, not complete. Do not repeat heavy checks, rebuild unchanged bytes, run
duplicate CI or toggle account settings while waiting for the human response.

Post-CI blocked audit 2, September 8 at 03:37 UTC: the previous goal turn made
substantial progress (final source push, twelve-job CI success, evidence and
cleanup). This continuation revalidated 1,823 MiB available host RAM against the
4 GiB guest gate, and found no live Pytxo/QEMU/waiter/CI-watch process. There is
still no human response authorizing temporary device-login enablement. No next
required action is available within the current resources and approval boundary;
this is a blocked/no-progress continuation, not a verified wait. The goal remains
active until the three-turn blocked threshold is met or the condition changes.

Read-only alert triage after GitHub's default-branch warning found all six critical
and 62 high alerts patched or removed in the five active candidate lockfiles.
Their hashes match the September 6 dependency audit record. The remaining 23 high
alerts concern legacy `apps/docs`, which current website/Desktop release paths do
not build. This is an existing-alert comparison, not a new whole-dependency audit;
no security setting, alert dismissal, package change or production claim occurred.

The following local build/evidence checks preceded that push.

Memory wait session 27039 was cancelled before guest launch at 02:16 UTC after
available RAM remained below 4 GiB; no host app was closed. Root launched the
exact extracted final MSI executable in a fresh isolated host Pytxo profile:
`target/astra-final-native/launch.json`, PID 38072 started 09:18:59.884+07.
Playwright CDP 9327 exercised the actual native WebView; this is host evidence.
Fresh fixture: `%TEMP%/PytxoAstraFinal20260908/fixture`, baseline `e9e6eb9…`.
Initial fixture 8 pass and independent 21 pass/5 expected fail are verified. The first
independent adapter failed from a Windows import URL; only the URL was corrected,
and both logs are retained. Original fixture assertions remain unchanged.

Run `267ba929-200f-48af-b5bc-6735e5810ae4` began 02:24:46 UTC and passed all three
combined checks at 02:27:47: Codex 0.153.4/Astra/high, three tasks, two waves, at
most two workers, Orbit/projfs-sparse-copy-v2. Primary baseline stayed unchanged
before Apply and after Cancel. Native UI first refused untrusted dispatch;
Orbit was then selected through Setup → Workspaces → Edit folders & permissions.
Saved draft restored workspace/Codex and cleared its old plan before rebuild.
The documentation task was explicitly clarified before Run. Every exact diff
was reviewed. Apply `43939199-b4d3-4a3b-bf67-d58b7201e194` committed package
`349ff5f7a504516497aa60f04887bf969dd4215d92f84701d54f8ca40f5bc93c` at 02:29:55.
All 3 changed hashes matched, other 3 unchanged; actual post-checks 11+26 passed.
Receipt survived native process restart. Independent evidence review found no
material issues and rechecked all 354 source inputs. Native test processes are
stopped; fixture/profile preserved. Six unmodified captures and sanitized result:
`tooling/benchmarks/results/astra-final-native-2026-09-08.json`.

The film source and website's current-candidate record now identify that run.
Old film/poster/sheets/source were hash-archived before replacement. The new
silent master passed format/black-frame checks: 52 seconds, 1080p/30fps,
H.264/yuv420p/BT.709, SHA256
`9c427fc2ae0ab8acd6303b5b0379c432b5dc4b136dcec1a968c47ca59a0015e4`.
Seven scene stills and 13 encoded transition frames were visually inspected;
full continuous playback was not watched. Typecheck, asset hashes, transcript
and provisional cues passed. No paid narration was generated. Record:
`tooling/benchmarks/results/astra-final-demo-2026-09-08.json`.

Website production build, lint and 163 internal links across 108 source files
passed. Three affected production browser tests passed (17.7 seconds), including
current-candidate identity at 1440/390. Both full-page screenshots were inspected.
Independent final film/web evidence review found no material issues. Logs and
captures: `target/astra-final-native/web-*`. Test server is stopped.
Consolidated CI, final guest and publication remain pending. No new Actions run,
account change or publication occurred. At 02:55 UTC, host free RAM was 2,321 MiB;
the guest's 4 GiB launch gate is retained. Current Actions API shows $0 net usage,
with minute quantities unchanged since the preceding read.

New MSI `cf8db2e0b41ceaaf678d3c74226b62dc62589f2a68880ec8ba49d42088dbd911`
built successfully at 02:04 UTC. Its actual packaged EXE is
`d75be6b767164e681b278c717551b84f62baf6987467751f2e98a39538c72db5`.
All 354 frozen inputs still match digest
`c721a49414e348dc93d7801c9d36fefdd445a75d44dd3ecaa6487824265e38f0`.
Both artifacts remain Authenticode unsigned. The actual MSI import guard passes.
Focused native tests (4), browser journeys (4), Desktop check and distribution
all-target Clippy passed. This includes failed-example retry and Skip at 1440/390.
Evidence: `target/astra-onboarding-prereqs-20260908/`.

The 16-file `final-packet/`, ZIP and packet-final ISO are frozen and roundtrip
verified; guest-tools-v4 also passed byte checks and bounded review. The final
ZIP hash is `47517631b8602ad671ec3cade5ff3530de2615648f7a0b2d1b008c79627bac46`.
VM launcher session 27039 was cancelled before launch, as recorded above.
No host app or setting was changed. On resume,
continue the existing disk with the final packet and V4 tools; record actual
installer behavior for these bytes. Prior 1f47 installation/launch is historical.

Latest: guest `80717a46…` stopped at 01:43:45 UTC. ACPI shutdown was requested
at 01:43:03; the resource guard also recorded low memory (475 MiB) at 01:43:42
before QEMU exited 0 / manager1. Its disk is preserved. Post-prerequisite inventory
was exported: SHA256 `4d94bf568b0cf40dd4f8652cca09b53c9c77c041cef5194aeded882350753c15`.
Device-code login is disabled on the existing ChatGPT account; the assistant
asked asynchronously for temporary enable/login/restore approval and cancelled
the unused guest login attempt. **No approval response yet; do not toggle it.**
Brave tab160932148 (`guestSignIn` in CUA) retains the official consent screen,
marked for handoff. No authentication succeeded or host credentials were copied.

The guest stayed stopped during the completed build and test commands.

The guest also reproduced a guided-example onboarding defect: without Git,
Try the guided example displayed `[io] program not found`. Root has corrected
`ipc_install.rs` to probe Git before file creation and retain executable context
in errors, plus accurate Git/Node helper text. A native missing-executable/no-files
regression and two-viewport UI retry/skip regressions passed.
Independent read-only review found no material issues; its Skip coverage note
was addressed. The new MSI above contains this correction; preserve `1f47…`
installation evidence as a separate observation.

Git and Node installers completed in the guest with observed exit 0. A fresh
non-admin PowerShell verifies Git 2.55.0.windows.5, Node24.20.0 and npm11.19.0
at 01:29:43 UTC (`crt-prerequisite-versions-result.png`). Codex0.153.4 is installed
to match the earlier rehearsal. Vendor authentication and
the complete mission remain pending; no host credentials have been copied.

Continuation `80717a46-e16c-4e58-afda-b395069f0cc7` started at 01:09:52 UTC
(launcher 90518). The installed replacement launched through Start and rendered
the actual welcome screen at 01:13:13; screenshot `crt-resume-app-20s.png`.
Maximized welcome and optional-CLI screen were inspected; the latter correctly
offers Continue with Desktop without a separate Pytxo CLI. This verifies the
missing-runtime repair in the guest. Full mission/recovery acceptance is pending.

The failed candidate was removed through Programs and Features. Guest inventory
confirms its EXE/registry entry absent and all six checked VC++ runtime DLLs
absent. The new MSI installed with exit 0 at September 8, 00:56:41 UTC. Start-menu
launch opened a native white window at 00:57:35; rendered UI is not yet verified.
At 01:06:00 the host-resource guard stopped only the disposable guest after two
low-memory samples (441 MiB available in the final sample); QEMU exited 0.
The existing disk and installer logs remain intact. The successful continuation
above supersedes this interrupted first observation; no reinstall was needed.

The final Windows distribution policy is opt-in: `.cargo/windows-msvc.toml`,
used by `npm run build:msi` and the Release workflow with an explicit target.
The first global policy caused Windows to deny a host build helper; removing
its automatic loading restored the ordinary Cargo core check (passed, 9.34 s).
No host security exception or setting changed. The Desktop build script disables
Tauri's linker shim only when the actual target already uses the static CRT.

Final replacement MSI: `1f47f34d673b552ae8ddebab222ad49989ef8d94f6032b94027e970ab980219f`;
packaged EXE: `44106e772f38ef4a1ba8123ec97128c091691b4592f6e0e6eb5bfc942d80d430`.
Both remain Authenticode unsigned. The 354-input digest is
`3e57d96e802b903bfab4ae73a6ce822c15b52159b1602b38351f3a733f6039ae`;
all inputs still match after testing. Actual MSI extraction/import check passes,
all four native libraries use static C/C++ runtimes, 41 release-tooling tests
and 12 voice contract tests pass, and formatting/version/diff checks pass.
Independent review closed its extensionless-DLL finding. Logs and identities:
`target/astra-crt-native/scoped-policy/`. These checks are not speech-transcription
or clean-guest mission acceptance. No additional Actions run has occurred.

The replacement packet's 16 files and helper media's 8 files passed full ISO
roundtrip checks. Git/Node installers match official checksums and have valid
Authenticode signatures; they are staged, not installed in the guest. Online
guest `f31c0cb9-197f-4aa2-82fb-12a5c9ab77f3` started September 8 at 00:40:36 UTC,
launcher 85048, after the normal memory/hash checks. That session recorded
pre-replacement and post-removal inventories, removed the failed candidate and
installed the replacement as described above. WebView2 remains from the first
installation; no VC++ redistributable was manually added.
CI is green only at `f64a0b0`; merge/publication approvals remain outstanding.

The following is preceding failed-candidate evidence and repair history.

The clean guest installed the frozen MSI `614d44e2d47e…` successfully (exit 0
at September 7, 23:49 UTC), then Start-menu launch failed with Windows reporting
missing `MSVCP140.dll`. This is a reproduced packaging defect, not a clean-install
pass. `target/astra-clean-vm-20260907/screens/guest-pytxo-first-start.png` captures
the actual loader error. Before/after guest inventories verify no developer
prerequisites and show WebView2 afterward. No VC++ redistributable was manually
added; the inventory's application filter did not explicitly enumerate it.
The guest shut down normally at 23:57:46 UTC, QEMU exit 0, preserving its disk.

The frozen executable imports 95 symbols from `MSVCP140.dll`. Current Whisper
and GGML C++ objects use `/MD`; their exact contribution to that older executable
is not independently attested. A consistent Windows x64 static CRT build and
actual executable dependency check are being implemented. No VC++ runtime will
be added to the guest to conceal this failure. Replacement installer, native
acceptance and final-build provenance must be verified before claiming closure.
The earlier MSI, film and native evidence retain their original identities.

The paragraphs below retain the preceding provisioning observations as history;
the successful installation and failed first launch above supersede their pending
installation status. No Actions run or publication was triggered by this test.

All twelve jobs in CI run `34112393479` passed on
`f64a0b092b02af917fea52180a4e76458e2f1934`, completed September 7 at 11:13:50 UTC.
The Windows workspace, transport lifecycle, audit and feature checks passed.
Final run/jobs records and Windows log are retained under
`target/astra-ci-34112393479-*`; `target/astra-ci-handoff-f64a0b0.md` records
the source/artifact relationship. Keep the earlier failures below as history.

The user's subsequent “do it for me” authorizes assistant-owned disposable
Windows VM setup and validation. Portable QEMU was checksum-verified and
extracted; WHPX executed firmware and accepted screenshots/input over private
QMP stdio without changing host features. The complete Windows 10 LTSC evaluation
ISO passed Microsoft's full SHA256 on September 7 at 15:20:26 UTC. The one-shot
launcher requires an untouched guest disk and stable available RAM/commit
headroom, with monitoring during setup; no host paging settings or apps change. The exact
16-file installer packet passed a read-only ISO roundtrip. See
`target/astra-clean-vm-20260907/PROVISIONING.md` and live process observations.
Windows setup began at 22:39:41 UTC after the memory checks passed; its captured
screen shows installation in progress. The installed Desktop workflow remains
unexecuted, and no clean-install pass is claimed.
The VM subsequently paused on WHPX errors after guest resets. Both occurrences
and preserved disk copies are recorded in the provisioning log. A documented
exit-on-reboot continuation reached the actual Windows desktop at 23:06 UTC
(`screens/windows-oobe-2306.png`). Before inventory or Pytxo installation, host
available RAM fell to 60 MiB; the monitored guard stopped only the owned guest.
Continuation now requires 4 GiB available host RAM, retaining the 2 GiB guest
and live pressure stop. The subsequent offline inventory verifies build
19044.1288, no Pytxo/WebView2/developer prerequisites, enabled Defender and the
exact frozen MSI hash. It then shut down normally. Online activation failed
once with `0x87E10BC6`; `/xpr` reports notification mode. Normal Windows Update
downloaded part of the offered August 2026 cumulative/.NET and Defender updates.
The guest then shut down normally to replace unreliable timed QMP keyboard input
with reviewed explicit key-down/up events. The guarded continuation is waiting
for 4 GiB available host RAM; its input probe and new short copy/install scripts
still require guest execution. Servicing and installed-app acceptance remain
incomplete. Activation is a recorded environment limitation, not a new blanket
Pytxo prerequisite. Live continuation details are in the ignored provisioning
record. No host apps or settings were changed.

After the green CI run, GitHub billing showed 2,520 of 3,000 included minutes
used, 480 remaining, and $0 billable Actions usage; that is a point-in-time
observation, not a reserved allowance. No additional Actions run, paid budget
change or publication was made for VM provisioning. Preserve the private brief
and unrelated untracked files. Merge, release, npm and site publication retain
their explicit approval gates.

## Preceding Windows test fixture correction

CI run `34107810014` on `402c5991cb064341bf240d2d28e3c46c424b9e13`
finished with eleven passing jobs. Linux/macOS Rust, website, Desktop, native
Linux, npm and benchmark checks passed. Windows alone failed: two transport
fixtures exceeded their ten-second whole-lifecycle deadline. The log does not
identify a startup delay versus a hang. All five transport tests passed unchanged
locally; do not call the hosted failure a proven harmless flake.

The follow-up changes only the Windows fixture inside `flow.rs`'s existing
`#[cfg(test)]` module: 60-second startup/non-Stop allowance, elapsed event and
observed-file diagnostics, and bounded async waits for owned-run cleanup on
failure. Post-Stop settlement remains ten seconds, below the probe's natural
30-second exit. Every timeout still fails; assertions and production code are
unchanged. Cleanup waits are not a hard process-level watchdog. All 95 affected
orchestration tests and affected Clippy pass; independent review has no findings.
Logs: `target/astra-ci-windows-{fixture-green,orchestrate-green,orchestrate-clippy}.log`.

Retain the MSI, native observation, film and Windows packet below with their
original source identities. All 351 other frozen native inputs still match;
the entire non-test `flow.rs` prefix matches after explicit newline normalization.
The full 352-file snapshot differs only in test source, so these artifacts were
not rebuilt or relabeled as builds of the later correction commit. Separate
verification: `target/astra-ci-test-only-provenance.json` and independent review
`target/astra-ci-test-only-provenance-review.md`. Hosted CI must still identify
the actual new commit. Clean Windows and public-download gates remain open.

GitHub billing refreshed through Brave after the second run: 2,291.3/3,000
included minutes used (708.7 remaining), $0 billable Actions usage. Keep the
existing paid-overage stop and run one consolidated correction, without blind
retries or unrelated rebuilds.

## Preceding production correction and retained native build

The reviewed 116-file candidate was committed as
`2964d5ad2ad677dec014d32ae227635de8dfea5b` and pushed once to draft PR31.
CI run `34100785788` executed after the Team upgrade: eight jobs passed, four
failed. Both Unix Rust jobs exposed the same Stop registry-lock/reaping cycle;
Windows Rust 1.98.1 rejected a test helper's constant `chunks_exact`; npm's
version validator still expected the old public-binary wording. Completed logs
are retained under `target/astra-ci-34100785788-*`.

The correction persists cancellation and releases the registry before waiting
for process termination, then removes only captured identities. Both Stop paths
retain PID checks, failed-termination evidence and unrelated new registrations.
A controlled lock regression failed before repair. All 468 Windows workspace
tests now pass with zero ignored; workspace Clippy and formatting pass.
The actual repository version check and 21 release tooling tests pass. Independent
runtime and workflow reviews report no findings. The cheap npm metadata checks
now precede Rust compilation without removing any check.

The runtime repair supersedes the earlier native source freeze for the new
candidate. Keep the preceding MSI, native observation, film and Windows packet
as historical evidence; do not relabel them as tests of changed bytes. The new
352-input freeze is `d54be69068c39cf8b14ce58bbad84a77a274ae878a7874d796c9da2644db8e4a`;
its rebuilt MSI is `614d44e2d47eeeea2e62c0689b3146183daea30816f4d4341be0333d7b8948b3`.
Run `53728815-a282-44f8-8be7-05f224e91223` completed three tasks and three combined
checks. All exact diffs were inspected in the native MSI WebView, Apply was
explicitly confirmed, all three changed hashes matched and the other three
source files were preserved. Eleven fixture tests and 26 separately authored
acceptance tests pass; five acceptance tests failed on the baseline. The committed
receipt survived a native process restart. Independent review reports no findings.
The new allowlisted record is `tooling/benchmarks/results/astra-ci-native-2026-09-07.json`.
Workers reported Astra/xhigh; do not relabel this as the earlier Astra/ultra take.
The same payload/profile used Computer Use for preparation and execution, then
WebView inputs for review after inconsistent native-helper state. Clean install
remains unexecuted. The refreshed 52-second silent film hashes to
`761b377c71a15eed5a59ec29592f36919d711736e08122cfc6a24c11a044a2ee`;
seven stills and 13 encoded transition frames were inspected. Current packet:
`target/astra-ci-windows-validation-packet.zip`, 16 entries, SHA256
`42845a33b2e7044787843b7a63c51cf77b1e5132c1a38f95311ff3d292e10503`.
Every archive hash and baseline file was checked. Both preceding artifacts are
preserved. One consolidated correction push follows final source/site checks.
The site rebuild/lint and 162-link check pass, as do all three affected browser
tests. Current evidence and its JSON were inspected at 1440/390px with matching
hashes/counts and no root overflow or page errors.
Consult the latest generated CI record and
PR31 checks for hosted acceptance; local tests do not establish it.

## Team upgrade and authorization

Matt upgraded Pytxo-dev to GitHub Team and explicitly instructed continuation,
organization editing through Brave, and careful Actions use. Brave verifies
2,000 of 3,000 included minutes used at 08:25 UTC, before this run, $0 billable Actions usage, and the existing
$0 Actions budget with Stop usage Yes. The prior proposed $10 overage allowance
was not applied. The org's blank description and URL were updated to the current
agent-hypervisor description and `https://pytxo.com`, with a saved confirmation.
The public `pytxo-releases` repository was pinned to the organization's public profile.

The first consolidated commit/push completed; the necessary correction push is
within the same authorization and included allowance. All 116 proposed file hashes matched before this
resumption's CI/docs edits. Preserve the private brief, unrelated untracked work
and unused demo Flow copy. The only new workflow behavior is 60-minute limits
for Rust/native jobs and 30-minute limits for the remaining CI jobs; PR-only
supersession cancellation was already prepared. Keep every existing check.
No active run was present, and existing caches already total about 10 GB; do not
create redundant runs or extra large caches merely to test availability.

The preceding MSI/demo/Windows packet remain historical proof described below.
The current native freeze and observation are identified above. CI must identify
the actual correction commit. A PR push
does not trigger the main-only website deployment; merging does and therefore
still needs separate approval. Releases, paid overages, security changes and a
new VM/clean-machine installation scope also remain separate boundaries. Matt
permits Computer use for later Pytxo testing; a development-host run does not
establish clean installation.

## Historical local freeze before the Team upgrade

The following blocked-status and no-write statements describe the earlier
freeze. The resumption above supersedes their next-action and authorization state.

The ASTRA goal has completed its selected local implementation, research,
native proof, website, film and portable acceptance packet. CI, clean Windows
installation and public download remain HUMAN-BLOCKED; do not mark the goal
complete or describe version 1.2.2 as publicly verified. No commit, push, merge,
publication, budget/account/security change, new dependency or paid narration
request occurred. HEAD remains `ff0b88fb71224d579267e697261ae4d831f0cd76` on
`codex/beta-candidate-verification`, with the ASTRA changes uncommitted.

The final impasse audit rechecked all 116 candidate file hashes, unchanged PR31
CI run 34038203378 (completed failure before execution), public Latest 1.2.1,
and the unexecuted clean-Windows result form. Independent review found no further
required safe local action. The same approval/environment condition persisted
across three consecutive goal turns; the goal is blocked pending owner input.
Automatic continuations are not approval. Current audit:
`target/astra-blocked-audit-final.json`. Revalidate candidate hashes and explicit
approval scope before resuming gated work.

Final native source: 352 inputs, digest
`9b9079cb25fc5dc01e749c14ffaccfc67895cbce60f6bbdef12dff671de39858`.
MSI `dfacd548f58ea149d84271de50c038288f6b19894f1496dde65916a4a67d20d3`;
payload `9d99646c9ac59c4504a5f4bd7133b1bd84d3327a59a25ccb7dd75926abd9e603`.
Both are Authenticode unsigned. `target/astra-native/msi-payload-run.json` now
correctly identifies this MSI and its original launch; it is no longer stale.

Actual run `d743c0dc-76d3-4ff8-b6a3-7d88e743eb22`: three Codex tasks, two waves,
three passing combined checks, all exact diffs inspected through native UI,
explicit Apply committed, three resulting hashes match, other source unchanged,
eight independent tests pass. The committed receipt survived a process restart.
Final documentation task received an explicit final-state prompt override via
the existing editor; exact guidance and observed worker input are recorded.
The previous ready package `55a36c03…` was retained without Apply because native
review found stale README prose. Pytxo did not automatically detect that error.
All prior failed attempts and the competent direct worktree baseline remain.

Verification: full Rust workspace 459 passed before the final receipt repair;
the full affected orchestration suite 95 passed afterward, with seven actual
regressions that reproduced old failures. Final workspace Clippy, CLI/MCP builds,
formatting and MSI build pass. Desktop 128 production browser checks pass;
website 23 checks, 161 links, lint and production build pass. The completion
audit repaired canonical/share identity, sitemap omission and a duplicate docs
heading; all 45 sitemap routes pass without JavaScript. Eight final-build local
performance observations carry explicit limits; no production speed claim.
Native source/MSI/film/packet identities were rechecked unchanged. Proposed
commit inventory: `target/astra-commit-inventory-final.json`, excluding the
private brief, preserved user files and unused generated Flow image. See logs and
limits in `docs/01-projects/astra-evidence-2026-09-07.md`.

Film: `apps/demo-video/out/pytxo-demo-silent.mp4`,52 seconds,1080p30, H.264,
BT.709, no audio. SHA256
`45aab4c6dee117d4792ac637fc181ad1445b65841359086e88d048751339317c`.
Six original native PNGs, seven scene stills and13 encoded transition frames were
inspected. Format/black-frame/hold checks pass. Provenance is
`tooling/benchmarks/results/astra-demo-2026-09-07.json`. Narration was not regenerated;
old audio and provisional caption timings cannot certify this edit.

Packet: `target/astra-windows-validation-packet.zip`,15 source-only/allowlisted
files, exact MSI inventory and all ZIP-entry hashes verified. SHA256
`2d259f66832a45049e89d0d52d6fd0522f0720a52ffd6fddd85cd38ce2c2d8ea`.
Its fixture baseline tests pass locally; the clean-machine result form remains
`not_executed`. It contains no vendor sessions, browser profiles or raw logs.

All owned native/model/build/test/preview processes are finished. Native PID31444
was restarted as31496 to verify History, then identity-checked and stopped.
No other project process was touched. Native draft reuse restores workspace/CLI
after detection settles; an early click requires reselection. That usability edge
and delayed per-worker terminal rows remain explicitly deferred.

At04:01 UTC, PR31 remained draft with the same rejected CI run34038203378;
public GitHub/npm remained1.2.1. Next: owner approval for a proposed $10 Actions
allowance with stopping retained, an approved clean Windows x64 machine and
installer/prerequisite/vendor-session scope, then the two-stage publication
sequence in `docs/01-projects/astra-release-proposal-2026-09-07.md`.
The earlier clean-machine question has no answer. Do not trigger CI spending or
publication without the required explicit approval. Preserve the private brief
and unrelated untracked paths listed below when preparing any future commit.

## Historical continuation notes

The following notes retain intermediate state; the current section above
supersedes their ownership, pending steps, counts and artifact identities.

Current continuation (03:23 UTC): the original receipt-cap repair is complete.
Five candidate and two Apply regressions reproduced the pre-existing collision;
all 95 orchestration tests pass. Independent review found no actionable issue.
Final workspace Clippy, CLI/MCP builds and formatting pass (`*-final.log`). The
earlier full workspace run passed459 before this last narrow receipt repair.

Root owns all source and compiler again. Final MSI build session50080 is active,
one Cargo job. It froze352 source inputs at03:17:36 UTC with digest
`9b9079cb25fc5dc01e749c14ffaccfc67895cbce60f6bbdef12dff671de39858`.
No native process/model is running. The prior unlaunched MSI78f9f8b3a108 is
preserved in `target/astra-native/pre-receipt-build/`; it is intermediate.
No schema expansion or permission/execution-domain widening occurred.

`msi-payload-run.json` still names OLD stoppedPID27480/MSI3e48087850e1: it does
not describe the new unlaunched MSI. Website/demo successful JSON imports remain
intentionally missing until actual native success. Packet generator requires
that success and matching MSI/source hashes. A text question about an available
clean Windows PC/VM is pending; no answer or approval has arrived. Refreshed
GitHub/npm remain1.2.1; PR31 is draft atff0b88f with the same rejected CI run.
Use `docs/01-projects/astra-evidence-2026-09-07.md` and the release proposal.

Prior continuation (02:16 UTC): the rebuilt MSI `3e48087850e1…` / payload
`6c65e5163e9f…` launched, and native History verified the visible preparation
refusal and correctly bound receipt after restart. Its new run
`98a3ac95-c3a7-47fa-9818-456fb88daa04` failed before model execution: the
PowerShell → npm CMD shim split quoted/multiline handoff arguments. Two workers
failed; the dependent task was blocked. All six primary baseline hashes match.
Build/records archived in `target/astra-native/pre-transport-build/`; owned native
PID27480 stopped. The prior scope-refusal and initial cwd failure remain separate.

Runtime specialist exclusively owns the narrow Windows Codex adapter transport
repair in `crates/pytxo-orchestrate/src/flow.rs`, runner handoff format and tests.
Root owns docs/site/demo and native integration. Exact stdin is the documented
Codex route; other adapters receive no new claim of arbitrary-prompt support.
Source freeze352 inputs `f328165c…638c8a` is superseded once those edits land.
Reproduce through the actual CMD→Node argv/stdin boundary, then integrate, refreeze,
rebuild and retry native; never fabricate the missing successful observation.

Last full checks before the transport repair: Rust453 passed, Clippy/format and
CLI/MCP builds passed; Desktop128/128 passed with zero Svelte/CSS errors/warnings.
Logs use `*-handoff*`. Direct worktree Codex completed:3 files,8 tests,252.172s,
primary preserved, no speed/cost claim. Read `docs/01-projects/astra-evidence-2026-09-07.md`.

Demo and website Evidence changes currently import the intentionally missing
`astra-native-codex-2026-09-07.json`; they must not be called build-ready until a
real accepted run supplies it. Recorder derives/validates plan/task/check IDs and
complete primary source inventory excluding only `.git`/`.pytxo`. Six native
captures, final film, public-safe records, web checks and portable acceptance
packet remain pending. Source fixture is in `tooling/benchmarks/fixtures/astra-first-mission/`.
Hosted CI, clean Windows and public download require the concrete owner actions
in the release proposal. No publication/spending/account change is authorized.

Older sections below retain historical checkpoints; this continuation supersedes
their pending counts and ownership. The goal remains active.

## Active goal and preserved baseline

Execute the private ASTRA brief; it is intentionally untracked and must not enter
commits, packages or public artifacts. Pytxo is an agent hypervisor. PR31 baseline
`ff0b88fb71224d579267e697261ae4d831f0cd76` on
`codex/beta-candidate-verification` remains intact. All ASTRA changes are local,
uncommitted at this checkpoint. No push, merge, release, deployment, billing,
security-setting, new dependency, or system-feature change has occurred.

Current decisions and evidence: [[astra-execution-2026-09-07]] and
[[astra-product-decisions-2026-09-07]] in `docs/01-projects`. The goal is active.
Preserve pre-existing `apps/web/captures`, `apps/web/scripts/.verify`,
`desktop-e2e.log`, `docs/superpowers`, and private `PYTXO_ASTRA_MASTER_GOAL.md`.

## ASTRA implementation and evidence

- Fresh product/competitive/economics, runtime, release and adversarial reports
  are under `target/astra-research/`. Native tools are strong substitutes; choose
  a recurring bug plus regression test, one existing harness, then second use.
  No exclusive capability, productivity win, demand or billable-token claim.
- Desktop retains Chroma Aperture and Work/History/Setup. Scoped templates select
  real-path placeholders; plans show complete editable prompts, paths and
  dependencies; unavailable CLIs collapse behind a disclosure; readiness can be
  rechecked. Reusing a draft restores its original workspace/CLI if available,
  otherwise requires a replacement, and always clears old planning authority.
- Onboarding makes the optional terminal CLI secondary to continuing with the
  already bundled Desktop core. Active onboarding was already accountless;
  an unused sign-in component is not evidence of a first-run account barrier.
- Planner no longer chooses external planning from ambient provider keys alone.
  Explicit local mode wins, and the network entry point requires opt-in.
  Both original selection failures were reproduced; all15 planner tests pass
  without external model requests (`target/astra-planner-*.log`).
- Context now reads the actual dependency-composed/retry workspace under the
  original repository policy. Fresh retry context omits deleted source, fidelity
  caps remain enforced, generated output ancestry is checked, and atomic output
  replacement preserves outside hardlink aliases. Unsafe project labels fail.
- Final review found and repaired Windows UNC normalization in the shared path
  helper, also preserving exact UTF-16 components and unsupported namespaces.
  Final focused count26: path5, core boundary2, runner boundary9, integration10.
  Logs `target/astra-research/runtime-unc-*.log`; no live SMB test or universal
  filesystem-race/sandbox guarantee. Runtime worker source is frozen.
- Website now leads with one existing CLI and reviewed outcome, qualifies preview
  imagery/source access, corrects optional CLI/setup/release/economics claims,
  and offers a minimal local support-report template instead of raw log upload.
- Release staging now validates actual SHA256 bytes and complete checksum rows,
  preserving the existing CLI5/manual-MSI1 contract. Genuine red2, then all16
  inventory/version tests pass (`target/astra-release-*.log`). Independent review
  found no implementation bug. PR-only CI cancellation preserves release/main.

## Current integrated verification

- Full workspace PASS after the UNC repair:
  `target/astra-workspace-tests-final.log`. The later native PTY launch repair
  has37 affected passing checks in `target/astra-research/runtime-shell-cwd-*.log`.
- Full clippy PASS after the native launch repair, warnings denied:
  `target/astra-clippy-native-fixes.log`.
- Desktop check: zero errors/warnings and CSS pass:
  `target/astra-desktop-check-final.log`.
- Desktop production browser suite:122/122 PASS
  `target/astra-desktop-production-e2e-2.log`. First invocation was stopped because
  npm/PowerShell swallowed the worker override; direct Playwright CLI used1worker.
  Earlier67 affected dev checks also passed. Relevant desktop/narrow screenshots
  in `apps/desktop/test-results` and before shots in `target/astra-ui/before`.
- Web lint PASS;156 internal links across107 files PASS; production build PASS.
  Web browser17/17 PASS `target/astra-web-e2e-final-2.log`; first run16/17 failed
  an old intentionally replaced headline expectation, then corrected.
  Final production build PASS with the later evidence caveat/changelog copy:
  `target/astra-web-build-final-2.log`. Desktop/mobile website screenshots were
  inspected; no page errors or horizontal overflow (`target/astra-ui/after`).
- Both initial native builds PASS (`target/astra-native-build.log`,
  `target/astra-msi-build.log`). MSI SHA256 `ccfdb862b9c06322665e1f1c368f3868b0fc2626241c87684b84260fc390451d`;
  exact extracted payload `68c1deb31669a49cb0c304f06028144c3a37f3252087a8fb64abda58ee1f56e8`.
  These bytes are preserved in `target/astra-native/first-build` and are now
  superseded by source fixes awaiting rebuild. Neither artifact is signed.
- Isolated native fixture created at `target/astra-native/fixture`, initial Git
  commit `c602e3e`, max_agents1/Orbit/worktree/PTY,2 baseline tests pass. No agent
  mission succeeded yet. Existing isolated Codex0.153.4 is `target/beta-tools`; no CLI
  download or global configuration changes. Use separate `PYTXO_HOME`, trust
  store and WebView2 user-data directory for upcoming native verification.

## Native findings and current continuation

The exact MSI payload launched with isolated Pytxo/WebView2 state on this
development host. Actual onboarding, vendor readiness, folder picker and a
three-task Codex plan were exercised. Run `9d04e018-a888-46ee-af60-449a902b9adc`
failed before model work: CMD rejected its verbatim local cwd, fell back to the
Windows directory, and Codex refused a non-repository cwd. No package or Apply
became available. Preserve `target/astra-native/first-run-events.json` and PNGs.

The runner now normalizes external shell cwd while retaining canonical policy
roots; real UNC/device cwd fails explicitly instead of falling back. Actual CMD
red reproduced the PTY defect, then affected checks passed. A final bounded
  review reproduced sibling-directory execution for dot/space-ending directory
  names. These now fail explicitly before normalization; real PTY regressions
  and affected controls pass24/24 (`runtime-shell-cwd-edge-*.log`). Long paths
  were not tested. Source is frozen for MSI rebuild.

Native agent detail also exposed short/full receipt ID mismatch and a false
shared-working-tree inference from optional project root IDs. Both are repaired.
The adversarial reviewer then reproduced cross-run receipt borrowing during a
delayed refresh; all review consumers now require the focused run identity.
The starting phase is labelled and stoppable consistently. All17 epistemic
browser tests pass (`target/astra-ui/receipt-switch-green.log`); final production
suite passed125/125 (`target/astra-desktop-production-e2e-native-fixes.log`).
Final check/CSS passed0 errors/warnings (`target/astra-desktop-check-combined.log`).

Next real fixture is outside source ancestry at
`%TEMP%/PytxoAstra20260907/fixture` (commit
`ac5e3fb09b429494d703f33e5525f2e3db8ea54b`, six baseline files, same source/test
bytes, two concurrent workers). Local fixture Git disables automatic newline
conversion so Git blobs match disk hashes; no global Git settings changed.
This avoids inheriting this monorepo's project instructions. First native process
was stopped after terminal failure. Rebuild MSI, extract into a fresh folder,
freeze provenance, then launch its payload at the existing registered generated
path `target/release/pytxo-desktop.exe` to preserve URL handlers. Reuse only the
isolated Pytxo profile and existing vendor session; no credential copying.

`target/astra-native/VERIFY-WINDOWS.md`, result template and packet-preparation
script are prepared, not executed clean-install evidence. Do not run the packet
script until its source fingerprint and MSI-native record describe the final
build. Native success, final demo/Bench, portable packet and external gates remain.

MSI rebuild is active in `target/astra-msi-final-build.log`, preceded by runner
clippy and workspace formatting. Native source freeze351 inputs:
`bcaf589b9d7d0f7fa18523d115475ac8d2099ab33abfee80c663bf4590eee3d0`
in `target/astra-native/final-build-inputs.json`; source is still uncommitted.

## External gates remain open

PR31 CI34038203378 failed before steps; organization Actions budget is $0 with
hard stop. Payment-method status remains unknown. Prepare exact source/artifacts
before requesting owner payment check and a finite allowance (research proposes
$10 hard stop; not authorized). No pointless reruns or reduced matrix.

Windows11 Home has no supported Sandbox or verified clean VM; a separate approved
clean Windows x64 environment is needed. Extraction/dev-host UI proof is not a
clean elevated install. Public latest/npm/deployed download remain1.2.1;1.2.2 is
absent. Publication and independent download/install checks require approval.

Unsigned releases marked Latest can break the existing latest.json URL; docs now
state this correctly. Private vulnerability reporting on the public distribution
repo is disabled; enabling and verifying a reachable private channel is an owner
security-setting decision. Do not invent a contact address or claim it is fixed.

The Sep6 silent52s demo and Bench records remain historical evidence. Recapture
changed states from the final native build and keep measurement scope explicit.
No audio/spending approval exists for fresh narration. Meaningful authorized
work remains, so this is not a blocked or complete goal.

---

# Historical checkpoint — Pytxo Beta candidate — 2026-09-06

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

## Final refinement candidate — September 20

Supersedes the initial refinement payload above. MSI SHA256:
61112E132C3090816A77BE39BAF9B3CBCC72CCA2020F8955D080637D156ABCE6.
Extracted EXE SHA256:
6FF3BCCFF65DBA94EC5B7567292B2A3641A30C3EC06AAF314EFA1091FC53DA99.
Build session33695 exit0; all479 input hashes unchanged; exact inventory and
extracted runtime dependency checks passed. Source: refinement-final-source.json.
Svelte/CSS checks passed. Combined affected browser run:49 passed, one incorrect
Setup heading locator; after correcting that test-only locator, all3 background
navigation tests passed. Independent review found no further actionable defects.

Setup lightweight refresh previously erased worker rows until the next full
refresh. Work entry now requests a full snapshot; superseded responses are
ignored and cursor progress is committed only with accepted snapshots.
Native final extracted payload: Work -> Setup, observed integrity refresh -> Work
restored Worker completed, recorded output and Apply recorded. First return frame
briefly showed Worker not recorded before the full response arrived: loading-state
polish remains. No new worker or Apply was performed. This is extracted-host UI
proof, not installed, clean-machine, upgrade or exact-new-payload recovery proof.

Windows evaluation ISO verified against Microsoft SHA256:
A61ADEAB895EF5A4DB436E0A7011C92A2FF17BB0357F58B13BBC4062E535E7B9.
Receipt: D:/pytxo-beta-lab/windows-media-verification.json. User said they would
install VirtualBox; latest default-path and uninstall-registry checks did not yet
find it. No VM exists. Resume after installation is complete; no new permission
request is needed for the already authorized disposable environment.

## Work cockpit continuation — September 20, native input interrupted

Goal remains active; not a verified beta. Native inspection found and repaired
long worker headers hiding output access, incorrect singular counts, and dual
inspection panes squeezing the canvas. Work now owns density-independent padding;
short Work output shares one dock. Summary panels retain a distinct semantic CSS
class. A zero-specificity density exclusion preserves Review's compact override.
Source changes are in DockInspection, MissionDock, ExecutionMap, WorkActive,
desktop2-shared.css and the long-worker preview fixture. Regression additions are
worker-detail-density.spec.ts plus updated dock/performance journey assertions.
The performance sequence uses Fit after deliberately panning nodes offscreen;
the 50ms long-task limit is unchanged.

Verification before final rerun: 34 affected functional tests passed, isolated
performance passed, and then 50 related Review/History/worker tests passed after
fixing full-suite findings. Final exact combined runs are LIVE and must be polled,
not restarted: MSI exec session 85788; e2e:release session 62280; check session 20844.
Logs: target/work-cockpit-polish-{msi-build,release-tests,check}.log.
The final build-input manifest is target/ui-evidence/work-cockpit-polish-20260920/source-inputs.json.

An earlier MSI of this turn passed extraction/runtime checks and was inspected
natively, but is superseded by the running final build. Its extracted exe is at
D:/pytxo-beta-lab/work-cockpit-polish-20260920/payload/PFiles/Pytxo Desktop/pytxo-desktop.exe
(PID35492 at launch). Do not claim that payload includes the last summary-class
and CSS-specificity corrections. Native Work, output, and read-only Review were
observed. No new agent execution or Apply was performed.

Computer Use reported physical Escape interruption on the last native observation.
No further Computer Use input was made. Respect the user's desktop control; exact
final-payload visual acceptance remains open. Final MSI extraction should use a
new evidence directory (the earlier directory already exists). VirtualBox remains
absent from both its default path and uninstall registry. Clean installation,
installed upgrade/signed updater, actual 100/150/200% Windows DPI, final-candidate
real workflow, and public release gates remain unverified. No publish/commit/push.

## Final cockpit package verification — September 20

Sessions85788 (MSI),62280 (e2e:release),20844 (check) all exited0.
Full final suite:331 functional tests PASS plus1 isolated performance test PASS.
Svelte:0 errors/0 warnings; CSS lint PASS. Final MSI extraction and runtime-import
checks PASS. All380 inventoried source inputs remained unchanged across build.

Retained installer:
D:/pytxo-beta-lab/work-cockpit-polish-20260920-final/pytxo-desktop-1.2.2-cockpit-candidate.msi
SHA256 7F32FE7A1C11E881C8D82AE17ED75872E6E734DDC2E7AAA0E05362205913E471.
Extracted EXE SHA256:
4D692970A9B63BD32E00079521575931CE9452359ECD235533F29797DEE181EF.
Full receipt: D:/pytxo-beta-lab/work-cockpit-polish-20260920-final/candidate-receipt.json.
The copied installer hash matches the verified build output; version is1.2.2.

This final extracted payload has NOT been launched. No native input was attempted
following the prior physical Escape interruption. Earlier native captures belong
to superseded candidates; native-work-final.png was renamed
native-work-before-final-css.png to avoid implying current-package proof.

Status remains local packaged candidate, beta acceptance incomplete. Pending:
exact-final native visual and real workflow/recovery checks; clean Windows install,
retained-data upgrade and signed updater; actual DPI matrix; hosted candidate CI;
website/docs/demo alignment; approved publication and independent downloads.
Goal remains active. No source changes or new tests were needed in this continuation;
it completed authoritative verification and retained an exact reviewable artifact.

## Cockpit documentation alignment — September 20

Updated apps/web/content/docs/concepts/desktop.mdx to describe the fixed Work
canvas, List fallback, contextual worker inspection, separate output console,
commit rail, state-driven motion, and New work controls. Explicitly labels the
layout as the unpublished 1.2.2 candidate and preserves native/upgrade uncertainty.
Validation: 192 internal links checked; 45-file docs source assertion passed;
Desktop guide compiled with @mdx-js/mdx. No Desktop source, installer, capture,
public website, or release was changed in this continuation. Exact-final native
acceptance remains open after the earlier physical Escape interruption.

## Current cockpit product captures — September 20

Refreshed the stale canonical browser-fixture screenshots through capture:marketing:
25 tests passed, 27 Desktop and documentation captures, 10 marketing assets, plus
existing demo-reference outputs. Production web build passed including source-parity
verification. Eight docs/presentation tests passed, covering 1440/390 widths,
keyboard tab navigation and reduced motion. Inspected Work 1600/960, Review 1600,
and rendered website previews. Product walkthrough explicitly labels unpublished
1.2.2; demo README now identifies the final cockpit package and separates historical
native films from refreshed fixture stills. No native recordings or Desktop runtime
source changed. Nothing deployed. Receipt: target/ui-evidence/cockpit-marketing-20260920/receipt.json.
Logs: target/cockpit-marketing-refresh.log, cockpit-web-build.log,
cockpit-web-presentation-tests.log. Native, installed upgrade/updater, actual Windows
DPI and final-candidate real workflow acceptance remain pending.

## Updater receipt recovery and stale frontend packaging — September 20

User explicitly requested keeping desktop control paused. Do not resume native
interaction without user resumption. VirtualBox still absent at its default path.

Reproduced malformed update receipt preventing all feed calls. Controller now
warns that previous completion cannot be confirmed but continues checking the feed.
Install still requires a writable receipt and all prior preflight/signature guards.
25 updater/menu tests and Svelte/CSS passed. Negative repro log retained.

Initial rebuild reused the earlier 4D692970 executable: current frontend asset keys
were absent. Rejected package retained in D:/pytxo-beta-lab/updater-receipt-20260920
with rejected receipt. Added rerun-if-changed=../dist in native build.rs; genuine
native recompilation followed. New verify-desktop-embedded-assets.mjs rejects that
stale binary and accepts all 4 current hashed JS/CSS asset keys in the rebuilt MSI
payload. build:msi now calls this guard. Rustfmt/script syntax passed. This checks
embedded asset identities, not native rendering or installed-update behavior.

Latest candidate:
D:/pytxo-beta-lab/updater-receipt-20260920-rebuilt/pytxo-desktop-1.2.2-receipt-recovery.msi
MSI SHA256 34A09D4C446098EB821C9851F0B21D4713BFFBE8F34EB09E3D4940C5BBEC21A2
Extracted MSI EXE SHA256 238EC3C56A2A7E2FE500A60BD262ACE7D2CC0593F2F17B00CFE3D36A92545A6A
candidate-receipt.json in same directory. MSI extraction/runtime imports passed.
This supersedes the cockpit-final package, whose frontend freshness is not proven.
No new package launch, install, feed mutation, commit, or publication. Goal active;
native/DPI/clean-install/updater/real workflow acceptance still pending.
Logs: target/updater-receipt-*.log; source inventory under
target/ui-evidence/updater-receipt-20260920 (wrapper guard added after rebuild start).

## Public updater feed audit — September 20

Anonymous live GitHub latest.json fetch succeeded. Current public version remains
1.2.1, below the local 1.2.2 candidate. Downloaded the exact referenced Windows MSI
without launching/installing it; SHA256 matches DESKTOP_SHA256SUMS.txt for the manual
installer: 2F7FCF4D1A6E4C3958ECC304E7E7F8480847FB1F2936A092DFAE39153E3F3A24.
Signature key ID matches the configured Desktop public key (588BE8565F07EB9D).
This does not verify the cryptographic signature or installed lifecycle. Receipt
and original fetched assets: D:/pytxo-beta-lab/public-feed-audit-20260920/.
No feed mutation/publication. A no-newer-update response on 1.2.2 is expected;
valid signed newer-artifact installed acceptance remains open. Desktop input stays
paused per user request; no equivalent workaround was used.

## Public updater cryptographic verification — September 20

Closed the previous signature-verification gap for public v1.2.1 only. Used the
cached minisign-verify 0.2.5 crate in an offline local helper, matching Tauri
updater's PublicKey::decode, Signature::decode, verify(data, signature, true).
The downloaded MSI verifies against Desktop's configured public key. A one-byte
in-memory mutation is rejected; original file unchanged. Helper/source/receipt in
D:/pytxo-beta-lab/public-feed-audit-20260920; log target/public-updater-signature-audit.log.
This is cryptographic artifact verification, not an installed update test, and
cannot establish signed forward updating from local 1.2.2. No private key accessed,
new signature produced, feed changed, installer launched, or desktop input used.

## Blocked acceptance audit — September 20

Retained latest installer/executable hashes and all four current embedded frontend
asset identities rechecked successfully. Local/browser/package and public v1.2.1
signature evidence cannot close native acceptance. Native control remains explicitly
paused across at least three consecutive goal continuations; independent updater
repair, packaging repair, web/doc previews and public signature checks are now done.
VirtualBox still absent at its standard path. No live verification process is being
awaited. Goal is blocked, not complete: resume condition is user resumption of native
Desktop testing and provision of the administrator-installed disposable Windows lab.
Installed signed forward-updater acceptance additionally needs an authorized signed
newer artifact/feed; publishing/CI/pilot actions still require their own scoped
approval. Actual 100/150/200% DPI, final native Work/Review/Apply/Stop/recovery,
installation/retained-data upgrade, and matching premium native footage remain open.
Do not keep making incidental polish edits or repeating green checks to stand in
for these gates. Do not resume desktop input on an automatic continuation alone.

## Native acceptance resumed — September 20

User explicitly authorized native testing again; prior desktop-paused restriction is superseded. Exact extracted 238EC3C5 candidate launched and exercised (older installed process was separately opened by launcher; not candidate evidence). Real isolated Codex job c6482860-16b3-45fa-aed7-35e1be10509e in D:/pytxo-beta-lab/cockpit-native-fixture completed, combined checks passed, stale README probe was refused without mutation, baseline restored, fresh review successfully Applied. All seven candidate inventory hashes match post-Apply checkout; npm test 5/5. Original dirty demo repo preserved. Agent included one CRLF-to-LF ending change in reviewed package; Apply exact-byte correspondence passed, pure append assumption did not.

Native canvas drag/wheel/Fit/List, details/output and Void/Light reduced-motion inspected at ~1280x800 logical window. Original Void/normal-motion settings restored. Evidence and precise limits: target/native-cockpit-resume-20260920/ACCEPTANCE.md. No new source changes, commit or publication. Stop/recovery, full native keyboard/multiworker/latency/DPI matrix, clean installation, retained-data upgrade, signed forward updater and matching footage remain open. Native permission remains granted; do not ask again solely because older checkpoint sections say paused.

## Native findings polish — September 20

Previous turn classified as progress: exact native real run/stale refusal/Apply evidence. Follow-up fixes FlowScreen Run guidance to show actual planner blockers beside the action, removes duplicated lower blockers, and gives stale-input guidance precedence over obsolete blockers. RunReview/History singular file copy corrected. Confirmation capture exposed collapsed narrow Project/Agent selectors; header now stacks at <=700px. No authority or backend changes.

35 focused tests passed before and after final layout correction (plan-blocker-guidance, execution-topology excluding performance, laptop-workbench). Svelte 0 errors/0 warnings and CSS lint pass. Impeccable detector returned []; inspected 1280 Void and 390 Light captures. Final 390 capture has usable full-width selectors and visible blocker/Run.

First MSI build overlapped the final frontend correction; embedded-asset guard correctly rejected it as stale. Do not promote its output. Final sequential rebuild started via exec session 97523, log target/native-blocker-polish-final-build-20260920.log. Wait on actual live handle; never infer terminal state from log silence. Need retain/hash/extract/verify final package and rerun affected native blocked-plan/narrow checks before claiming these latest edits natively verified. Earlier extracted 238EC3C5 candidate remains the last retained native-tested artifact. No release or commit.

## Final polish package and native regression checks — September 20

Final sequential build session 97523 completed exit 0. MSI retained at D:/pytxo-beta-lab/native-polish-20260920/pytxo-desktop-1.2.2-native-polish.msi (SHA256 89F69E8FE12EC5494485A1A0BA24022025491C2D1F228595EAAD705B85450930); extracted EXE SHA256 60CA32B95140B84FA1FA6454202527E66DE85CC8DF783330B6609DB1FF4BCF99. MSI extraction/runtime imports and all four current embedded frontend assets passed. Source hashes and browser captures retained in target/native-polish-20260920.

Exact extracted package PID 15800/window 139186 launched. Native dirty-checkout plan shows the actual README.md blocker beside disabled Run without page scrolling. History and Review show singular prepared-file copy; existing applied result restores before/after contents and recorded outcome. Screenshots, UIA evidence, and candidate-receipt.json retained in the package directory. No new dispatch/Apply performed in this regression check; previous candidate's real run/stale refusal/Apply evidence remains separately identified.

First launch attempt exited before a window; retry after closing the previous extracted candidate succeeded. Shared-instance conflict is an unproven hypothesis. Older installed process remains separate and is not candidate evidence. Native permission remains granted. Stop/recovery, full native DPI/keyboard/multiworker/latency acceptance, clean installation/retained-data upgrade, signed forward updater lifecycle, and matching footage remain open. No commit, release, or publication.

## Native Stop acceptance — September 20

Final 60CA32B9 extracted package ran a real one-worker Codex/Orbit task in a new independent clone D:/pytxo-beta-lab/cockpit-stop-fixture. Run cfd813a2-bfb7-465c-9c91-ee2352b3d41c reached Running; native Stop and its confirmation terminated the process tree. Database run and agent status are cancelled, run finished_at is populated, process registry has no entries and includes cancelled run ID. Native UI settles to Stopped, candidate not prepared, no confirmed Apply. Tracked checkout remains unchanged (git diff empty; only .pytxo untracked). Screenshots, UIA and read-only database records retained in D:/pytxo-beta-lab/native-polish-20260920/stop-*. This verifies ordinary active-worker cancellation, not crash/restart or partial-Apply recovery.

Reproduced separate UX defect: after selecting a new workspace, empty-state New work restores a prior workspace draft and switches context back. Explicitly selecting cockpit-stop-fixture in the composer avoided misdispatch during acceptance. Fix draft/workspace routing with regression coverage next; preserve existing drafts. No source edit or publication in this pass.

## Workspace draft routing repair — September 20

Native Stop pass was progress. Fixed reproduced empty-workspace New work restoring another workspace's unfinished request: DesktopShell now separates contextual New work from explicit Continue draft, retaining per-workspace drafts in this app session. Existing Continue draft intentionally restores its original project; contextual New work restores only the current workspace's retained request. No backend/Apply change.

Svelte check 0 errors/0 warnings and CSS lint passed. All 16 menu-workflow tests passed, including new two-workspace retention regression and existing explicit Continue/dispatch-clearing tests. Initial regression fixture incorrectly retained preview runs and lacked the empty-state action; corrected to existing empty-history fixture, without weakening assertions. Evidence target/workspace-draft-tests.log.

MSI rebuild started for native regression validation; inspect the live exec handle returned in this turn and target/workspace-draft-native-build.log. Last native-tested retained package 60CA32B9 predates this routing repair. Do not claim new source natively verified until final rebuilt package is retained, hashes checked, and workspace transition exercised. Full beta gates remain open.

## Workspace draft package native return-path finding — September 20

Build session 66404 completed exit 0. Retained MSI D:/pytxo-beta-lab/workspace-draft-20260920/pytxo-desktop-1.2.2-workspace-draft.msi SHA256 A291DB43154F9752B48081709FF447794D0599DA191B17ACBD4E2E2FF087EF86; extracted EXE F5B98B1D9C9A889D375EFE678363367BF5D08D3B3719858056118645A7C8A2A1. Runtime imports and four embedded asset identities passed. Exact process PID22344/window270464 running. First launch PID4188 exited while previous candidate remained open; retry after closing previous candidate succeeded (cause unproven).

Native contextual New work now stays in newly registered draft-routing-fixture with empty editor; screenshot retained. Return to cockpit-stop-fixture exposed an additional failure: retained draft is stored but sidebar does not recover it after another workspace's empty composer clears the last-draft pointer. Fixed openCompose fallback and sidebar hasDraft to consult current-workspace retention. Added sidebar return regression. Tests and check started; use live handles from current turn. This newest follow-up is not in retained F5B98B1D package; requires rebuilt native confirmation. No dispatch/Apply in routing acceptance. VirtualBox still absent. No publication.

Follow-up validation: sidebar return regression and other functional cases passed (16/17 menu tests); Void Setup capture timed out during screenshot/Chrome teardown, no failing product assertion. A single bounded retry of that capture started. Initial concurrent check exited 1 without diagnostics; sequential npm run check passed 0 errors/0 warnings plus CSS lint. Latest source still requires package rebuild/native return-path confirmation. Do not reuse F5B98B1D as evidence for the follow-up fallback.

## Final draft build resource failure and bounded recovery — September 20

Screenshot retry session99550 passed; all 17 workflow cases now have passing evidence, Svelte/CSS sequential pass. Final MSI session24329 failed with rustc-LLVM out of memory, not a source diagnostic. C: had only 233664512 bytes free; physical/virtual free memory ~2.4GB at sample.

Relocated inactive C:/pytxo/target/local-ux-checkpoint-staged (2.3GB) to D:/pytxo-beta-lab/retained-target/local-ux-checkpoint-staged. Verified no running executable under source path, checked resolved paths, wrote full file SHA256 manifest, moved via PowerShell, created original-path junction, and verified every file through junction. Session74600 exit0. Manifest beside destination. No artifacts deleted, source/build dependencies untouched. One sequential Cargo jobs=1 MSI retry started, log target/workspace-draft-final-native-retry.log. Native acceptance of follow-up remains pending; do not promote older artifacts.

## Build retry and requested VirtualBox installation — September 20

Retry session26560 failed terminally with Windows error112: insufficient space creating pytxo_desktop_lib.lib. This is distinct from prior LLVM OOM. User explicitly requested we perform VirtualBox installation. Verified bootstrap SHA256 and Oracle Authenticode, launched normal RunAs (PID27328). Elevated UI inaccessible to Computer Use due integrity separation; did not bypass it. Verified extracted MSI Oracle signature and actual feature table; attempted standard elevated msiexec core-only install (ADDLOCAL=VBoxApplication, no restart; omit optional bridged/host-only network drivers). Windows rejected launch as operation cancelled by user. No installation success claim; do not repeatedly reopen UAC without new user steering.

Relocating two more inactive build-evidence folders with full SHA256 manifests and original-path junctions: ui-audit-fullscreen-build-2026-09-14 and pytxo-interface-polish-2026-09-13 to D:/pytxo-beta-lab/retained-target. Session13674: first verified, second in progress; poll actual handle. No source edits this turn. Latest source native package remains pending. Earlier active native candidate PID22344/window270464 remains old F5B98B1D, not sidebar fallback build.

Relocation session13674 completed exit0: both folders fully hash-verified through original-path junctions, C: free 4.07GB. Retry after this verified resource change started with Cargo jobs1; log target/workspace-draft-final-native-space-retry.log. Keep final package gate open until actual build and native acceptance complete.

## Retained final routing package; desktop interruption — September 20

Build session45439 completed exit0. Final retained MSI D:/pytxo-beta-lab/workspace-draft-final-20260920/pytxo-desktop-1.2.2-workspace-draft-final.msi SHA256 6BCE9628C3E36F534B0BB8D0FFDB0CF74DD1CE7D6788441D9050A61ACE16514D; extracted EXE SHA256 690236404B1FB82A1B2492CA4B6004FBCB6E8916A68705A3C4D2AE3133EB56E9. Extraction/runtime import checks passed; four current embedded frontend asset identities passed; retained two changed routing source/test hashes match. Exact receipt saved beside installer.

Computer Use activation of previous candidate was stopped by physical Escape. No further desktop input performed; final candidate has NOT been launched or natively accepted. Await explicit native-control resumption; automatic goal continuation does not revoke interruption. Prior candidate-specific Stop/Apply evidence remains valid only at recorded scope. VirtualBox elevated MSI request was cancelled by user; do not retrigger UAC without new steering. First blocked native-control audit after final build progress; independent receipt completion performed. No live build process remains; no commit/publication.

## Final routing package native regression passed — September 20

User explicitly resumed desktop control. Closed superseded extracted candidate; exact final EXE 690236404B1FB82A1B2492CA4B6004FBCB6E8916A68705A3C4D2AE3133EB56E9 launched PID29228/window139494. Native request retained in cockpit-stop-fixture; switching to empty draft-routing-fixture and contextual New work produced empty editor in correct project; returning to cockpit-stop-fixture exposed Continue draft and restored exact original request. Build plan available; no dispatch or Apply. Evidence and updated candidate-receipt.json in D:/pytxo-beta-lab/workspace-draft-final-20260920. UIA set_value timed out but subsequent observation confirmed text; no duplicate entry.

VirtualBox is now installed: VBoxManage 7.2.18r175117. Windows evaluation ISO detected as Windows11_64 EnterpriseEval 10.0.26200.6584, unattended supported. No registered VMs yet. Prior cancelled-install/paused-control notes are superseded by this evidence and explicit permission. Clean installation, retained-data upgrade, signed forward installed updater, complete native keyboard/multiworker/latency/DPI matrix, and final footage remain open. No commit or publication.

## Clean Windows VM provisioned; host boot failure — September 20

Created registered Pytxo-Beta-Win11 UUID35684155-43b9-489c-b3a7-ecc6ec2f470f in D:/pytxo-beta-lab/vms, 4GB/2CPU, EFI/Secure Boot/TPM2, NAT, dynamic80GB VDI, no clipboard or host folders. Verified Microsoft media receipt; unattended installation preparation exited0. Disposable generated guest credentials remain private in lab, never copy to repository. Headless boot terminated exit1 before guest execution: VBoxHardening.log identifies VERR_SUP_VP_FOUND_EXEC_MEMORY (-5619). VBoxSup running; VM poweroff verified. No Windows security changes. Exact injecting component/root cause unproven; avoid speculative process kills or repeated identical boots. Native Pytxo testing remains authorized and independent. Clean-install/upgrade gate remains open; next independent work is remaining final-package native interaction/theme verification. This turn made concrete lab preparation/diagnostic progress, not a blocked goal turn. No publication.

## Final-package native interaction/theme sample — September 20

Exact final extracted EXE remains PID29228/window139494. Recorded stopped worker opens lightweight summary without log rows. Escape closes summary and restores visibly focused selected node; Return reopens summary. Open output creates distinct bottom dock, preserves canvas and candidate/checks/review/repository rail; summary and output share tabs at this window size. Output close removes its tab, closing remaining summary restores full canvas and Fit adjusts from55% to97%. Escape while output had focus did not close the output dock; used its explicit close control instead (no claim of general Escape support). Native Canvas/List toggle shows same stopped recorded worker. No dispatch/Apply.

Void and current Light theme sampled at existing host scaling/app100% and ~1282x802 capture; screenshots native-worker-summary.png, native-output-dock.png, native-light-canvas.png, native-light-list.png in final package directory. Current source theme.ts and native UI label are Light, not Aluminum; do not claim an Aluminum-specific acceptance. Void restored after testing. UIA often returns previous tree immediately after transitions; settled observations used for verification. No measured latency/CPU claim, no native multiworker or full DPI-matrix completion. Host VirtualBox -5619 blocker unchanged; no identical boot retried. Goal remains active; no publication.

## Native idle resource concern — September 20

Exact final PID29228 stopped-worker Work canvas, Void/app100%, output closed. 30.06s sample: root24.791% of one core,51.2MiB working set; descendant WebView tree total70.32% one core,534.85MiB summed working sets (shared pages double-counted). Evidence native-idle-resources.json in final package dir. Controlled reduced-motion30.02s sample: root25.089%, tree54.03%; renderer12.682% ->2.811%, GPU7.848% ->2.759%, browser23.908% ->22.226%. Motion accounts for some renderer cost but not root/browser cost. No UI captures during measurement windows. Source idle domain polling8s/integrity60s; no root-cause inference from intervals alone. Native reduced-motion renders static ASCII glyph; normal motion restored afterward.

Process roles identified: root29228, WebView browser27272, renderer26696,GPU37792, utilities3548/36220,crashpad4212. Main process thread8156 dominates cumulativeCPU. Bitdefender bdhkm64/atcuf64 modules present; this is not proof they cause CPU or VirtualBox failure; do not disable protections. WPR available at C:/Windows/system32/wpr.exe, status says not recording; WPA not on PATH. Next useful action: scoped native profiling or clean-process comparison to separate product work, accessibility/capture instrumentation, and host overhead. Resource baseline not accepted. No source changes or publication this turn; this was measurement progress, not blocked goal state.

## Fresh-process CPU comparison — September 20

Closed old exact candidate PID29228 normally; verified absent. Relaunched identical EXE PID25292,10s warmup then30.01s sample before any sky list/state/activation against new window. Root24.678% one core/48.46MiB; process tree49.928%. native-fresh-idle.json retained. This rules out merely stale per-process inspection state as sufficient explanation; host/UIA external effects not excluded. No claim of measured visible Work state because fresh startup window was not inspected during sample. Normal motion had been restored before old process exit. Only unsent test marker draft was in old process.

Read-only thread probe on old process shows hot initial thread8156 starts in pytxo-desktop.exe, mostly kernel CPU; second cumulative thread32812 starts in UIAutomationCore.DLL. No stack attribution yet. WPR normal start CPU failed0xc5585011 profiling-policy privilege; verified not recording. Standard RunAs WPR start requested, exec session56026 is currently live waiting on elevation. User async question pending for Windows approval; do not automate security dialog, do not retrigger RunAs blindly. On continuation first poll56026 and wpr-status. If trace starts, stop promptly and save private local ETL; do not leave recorder running or infer terminal from a polling timeout. Next trace analysis must distinguish host overhead from actual product defect. No source changes/publication.

## Output lifecycle coverage and cancelled trace request — September 20

Previous turn was fresh-process measurement progress. Re-polled existing elevation exec56026; it terminated exit1 with operation cancelled by user. WPR status confirms not recording. Do not re-open UAC without new user steering. Native-control permission remains separate; no general desktop stop reported.

Extended existing untracked e2e/execution-topology.spec.ts dense-output test to close the actual output view, assert output region and event rows unmounted, then wait2200ms and assert event-read count remains unchanged. Bounded production-preview run `npx playwright test e2e/execution-topology.spec.ts --grep 'dense output stays dormant' --workers=1 --reporter=line` passed1/1,31.4s; exec76292 terminalexit0. This validates output lifecycle including timer disposal, not native CPU root cause. No app/runtime source changed; existing untracked test file preserved with narrow additional assertions. Exact native candidate remains same retained EXE. Idle CPU concern, clean Windows host failure, full DPI/multiworker/native latency and signed installed updater remain unresolved. No publish/commit.

## Native CPU comparison and multiworker fixture preparation — September 20

Compared installed PID17228 and fresh candidatePID25292 concurrently20s: installed18.581% one core/30.24MiB, candidate13.740%/48.25MiB. This is not a controlled version benchmark (window state differs) but confirms high idle cost is not exclusive to final candidate; host/root cause still unproven. Receipt native-installed-comparison.json. Source event loop has no explicit busy-loop override. Local-preview expiry posts to UI every250ms but its body only checks existing previews; no basis to blame it for25% CPU without stacks. WPR request remains terminal-cancelled, no retrigger.

Existing smoke-echo fixture has3 tasks suitable for recorded native multiworker UI acceptance. Local CLI binaries datedSept6/7, so current CLI release build started with -j1 (exec86245), log D:/pytxo-beta-lab/workspace-draft-final-20260920/current-cli-build.log. Resource check before build: ~4.6GiB free physical,~8.1GiB virtual,9GB Cfree. On continuation poll exacthandle; no duplicate build. Next: isolate a clone under D:/pytxo-beta-lab, execute clearly labeled echo/stub fixture through current CLI, inspect recorded tasks in exact Desktop. Do not present stub output as real agent execution or Apply proof. No Rust source modifications. Current native candidatePID25292 has not been targeted by sky since fresh launch; window needs fresh discovery before input. No publication.

Prepared independent clone D:/pytxo-beta-lab/native-multiworker-fixture from clean committed cockpit-stop-fixture history via --no-hardlinks. Only fixture pytxo.toml modified: Orbit/worktree/PTY,3 clearly named fixture tasks, two initial paths and one dependency on fixture-plan. No fixture execution yet. Config is intentionally uncommitted lab data; do not commit user source or claim real coding agents. Current CLI build last observed compiling pytxo-planner; exec86245 remains live.

## Native three-worker recorded canvas verified — September 20

Current CLI release build exec86245 exit0,5m12s, SHA2563AF42192BCAA7BBC69DE3BE36934D65524BFB0371853C298DA48A90E0619BCA5. Isolated native-multiworker-fixture run83f8a48b-5b04-4941-9663-8f815c6df45d completed3 echo workers in2waves, exit0each. Actual receipt Orbit, overlay/projfs-sparse-copy-v2 (config said worktree; report actual receipt). Stub command prints PYTXO_NATIVE_FIXTURE_ONLY_NO_AGENT_CLAIM. No tracked changes or Apply. Two failed-startup records preserved: modified config dirty refusal, then LF restoration while core.autocrlf=true still reported config dirty; restoring original CRLF checkout fixed it. Config now external multiworker-config.toml; no commits/reset/stash. Potential Git status/cache/line-ending behavior is observation, not diagnosed product defect.

Exact Desktop final candidate PID25292/window401314 discovered, selected new registered fixture, entered Work. Native canvas shows3tasks/2waves, single recorded fixture-plan -> fixture-followup edge and independent node with none. Background drag pans scene while header/rail remain fixed; zoom100->110%; Fit returns100% with every node visible. native-multiworker-fit.png/.txt and multiworker-status.json retained beside candidate. This is genuine native rendering of actual stored stub execution, not real coding-agent or Apply evidence, and not24-node performance/DPI proof. No new desktop build needed. Source runtime untouched; goal remains active. Earlier fresh CPU sample after launch was Setup (confirmed when first observed), not a verified Work comparison; keep its recorded scope limitation.

## Native dense canvas acceptance and remount repair — September 21

Exact retained EXE 690236404B1FB82A1B2492CA4B6004FBCB6E8916A68705A3C4D2AE3133EB56E9 remains PID25292/window401314. Current CLI dense echo fixture run7ca5f03c-25af-4a14-a7e0-97993a6e3777 completed24 workers in8waves under Core concurrency3; initial concurrency6 correctly refused. External config and logs in D:/pytxo-beta-lab/workspace-draft-final-20260920. Stub execution is not coding-agent or Apply evidence; tracked fixture checkout unchanged.

Native 24-node graph and minimap rendered, List retained pane-owned scroll and fixed boundary. Fit clamps55% and clips first/last waves at1282x802 window; plan's55% minimum conflicts with all-node Fit for this graph. Async question pending whether Fit may use overview scale below55% while normal zoom retains55–160%; no change to agreed zoom policy yet. Minimap is currently a non-interactive SVG and still fails keyboard/navigation acceptance. Evidence native-dense-fit-clipped.png, native-dense-list.png, native-dense-camera-remount.png beside candidate.

Native Canvas->List->Canvas reproduced transform loss: zoom label55% but graph reset to full size. Source observer/wheel listener was bound only on component mount, while Canvas/List replaces viewport/scene DOM. Added focused regression; initial run71580 failed with expected transform versus empty string. ExecutionMap now binds observer/listener to current viewport via effect, cleans old listener/RAF, reapplies camera when scene changes. Focused remount and existing pan/zoom/Fit tests passed2/2 (session55067 exit0); npm run check passed Svelte0errors0warnings and CSS (78572 exit0). No Rust/Apply changes. Two already-untracked files narrowly edited: ExecutionMap.svelte SHA256E6932A722DA4AAD006016F2B4D8F3FF6DDA13C892D93677BD5CD9E61811F1E45; execution-topology.spec.ts SHA2563B6AEA24F2312CBD03ABF248042D519D4788F65607DE5F494FFFB5A265611E95.

Source repair is NOT in retained native package; rebuild/native confirmation pending. Batch remaining authorized canvas repairs before another package build. No active test/build process remains. CPU attribution, VM -5619, true DPI matrix, signed forward updater and final footage gates unchanged. WPR remains terminal-cancelled; do not reopen elevation. Goal active; no publication.

## Minimap navigation implemented — September 21

Previous turn was progress: native dense evidence identified camera remount defect and source fix passed. Added minimap click-to-center using inverse SVG viewBox transform; keyboard arrows/zoom/Fit reuse camera controls, Enter/Space centers selected worker, camera viewport outline updates directly with scene transform. Minimap remains available at narrow widths. No selection, node position, dependency, agent authority or Apply changes;55–160% zoom policy unchanged pending user Fit question.

Focused four browser tests passed (26333 exit0): minimap pointer/keyboard at1280/390px, Canvas/List remount, existing pan/zoom/Fit. Rendered both screenshots inspected; header/rail remain viewport locked. npm run check passed0errors0warnings and CSS (13971 exit0). Browser screenshots under apps/desktop/test-results/execution-topology-minimap-*/minimap-*.png. Native verification of new source still pending.

Sequential MSI build CARGO_BUILD_JOBS=1 started (exec12078), log C:/pytxo/target/canvas-navigation-native-20260921.log. Last observation confirms live rustcPID4932 compilingDesktop, no terminal result yet. Poll this exact session; do not launch duplicate or infer failure from silence. Resources before build~4.8GBfreephysical/8.7GBvirtual/9.6GB Cfree. On completion extract via tooling/scripts/verify-windows-msi.ps1 to fresh D:/pytxo-beta-lab/canvas-navigation-20260921, retain MSI/source hashes, check embedded frontend assets, and exercise exact new EXE against24-task stored fixture. Current runningoldnativePID25292 is still69023640 candidate, not new source. No publication.

## Canvas navigation package native verification — September 21

Previous turn was progress plus verified live build. MSI build12078 now terminalexit0. Retained D:/pytxo-beta-lab/canvas-navigation-20260921/pytxo-desktop-1.2.2-canvas-navigation.msi SHA2562941D8DD51A1634FD070FC07A6680BB8264C2F03D6A8FD4AC213A688B2881294. Extracted EXE SHA25620A9FAD565066EC222C004522F03FBAB4A517EBB6136B75161222744CC9BF791. MSI extraction/runtime imports and four embeddedfrontendasset checks passed. Source hashes and candidate-receipt.json retained. No active build remains.

Closed supersededPID25292 normally, verified gone; launched exactnewEXEPID22948/window1777400. Native stored24-task/8wave echo fixture: explicitFit55%; Canvas->List->Canvas retains identicalcamera; wheelpanworks afterremount; minimapclickright bringswave8into view; Right pans28px; Return centersselectedfixture-01; F returnsFit. Headerandreviewrail fixed; no dispatch/Apply. Evidence native-camera-restored.png, native-minimap-keyboard.png, native-minimap-center.png in newpackagefolder. This is nativebehavior proof for boundedrepairs, not measuredlatency orbetaacceptance.

Fresh process initiallyshowed100% despite defaultFit requirement; likely plan arrivesafterinitialfit, needsreproduction/sourcefix separately. Do not confusewithuserpending55%minimumFitconflict. CurrentFitstillclipsouterwaves; minimap now suppliesworkingnavigation. Userdecisionquestion remainsunanswered. IdleCPUattribution, VMhost-5619, nativeDPI/latency, signedforwardupdater, finalfootage andpublicreleasegates remainopen. No sourceeditsthisturn; no publication.

## Delayed-plan automatic Fit repair — September 21

Prior turn made native acceptance progress for exact20A9FAD5 package. Reproduced startupFit timing defect in delayed-plan browserfixture: firstfit usesempty640x360bounds; loaded24-taskplan leavesoldcamera. Red test10963: automaticFit failed with oldscale0.797222 versusexplicitFit0.55; manualcamera preservation passed. Added preview-only1200ms plan delay fixture and two cases. Source effect now observes graphbounds and refits only whilecameraMode=fit; beginningpointerdrag marksmanualimmediately so delayeddata cannot interruptgesture. Zoompolicyremains55–160%; pendingFitexceptionquestion unchanged.

Six focusedcombinedtests pass (1098exit0), Svelte0errors0warnings/CSSpass (50839exit0), densebrowserperformance1/1pass (70304exit0): no>50mslongtask duringpan/zoom/tenselections andonereviewread. This is notnativeCPU orlatencyproof. AlreadydirtyExecutionMap.svelte, desktop-backend.preview.ts andexecution-topology.spec.ts narrowlyedited; sourcehashes D:/pytxo-beta-lab/canvas-autofit-source-20260921.json.

MSI buildstarted CARGO_BUILD_JOBS=1, exec26689, log C:/pytxo/target/canvas-autofit-native-20260921.log. Poll exacthandle; do notduplicate. Resourcesbeforebuild~4.7GBfreephysical/8.7GBvirtual/9.6GB Cfree. Retainedcurrentnative20A9FAD5PID22948/window1777400doesNOTcontainautomaticFitrepair. Onsuccessfulbuild extracttofresh D:/pytxo-beta-lab/canvas-autofit-20260921, retainMSI/sourcehashes, verifyembeddedassets andnativefreshlaunch densefixturedefaultFit plusmanualcamera preservation. Othergatesunchanged; no publication.

## Automatic Fit exact native acceptance — September 21

Build26689 completedexit0, releasecompile6m24s. RetainedMSI D:/pytxo-beta-lab/canvas-autofit-20260921/pytxo-desktop-1.2.2-canvas-autofit.msi SHA25659AAB13F8426C6B9A6466A5471013E5B1C8A7D5EA9FA8C7A00FAD11009CFDF9E; extractedEXE0E677C88A9B971F0B18C5B1BECF5870CDB56CE3CED754906A91422FDEA0D24E2. Extraction/runtimeimports/fourfrontendassetidentitiespassed. Sourcehashesandcandidate-receipt.json retained. No livebuild.

ClosedoldPID22948normallyandverifiedgone; exactnewcandidatePID18708/window68618520. Freshstart shows24tasks8waves at55% without pressingFit, confirmingdelayed-planfix natively. Minimapclick navigatesfinalwaves; Canvas->List->Canvas retainsmanualcamera. native-startup-fit.png andnative-manual-camera-restored.png retained. No dispatch/Apply; fixtureecho scopeunchanged. No sourceedits thisturn. This is acceptanceprogress, notblockedgoalturn.

Next unresolvedscope: Fitminimum/allnodes requirementconflict pendinguseranswer; native idleCPUattribution remainsunproven andWPR elevationwasexplicitlycancelled; cleanhostVM-5619; trueDPI/nativeinteractionlatencymatrix; installedsignedforwardupdate; finalauthenticfootage/publicrelease. Goalnotcomplete. Currentnativepackageandnormalmotion remainrunning. No publication.

## Non-elevated debugger route prepared — September 21

Previous turn completed nativeautomaticFit acceptance. Read-only checks: no cdb/procdump/wpa onPATH orSDKdebuggerpath; noWinDbgAppXregistered. WPRnotrecording; VirtualBoxguestpoweredoff; QEMUREADME confirmsWindowsbuildlacksTPMdevice, so notvalidWin11fallback. No WPR elevation retry, VMbootretry orsecuritychanges.

Microsoftdocs provideWinDbgviawinget andnoninvasiveuser-modeinspection (-pv -p PID), withqd toquit/detach leavingtargetrunning. Sources https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger and /debugger/noninvasive-debugging--user-mode- and /debuggercmds/qd--quit-and-detach-. This offersindependentdiagnosis beforedeclaringnativeCPUgateimpassable.

wingetshow40340completed0: Microsoft.WinDbg1.2606.22001.0, officialMSIX https://windbg.download.prss.microsoft.com/dbazure/prod/1-2606-22001-0/windbg.msixbundle, manifestSHA25612e63fb884347567bdd35f67f7aad61b26a08f8404553dad6951a10776f7d771. Startedper-usernoninteractiveinstall viawinget (--scope user --source winget --silent), exec4884 LIVE. Hashverificationpassed; WindowsStagePackageAsyncoperation0active perwingetlog. Get-AppxPackage stillabsent; donotclaiminstalled. Noelevationrequest orsystemsecuritychange. Poll4884; do notduplicateorassumeterminalfromslowstaging. Onsuccesslocateinstalleddebuggingtools, inspectonlytestPID18708 (verifyexactpath), retainstackoutputlocally, detachpromptlyusingqd; do notleaveprocesssuspended oruploadlogs/dumps. NativeCPUrootcause stillunproven. Noappsourceedits/publication.

## Native CPU stack evidence and Rust scope question — September 21

WinDbgper-userinstall4884completedexit0; Microsoft.WinDbg1.2606.22001.0 installedunderWindowsApps, signedamd64/cdb.exeavailable. Noelevationorsecuritychanges. Noninvasive CDB -pv againstexacttestPID18708 with boundedcommands andqd succeeded; processrespondingafterdetach. No livedebugger/sessionleft. Stackfiles retained privatelyin D:/pytxo-beta-lab/canvas-autofit-20260921; do notpublishrawstacks/PDB.

20sper-threadsample69955exit0: mainthread37512(hex9288)28.65%onecore,22.889%kernel; nextthreads0.311% and0.156%. native-thread-cpu.json. Stack01/03 andsymbolicstack showmainmessagehandling withbdhkm64hookframes; stack02 catchesNtCreateFile/CreateFile work. MatchingbuiltPDBcopiedtolocalcandidate symbols andresolvedaddresses: rusqlite::Connection::execute_batch, PytxoStore::open, domain_changes+0x102 (neighborload_desktop_snapshot symbolalsoappears). Native-sql-callers.txt retainsresolution. PresenceofhookisNOTproofBitdefendercausesCPU. SnapshotcorrelationisNOTfullCPUattribution.

Sourceconfirms synchronous #[tauri::command] domain_changes opensstore viaopen_store_for_domain; PytxoStore::open setsWALandappliesmigrations. Shellpollsidle8s/active2.5sacrossdomains. Concretecandidateforlag: blockingSQLiteinitialization/readwork onUIthread; avoidguessingrootcauseorUIthrottlingawayfreshness. Newasyncscopequestionpending: originalapprovedcockpitplanexplicitlyNoRustchange; requestboundedRustread-pathfix (offUIthread,avoidreinitializationonobservation,preservemissing/incompatiblestoreerrors/Applyauthority). Do NOT editRustuntiluseranswers. No appsourceeditsthisturn. Thiswasdiagnosticprogress,notblockedgoalturn. Fit55%decisionalsopending. Allotherbetagatesunchanged.

## Pending-decision audit 1 — September 21

Prior turn made diagnostic progress (successful debugger installation, detachedstackcapture, nativeCPUthreadattribution). Currentturn reconfirmedlatestEXEhash0E677C88..., candidate receipt, no livecargo/rustc/cdb/winget; reconciledstaleCURRENT_GATESheader. This is status reconciliation/no-progress for goal accounting, notnewacceptance orverifiedwait.

At current approvedboundary, next corrective engineering depends on unansweredRustscopeexception (explicitNoRustplan) andFit55% policydecision. Otherremaininggates require supportedcleanWindowsenvironment/profiling orDPIhostinteraction/releaseauthorization; existingVMfailsbeforeboot, signedforwardfeednotapproved. Do not inventapproval fromautomaticcontinuation, repeatunchangedbuilds, weakenhostsecurity, orproducefinaldemoasifbetaverified. No new safe independentimplementation identified aftercurrentread-pathinvestigation. First consecutivepending-decisionblocked audit; goalremainsactive per3turnthreshold. No appsourceedits, no publication.

## Pending-decision audit 2 — September 21

Previous turn was no-progress/blocked audit1. This turn revalidated all three latest source-receipt hashes against current files: unchanged. No new user answer to Rust scope or Fit exception; automatic goal continuation is not authorization. Same approved-boundary impasse, no live operation to wait on and no independent safe implementation identified. Second consecutive blocked audit; goal remains active until threshold. No source changes, no repeated build, no publication.

## Pending-decision audit 3 — September 21

Same Rust-scope and Fit-policy questions remain unanswered after three consecutive blocked-audit turns. No new authorization, source change, live build/debugger/install operation or independent safe next implementation. Latest package/evidence remain retained; beta objective is not achieved. Set goal blocked pending explicit scope decisions and remaining host/release prerequisites; do not keep automatically rechecking unchanged state. No publication.

## Approved Fit and observation repairs — September 21

User explicitly approved both pending exceptions. Fit now allows whole-graph overview below 55%, while manual zoom remains 55–160%; zoom-out from an overview holds its scale instead of jumping inward. Panned overview cameras survive Canvas/List remounts. Added 1280px and 390px whole-graph geometry and camera regression checks.

The domain_changes Tauri command is now async, with SQLite observation on spawn_blocking and an existing read-only connection instead of store initialization/migration per poll. Full snapshot loading also runs on spawn_blocking. Apply, orchestration and write paths are unchanged. Read-only store coverage verifies live change observation, rejected writes, missing-file preservation and incompatible schema errors.

Verification: Svelte/CSS clean; all 28 execution-topology functional tests pass; pytxo-store 27 unit + 2 integration tests pass; cargo check -p pytxo-desktop passes (an initial missing qualification was corrected). Native MSI build is running in exec session 95443, log target/cockpit-read-poll-build.log. Do not launch a second build. Prior package remains running, PID18708. No publication. New native CPU and exact-package acceptance remain pending.

## Exact candidate retained; native control stopped — September 21

Build session 95443 exited 0 after 12m03s. MSI administrative extraction, runtime import checks and four embedded frontend asset identities passed. Candidate receipt: D:/pytxo-beta-lab/cockpit-read-poll-20260921/candidate-receipt.json. EXE SHA256 834AD609777D159846583453B585ED200A5C60AC1E71D300909A2DCA6E3F32C1; MSI SHA256 519421AEA5D361CF5DEE2DF79C1E480E8168677929A20330613FAB3B575A7F80.

Physical Escape stopped Computer Use before closing the prior candidate. No native launch or CPU improvement claim for the new package. Desktop control remains paused; automatic goal continuation does not revoke the stop. A fresh non-desktop verification pass completed the dense browser performance test (session64194, exit0): 24 tasks, pan/zoom/ten selections, no recorded >50ms task, one review-detail read. Four source hashes still match the packaged source receipt. Updated the architecture note to describe the observation scheduling boundary.

Last turn and this turn are progress, not blocked audits. Remaining beta gates are native package/CPU/latency/DPI, clean install/retained upgrade, signed forward update and final authentic demo. No build or debugger is running. No source changes beyond architecture documentation this continuation, no publication.

## Native-control blocker audit 1 — September 21

Previous turn was progress: dense browser performance acceptance and candidate records completed. This continuation revalidated all four packaged source hashes and confirmed no live cargo/rustc/cdb process. No changed evidence or safe independent implementation remains within the approved acceptance work. Native control is still stopped after physical Escape; a single explicit resume question is pending. Do not treat automated continuation as permission or bypass the stop with another input mechanism. Clean Windows environment and signed forward-update prerequisites remain unresolved. This is the first consecutive no-progress audit for this blocker, not a verified wait. Goal remains active; no new build, app input, source edit or publication.

## Native testing resumed: Fit passes, CPU remains open — September 21

User explicitly resumed native testing. Old candidate PID18708 closed through its window; exact new EXE SHA834AD609... verified and launched as PID22408/window2629562. Recorded 24-task, eight-wave echo run now fits all nodes at39% in the native1282x802 window. Horizontal wheel pan and Canvas/List/Canvas camera preservation passed; Fit restores all24. Retained native-fit-24.png and native-overview-restored.png in the candidate evidence directory. No new dispatch or Apply.

Native CPU acceptance fails: prior candidate idle draft48.55% one core in20s (no build); new candidate draft21.27%, completed dense canvas19.70%,49.89MB working set. Different timing/host load prevents causal before/after claims. Mainthread6996 accounted6.59%, blocking workers8344/30572 accounted4.03/3.57%, UIAutomation thread29348 accounted3.34%. Two bounded noninvasive CDB captures with matching local PDB detached using qd; app responding afterward, no debugger left running. Snapshot shows main message-loop/WebView work and hot worker threads waiting; it does not attribute the remaining aggregate CPU or prove a fix. No security changes or remote uploads.

Candidate receipt updated. Native control is authorized, not paused. This turn is progress (new exact-package acceptance and performance evidence), not a blocked audit. Remaining: native CPU/latency, actual DPI, clean environment installation/upgrade, signed forward updater and final demo/release gates. No source changes or publication this turn.

## CPU burst attribution and isolated catalog — September 21

This turn made diagnostic progress. Source idle polling is8s plus completion time; no tight timer loop found. Minimized PID22408 sample12.24% one core;24 one-second samples show near-zero gaps then3-second CPU bursts about10seconds apart. Triggered noninvasive CDB capture during a burst reached domain_changes -> PytxoStore::changes_since -> SQLite prepare. Detached with qd. Evidence: minimized-cpu.json, minimized-cpu-series.json, native-burst-stacks.txt in current candidate directory.

Read-only catalog inspection found149 active registered domains,138 under temporary directories. This is material polling load. Original records were not removed or modified. Created a separate PYTXO_HOME at D:/pytxo-beta-lab/cockpit-read-poll-20260921/isolated-home containing only the native-multiworker-fixture domain row and normal app-created schema. Existing domain run DB is reused; no dispatch or Apply. Original catalog count149 and isolated count1 verified.

Closed PID22408 normally, launched identical EXE with process-local PYTXO_HOME as PID26632/window63571426. Current app is the isolated diagnostic profile, not the operator catalog. First12.85% sample opened the older3-worker run/rightdock and is explicitly labeled non-comparable. Selected newest24-worker run through History, returned to Work: all24 visible at39%. Matched25s sample10.73% one core,47.83MB; catalog size contributes but cannot establish full causality or pass1.2% baseline. No native performance fix claimed. Evidence isolated-dense-cpu.json.

Additional native UI defect observed: at1282x802 with right dock open, clicking Runs expands a disclosure clipped by the command-actions horizontal overflow. History remains usable. Source pointer WorkActive.svelte: run-reference details + command-actions overflow-x:auto at narrow container. Next bounded implementation: reproduce this clipped run chooser in browser and repair disclosure placement without route scrolling, then verify. CPU diagnosis remains open; do not delete temporary domain records as a performance shortcut. No code edit, build or debugger left running; native control remains authorized.

## Runs chooser clipping repaired — September 21

Native-observed defect reproduced by a browser hit-test regression at1280x900 with the right evidence dock open. Before fix, run tab was present/visible to layout APIs but document.elementFromPoint at its center returned another element; test session79342 failed as expected.

WorkActive.svelte now wraps the narrow action row instead of clipping overflow, aligns the chooser inward from the left edge at that breakpoint, raises the command strip above the canvas, and closes the disclosure after selecting a run. Existing route locking, actions and Apply semantics are preserved. Added pointer and keyboard selection/close assertions plus route-scroll assertion to execution-topology.spec.ts.

Svelte/CSS passed. Nine affected responsive Work/NewWork/chooser cases passed (session12963); final screenshot confirmation test passed (session2807). Inspected menu screenshot with right dock: unobstructed and readable. Evidence D:/pytxo-beta-lab/runs-chooser-fix-20260921. No native rebuild yet: retained834AD609 candidate does NOT include this fix; source is ahead. Do not claim latest source hashes match that old package. No build/test process remains live. Current native PID26632 is the isolated one-domain diagnostic profile. CPU and broader beta gates remain open. This turn is implementation progress, not a blocked audit.

## Indexed change bounds and combined build — September 21

Before the next package build, repaired a measured polling query inefficiency. The combined MIN/MAX aggregate forced a full domain_changes index scan. New CHANGE_BOUNDS_SQL uses separate scalar MIN/MAX subqueries in one statement, retaining a single read snapshot and allowing SQLite endpoint seeks. No schema, cursor, orchestration or Apply authority change.

Regression uses10000change rows and EXPLAIN QUERY PLAN to reject a full ledger scan while checking exact bounds/current cursor. Red result session30077 confirmed SCAN domain_changes USING COVERING INDEX; an initial fixture compile error was corrected to use a tempfile store. Green full store suite session39572 exited0:28unit+2integration tests including reset/gap and read-only observation coverage.

Combined native MSI build now running in exec session94049, log target/cockpit-chooser-indexed-build.log. Includes prior source-only Runs chooser fix plus indexed bounds. Do not start a second build or mutate frontend dist while packaging. Existing running native PID26632 remains previous834AD609 candidate with isolated PYTXO_HOME. CPU acceptance still open; query-plan improvement alone is not a native CPU fix. This turn is concrete implementation/test progress.

## Combined package native acceptance — September 21

Build94049 exited0. Retained candidate D:/pytxo-beta-lab/cockpit-chooser-indexed-20260921; EXE SHA396AABBCBA4023981985C706D582DD4AC0980CDA1BFD4BACD266C81C35415D79, MSI SHAA615F931A07B8C2626C95DCE6240DFB6B3DBE409013A43A949D085F9C678EA02. Administrative extraction, runtime imports and4embeddedfrontendassetchecks passed. Includes Runs chooser repair and indexed ledger bounds query.

Native1282x802: Runs chooser opens unobstructed with right dock; pointer selection switches to dense run and closes chooser. All24workers/eightwaves visible at39% Fit. Screenshots and receipt retained. No dispatch or Apply.

Matched isolated one-domain dense canvas sample:0.9342% one core,47.96MB working set over25seconds, versus previous10.73%. Full operator catalog149records still12.8278%,48.54MB. These are root-process idle samples, not total WebView tree or interaction latency. Catalog-scale polling remains the next performance target; do not call CPU globally fixed. Original catalog count149 verified read-only.

Launch correction: clearing PYTXO_HOME through .NET yielded an empty-profile launch; it was closed without mission activity. Explicit PYTXO_HOME=C:/Users/mattbaconz restored the operator catalog. Current exact candidate PID31128/window288557256, Work24canvas39%, no docks. Native control authorized, no build/debugger live. This turn is native validation progress, not blocked. Remaining DPI/latency, clean installation/upgrade, signed forward updater and final demo/release gates still open. No publication.

## Catalog polling attribution — September 21

Previous goal turn was progress (exact-package/native chooser/Fit and isolated/operator CPU evidence). This turn narrowed the remaining performance cause without altering product source. Source consumeDomainChanges makes sequential native calls for all149domains every idle cycle; each native domain_changes opens a fresh read-only SQLite connection. One bounded noninvasive matching-PDB capture detached with qd, but did not catch hot query work. No debugger remains.

Read-only Python diagnostic using all149catalog DB paths succeeded149/149 with no store errors: repeated open/query/close sweeps0.92/0.57/0.56seconds wall and0.81/0.58/0.56CPU seconds. Retained connections/prepared statements: initial0.53s then0.0096/0.0097/0.0136s warm sweeps. Evidence direct-sqlite-diagnostic.json and reused-sqlite-diagnostic.json in D:/pytxo-beta-lab/cockpit-chooser-indexed-20260921. These diagnostics exclude Rust config loading, IPC, rendering, file-replacement validation and are NOT native acceptance or an implemented cache.

Next bounded repair hypothesis: reuse read-only observation connections/prepared statements, with explicit identity/replacement/deletion/schema/config-path validation and bounded ownership/lifetime; preserve errors, cursor reset/gap, cross-domain observation and Apply semantics. Do not simply skip inactive domains or delete149catalogrecords. Batch IPC may also reduce round trips but direct DB reopen cost is independently significant. Need regression tests for concurrent writes, replacement/reset/deletion and incompatible schema before integration. No new build or implementation this turn. Current exact candidate PID31128 remains operator profile. Full beta goal active; this is diagnostic progress, not no-progress.

## Batched observation implementation — September 21

Previous turn was diagnostic progress. Persistent connection caching was rejected after a disposable Windows SQLite read-only connection prevented database rename with sharing violation32. Retaining handles would interfere with replacement/recovery. No dependency or connection cache added.

Implemented bounded batch observation instead: domain_changes_batch handles <=128domains in one blocking task, resolves current config per domain, opens read-only and closes each store within the sweep, propagates errors. Shared DTO conversion preserves single-command semantics. Desktop sync optionally consumes first pages in128domain batches, retains sequential follow-up pages/reset handling, validates response length; preview backends retain single-domain fallback. Consistent snapshot priming and normal polling both use batching.149domains now require2initial native calls instead of149, without skipping domains or changing polling cadence. Apply/orchestration/write paths unchanged.

Svelte/CSS passed(session61428);14review-state/synchronization tests passed using direct desktop Playwright CLI and target/cockpit-batch-playwright.config.ts. Initial npm exec invocation ignored intended test scope and hit duplicate Playwright discovery; corrected direct CLI run passed. New tests cover149domains/128+21batchbound, pagination/reset, failed/incomplete batches preserving cursors. Native test session85659 exited0: change_batches_preserve_errors_and_release_database_handles passes on Windows, covering rename afterread, missing-file noncreation, reset, incompatible store, oversizedbatch.

Combined MSI build now live in session17167, log target/cockpit-batched-poll-build.log. Do not start another build or mutate frontend dist while packaging. Running native candidate remains396AABBC... PID31128/operatorcatalog; it does NOT contain batching. Native CPU benefit remains unmeasured. Next: await17167, extractfreshcandidate, verifyassets/imports/hashes, launch andrepeat149domain24workercanvas CPU. Goal active, this turn implementation/test progress. No publication.

## Batched candidate retained; window activation failure — September 21

Previous turn implemented/tested batching. Build17167 exited0 this turn; MSI extracted and all runtime/4embeddedassetchecks passed. Receipt D:/pytxo-beta-lab/cockpit-batched-poll-20260921/candidate-receipt.json. EXE06273DACB5F5D68CBC3002873B1A5DC3F6DC80584DE08B8D59E8EBEA93E825E9; MSI56A3B15DDF86300713B082A0B0D4C79B930B458DD8E80761217CE6536EB51711. Source hashes retained. This is packaging/verification progress.

Computer Use list_windows returned old candidate288557256 but activate_window failed 'failed to activate captured window'. Fresh list/get_window and one retry failed identically. Stopped app inputs under bounded recovery guidance. No physicalEscape stop, native authorization still valid. User asked to bring Pytxo foreground and reply ready; do not re-request permission. Old PID31128 remains intended running candidate; new06273DAC not launched, no new CPU measurement. Do not restart build or claim batch performance. Native activation prerequisite pending; no blocked threshold yet. No publication.

## Batch snapshot race coverage — September 21

Previous turn was packaging progress. All six packaged production-source hashes revalidated unchanged. Added149-domain cross-chunk mutation regression: repo-0 advances after firstchunk during cursor priming; consistent snapshot reloads and returnsrevision2 with matchingcursor2. All15review/synchronization tests pass in2s; target/cockpit-batch-final-sync-tests.log. Only test source and receipts changed; no rebuild needed. Candidate receipt updated.

This turn completes a meaningful previously uncovered synchronization boundary, so progress. Native activation still awaits user foreground action; no repeated desktop input or new permission request. Next remaining exact-candidate work is launch/CPU/latency/DPI; clean Windows and signed forward updater prerequisites persist. Do not generate repeated tests solely to avoid reporting an unchanged blocker. No live build, no publication; goal active.

## Native activation prerequisite audit 1 — September 21

Previous turn was meaningful race-test progress. Current exact06273DAC executable hash revalidated unchanged; no cargo/rustc/cdb process remains. No user foreground-ready answer arrived. Same bounded Computer Use activation failure remains prerequisite for exact-package CPU/latency/DPI work; clean Windows and signed-forward-update prerequisites remain unresolved. No further independent implementation justified by current evidence. First consecutive no-progress audit for this activation blocker; goal remains active. No retry loop, new build, publication or completion claim.

## Native activation prerequisite audit 2 — September 21

Previous turn was no-progress audit1. Same old candidate PID31128/window288557256 remains responding; no foreground-ready user reply or changed activation evidence. No safe independent remaining implementation identified. Second consecutive no-progress audit for the same native activation prerequisite; goal remains active until threshold. No repeated app inputs, build or tests; not a verified wait and not complete.

## Native activation prerequisite audit 3 — September 21

Third consecutive no-progress audit for unchanged native activation prerequisite. No foreground-ready reply or external-state change; bounded activation retries already failed. No independent safe acceptance work remains identified. Candidate06273DAC and15sync tests retained; native CPU/latency/DPI, clean installation/upgrade, signed forward updater and final demo/release gates remain incomplete. Mark goal blocked pending restored desktop activation, not complete. Resume when user brings Pytxo foreground and replies ready. No new inputs/build/test/publication.

## Redundant Work commit rail removed — September 21

User resumed the blocked goal with a current native screenshot and explicitly asked to remove the highlighted Candidate → Combined checks → Human review → Repository rail. Removed the rail from ExecutionMap, including its duplicate derived state, one-shot convergence markup, and responsive/dead CSS. The command-strip Review changes action remains the single entry into exact-byte Review; the existing status sentence and recovery alert retain current outcome/recovery truth. Apply remains confined to Run Review. The canvas now uses the reclaimed height.

Svelte/CSS reports0errors/0warnings. Ten affected browser cases pass: six inspection/recovery/navigation cases plus viewport-lock checks at1920×1080,1280×720,860×760,and390×760. Screenshots at1920 and1280 were inspected; the rail is absent, Work remains viewport-locked, and the command strip/canvas hierarchy is intact. The Impeccable distill direction was applied as a bounded refinement of the approved cockpit.

Combined source plus batched-poll MSI build is live in session49792, log target/cockpit-distilled-batched-build.log. Do not launch a second build or mutate dist while packaging. Existing native PID31128 remains an older candidate. Native activation authorization is resumed by the user's continue request and current screenshot, but the new executable is not yet built/launched. Goal active; no publication.

## Distilled Work cockpit R2 and catalog-scale idle fix — September 21

The highlighted duplicate Candidate → checks → review → repository rail is removed from the Work canvas. The command strip keeps the single Review changes entry; Apply remains exclusive to Run Review. A fresh exact native package shows all 24 recorded workers/eight waves at 39% Fit with no route scroll and no bottom rail.

Catalog polling now uses one native batch per up to128 domains, indexed change-log endpoint queries, settled file fingerprints with no retained SQLite handles, snapshot-resolved store paths, and catalog fingerprint refresh. Incremental scheduling always includes the selected workspace, active runs and pending approvals; dormant domains rotate through a one-minute focused coverage window and five-minute background coverage. Same-process mutations still use native domain-change events. No orchestration, permission profile, execution domain, review or Apply semantics changed.

Validation is green: Svelte/CSS0errors/0warnings; desktop Rust46/46; strict desktop Clippy; targeted polling/catalog reload tests; prior affected viewport/inspection/recovery/navigation cases; browser long-task fixture produced no task over50ms through pan, zoom and ten selections. Exact MSI administrative extraction, PE runtime-import policy and four embedded asset identities passed.

Final retained local candidate: D:/pytxo-beta-lab/cockpit-beta-candidate-r2-20260921. MSI SHA2568E9E41935F21622D3ED0290DB4DA70AE3E2100413F269A0D99DDAA7065A325E2; extracted EXE SHA2566C92F55040185DF3F486C4470BE6C3303DED7BFE65EC289EB913AA3F3401763C. Exact EXE is PID29052/window23993946 with process-local PYTXO_HOME=C:/Users/mattbaconz and149-domain operator catalog. Native worker summary, bottom output dock, output close and summary Escape behavior passed. No dispatch or Apply.

Performance evidence preserves startup and steady states separately. First25.844s after launch:1.5719% one core,49.208MB average working set. Settled36s cadence:0.651% one core and0.03125s maximum one-second CPU delta. Final steady25.867s:0.4228% one core,49.309MB average/max. These measure the root Desktop process only, not summed WebView descendants. Receipts, raw samples, screenshot and source hashes are beside the MSI.

Still open for beta release: clean-install/retained-data upgrade because the prepared VirtualBox guest cannot boot on this host (VERR_SUP_VP_FOUND_EXEC_MEMORY -5619); signed installed forward updater; full Windows100/150/200% scaling and final theme/reduced-motion matrix; real coding-agent Work → Review → Apply mission; public release/download/website state; final Remotion launch footage. No commit, publish or release action occurred. Goal remains active.

## Exact R2 theme and reduced-motion acceptance — September 21

The packaged R2 Work cockpit was inspected natively in both shipped themes, Void and Light, with the duplicate bottom commit rail absent and the command-strip Review changes action intact. The repository implements only Void and Light; Aluminum is not a current theme and is not claimed as tested. Evidence: `native-work-no-rail.jpg` and `native-work-light.jpg` beside the exact MSI.

Reduced motion was enabled through the packaged app, the Work cockpit was captured twice 1.6 seconds apart, and the decoded 70x75 Aperture crop produced the same MD5 (`1b4e6a506b54c59a3585107cbe1f09f0`) in both frames. Evidence: `native-work-reduced-motion-a.jpg` and `native-work-reduced-motion-b.jpg`. This verifies the visible ambient Aperture frame settles under reduced motion; it does not establish every hidden animation path. Operator settings were restored to Void, 100% in-app scale, compact density and normal motion, and the app was left on Work.

The exact package receipt now records these checks. This closes the shipped-theme and native reduced-motion sample for R2. Windows display scaling at 100/150/200%, clean installation/retained-data upgrade, installed signed forward update, a real coding-agent Work → Review → Apply mission, public release state and final launch footage remain open. No dispatch, Apply, commit, publish or release action occurred; the beta goal remains active.

## R3 updater fallback and release-channel evidence — September 21

The redundant Work commit rail remains removed in exact candidate R3. The command-strip Review changes action is the only Work entry into exact-byte review, and Apply remains confined to Run Review. Exact native R3 rendered the completed 24-worker/eight-wave canvas at 39% Fit with no bottom rail. Evidence is retained in `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/native-work-r3.jpg`.

Candidate R3 packages the two-endpoint updater configuration: the preferred stable raw-main manifest followed by the existing GitHub Latest-release manifest. On September 21 the preferred endpoint returned HTTP 404, the fallback returned HTTP 200 with public signed-manifest metadata for v1.2.1, and exact candidate v1.2.2 correctly reported no newer version without an updater error after automatic and manual checks. This verifies endpoint failover and version comparison, not a signed forward installation. Native evidence is `native-updater-fallback.jpg`.

Release publication now mirrors a signed `desktop-dist/latest.json` into the public repository main branch only after release asset publication. Unsigned releases preserve the previous signed manifest. The exact release-inventory verifier now rejects oversized or malformed manifests, invalid SemVer/date/platform/signature metadata, and updater URLs that do not bind the exact version and filename. Fourteen release inventory/channel tests, ten updater-controller cases, and Svelte/CSS checks pass.

The currently public v1.2.1 Desktop release was downloaded to `D:/pytxo-beta-lab/public-updater-audit-20260921` and passed the current exact signed-inventory verifier. Its manual and updater MSI copies are byte-identical at SHA256 `2F7FCF4D1A6E4C3958ECC304E7E7F8480847FB1F2936A092DFAE39153E3F3A24`; `latest.json` SHA256 is `B7DE0572248AC3D4D5F4C07214B392810EFEB227F361BA713FAF9FB4D1C061EF`. This validates inventory shape, checksum coverage and signature metadata shape; Tauri cryptographic verification still requires an actual signed install flow.

Retained candidate: `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921`. MSI SHA256 `8B8E666B0354616C4F2C082917DD184A9AEE8846614EB411444538BE88F27883`; extracted EXE SHA256 `FFC6DCCD75CACF193B46AF4BBAE9FED329C4F5AB8D9E4F457F754213A32154BC`. Administrative extraction, PE runtime imports and four embedded frontend identities passed. Exact EXE remains PID29992/window3415854 under the 149-domain operator catalog. First post-updater-check 25.874-second sample measured 1.8117% of one core and 49.32MB average working set; the settled 25.838-second sample measured 0.5442% and 50.052MB. Both are root Desktop process measurements, not summed WebView descendants.

The installed legacy product was inspected read-only at version0.9.0. Candidate R3 keeps its per-machine scope and UpgradeCode `{A941B0FB-47E1-5D2C-8982-D79D9DE352A6}` with a new ProductCode, so MSI upgrade metadata aligns. The installed UI reported v1.2.1 available, but no updater action or candidate installation was triggered. This is not retained-data upgrade or signed-forward-update proof.

Machine-readable receipts are `candidate-receipt.json`, `public-updater-audit.json`, `installed-legacy-observation.json`, and `source-hashes.json` beside R3. Source receipt SHA256 is `CF61A90F95EF0097AFC141B02F885ADF49A83540EEBB8F38A2F701ED378AFD6E`; all 27 recorded hashes recomputed with zero mismatches. Clean install/retained-data upgrade, a genuinely newer signed forward update, Windows display scaling at 100/150/200%, a real coding-agent Work → Review → Apply mission, public release/download state and final Remotion launch footage remain open. No dispatch, Apply, install, commit, publish or release action occurred; the beta goal remains active.

## Exact R3 real-agent Work → Review → Apply mission — September 21

The outstanding real-agent journey is now verified against a disposable clone at `D:/pytxo-beta-lab/real-codex-mission-r3-20260921`. Baseline was five passing dependency-free tests. The local plan preview produced one Codex worker, one wave, explicit ownership of `src/risk-policy.mjs`, `test/risk-policy.test.mjs`, and `README.md`, and `npm test`, with no warnings or blocked reasons. Run `a180a916-023b-4ada-99ad-7e7d20bb8cd4` used Orbit, PTY execution and the recorded `projfs-sparse-copy-v2` overlay backend. The canonical fixture remained unchanged while the worker ran.

Real OpenAI Codex CLI0.144.4 completed with exit0. It added concise review reasons for network and destructive-command paths, regression tests, and two README examples without changing legacy result shapes. Pytxo then ran its separate bounded verifier and recorded `npm test` passing all7 cases. Candidate manifest version3 contains exactly3 modified files,0 additions,0 deletions,4029 candidate bytes and package digest `350cbbc5810f375152c9eb098ecfd840503adc73d0b0601c93c1fb5c32e29f85`.

Exact R3 Desktop showed the genuine worker running in the one-node canvas with Stop available and Review disabled, then settled to Completed with saved checks passed. Run Review displayed the3 prepared files, exact before/after bytes, combined checks passed, canonical destination and Ready to Apply. Apply required a second exact-package confirmation and committed attempt `457f6694-c682-4215-9700-6d1dcce0e90e`. The durable journal phase is `committed`, all3 operations completed, no directories were created, and canonical hashes match the candidate. Post-Apply `npm test` passed7/7. The canonical checkout shows only those3 modified files plus the untracked `.pytxo` runtime directory.

Native evidence beside R3: `native-real-codex-running.jpg`, `native-real-codex-completed.jpg`, `native-real-codex-review.jpg`, and `native-real-codex-applied.jpg`. Machine-readable evidence is `real-codex-mission-receipt.json`; candidate receipt was updated to record the fixture-scoped dispatch and Apply. This closes the real coding-agent Work → Review → Apply release gate for exact R3. It does not prove clean installation, retained-data upgrade, a newer signed forward update, full Windows100/150/200% display scaling, public release/download state, or final Remotion footage. No Pytxo source commit, push, publish, installer action, public release or external deployment occurred; the beta goal remains active.

## Native scaling baseline and clean-host blocker narrowed — September 21

The exact R3 process reports window DPI120, so all current R3 native evidence was captured at the host's actual Windows125% display scaling with Pytxo's in-app scale at100%. The 1282×802 Work, real-agent running/completed, Run Review, committed Apply and updater surfaces render correctly at that scale. Receipt: `native-display-scaling.json`. This adds a real125% baseline; it does not substitute for the requested100/150/200% system-scaling matrix.

Windows Sandbox is not installed. VirtualBox7.2.18 and the prepared `Pytxo-Beta-Win11` VM are present, but the guest remains powered off because VirtualBox hardening stops before boot with `VERR_SUP_VP_FOUND_EXEC_MEMORY (-5619)`. `VBoxHardening.log` records Bitdefender `atcuf64.dll` and `bdhkm64.dll` modules in the hardened process, then reports executable memory when opening `VBoxDrvStub`. Hyper-V host services are also active, but the fatal logged condition is the hardening/executable-memory rejection; do not claim Hyper-V as the established cause. No Bitdefender exclusion, Windows security change, VM mutation or guest install was attempted. Receipt: `clean-install-blocker.json`.

Clean-install and retained-data upgrade acceptance now has a concrete resume condition: use a separate clean Windows host, or explicitly authorize a temporary security-product compatibility change for this isolated VM and restore protection immediately afterward. The updater and application receipts remain valid without that gate, but beta release readiness does not. Goal remains active; no install, commit, publish or release action occurred.
## Duplicate commit rail removed and product captures refreshed — September 21

Removed the redundant bottom Candidate to Repository commit rail from the Work canvas. The command-strip `Review changes` action remains the single entry to Run Review, and Apply authority remains confined to the exact-package review screen. The exact R3 native candidate had already passed with this rail absent.

Refreshed the deterministic product capture set against the final cockpit source. All 25 capture cases passed. The Work image is now identical across Desktop, documentation, website, and Remotion at 1600x1000; the 960x640 Desktop, docs, and website mirrors also match. Website verification passed for 27 Desktop source captures, 27 docs references, and 10 marketing captures, including dimensions, distinct content, and source parity. Silent Remotion asset validation passed. Svelte reports 0 errors and 0 warnings, CSS lint passes, and all 71 focused Playwright checks pass across topology, viewport ownership, camera and minimap, output loading, epistemic states, themes, accessibility, and the dense-canvas performance fixture. Evidence is retained in `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/work-capture-refresh.json`; all 27 packaged source identities still recompute with zero mismatches.

The capture command refreshed every deterministic route because npm treated the attempted grep text as a positional argument. This was harmless and the complete 25-case suite passed; the full source/docs/site capture set now reflects the same current UI. No publication, deployment, commit, release, or installer mutation occurred. Remaining beta gates are unchanged: clean-host install and retained-data upgrade, the 100/150/200 Windows scaling matrix beyond the verified host 125 percent, and a signed forward updater installation.

The first complete Desktop release rerun exposed two stale background-refresh assertions: the lag repair intentionally changed the full recovery audit from 60 seconds to five minutes, but the tests still required `Updated just now` at one minute. The runtime was left unchanged. The tests now prove the actual contract: Setup honestly reports a one-minute-old lightweight snapshot, entering Work immediately restores a full worker snapshot, incremental polling remains active, and the five-minute recovery audit refreshes without dropping worker evidence. The isolated file passes 3/3. The final release gate then passes all 355 non-marketing Playwright checks plus the isolated dense-canvas performance check. Full output is retained as `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/desktop-e2e-release-final.txt` with SHA256 `ecbeee11500c5b35ed796b7014062f9c47ed79f4efaab06ef0bcabcf976986ca`.

## Website, docs and SEO candidate verified — September 21

The unpublished website now matches the beta product boundary: Windows Desktop plus a vendor agent CLI is the supported first path, the current public GitHub and npm release is 1.2.1, and workspace 1.2.2 remains explicitly unpublished. The homepage carries `WebSite` and `SoftwareApplication` structured data, all 45 public MDX pages have explicit descriptions, and Account/sign-in/sign-up are `noindex`. Dormant account copy no longer presents Pro, Max and Ultra as ordinary public subscribe tiers, and Account is absent from primary footer navigation.

The fresh Next production build passed TypeScript and generated 63 static pages. ESLint passed; all 192 internal links across 114 source files passed; product assets remain aligned across 27 Desktop captures, 27 documentation references and 10 marketing captures. The fresh production build then passed 19/19 marketing, presentation and docs Playwright checks at desktop and mobile widths. The expanded site audit passed hero geometry at 1280/1440/1920, AA state-chip contrast, zero console warnings on the primary marketing/docs routes, canonical/title/description metadata, structured-data parsing, robots and sitemap coverage, and private-route `noindex` behavior.

Live public evidence remains deliberately separate from the local candidate. `pytxo.com` returns HTTP 200 but still serves the older “Run coding agents in parallel” title and prior product narrative. GitHub Releases and npm both report 1.2.1, and the public Windows MSI exists. The preferred branch-backed updater manifest still returns HTTP 404; the signed GitHub Latest-release fallback returns HTTP 200 for v1.2.1. No website deployment, stable-manifest publication, package publication, source commit, push or release occurred.

Machine-readable evidence is `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/web-beta-audit.json` with SHA256 `d2ffa091f52c0428d43bcd56e3d5d3d7ece0af53a584127a0e00006a5efaea24`. Updated candidate receipt SHA256 is `bb490243f60527058e634301632a9886fa14bf426d2f2ac56086372f7808c197`. Remaining beta gates are clean-install and retained-data upgrade, Windows 100/150/200 percent scaling beyond the verified 125 percent host, a genuinely newer signed forward update, publication of the stable updater mirror, approved website deployment/public release, and final Remotion footage.

## Remotion 4K pipeline verified; current R3 film still open — September 21

The demo guide now binds the exact R3 MSI `8b8e666b…` and executable `ffc6dccd…`, the verified real Codex Work → Review → Apply journey, and the current product-capture boundary. Stale September 20 package language was removed. The premium R3 shot plan requires a pointer-visible native sample first, then one bounded request through plan, canvas execution, worker inspection, exact Review, continuous confirmation-to-Apply receipt, and History. Current R3 evidence includes exact native stills but no continuous video.

Added a resolution profile to the master validator and a bounded 4K clip renderer. The historical proof command rendered frames0–29 of the September8 evidence composition at3840×2160,30fps,H.264 CRF16,yuv420p,BT.709 with concurrency1. Full decode passed and the inspected midpoint frame is sharp and visibly labeled September8. Clip SHA256 `78a81600ae8b6d247d3246f993aa637acb15eb0b84cf435916f1c5b8919d20c7`; receipt `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/demo-4k-pipeline.json` SHA256 `e138dcf826e7b80baa879d89faac81ddacb1c4f2eac5a0e7852b82f82a278c30`. TypeScript, composition discovery, the existing52-second1080p master regression, asset validation and diff checks pass.

This closes only the export pipeline. The current host is1920×1080, so existing product pixels remain 1080p/1282×802 source when upscaled; a true native4K R3 master needs fresh continuous footage from a3840×2160 target and a new provenance-bound composition. Updated candidate receipt SHA256 is `72cfe6545cbd17e0daee7c802bbcac655cf8bbed854d1a5676ef4e86f4b2b8d1`. No recording, upload, deployment, publication, release, commit, or push occurred.

## Exact R3 motion storyboard rendered and reviewed — September 21

Added `PytxoR3Storyboard`, a24-second review composition using the four exact native R3 mission stills. The manifest binds every input hash and dimension and explicitly disables continuous-footage, pointer-motion and native4K claims. Two source-coordinate masks cover the disposable validation path in Review and applied stills. The masks now share the source-image transform so they stay aligned through zoom and dissolves. The design uses a sparse operator briefing,12fps Chroma Aperture, native-pixel cockpit panels and Execute→Candidate→Review→Apply progress rail.

The first26-second render passed format validation but boundary inspection exposed dips toward black where touching sequences both faded out. It was superseded rather than retained as evidence. The next24-second render fixed those gaps, but source review then found that its fixed-position privacy masks could drift while the screenshots zoomed; output `d8f81a68…` was also superseded. The accepted24-second composition overlaps each handoff by12frames and transforms each mask with its source image. The final transition sheet shows continuous dissolves without black gaps; the encoded six-scene contact sheet preserves readable hierarchy and both privacy masks.

Final review artifact: `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/pytxo-r3-storyboard-review.mp4`,10,547,264bytes,SHA256 `d57c73c2ecfaa9e31abd05c10da1bbd058e845b2cd527866a62490211aa9d683`. H.264/yuv420p/BT.709,1920×1080,30fps,24.000seconds, silent. Exact-asset validation, mask bounds and transform alignment, TypeScript, composition discovery, full machine decode, contact-sheet and transition inspection pass. The exact encoded file completed a full 1× local agent playback with playbackRate1, readyState4 and ended=true; normal-speed human playback remains open. Receipt SHA256 `f491d23456a67c982efd9bf24df1eea00d2d5aaefa13100672e3f7f18e1bf3b0`; updated candidate receipt SHA256 `1655a1b0a02b087c487a896d4b5df6644103ad80ad6272262f06881f62637001`.

This is a motion-design storyboard, not continuous native interaction, pointer evidence, native4K footage, normal-speed human playback or a public launch master. Fresh pointer-visible R3 capture can now replace the still scenes without redesigning the narrative. No upload, publication, deployment, release, commit or push occurred.

## Work cockpit semantic zoom R4 — September 21

The dense Work canvas now changes representation with camera scale instead of shrinking the full worker card indefinitely. Before: the eight-wave Fit view preserved all 24 workers at 39 percent, but card descriptions and secondary status text were too small to scan. After: below 55 percent the canvas shows concise topology tiles with status, wave, task ID and agent/state; below 28 percent it reduces to status plus task ID; at 55 percent and above the full worker card returns. The redundant Candidate → checks → review → repository rail remains removed. `Review changes` remains the single Work entry into exact-byte Review, and Apply remains confined to Run Review. No Rust, orchestration, permission, execution-domain or Apply semantics changed.

Exact local R4 candidate: `D:/pytxo-beta-lab/cockpit-beta-candidate-r4-20260921`. MSI SHA256 `98186AF22339E01A37F4853F31D10729D6E381A3877F683A5E2D8D7A9C30FB99`; administratively extracted EXE SHA256 `210CD5612E369B9B57BA5CD0579705857304CA104436B1D525D995D430852CC8`. MSI extraction, the packaged runtime-import policy and all four current embedded frontend asset identities pass. Machine-readable candidate receipt SHA256 is `5DD37935C4F7D8FDA7082F1CECE6430F03125740EACBE23F9081BE2983CF6D41`.

Native acceptance used the exact extracted R4 executable at the host's actual Windows 125 percent scaling/window DPI 120. `native-dense-r4-dpi.png` shows all 24 recorded workers and eight waves simultaneously at 39 percent Fit, readable task IDs, the minimap, no route scroll and no duplicate commit rail. `native-work-r4-operator.png` shows the full one-worker card and command-strip Review action under the operator catalog. Only those DPI-aware full-window captures are authoritative; earlier generic and unscaled capture attempts in the evidence directory are explicitly excluded by the receipt. Exact R4 remains running as PID 28216 with process-local `PYTXO_HOME=C:/Users/mattbaconz`.

Final combined validation is green: Svelte reports 0 errors and 0 warnings; CSS lint passes; the definitive non-marketing Desktop suite passes 355/355 serially; the isolated dense-canvas performance fixture passes 1/1 with no browser main-thread task over 50 ms. An earlier four-worker run produced 349 passes plus six setup/load timeouts; every exact failure then passed 6/6 serially before the definitive 355-test run. A settled 25.979-second native sample on the isolated lab profile measured 0.2406 percent of one core and 43.051 MB average/max working set for the root Desktop process only. It is not a summed WebView measurement or a like-for-like 149-domain catalog comparison.

R4 is a verified local UI/package candidate, not a release. Still open: clean installation and retained-data upgrade, Windows 100/150/200 percent scaling beyond the verified 125 percent host, a genuinely newer signed forward update, publication of the stable updater mirror, approved website/release publication, continuous pointer-visible R4 footage, native 4K source capture and human playback review. The prepared VirtualBox guest remains blocked by `VERR_SUP_VP_FOUND_EXEC_MEMORY (-5619)`; no Bitdefender or Windows security setting was changed. No R4 dispatch, Apply, install, commit, push, deployment or publication occurred.

## R4 clean-backend initialization and retained-data clone — September 21

The exact extracted R4 executable was launched against two new process-local homes without installing it. An empty backend home created only `.pytxo/hypervisor.db`, schema version 1, with the expected `domains`, `fleet_nodes`, `fleet_runs`, `flow_drafts` and `project_roots` tables and zero rows. The native empty Work state rendered successfully at Windows 125 percent scaling. Existing sidebar recents remained visible because WebView storage is shared independently of `PYTXO_HOME`; this evidence therefore proves clean backend initialization, not a clean Windows user or WebView profile.

For retained-data compatibility, the closed operator `.pytxo` tree was copied byte-for-byte into a private lab home: 12 files, 236,497 bytes and zero initial hash mismatches. The clone contained catalog version 1 with 150 domains, 24 flow drafts and eight project roots. Exact R4 reopened the completed real-Codex workspace natively, then closed with all 12 clone hashes unchanged, no new clone files, identical schema/table counts and zero hash changes in the live source data. A later Python schema inspection created SQLite SHM and zero-byte WAL sidecars 16 seconds after the launch comparison; the receipt records that provenance and does not attribute them to R4. This proves current-data compatibility and read-only preservation for the original cloned operator payload. It does not exercise a legacy schema migration or an MSI retained-data upgrade.

Evidence is retained at `D:/pytxo-beta-lab/state-compat-r4-20260921`; `state-compatibility-receipt.json` SHA256 is `C45A6290D2F40281805E78C213B92761B909A2E05B9DC49CE238C63686E60421`. No dispatch, Apply, installation, live-catalog mutation, commit, push or publication occurred. Clean MSI install and legacy retained-data upgrade remain blocked on an isolated Windows environment.

## R5 clean first use and final cockpit candidate — September 21

R5 supersedes R4 as the current local candidate. It carries the same scale-aware recorded-worker canvas and removed duplicate commit rail, then corrects the first-use promise to `Review every change` with `Inspect the exact changes before you decide what to save.` Work still routes through the command-strip `Review changes` action, and Apply remains confined to Run Review. No Rust, orchestration, permission, execution-domain or Apply semantics changed.

Exact local R5 candidate: `D:/pytxo-beta-lab/cockpit-beta-candidate-r5-20260921`. MSI SHA256 `1B73E659F9CCDDFD9AD6B954A70308BC84A5944EF18E8E83CFD2F8D37E6B4A0C`; administratively extracted EXE SHA256 `B3119218F205AB7ECD7A1516813950B14482309990A5FEB12C84FADB76D9A722`. MSI extraction, runtime-import policy and all four embedded frontend asset identities pass. Svelte reports 0 errors and 0 warnings; CSS lint passes; the focused shell/topology suite passes 82/82; the definitive non-marketing Desktop suite passes 355/355 serially; the isolated dense-canvas performance fixture passes 1/1. The failed focused invocation retained beside the candidate used Windows path separators, matched no tests, and was superseded by the corrected 82-test run.

Native acceptance used the exact extracted R5 executable at Windows 125 percent scaling/window DPI 120. A fresh `PYTXO_HOME` plus fresh WebView profile opened the actual Welcome step without inherited recent workspaces; the revised copy and full `Get started` action are visible in the DPI-aware 1618×1010 capture. A separate empty backend home initialized schema version 1 with the expected five tables and zero rows. The shared WebView profile still supplied recent-workspace labels in that second check, so it proves catalog initialization rather than clean-user state. The dense lab capture shows all 24 workers/eight waves at 39 percent Fit with readable IDs, dependency edges, minimap and no bottom rail. A settled 20.017-second root-process sample measured 0.3122 percent of one core, 40.559 MB working set and 11.461 MB private memory. This is not a summed WebView-tree measurement.

Candidate receipt: `D:/pytxo-beta-lab/cockpit-beta-candidate-r5-20260921/candidate-receipt.json`, SHA256 `F3ABD6A1DF0A6EBECD4D130A6C1F9B54E6574035D01E3FFDF848EDA0FE0E7F64`. State receipt: `D:/pytxo-beta-lab/state-compat-r5-20260921/state-compatibility-receipt.json`, SHA256 `D9B90003AA2DCC7FD80373C41C10AD44109DD994CD2153F8D2F4E0B01394C6E0`. Exact R5 is left running under the operator catalog as PID 30360. R4's byte-identical current-data clone remains useful historical evidence; an exact-R5 retained clone or installer upgrade was not rerun and is not claimed.

R5 is a verified local UI/package candidate, not a releasable beta yet. Still open: clean/elevated MSI installation and retained-data upgrade, native Windows 100/150/200 percent scaling beyond the verified 125 percent host, a genuinely newer signed forward update, stable updater mirror and approved website/release publication, continuous pointer-visible R5 footage, native 4K source capture and human playback review. The prepared VirtualBox path remains blocked by host security hardening; no Bitdefender or Windows security setting was changed. No R5 dispatch, Apply, install, commit, push, deployment or publication occurred. Goal remains active.

## R6 New Work command ownership — September 21

R6 supersedes R5 as the current local candidate. New Work now has one command for each phase: the request composer owns the single `Build plan` action, and the prepared-plan pane owns `Run`. The second `Build plan` button beside `Run` was removed. This reduces duplicated authority without changing plan generation, dispatch, Rust, orchestration, permission, execution-domain or Apply semantics. The existing scale-aware Work canvas, single command-strip `Review changes` entry and exact-package Apply boundary remain unchanged.

The updated planned-flow captures at 1600×1000, 1280×800 and 960×640 show one `Build plan` action and one `Run` action. The initial affected run passed 133/135; both failures came from a new role query that excluded the intentionally hidden compact pane at 860 and 390 pixels. The corrected DOM-scoped assertion passed all four target viewports, and the definitive combined suite then passed 355/355 serially. Svelte reports 0 errors and 0 warnings, CSS lint passes, and the isolated dense-canvas performance fixture passes 1/1.

Exact local R6 candidate: `D:/pytxo-beta-lab/cockpit-beta-candidate-r6-20260921`. MSI SHA256 `D2950419F3F843B5D615E46F0986544650F5B57DF48426795B9D1FADDDA8183A`; administratively extracted EXE SHA256 `0B9BCACEAF1F5A04867CBDB86DDF4DBCAE7DD9956D1970D3EE227F498CDA02FA`. MSI extraction, runtime-import policy and all four embedded frontend asset identities pass. The specific planned-flow interaction was verified in the production-preview browser suite and captures; exact-package native launch was verified separately.

Native acceptance used the exact extracted R6 executable at Windows 125 percent scaling/window DPI 120. A fresh Pytxo home plus fresh WebView profile opened Welcome without inherited workspaces and kept `Review every change` plus the full `Get started` action visible in a DPI-aware 1602×1002 capture. The isolated dense fixture shows all 24 workers/eight waves at 39 percent Fit with readable IDs, dependency edges, minimap and no bottom rail. A settled 20.009-second root-process sample observed no CPU-time increment at the timer resolution, 40.832 MB working set and 11.383 MB private memory; it is not a summed WebView-tree measurement.

Candidate receipt: `D:/pytxo-beta-lab/cockpit-beta-candidate-r6-20260921/candidate-receipt.json`, SHA256 `225D72B36C7AD971B58379AFF2BDC8F5DDAB94A81710D8C837BCF691C2C92855`. State receipt: `D:/pytxo-beta-lab/state-compat-r6-20260921/state-compatibility-receipt.json`, SHA256 `0B3049F5BF1A1516C65E3330BA394F9DC211A25D6C0DFEF106A006332074A757`. Exact R6 was closed cleanly before the installed-updater handoff; its prior isolated 24-worker native process and capture remain recorded.

R6 remains a verified local UI/package candidate rather than a releasable beta. Still open: clean/elevated MSI installation and retained-data upgrade, native Windows 100/150/200 percent scaling beyond the verified 125 percent host, a genuinely newer signed forward update, stable updater mirror and approved website/release publication, continuous pointer-visible R6 footage, native 4K source capture and human playback review. VirtualBox 7.2.18 is now installed and the prepared `Pytxo-Beta-Win11` VM is registered, but a fresh bounded start attempt terminated before boot with VirtualBox hardening error `-5619` / executable memory; the VM remains powered off. Evidence: `clean-host-blocker.json` beside the R6 candidate. Windows Sandbox and Hyper-V Manager are unavailable on this Windows 11 Home host. No Bitdefender or Windows security setting was changed. No R6 dispatch, Apply, install, commit, push, deployment or publication occurred. Goal remains active.

## Installed updater handoff pending secure-desktop confirmation — September 21

The installed 0.9.0 Desktop exposed the public signed v1.2.1 update and had already prepared `Restart to update`. Invoking it caused Desktop to exit cleanly and launched the real passive Windows Installer. The downloaded temporary MSI is byte-identical to the independently audited public updater artifact: 10,661,888 bytes, SHA256 `2F7FCF4D1A6E4C3958ECC304E7E7F8480847FB1F2936A092DFAE39153E3F3A24`. The installer is currently waiting at Windows' secure UAC prompt. That prompt requires physical user confirmation and is not automated. Evidence: `D:/pytxo-beta-lab/installed-updater-r6-20260921/updater-handoff-pending.json`, SHA256 `DC7FD394CB85397C43E0F7636151887267D0A335F104EA5B9BE785E3A30BD9F4`.

This verifies signed download identity, controller-to-installer handoff and clean Desktop exit. It does not yet verify elevation, MSI upgrade completion, retained data, relaunch or the running 1.2.1 version. The exact R6 lab app was closed before this handoff and can be relaunched after the installed-upgrade result is known.

## Exact R6 cockpit motion proof rendered and reviewed — September 21

Added `PytxoR6CockpitProof`, a 12-second review composition bound to the exact R6 candidate and two hash-locked native frames: clean first use and the 24-worker/eight-wave canvas at 39 percent Fit. The film retains the Chroma Aperture palette and 12 fps ambient ASCII glyph while keeping the dense completed graph static. Its persistent disclosure says `R6 cockpit proof · exact native stills · continuous capture pending`. It does not synthesize worker motion, a pointer, elapsed time, Review/Apply interaction or repository effects.

The retained master is `D:/pytxo-beta-lab/cockpit-beta-candidate-r6-20260921/pytxo-r6-cockpit-proof.mp4`, 3,888,454 bytes, SHA256 `E47E651C88AC32A68040B124151D3018CB675D3986BA5F21525BF98E4AA87FE8`. It is H.264/yuv420p/BT.709, 1920×1080, 30 fps, 12.000 seconds and silent. Exact-asset and observation validation, TypeScript, composition discovery, full master validation, black/freeze full decodes and six-frame contact-sheet inspection pass. The contact sheet SHA256 is `AAFF9A7D10EFA07C1F6047315D57424C726AD3B2CF012C6297E767DFB79182D6`.

The 3840×2160 delivery raster is `pytxo-r6-cockpit-proof-4k.mp4`, 9,639,231 bytes, SHA256 `29CB57C8B47CBB2543C5ED7B4F1F6DAB7A692558E0E508138E92ECF857F08F5C`. Its full master validation, full decode and original-raster midpoint inspection pass. The vector overlays are 4K; the embedded 1602×1002 native frames remain upscaled stills rather than native 4K capture. Updated proof receipt SHA256 is `069F978C2593F5C718920C3297B34FEC15CAA4AE5BE920206C4EEEAACB36A292`. Continuous pointer-visible R6 capture, the real Review → confirmation → Apply sequence, native 4K source capture, privacy review and normal-speed human playback remain open. No upload, publication, deployment, release, commit or push occurred.

## R7 explicit Windows updater handoff — September 21

R7 supersedes R6 as the current local package candidate. The Work cockpit source identities remain byte-for-byte equal to R6, so the recorded-worker canvas, single `Review changes` entry, one-command New Work flow and removed Candidate → Checks → Review → Repository rail remain intact. R7 changes the updater handoff that made the installed update feel as if Desktop disappeared: `Download update` now downloads and verifies while Pytxo stays open, then a distinct `Install and restart` action explains that Pytxo closes and Windows asks for administrator permission. Re-check is disabled while verified bytes are owned by the ready or restart-required state. Installer, handoff and receipt failures retain the verified resource for retry. Rust, orchestration, permission and Apply semantics are unchanged.

Verification is clean: Svelte reports 0 errors and 0 warnings, CSS lint passes, focused updater/controller tests pass 12/12, the definitive non-marketing Desktop suite passes 357/357 serially in 7.6 minutes, and the dense-canvas performance fixture passes 1/1. Exact local R7 candidate: `D:/pytxo-beta-lab/cockpit-beta-candidate-r7-20260921`. MSI SHA256 `76D5F98ED0FB7541763F07C331835F713029C547BA8839A4A5D829E6B3D4AA13`; administratively extracted EXE SHA256 `9303857141D6A3727BDF2D20B54181CCED87335DCEC9FC20EB51F5E68BF4076A`. Administrative extraction, runtime-import policy and all four embedded frontend asset identities pass.

The exact extracted R7 executable launched from a fresh Pytxo home and fresh WebView2 profile, stayed responsive and closed through its main window. At Windows 125 percent scaling/system DPI 120, the DPI-aware 1938×1038 capture shows fresh onboarding without inherited workspaces and with `Review every change` plus the full `Get started` action visible. Candidate receipt: `D:/pytxo-beta-lab/cockpit-beta-candidate-r7-20260921/candidate-receipt.json`, SHA256 `790A65382C183053F1E0721CD20CD30BD6AD491E1016699F9E7841C4B47E6E15`.

The earlier public installed-updater handoff is no longer at a visible secure prompt. The installed executable remains v0.9.0; no `consent.exe` or interactive-session `msiexec` is visible, so elevation, upgrade completion, retained data and relaunch remain unconfirmed. Current observation receipt: `D:/pytxo-beta-lab/installed-updater-r6-20260921/updater-handoff-current-observation.json`, SHA256 `711EFAC0D8BFA69E1DBA464CA6B1DE498D984F547158A583C7AD126FC078D086`. This does not distinguish cancellation, installer failure or an elevation handoff that was never accepted.

R7's two-stage updater is controller/browser/package verified, not exact-native ready-to-install verified: v1.2.2 needs a genuinely newer signed update to exercise that state in the packaged app. Clean/elevated installation, retained-data upgrade, native 100/150/200 percent scaling, a real coding-agent Work → Review → Apply mission, stable updater mirror, release publication and final authentic launch footage remain open. No R7 install, dispatch, Apply, commit, push, deployment or publication occurred. Goal remains active.

## R8 Work focus panel, updater hierarchy and exact package — September 21

R8 supersedes R7 as the current local package candidate. The redundant Candidate → Checks → Review → Repository rail remains removed, `Review changes` remains the single Work entry into exact-byte review, and Apply remains confined to Run Review. Selecting a recorded worker now opens the lightweight summary as a right overlay on wide Work layouts, so the canvas keeps its geometry instead of relaying out every node. A separately opened output dock remains usable below the summary; compact and short layouts continue using the shared tabbed inspection surface.

The final first-selection trace initially exposed an intermittent 53–64 ms browser task. The repair keeps canvas transforms on one composited scene, holds topology stable across ordinary dock-width changes, defers hidden inspection trees, uses the native animation API for bounded evidence motion, and removes the worker-summary reflow. The exact-final functional suite passes 358/358 serially in 9.0 minutes. The strict dense-canvas performance case passes once in the final release run and then 10/10 in a repeated final-source stress run; every run rejects browser main-thread tasks above 50 ms. Svelte reports 0 errors and 0 warnings, CSS lint passes, focused updater tests pass 13/13, and focused native motion tests pass 3/3.

R8 also gives each updater state one primary next action. `Check for updates` disappears after a concrete version or action exists; `Download update` owns preparation and `Install and restart` owns the Windows handoff. Error treatment now uses the semantic refuted color while quiet manual recovery stays available. Verified update bytes remain protected across restart-required and retry states. Rust, orchestration, permission and Apply semantics are unchanged.

Exact local candidate: `D:/pytxo-beta-lab/cockpit-beta-candidate-r8-20260921`. MSI SHA256 `D7CA3875D477525C264070B75C8AD4BD032323425917A89545C081215701A58F`; administratively extracted EXE SHA256 `0811CC5BDF8443F5D735BB29EB6E0A9E3F1B131E0EF534263822455F50DAF073`. Administrative extraction, runtime-import policy and all four embedded frontend asset identities pass. Artifact manifest SHA256 is `929D72A2A9E44B9CB99B06575F052F3B998A8272202AE5032065745D5B7B9E28`.

The exact extracted R8 executable launched from fresh Pytxo and WebView2 profiles, stayed responsive and closed normally. At Windows 125 percent scaling/system DPI 120, the 1618×1010 window-only capture shows the complete fresh welcome card, `Review every change`, and the fixed `Get started` action. The 10.045-second settled sample measured the main process at 32.93 MB working set and 0.47 percent of one core; the seven-process WebView tree measured 397.9 MB and 1.56 percent of one core, which is recorded separately from the prior main-process-only baseline. Candidate receipt: `D:/pytxo-beta-lab/cockpit-beta-candidate-r8-20260921/candidate-receipt.json`, SHA256 `30E5E2BD19AC59D0DF1A096A85056F88062A7001D44C210E326AE95DD24450F6`.

The R8 MSI and executable are unsigned, so the installed v0.9.0 application was preserved. Clean/elevated installation, retained-data upgrade, exact-native signed-update handoff, Windows 100/150/200 percent scaling beyond the verified 125 percent host, a current-candidate coding-agent Work → Review → Apply mission, hosted release gates and authentic launch footage remain open. No R8 install, dispatch, Apply, commit, push, deployment or publication occurred. Goal remains active.

## Experimental Jev Routing account bridge — September 28

Jev Routing remains an opt-in, advisory local candidate under Orbit in one repository domain. Core owns eligibility, permission, budget, candidate identity, verification and Apply; paid Jev sends and Live execution are off. The website/Link/Desktop one-use PKCE account return is implemented behind separate default-off experiment flags. Link's disposable PostgreSQL suite passes 48/48; Desktop Rust passes 49/49, including expired/malformed credential cleanup; strict Link/Desktop Clippy, Svelte/CSS, web helper/TypeScript and the active Settings browser check pass. The last browser capture shows Connect disabled by default. The installed/deployed cross-app return has not been run.

The exact-source local Routing candidate is `D:/pytxo-beta-lab/jev-routing-account-bridge-candidate-20260928`. MSI SHA-256 `29AC90E6F96444845DAF747E5A8A83BDD1EBD3434731BCD11E14AF091D0A2970`; loose EXE SHA-256 `AB3F0770334455E503C8C9D299E5D2628A813519E24163C187D5CA27EE8E1B45`. Four embedded frontend asset identities passed. The extracted MSI executable passed empty-host refusal and opened/closed a native window from an isolated home. Receipt SHA-256 `681D157D66B2B2A85BF1584779AEB78951DD1A25D456E5A0FB911CE5956170CD`. Both artifacts are unsigned. The quarantined helper and Cargo files remain quarantined; the portable toolchain and embedded host provided local verification without changing security settings. The offline benchmark checks now pass 32/32 including a genuine Store rules trace through the recorder, while the Jev arm remains synthetic and gives no efficacy claim.

This candidate is not a Jev beta release. A real prompt-bearing everyday/strong execution pair, routed Work → Review → Apply, deployed hosted recipient with workspace opt-in and browser return, held-out Jev-versus-rules evidence, signed install/upgrade and approved publication remain open. The available Codex child ran read-only under policy, so no real editing profile was qualified. No paid Jev inference, production Live routing, subscription/API billing switch, install, commit, push, deployment or publication occurred.

## Jev Routing local candidate and hosted disclosure preview — September 28

Catalog v7 now issues a durable opaque hosted workspace ID only when an active domain and immutable pinned consent Store path/file identity match guarded caller evidence. It has no production caller yet. Desktop Work/History resolve the current routed retry by exact agent/task/run/domain and revision, preserving the prior attempts. A current persisted one-task Orbit Shadow review also exposes a read-only **proposed hosted packet** beside the exact local no-network fixture packet. It names the hosted recipient and scope but does not grant consent, mint an ID, contact Link, issue a token or send. A shared fixture links the staged Flow packet to the proxy's real wire validator. Independent read-only review found no concrete P1/P2 in the canvas, Catalog or hosted-preview slices.

Final-source checks: Store 214 tests, routed Flow seam 50/50, Orchestrate library 81 passed with one ignored account-dependent test, Desktop library 49/49, proxy 17/17, offline benchmark plumbing 32/32 with zero skips, Desktop release browser 384/384 plus performance 1/1, focused routing browser 21/21, Svelte/CSS zero diagnostics, strict Orchestrate/Desktop/Proxy Clippy and scoped formatting/diff checks passed. The Desktop browser proposal was visually inspected. This does not prove a real Jev advantage; the benchmark's RJ arm is synthetic.

The latest **local package candidate** is `D:/pytxo-beta-lab/jev-routing-hosted-preview-candidate-20260928-01`: MSI SHA-256 `F976E572ECB9E57EC77796BBDFB559CC9A5A48C77EB4D85810BBF83DAEA94B00`, extracted EXE SHA-256 `9ABE0D287E1F37B22D94F7112D550F9B42160543B7E917B0BD739D1C189A5C8F`, receipt SHA-256 `0FE5AF90DE1688E0B61C817B9A018F213267B1899012975EE032EAF1E93D1C8E`. The final MSI build verified four embedded assets; MSI integrity/extraction, empty embedded-host refusal (125), and isolated native Desktop launch/close passed. Both artifacts are unsigned and uninstalled. Earlier Jev local candidates are superseded as exact-source packages.

**Not beta-ready:** hosted recipient-specific review, consent/revocation and guarded Shadow send are absent; no real prompt-bearing everyday/strong pair or routed Work → Review → Apply has been qualified; no installed/deployed browser return, authorized paid Jev trial, held-out paired result, signed install/update or publication exists. Production Live and paid Jev remain off. The quarantined files and security settings were left unchanged.

## September 28 — hosted Shadow execution boundary audit

The read-only proposed packet is compatible with the proxy's packet validator,
but it is not executable hosted authority. Current saved Shadow policy, local
consent and one-send journal are fixture-specific; the controller rejects a
hosted advisor. Its `jev:<digest>` request IDs cannot pass Link's time-bound
UUIDv7 admission. An executable hosted path therefore needs a distinct
recipient-bound reviewed mission; recipient-aware local consent and durable
request identity; a private Desktop-to-Link/Proxy client; and local plus Link
revoke/status reconciliation with race and crash tests before any grant, token
or send. Link checks grant/token at claim, but a claim committed before revoke
can still send afterward. Disclose such work as in flight rather than promise
zero post-click bytes. No hosted state was enabled.

Current TypeSafe model/API docs still show `jev-1.13.0` at $0.042/M input
tokens and no documented maximum billable amount per admitted request. Its
September 23 customer agreement says requests *may* be declined after credits
run out without automatic refills; it does not establish a guaranteed hard
external spend cap. The service spec now records this evidence and retains the
paid-send gate. Source links: https://docs.typesafe.ai/models,
https://docs.typesafe.ai/api, https://typesafe.ai/legal/mca. No API call,
credit purchase, billing-mode change, security change or release occurred.

## September 28 — hosted recipient identity foundation, still no network send

Store v15 adds a separate recipient-scoped hosted consent table and nullable
recipient/scope fields on the one-send journal. Legacy rows remain NULL and
readable/settleable; they cannot become hosted requests. Core's optional policy
recipient is omitted for old missions, preserving their serialized bytes and
digests. Hosted preparation requires the fixed recipient, exact reviewed scope,
current UUIDv7, one-use journal slot and Shadow mode; the final mark rechecks
scope, consent, task state and mode. The local no-network fixture remains on its
old path. An independent read-only review found no remaining high-priority
issue in this local foundation after the Shadow gate and migration test fixes.

Evidence: v14→v15 migration tests load a real frozen staged review, check its
canonical digest and preserve requests in all five journal phases; rollback and
false-v15-stamp tests pass. Core and Store suites, routed Flow seam (50/50),
orchestration lib (81 passed, one native-account test ignored), Desktop Rust lib
(49/49), formatting and strict scoped Clippy passed before the latest focused
Shadow/cross-scope test additions; their focused tests also passed. A final
combined rerun and package are still required. No hosted Flow review, grant
action, request client, Link reconciliation, real paid send or Live activation
was added. Store's caller-supplied packet digest is not network authority; the
future trusted client must derive exact bytes from the persisted review.

During the edit, C: reached zero free bytes and `migrate.rs` was truncated by
a failed patch. It was restored byte-for-byte from local Git blob
`96cfdfcb6b26b3bb8fae1c55baacf890ed20ae16` before subsequent changes.
Forty-four old build artifacts were moved intact from C: Temp to
`D:/pytxo-beta-lab/preserved-c-temp-artifacts-20260928`; no source reset,
quarantine restoration or security-setting change was made.

## September 28 — hosted Shadow consent receipt and exact local candidate

The experimental hosted-recipient review is now distinct from the local no-network fixture review. It binds the fixed hosted recipient, exact redacted packet and disclosure scope to the persisted one-task Orbit Shadow mission. Preview and enable recheck the physical Store identity and the current reviewed bytes. Catalog schema v8 retains a monotonic hosted-review receipt and latest enable/revoke fence, so a restored older consent row in the same SQLite file cannot become a current grant. The local consent operation never creates a Link grant, token or network request; dispatch still rejects the hosted review because no guarded hosted send controller exists. Core remains the authority for eligibility, spending, permission, candidate verification and Apply.

The full routed Flow seam passed 60/60, including 11 hosted consent/review tests. Store's full package suite passed, as did Proxy 17/17, Desktop Rust 49/49 and Link 48/48 in the attempted workspace run, Svelte/CSS checks with zero diagnostics, strict workspace all-target Clippy, and the offline benchmark plumbing 32/32 with its existing compiled Store-export bridge. The Runner library passed 94/94 serially and again with normal test parallelism after two deadline tests were made tolerant of loaded Windows startup while retaining long-child timeout assertions. The first workspace-wide test run stopped at a Runner timing assertion under concurrent load before those test-only repairs; a final full workspace run was not completed. No Jev efficacy or real worker qualification is inferred from these tests.

Exact post-change local candidate: `D:/pytxo-beta-lab/jev-routing-hosted-shadow-candidate-20260928-03`. MSI SHA-256 `ACD855EFBD98FE23AEF2DAC6E24B9DB9A2459D2D67746F911E539C480B3A8C92`; extracted executable SHA-256 `53FABE2B8921700CEA9287998ED7B900BCEBB9C4EEF91CEA2E93F7ECC24F3A26`; receipt SHA-256 `FB4A7D31D79CB3D39B4320476707C5E5CF73024F6DC86B267A79FE8C0DD81F5F`. The MSI build verified four embedded frontend assets. Non-installing MSI integrity/extraction passed; its embedded attempt host refused an empty request with exit 125; its Desktop window remained responsive after eight seconds in an isolated `PYTXO_HOME` and closed normally. MSI and EXE are unsigned and uninstalled. The prior v15 local package is superseded as an exact-source candidate.

This remains a local Shadow foundation, not a Jev Routing beta. A real prompt-bearing everyday/strong pair in the routed Work → Review → Apply path, guarded hosted send with Link grant/revocation reconciliation, a held-out Jev-versus-rules result, signed install/upgrade and approved publication remain open. The user's quarantined files remain quarantined; no security setting, paid Jev inference, Live routing, subscription/API billing mode, source commit, push or deployment changed.

## September 28 — real Claude subscription owned-host qualification probe

A Windows-only, explicitly ignored Runner integration probe now launches the installed native Claude CLI through the exact MSI-extracted embedded attempt host with a cleared, allowlisted child environment. Its read-only auth status reported `claude.ai` subscription mode under an owned Job. Separate explicitly invoked Haiku and Sonnet probes each edited only `result.txt` in a disposable Git repository using private stdin, Read/Edit tools and no API-key environment; both exited successfully, recorded registration and Job-zero settlement, and preserved an unrelated starter file. These are real subscription CLI/provider calls on disposable data, not paid Jev inference or an API billing-mode switch.

This proves a usable *native probe shape* for two requested Claude aliases on the current machine. It does not yet bind two reviewed `ExecutionProfile`/account records, establish reported immutable model identities or quota/cost, prove cancellation for these exact CLI invocations, or run through routed admission, frozen checks, Review and Apply. The installed CLI and account can change, so readiness must be fresh and exact at admission. The probe is ignored by routine CI because it requires a specific native account and live provider access. No production routing was enabled.

## September 28 — narrowed Claude subscription template and native Stop proof

The experimental Orchestrate adapter now constructs an inert, exact Claude subscription launch from reviewed `ExecutionProfile`, binding, selected account and one claimed file. It filters the child environment to the selected subscription home and Windows root, passes the prompt through private stdin, pins the native CLI and embedded host, requests only Haiku or Sonnet, and keeps model identity at **requested** and metering **unknown**. A separate owned auth observation executes and inspects its own receipt in one trusted call; a caller cannot pair a receipt from one account with a spec for another. The routed worker's native-create gate reconstructs the selected command and exact claim from the reviewed task, rechecks executable pins, account/worktree overlap and worktree reparse points. This is Orbit in one execution domain; Core still owns permissions, budget, verification, candidate identity and Apply.

An initial broader Claude tool permission wrote a file outside a disposable worktree, so that command was discarded. The current command uses Claude's restricted mode with Read and a single path-scoped Edit permission. The explicitly invoked **production-builder** native tests for Haiku and Sonnet each edited only their disposable claimed `result.txt`, preserved an unrelated file, and recorded successful owned registration and Job-zero settlement. Separate native Stop probes for both aliases returned Cancelled with Job zero and left the claimed file unchanged. The selected installed `claude.exe` pin was SHA-256 `9DBE16DAFED59DA5CDABBFE11AD0335738C753FAD794989B47F9446ACCD6DE3A`; the MSI-extracted Desktop executable serving as embedded host was SHA-256 `53FABE2B8921700CEA9287998ED7B900BCEBB9C4EEF91CEA2E93F7ECC24F3A26`. The subscription auth observation is point-in-time, not a future invoice guarantee.

Focused adapter/native-gate tests, strict Orchestrate Clippy and the full Orchestrate package suite passed (library: 89 passed, 3 account-dependent probes ignored in the ordinary run; routed Flow seam: 60/60; native fixture: 7/7; Review/Apply and Stop integrations passed). The new ignored Stop probes were then run explicitly for both aliases and passed. The linked-outside Edit probe remains an explicit **unmet beta gate**: the selected CLI has not produced a denied-Edit receipt under the exact launch. The worktree link scan is a pre-create check, not an atomic Windows filesystem sandbox against another process changing a path after inspection.

This is **not beta-ready**. Claude has no production admission call site; the routed Flow still dispatches only the synthetic fixture and hosted reviews still fail closed before network send. No real routed one-task Work → frozen check → Review → Apply journey, held-out Jev-versus-rules advantage, deployed Link grant/revocation flow, signed install/upgrade or approved publication has been demonstrated. The quarantined bootstrap/helper/Cargo files and security settings remain unchanged; portable Rust and the previously extracted embedded host are the local test workaround. Paid Jev inference and Live routing remain disabled, and no billing mode, commit, push, deployment or release changed.

The current-source debug Desktop binary also built through portable Rust and its embedded host returned the expected 125 for an empty request. Its SHA-256 is `0F68D5200E3FE0EAA80DEE420D4214F88501ABE221035034550F48606F7E4D13`. Repeating the exact production-builder edit probes against this freshly built host passed for both Haiku and Sonnet. For both aliases, an unclaimed sibling file inside the disposable worktree was readable but its Edit was explicitly denied and its bytes stayed unchanged. A separate absolute-outside probe left its file unchanged yet recorded only a denied **Read**, not a denied Edit; the strict denied-Edit test correctly failed. This remains an unproven outside-Edit boundary, not a green beta qualification. The full Orchestrate package suite, strict Clippy, formatting and Desktop Svelte/CSS checks passed; the current-source Desktop debug build passed. No new installer or signed artifact was produced.

## September 28 — Claude v2 launch, honest usage, and one-task no-create recovery

The exact Claude subscription launch now includes `--disallowedTools mcp__*` in addition to restricted mode and strict MCP configuration. Adapter/tool-bundle identities advanced to v2 so older qualifications cannot authorize this command. The native gate reconstructs the complete argument vector and extracts the single claimed Edit path without a fixed argument index. The installed CLI and current-source embedded host passed fresh v2 disposable Haiku and Sonnet claimed-Edit probes and separate Stop/Job-zero probes; Haiku also recorded a denied unclaimed-sibling Edit. The direct-outside-file probe left the outside bytes unchanged but recorded **no denied Edit**, so its strict test failed again. The CLI controls are not an OS filesystem boundary and the outside-Edit beta gate remains open. No fake-MCP native denial probe was run.

The attempt supervisor now records launched Claude/subscription usage as `Unknown` rather than the local fixture's known zero, and labels normal/fallback agent ledger rows from the selected durable harness. A trusted, never-created Claude attempt may record known zero only when its retained no-create receipt, cancellation event, physical Store and Catalog owner, and closed capacity release all match. Dead-controller reconciliation now handles exactly one prepared task as well as the original two independent siblings; it never resumes the worker. An injected one-task crash test passed, including replay without a second event. This crash test uses a local fixture; a Store-backed Claude/subscription no-create test is still needed before relying on provider recovery.

Final-source checks for this slice: Orchestrate library 91 passed / 6 intentionally ignored; routed Flow seam 60/60; feature-enabled native routing fixture 32 passed / 1 intentionally ignored (23.2 minutes, including the new crash/replay case and existing sibling/DAG/Review→Apply cases); strict Orchestrate all-target Clippy with test faults, formatting, and scoped diff check passed. The direct-outside native qualification test failed as described above and is not counted as a passing beta gate. The one-task real Claude route is still absent from Flow/Desktop dispatch, hosted Jev send still fails closed, paid Jev and production Live routing remain disabled, and no signed install/upgrade or held-out Jev-versus-rules result exists. Quarantined files and security settings were unchanged; portable Rust and the existing embedded host remain the local workaround. No source commit, push, deployment or release occurred.

## September 28 — reviewed Claude route shape before any native admission

The experimental reviewed Flow now rejects a Claude-intended mission unless it is exactly one Orbit/worktree/subprocess task with one plain claimed file, one worker, no dependency, a shared Claude subscription account pool, Haiku as everyday and Sonnet as strong, v2 adapter/tool-bundle identities, requested-model evidence, one allowed attempt, and Rules or local Shadow only. The task and mission explicitly name the Claude subscription endpoint as reviewed egress; that policy identity is **not** an OS socket fence. The spend guarantee is explicitly risk-bounded, never a hard dollar cap. Review applies the shape before private staging and dispatch recomputes it from the persisted mission. A valid pair stages and reloads, but unsupported dispatch durably cancels the run with no attempts or native launch. Legacy Flow and the local fixture executor remain unchanged.

A further disposable native probe requested `Edit` on `../outside.txt` through the exact Claude v2 launch. The file remained unchanged, but Claude recorded a denied `Read`, not a denied `Edit`; the strict outside-Edit test still failed. Do not treat this as native write-boundary qualification. Final-source Orchestrate library: 93 passed / 6 account-dependent probes ignored; routed Flow seam: 61/61; strict all-target Orchestrate Clippy with fault tests, formatting and scoped diff checks passed. The earlier 32-case native fixture run predates only the route-shape validator and probe wording; it was not repeated. A Store-backed Claude/Subscription no-create recovery test, trusted owned qualification, real routed Work → Review → Apply, hosted recipient send/revocation, held-out Jev efficacy, signed install/upgrade and approved publication remain open. Quarantined files and security settings stayed unchanged; paid Jev and production Live routing stayed off. No commit, push, deployment or release occurred.

## September 28 — packaged, explicitly gated Claude proposal route

The experimental Windows Desktop route now constructs a trusted one-task Claude subscription Rules mission at preview, offers Haiku/Sonnet as the everyday/strong pair, and dispatches from the persisted reviewed plan. Its explicit UI choice does not change the ordinary Codex default. The production builder freezes one claimed file and one check under Orbit/worktree/subprocess with one worker and one durable attempt, an existing one-slot account capacity pool, a one-hour review deadline, requested model identity and unknown subscription charge. Dispatch requalifies the account and both profiles through the embedded owned host. The v3 no-tools worker returns a JSON file proposal; Pytxo alone writes the isolated candidate, verifies it and prepares existing Review → Apply. This replaces the earlier v2 direct-Edit probe path for this narrow route. Hosted Jev, Live advice and paid inference remain disabled.

The exact MSI-extracted host passed the explicitly invoked real Claude subscription journey through one passed attempt, frozen check, exact candidate and Apply. The ordinary full Rust workspace suite passed after a test-only Windows crash-controller setup wait was raised from 20 to 60 seconds for the larger embedded host; the crash/cleanup assertions were unchanged. Strict workspace all-target Clippy, formatting, Svelte/CSS checks, benchmark recorder 32/32 including the compiled Store bridge, and 11 Desktop browser workflow tests passed. The MSI build verified four embedded assets. Administrative extraction, runtime-import inspection, extracted asset check, empty embedded-host refusal (125), and isolated native launch/respond/close passed. The quarantined bootstrap/helper/Cargo binaries and security settings were unchanged; portable Rust and the Desktop embedded host supplied the workaround.

Local candidate: `D:/pytxo-beta-lab/jev-routing-claude-proposal-candidate-20260928-01`. MSI SHA-256 `74C8CA9E76D4B51CD59EE6D1557297560C189F398117B0BAF86BD432B604C258`; MSI-extracted EXE SHA-256 `2A42693A8DF5C75386F78AA93F25F63DED4C9719B7339E35604AAE8B86AC141A`; candidate receipt SHA-256 `58E215A064FCBBCEDA8E9DEAC13487FB70BC3EEE1ED47B9C126EA55265B239EB`. Tauri's MSI bundle patch changes the executable hash from the loose release build. Both files are unsigned; the MSI was extracted, not installed. The existing dirty repository state was preserved. No source commit, push, deployment or release occurred.

This is a local experimental Rules-route candidate, not a Jev Routing beta release. The hosted send controller and Link grant/revocation reconciliation are still absent, and no funded, frozen held-out Jev-versus-rules result exists. Signed install/upgrade and approved publication are also open. Local account readiness is point-in-time; CLI/provider behavior, subscription limits and requested model identity remain external uncertainties. Do not enable paid Jev inference or production Live routing on the strength of this package.

Final read-only review found no Core-authority or billing-mode bypass but did identify a Desktop beta control gap: routed dispatch completes its model probes and worker synchronously, leaving New work at **Starting** without a Stop control during that interval. A cosmetic Stop button would be misleading because qualification happens before the durable Run claim. The experiment remains gated; broader beta needs a durable preflight cancellation intent and early Run navigation/status before this can be treated as interactive Work. The browser-preview Claude fixture also displayed copy-on-write while the real route uses a worktree; its displayed isolation was corrected and the browser route test and Svelte/CSS check passed again.

That browser-only source correction supersedes the `-01` package as the latest exact-source artifact. Retained final local candidate: `D:/pytxo-beta-lab/jev-routing-claude-proposal-candidate-20260928-02`. MSI SHA-256 `1DD86BF0294E28093E5A994F8FB5009E16A94A3B9B9D25361B3560E26617C5DF`; MSI-extracted EXE SHA-256 `96AB652A87B4D877EC4852A7D6D40D5AA48ED130CE120F7DB01B03F5E453FD52`; receipt SHA-256 `CB53DBD60300BB85D11AB5C06D9B90C1ACBD7D92AB6355EF3B062EFDA502E2C2`. Exact `-02` MSI extraction/runtime-import, embedded asset, empty-host and isolated native-window checks passed. All selected source hashes match `-01` except the browser-preview fixture; the real Claude subscription Review → Apply test passed on the `-01` extracted host and was not repeated on `-02`. Both MSI and EXE are unsigned and uninstalled. No quarantined file, security setting, billing mode, paid Jev gate, Live routing, commit, push, deployment or release changed.

## September 28 — durable routed Stop through preflight, active worker and recovery

The experimental routed Flow now accepts an exact reviewed Stop while dispatch is pending. Catalog v9 records a monotonic Stop bit before touching a worker. The controller checks it around owned qualification, reservation and registration, and the Desktop keeps dispatch off the async executor so Stop can run concurrently. A preflight Stop settles without a Run, attempt or capacity owner. After reservation, Stop uses the immutable original Store locator and physical file identity from the dispatch claim, so an edited data_dir cannot retarget termination. Registered workers are stopped by their exact owned Job; unresolved ownership remains recovery_required until the controller or recovery proves quiescence. The UI says “Stop requested” until that proof exists and exposes Stop again from saved dispatching history.

The terminal acknowledgement now respects the proven Store outcome. A Stop that kills a registered worker leaves the Run and Flow cancelled and dispatch reports cancellation. If the Run completed before a late Stop request, verified terminal reconciliation keeps it completed/dispatched. Dead-controller recovery after a passed parent wave treats a durable exact Stop as cancellation and never admits its child. This is Orbit in one execution domain; Core still controls admission, verification, candidate identity and Apply.

Focused final-source evidence: full Store package 54 + 25 + 141 + 2 passed; routed Flow seam 64/64 passed; fault-enabled native Stop group 7/7 passed (including preflight, post-reservation, config drift, active Job termination, between-wave Stop and late terminal race); the dead-controller-after-parent durable Stop recovery test passed. Strict workspace all-target Clippy and fault-enabled Orchestrate all-target Clippy passed, as did formatting and scoped diff checks. Desktop Svelte/CSS checks and 24 routing browser tests passed before the final Store acknowledgement changes; those changes did not touch the frontend. A prior full workspace run had one candidate_run error-message assertion fail under load; its isolated rerun passed. The broad suite is not claimed green on the final source.

The quarantined bootstrap/helper/Cargo binaries and Bitdefender settings remain unchanged. Portable Rust and the existing Desktop embedded attempt host provided local test execution. The retained -02 MSI predates these source changes, so it is no longer an exact-source candidate. Hosted Jev send/grant reconciliation, a real held-out Jev-versus-rules result, a newly packaged and signed installed candidate, retained-data upgrade and release approval remain open. Paid Jev inference and production Live routing stay disabled; no billing mode, commit, push, deployment or release changed.

## September 28 — hosted Shadow review can be staged, never sent

A separate default-off PYTXO_EXPERIMENTAL_ROUTED_HOSTED_SHADOW_REVIEW gate now lets a trusted Rust caller stage the exact one-task Claude subscription hosted-Shadow review. The builder fixes Haiku/Sonnet profiles and account binding, the hosted recipient, current packet/template/scope identity, and the next local hosted-consent revision. It does not accept a serialized UI policy. The ordinary Claude Rules preview is unchanged. A hosted review can be inspected from its persisted packet, but dispatch rejects it before Claude qualification, a run claim, network send or paid inference. Desktop has no hosted review action yet.

The Windows review integration test passes: gate off rejects; gate on stages the hosted mission, preserves Rules behavior, yields an exact reviewed hosted packet with consent revision 1, and refuses dispatch while leaving the draft ready and Run absent. The full routed Flow seam passes 64/64 and strict workspace all-target Clippy passes after this change. Local consent still lacks an account binding and Link grant/revocation reconciliation; do not expose or send this route until those fences and a guarded client are implemented and tested. The proxy paid-send startup gate remains closed.

## September 28 — exact local Stop candidate retained

The final-source Windows release MSI built with portable Rust; the build verified four embedded frontend identities. Local candidate: D:/pytxo-beta-lab/jev-routing-stop-candidate-20260928-03. MSI SHA-256 4E74F424473833B6F963B678A3B0F439E59705FD5D184ABC289E5CFD15B62ED1; MSI-extracted Desktop executable SHA-256 884C1451B8CA2074E1D8CC34EB48B04D186C38B56DF68286DF811CC0293B69FB; loose release executable SHA-256 770705F1BDC422FBCC4EE3E3EB317D2F4292667CA065E5EB2C6F69FCE09B32F2. Candidate receipt SHA-256 472FDC355046E22A160D01723042628C30C6D172555F6D3EB1D7D9E946C58C8D, with selected source-input hashes and verification limits.

Administrative extraction and all packaged PE runtime-import checks passed; four current JS/CSS asset identities matched the extracted executable. The embedded host refused an empty request with exit 125 and no output. The exact MSI-extracted host passed the fault-enabled active-worker Stop/config-drift test: one owned process was killed; the Store run, Flow and routing mission settled cancelled with no owned Job left. The MSI is unsigned and uninstalled. No exact-package GUI Work → Review → Apply or signed update was claimed. The current source has no hosted send; paid Jev and Live routing remain off. Quarantined files and Bitdefender settings were unchanged.

## September 28 — hosted Shadow is explicitly review-only; exact local candidate -04

The trusted hosted-recipient Claude preview now persists `review_only`, with an explicit warning and dispatch error. It remains inspectable for the exact coarse packet and local consent receipt, while the ordinary Claude Rules preview stays runnable. The hosted integration test checks that the packet omits the task text, claimed path, check command and account path, and that recipient/scope/request identities stay pinned. Desktop understands the review-only state and never offers Run for it. No hosted network send or paid Jev inference was added.

An independent read-only review found that the current one-file Claude builder fixes all coarse packet categories, so every accepted task has the same Jev packet. The benchmark protocol now says this path cannot establish routing efficacy; a varied, locally reviewed feature source is required before any funded Jev comparison. Core still owns permissions, budgets, mission/attempt state, verification, candidate identity and Apply.

The initial broad Windows run used the MSI-extracted GUI executable as a mixed PTY/subprocess test helper. Its PTY stdout test failed because that helper is not a console host. A current-source Pytxo console CLI host passed both transports. The cross-process Stop test also exposed a one-second pre-Stop fixture race under concurrent load. The fixture now holds its delayed write behind a file gate, verifies the worker is waiting, requests Stop and Job zero, then opens the gate and asserts no later effect. The Runner group passed 19/19 plus two intentionally ignored tests serially, and the final **full Rust workspace suite passed** under normal concurrency with the console test host. Strict workspace all-target Clippy, formatting, scoped diff checks and Desktop Svelte/CSS checks passed. Routing browser tests passed 25/25. Offline benchmark contract tests passed 31/31 with one optional compiled Store-export bridge skipped in this run; these use synthetic data, not Jev observations.

Final-source unsigned MSI and local receipt: `D:/pytxo-beta-lab/jev-routing-review-only-candidate-20260928-04`. MSI SHA-256 `127D6E9E79E534188617382E28AF848480DC2FC71A4B9A2CD1598F7688E1313A`; administratively extracted executable SHA-256 `D29EC061B286131E2986554DD9312F3C0253E3D12CDF5D313D025A770D72ECCA`; loose release executable SHA-256 `BA72C04FD55B5DDDF61CB615F7AF716D8D14037D91B5A1910CF743C490E428AF`; receipt SHA-256 `336DC1435DAE54348AE61EF556FAA19B142A26B75BCD9C2CDE4EC65AB2C65E27`. Selected source hashes in the receipt were rechecked with zero mismatches. MSI extraction and PE runtime-import policy passed; four embedded frontend assets matched; the extracted host refused an empty request with exit 125 and no output. Using that exact extracted host, the local Claude stub completed a routed one-task Review → Apply journey, and the active-worker Stop/config-drift test killed one owned process and settled cancelled. These tests do not prove current provider behavior or a native GUI journey.

This is a verified **local experimental candidate**, not a public or Jev Routing beta. The hosted account-bound grant/revoke/send controller, a task-discriminating redacted packet, held-out Jev-versus-rules result, exact-package real-provider/native GUI acceptance, signed installation and retained-data update, and approved publication remain open. The quarantined bootstrap/helper/Cargo files and Bitdefender settings remain unchanged; portable Rust plus the console CLI and MSI-extracted Desktop host are the separate local test workarounds. Paid Jev and production Live routing remain disabled. No billing-mode change, commit, push, deployment or release occurred.

Final read-only review found no new hosted dispatch or consent authority bypass. It also found that the existing Codex transport fixture starts its ten-second Stop-settlement timer **after** the synchronous `stop_run` call. That fixture proves eventual quiescence but does not bound the Stop call's own latency; this remains a beta responsiveness check. The packaged active-worker test proves owned-process termination and durable cancellation, not a native UI latency bound.

## September 28 — bounded Stop probe and exact local candidate -05

The Windows Codex transport fixture now measures the synchronous stop_run call through a blocking task with a ten-second deadline, then separately bounds worker settlement to ten seconds. The focused PTY and Subprocess Stop test passed (about 2.3 and 2.0 seconds per transport); strict Orchestrate all-target Clippy, formatting and diff checks passed. This closes the prior test-timing gap, but it is not a native GUI Stop latency measurement. The previous full workspace green run predates only this test assertion; runtime source behavior is unchanged.

The rebuilt local candidate is D:/pytxo-beta-lab/jev-routing-stop-bounded-candidate-20260928-05. MSI SHA-256 00EA75C282E950094D964C3B48F922EA7A31D7FB38E98B5343750309E364D975; extracted executable SHA-256 4142ECFB6C54A2CBA46EAB3F32153B1DC56B2D94982DBB33829FA79972130575; receipt SHA-256 56FA392FD34EC12030AE39A7B9A9BA7224C57F929950A6F60B631896BE1C444A. Its 14 selected source inputs and three artifacts rehash with zero mismatches. MSI administrative extraction, packaged runtime imports, four current embedded asset identities and empty-host refusal passed. The exact extracted executable passed a local-stub routed Review to Apply journey and the registered-worker Stop/config-path-drift test. The MSI is unsigned and uninstalled.

A read-only hosted-client review identified a future revoke hazard: Desktop currently removes the routing bridge credential even when Link revocation fails. Before any hosted grant/send client is exposed, local consent must be fenced first and a durable pending remote-revoke receipt or equivalent recovery must preserve reconciliation across disconnect/restart. Hosted grant/token/send remain absent and paid Jev and Live routing stay off. Quarantined files and security settings remain unchanged. This candidate is local experimental proof, not a beta release; real-provider/native GUI, signed install/upgrade, hosted account-bound reconciliation, task-discriminating packet and held-out Jev efficacy remain open.

## September 28 — hosted grant recovery and exact local candidate -06

The hosted Shadow path now lets Desktop inspect the exact saved packet, record local opt-in, and reconcile an account-bound workspace grant with Link. Local send authority is fenced before remote revocation; a missing original workspace Store can be fenced from Catalog for revoke-only recovery. The Link/Desktop account return has one-use PKCE codes and a narrow `routing:revoke:v1` scope. A pending grant can reconnect for cleanup even when Desktop and Link experiment switches change independently. Link's own default-off grant switch blocks new full-scope code issue **and exchange**, new grants and evaluation tokens while retaining reads, revocation and revoke-only account recovery. Desktop labels its account-wide credential DELETE explicitly and asks before it revokes all Routing access. These are Orbit controls in one repository execution domain. Core remains the authority for permission, spending, durable attempts, verification, candidate identity and Apply. The Proxy's paid-send startup gate is still hard closed and all hosted reviews remain `review_only`.

Current-source verification: the full Rust workspace suite passed before two final narrow Desktop/Link off-switch refinements; the affected Desktop (57 + 1) and Link (50) tests passed afterward. Strict workspace all-target Clippy and `cargo fmt --all --check` passed after those edits. Desktop Svelte/CSS reported zero errors/warnings, 23 routing browser checks passed, and the website production build, TypeScript, focused ESLint and three redirect-helper tests passed. The benchmark contract suite passed 32/32, including a real Store-exported **rules** trace crossing into the recorder; Jev-arm fixtures remain synthetic. Link's disposable PostgreSQL integration tests were not executed because no test database was configured. A read-only final review found no further cross-flag revocation or authority hole.

The exact-source unsigned local candidate is `D:/pytxo-beta-lab/jev-routing-hosted-recovery-candidate-20260928-06`. MSI SHA-256 `FDE6AB2480625C7E0F8BCEAA8DD8DC7DB6AAFB273BFD9C9980F6B68A3A0DD029`; administratively extracted EXE SHA-256 `9A07F66A82084B4B0F90F43F1D3FFF749F0ADC5352D260D6B0B8817AC75BFFC1`; loose EXE SHA-256 `0F47AF6C15A3FC26853F89D933B369B934B210A835569C081D92AFFC6E059543`. Receipt SHA-256 `0A449F4C7A177CDCDE6AFE947701F9499373C3B4BBAEDD51E287370EC0A8BACE` records 25 selected source inputs and limits. MSI extraction/runtime-import inspection and four embedded asset identities passed; its extracted host rejected an empty request with exit 125 and no output, passed an owned cancellation/settlement test on PTY and subprocess, and completed the disposable Claude-stub routed Review → Apply journey. The MSI is unsigned and uninstalled. No real provider, native GUI, deployed bridge, signed install/upgrade, funded Jev trial or publication was verified. The one-file hosted packet still lacks task-discriminating variation, so a Jev efficacy claim is impossible from this route. Quarantined files, security settings, billing modes, paid Jev and production Live routing were unchanged; no commit, push, deployment or release occurred.

## September 28 — reviewed demand facts and benchmark packet binding

The experimental one-file Claude proposal review now records caller-declared task kind, context completeness and cross-component need in its frozen mission. Absent facts default to other/incomplete/unknown, and editing the mission or restoring a saved draft clears stale labels. The regular Desktop route is unchanged. Core's evaluated, explicitly authorized Live override is further limited to complete, explicitly single-component diagnoses; unknown or cross-component work stays on the strong rules choice. Hosted reviews remain inspectable and `review_only`; the Proxy paid-send gate stays hard closed. Permission, budget, attempt, verification, candidate and Apply authority remain in Core for Orbit in one execution domain.

The offline benchmark freezer now requires a reviewed packet digest per case and binds it across both arms. The recorder derives the assignment-keyed packet alias used by the Store export and rejects a mismatched advisor journal row. The low-cardinality raw packet digest makes frozen schedules, studies and analysis bundles **private** despite their omission of task text. A real Store journal integration probe proves the Rust-to-Node alias match with synthetic advice; it makes no Jev call. This is consistency evidence only. The current eligible one-file diagnosis cohort still yields identical coarse packets, so this route cannot establish Jev efficacy. An independently reviewed, varying feature source and pre-registered held-out comparison remain required.

Verification on current source: the first full Rust workspace run reached Store with one old Live-test expectation (unknown kind previously applied); after updating the test to the new diagnosis gate, all 141 Store routing tests passed. The feature-enabled CLI trace-exporter suite passed 6/6. The three benchmark contract suites passed 33/33 including real Store rules and advisor-journal bridges. Strict workspace all-target Clippy, Rust formatting, Desktop Svelte/CSS checks and the four experimental Claude browser cases passed. The earlier `-06` unsigned MSI predates these source changes and is not an exact-source package. No native GUI/provider journey, deployed hosted send, signed install/upgrade, or funded Jev trial was run in this slice. Quarantined files and security settings stayed unchanged, and no billing mode, commit, push, deployment or release changed.

## September 28 — local package candidate -07 from reviewed-demand runtime source

The Windows release MSI built with the portable Rust toolchain and verified four current embedded frontend asset hashes. Retained unsigned local candidate: `D:/pytxo-beta-lab/jev-routing-demand-facts-candidate-20260928-07`. MSI SHA-256 `F2756959596FF951A806F665DBF6DEFD935E02B59AC4D43D75DC953F30B64BB0`; loose executable SHA-256 `D71EE31A88E4F163A4D99FACF3D95E78896252377F76DA033B9B03777ABDC0B8`; MSI-extracted executable SHA-256 `FD62D64FBEC0D847E530A0C7B45319BD3439089080558078C7D46A2A10ED0906`. The local receipt is `candidate-receipt.json`, SHA-256 `E6BBFE025F0B2EF15B90EEA6D1BDFB9D50FDF7E79829155CD27FD79182009553`.

Administrative extraction, packaged PE runtime imports, four extracted asset identities and embedded-host empty-input refusal (exit 125 with empty output) passed. The exact MSI-extracted host passed the disposable Claude-stub routed one-task Review → Apply journey after the test explicitly reviewed the local-transformation facts; the first run's old everyday-profile assertion failed because the new conservative default correctly selected strong. Only the test fixture changed after MSI build. Fault-enabled Orchestrate strict Clippy and Rust formatting passed after that change. This is not a real-provider or native GUI acceptance run. The MSI remains unsigned and uninstalled; no paid Jev inference, production Live route, billing switch, commit, push, deployment or publication occurred.

## September 29 — hosted Shadow recordability and exact local package -08

Core now exposes one predicate for whether a task can accept a recorded advisor answer: complete context, an explicit kind, and valid kind evidence. Store enforces it before hosted request preparation and again at the one-use send mark, including for an older prepared row. Flow includes this readiness in the reviewed hosted packet preview and refuses local hosted consent when it is false; Desktop also refuses a new account-bound hosted grant and explains the missing facts. No paid send or production Live route was enabled. The account, budget, permissions, attempts, verification, candidate identity and Apply boundaries remain Core/Store-owned for Orbit in one repository domain. A test-only fault-build diagnostic reports safe owned-auth receipt fields without output text.

The hosted Store regressions pass 4/4, including a second request ID for the same task ordinal and incomplete/unknown/missing-evidence cases; the focused Flow review test passes. The full Rust workspace suite passed using the current-source embedded console host and the existing 30-second fault-test timeout override. Its first pass with the default eight-second bound had three Runner fault-case timeouts; all ten affected fault cases passed with the 30-second bound, followed by the full workspace pass. Strict workspace and fault-enabled Orchestrate Clippy, Rust formatting, Desktop Svelte/CSS and 27 affected routing browser tests passed. An initial real-provider test pointed the selected account home at .claude and correctly observed logged-out status; the actual Windows user-profile home yielded the existing Claude Pro subscription. The disposable one-file real Claude Review to Apply journey passed on the current-source debug host and again against the exact MSI-extracted host. Both used test-only synthetic pre-admission qualification, so neither closes the native sandbox qualification gate.

The unsigned, uninstalled local candidate is D:/pytxo-beta-lab/jev-routing-recordability-candidate-20260929-08. MSI SHA-256 0498449F3B7A121A54DEE2E0F62DBFE0CE7977388642B0A6FF503D5FA6FFBD76; loose EXE SHA-256 EE2ED54F7116CC307C0D70D66F48F2ED296FC4A31FBB5524F5E955E0AD2FD2CF; MSI-extracted EXE SHA-256 1EAB425CD45D47D6C4661E41B1C05EC52C17DCA16838E513ED2665A99E0C1942. Non-installing extraction, all packaged PE runtime imports, four embedded asset identities and empty-input host refusal (exit 125, empty streams) passed. candidate-receipt.json SHA-256 DD87A26C0E562E0775D4CF15468BFD35E370746718D81DEDBBDCF5E7FEC9B4C9 rehashes 11 selected source inputs and three artifacts with zero mismatches.

This is a local experimental candidate, not a Jev Routing beta release. Hosted reviews are still review-only, Proxy paid send remains hard off, native outside-Edit qualification remains unproven, and there is no exact-package native GUI journey, signed install/update, deployed hosted grant/send, or held-out Jev-versus-rules result. The eligible one-file diagnosis packet still lacks task-discriminating variation. Bitdefender-quarantined files and security settings stayed unchanged; no billing-mode switch, paid Jev call, production Live route, commit, push, deployment or publication occurred.

## September 29 — hosted grant revoke race and local package -09

A local hosted-consent revoke now writes the Catalog disabled fence and any pending Link grant's `revoke_pending` state in one transaction. An enabled Link reply arriving after that fence triggers a remote cleanup attempt; failure leaves a durable pending obligation for same-account retry. Desktop hides the Link confirmation action for an older unrecordable review, labels its saved consent ineligible, and calls its task type explicitly declared. A restart test covers the Store-committed/Catalog-not-yet-fenced gap, and a local fake-Link test covers late enable, failed immediate cleanup, and retry to a disabled tombstone. The remaining gap is operational: a crash in that cross-database interval still needs replay before Catalog reflects the disabled Store row. Core/Store retain all routing and Apply authority for Orbit in one repository domain.

The full Rust workspace suite passed with the current-source debug host and the existing 30-second owned fault-test override; the two strengthened tests were compiled and passed separately afterward. Hosted Store tests passed 8/8. Strict workspace and fault-enabled Orchestrate Clippy, Rust formatting, Desktop Svelte/CSS, 27 routing browser tests and all 33 benchmark contracts passed. The exact MSI-extracted executable passed a disposable Claude-stub routed Review → Apply journey. An independent read-only review found the earlier grant race and ineligible UI dead-end resolved, with no new beta-blocking bypass in this fix.

The unsigned, uninstalled local candidate is `D:/pytxo-beta-lab/jev-routing-race-candidate-20260929-09`. MSI SHA-256 `3073069107701C6A444D3E5FD4AC121D54B837E6279C65FB3619C871ADE3AC18`; loose EXE SHA-256 `9FA776A99B9F6A132BB4C4787BBF155ABB18D2814E3A01428F011EBFEEA31DC3`; MSI-extracted EXE SHA-256 `CCA081CD0F4B0C40E8BD3F45CC72C66462759A5D0B2AEAD30BC190C8B991E048`. Non-installing MSI extraction, all packaged PE runtime imports, four current embedded frontend identities and empty-input host refusal (exit 125, empty streams) passed. `candidate-receipt.json` SHA-256 `860905898115767613B93B7CAA14A312AC380434B7028A9D1DB625F25E3CC053` rehashes 13 selected source inputs and three artifacts with zero mismatches. The earlier -08 package predates this fix.

This remains a local experimental Shadow candidate, not a Jev Routing beta release. Hosted dispatch is review-only and Proxy paid send remains hard off. The exact -09 package has local-stub execution proof, but no native GUI journey, signed install/update, deployed hosted bridge, strict outside-Edit denial, task-discriminating eligible packet, or funded held-out Jev-versus-rules result. Quarantined files and security settings were unchanged; no billing mode, paid Jev inference, production Live route, commit, push, deployment or publication changed.

### Native subscription qualification after freezing -09

Against the same MSI-extracted host, the real one-file Claude proposal path passed its own native preflight and disposable Review → Apply test in 360.00 seconds with `PYTXO_TEST_ROUTED_CLAUDE_SYNTHETIC_QUALIFICATION` absent. This exercised the no-built-in-tools proposal response, unchanged probe files, cancellation/Job-zero settlement and one verified candidate before Apply using the existing Claude subscription. It is stronger than the earlier synthetic pre-admission journey but still not a native GUI or installed-package journey. The separate edit-capable adapter's direct outside-Edit test failed to qualify on both Haiku (no denial tool event) and Sonnet (denied `Read` only); both left the disposable outside file unchanged. Do not count an untouched file as proof that `Edit` was denied or expose that edit-capable adapter as ready. The addendum is `D:/pytxo-beta-lab/jev-routing-race-candidate-20260929-09/native-qualification-addendum.json`, SHA-256 `AABE606C556E09891275EAA8B7B00EABE9901904F77DB7837FA20CA8E7A473AB`, bound to the frozen candidate receipt and extracted-host hashes. No API billing switch or Jev inference occurred.

The exact extracted executable also opened a 1294×808 native `Pytxo Desktop` window and closed cleanly. This is startup/render evidence only; no native GUI Work → Review → Apply action was exercised. `PYTXO_HOME` isolated the backend Catalog but WebView2 still displayed existing recent-workspace state, so the startup screenshot was discarded rather than included in candidate evidence. The non-sensitive smoke receipt is `D:/pytxo-beta-lab/jev-routing-race-candidate-20260929-09/native-smoke/native-gui-smoke.json`, SHA-256 `BF39F3B35176CC66D80F6D747DCA5FE1AB19BA0E045FB4DE5671C27B096F6CD9`. Native test isolation must cover WebView2 data before a clean-state UI journey can be claimed.

## September 29 — exact native UI routing candidate and checkout boundary

The experimental one-file Claude subscription route now normalizes Desktop/registry domain paths, validates direct Windows owned-launch paths, and runs combined-candidate checks through pinned System32 `cmd.exe /D /C` with the command tail preserved. A default Windows CRLF checkout previously passed Git-clean checks but failed Review against Git-blob bytes. The routed preview and dispatch now require the complete included physical checkout to match the reviewed Git snapshot, including ignored files; Windows junctions/reparse points are refused before inventory recursion. Transformed checkouts are rejected before a run claim, with an actionable `core.autocrlf` error. This is a conservative v1 eligibility restriction, not dual-baseline support. The native child-termination fixture now holds its late effect behind a gate so full-suite load cannot make it fire before termination.

The final Rust workspace suite passed after the Runner/junction changes and gated fixture (`D:/pytxo-beta-lab/jev-routing-final-workspace-test-20260929-17.log`). A prior full run had only the one-second child-fixture race fail; the gated test passed alone and in the full rerun. After the final checkout-error presentation change, the focused Claude review test (including a clean CRLF clone), strict workspace all-target Clippy, Rust formatting, Desktop Svelte/CSS and scoped diff checks passed. The offline routing benchmark contracts reported 31 passed, 2 optional Store-bridge skips, 0 failed; they provide no Jev-versus-Rules efficacy result.

Exact local candidate: `D:/pytxo-beta-lab/jev-routing-beta-candidate-20260929-16`. Unsigned, uninstalled MSI SHA-256 `EB60DB91A6CA02089FC6BF906549368934865B54392E9146900326A9C38B7E86`; extracted Desktop EXE SHA-256 `2D757AF1676C1A2AC697A30348130DC0F984B1A6616CD2024DAB8F6990A69F92`; receipt SHA-256 `F50F0A57C7572A70E83E7E012EDF45883CA02D14E1E42337FD7C1A78DAE5B3F3`, with 22 selected source hashes and three artifact hashes rechecked without mismatch. MSI extraction/runtime imports, four embedded asset identities and empty-input host refusal (exit 125, empty streams) passed. The exact extracted executable completed an isolated native Desktop UI journey: New work → reviewed Orbit one-file plan → real Claude subscription qualification/worker → one passed Everyday attempt in Rules mode → combined-candidate Review → explicit Apply. Run `9bab556f-d017-48b1-ab80-3badc6d0e45e` is completed/applied with prepared digest `5235cb9eede1ab2b11ef3b9672bef7fb8a28874cded63b1f00cd3606534febd3`; only claimed `result.txt` changed in the disposable source, plus local `.pytxo` ledger files. The native Review capture is `native-smoke/native-review-applied.png`. On a separate CRLF clone, the same extracted app returned the specific checkout mismatch before any saved draft, run or attempt and left tracked files unchanged.

This is a **local experimental beta candidate for the narrow LF-checkout Claude Rules route**, not a public Jev Routing beta or a release. Hosted Jev remains review-only, paid send and production Live routing stay disabled, and no held-out funded Jev-versus-Rules benchmark exists. The edit-capable adapter's outside-Edit denial is still unproven; the proposal-only route does not expose that adapter. Signed install/update, retained-data upgrade, deployed hosted grant/send, broader checkout support and approved publication remain open. Quarantined bootstrap/helper/Cargo files and Bitdefender settings stayed unchanged; portable Rust and the embedded host were the workarounds. No billing mode, source commit, push, deployment or release changed.

## September 29 — bounded Claude check repair and exact local package

The default-off `PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_REPAIR=1` path now permits one failed pinned check after an eligible Everyday Haiku attempt to start one explicit, durable Strong Sonnet attempt from the reviewed base. The failure must retain actionable check evidence; a changed checker, checker error, Stop, exhausted capacity or usage, or Strong-first selection does not become a hidden retry. Core still owns admission, attempt state, limits, verification, candidate identity and Apply. The existing dependency graph remains the scheduler; this does not enable general swarms or paid Jev. Hosted review remains `review_only`, and Proxy paid send remains hard off.

An independent review exposed Stop races around Ready-to-dispatch, native checker creation, winner publication and sibling Review. Catalog now atomically fences Ready cancellation; a dispatched Stop resolves the original Store and launch gate. Native create and publication recheck the exact Catalog Stop, and the controller reconciles a late Stop into Core's durable cancelled state. Regression tests cover these boundaries, including a dead controller after the second attempt is admitted. A stale fixture that described an unsupported custom-pair route was corrected to expect rejection before dispatch. The final Rust workspace suite passed (`D:/pytxo-beta-lab/jev-routing-repair-workspace-final-20260929.log`); strict workspace all-target feature Clippy, Rust formatting, scoped diff checks, Desktop Svelte/CSS and the affected browser route had passed. The benchmark contracts passed 31/33 with two optional Store bridge skips; this is protocol validation, not a Jev efficacy result.

Final-source unsigned, uninstalled local candidate: `D:/pytxo-beta-lab/jev-routing-repair-candidate-20260929-19`. MSI SHA-256 `15F6D7F7613F7DF69474F6972B630D3F0405D2E622EB9A2B47230D145573BBB7`; MSI-extracted EXE SHA-256 `75CE1F19E4750E0D98251C064B4D59218F0E02279E37052AA2D6F2AB02F9342A`; candidate receipt SHA-256 `B14B854FE473CCB23B030D0E2FCA52B24BD439E000761A7133945B119F8BADA9`, with 27 selected source hashes and three artifact hashes. Non-installing MSI extraction and packaged runtime imports passed; four embedded frontend asset identities matched; the extracted host refused an empty request with exit 125 and empty streams. Against that exact extracted executable, a local Claude-shaped stub completed Everyday failed check to one Strong repair and prepared Review, and a separate Catalog Stop before checker creation prevented launch. Both used synthetic pre-admission qualification and do not prove current provider behavior.

The same extracted app then completed a disposable **real Claude subscription** Rules run in isolated native Desktop/WebView2 data. The reviewed one-task Orbit plan had a pinned `findstr` check; one Everyday Haiku attempt passed, the Run reached Review with matching prepared/package digest `6e2d8fa529edb898d53baa346e196a47f47074289e303974f700aea5a97f18ee`, and explicit Apply changed only `result.txt` in tracked source to `Pytxo native routing verified`. Run `3c7dabde-904c-4b0a-b4af-6e26b95e568f` is completed/applied. Onboarding was clicked in the native WebView, while route preparation, dispatch and Apply used the app's Tauri IPC; the completed Work screen and routing/attempt details were visually inspected and captured in `native-smoke/native-routing-details.png`. This is a native runtime and rendered-state proof, not a click-only New work → Apply journey and not a real-provider Strong repair. The separate `native-acceptance-addendum.json`, SHA-256 `7A2016CC7D0682E96D2F0582075AB6FB8F4F0F2BAAF65F9A072B57DA8F5F8237`, binds the frozen candidate receipt and extracted EXE hashes to this run and screenshot. The app closed cleanly.

This is a local experimental candidate for the narrow Rules route, not a public Jev Routing beta or release. A click-only current-package repair journey, real-provider Strong repair qualification, signed installation/update, retained-data upgrade, deployed hosted consent/send, task-discriminating packets and a funded held-out Jev-versus-Rules win remain open. The separate edit-capable adapter's outside-Edit denial remains unproven. The quarantined files and Bitdefender settings remain unchanged; portable Rust and the MSI-embedded host were used instead. No subscription/API billing mode, paid Jev gate, production Live route, source commit, push, deployment or publication changed.

## September 29 — failure-receipt handoff, cross-adapter fixture, and local candidate -20

The explicit Strong retry now reconstructs a bounded check cue from the exact retained owned checker receipt. Its private input contains only the frozen check ID, nonzero exit code and receipt digest; checker stdout/stderr, including untrusted secret-shaped output, are not carried into the repair prompt. Admission and native launch re-render the prompt against the same Store owner and ledger evidence. The feature-gated local dependency fixture also runs a distinct pinned PowerShell child after a `cmd.exe` parent, asserts the child consumes retained winner bytes after the producer worktree is mutated, and reaches Review → Apply. This is local cross-adapter evidence, not a real Claude-to-Codex handoff.

The final routed native fixture suite passed 42 tests with one intentionally ignored separate-host case (`D:/pytxo-beta-lab/jev-routing-final-fixture-20260929.log`). The full Rust workspace suite passed with an explicitly pinned console embedded host and a 60-second *test-only* owned-launch budget (`D:/pytxo-beta-lab/jev-routing-final-workspace-pinned-20260929.log`); the first unpinned run failed seven Runner setup cases, and an 8-second test budget caused three timing failures before the test-only budget was raised. Strict workspace feature Clippy, Rust formatting, Desktop Svelte/CSS, 28 routing browser tests, and 31/33 offline benchmark contract tests passed; the two benchmark skips need optional Store bridges and no Jev efficacy trial occurred.

Fresh unsigned, uninstalled local candidate: `D:/pytxo-beta-lab/jev-routing-handoff-candidate-20260929-20`. MSI SHA-256 `BDE5E0A7EBABD99A1BC5A64898ADC683C1F0C861C99EEE7FD4CE2E360D2BB1DF`; MSI-extracted Desktop executable SHA-256 `9B674C99CA29BEB7E2CAD178412018291D243491EDFE1F30E18760B1D49CE271`; candidate receipt SHA-256 `C577F686D40B8D2CC31F47AFA1CAB1C8A398D111848D7CC87C9B37AFBA712D14`. The receipt rehashes 31 selected source inputs and three artifacts without mismatch. Non-installing MSI extraction, runtime imports, four embedded JS/CSS identities and empty-host refusal passed. The exact extracted host passed the local failed-check receipt repair fixture and three Stop-before-create gates. A separate optional hosted-payload fixture refused the MSI host by design, then timed out twice before process registration with a separate CLI test host at its fixed eight-second probe deadline; no payload launched. It remains unverified on this machine.

The exact extracted Desktop also completed a disposable native Rules run using the existing Claude subscription: Run `7202ecfc-f192-47fc-9206-9a7dc56868b2` had one passed Everyday attempt, matching prepared/package digest `40ed733d8b16eafadf2e93dcf06516af4a0cb0c809a7327f70625b951f9b4627`, and explicit Apply changed only `result.txt` in tracked source. The native Work screen and routing details were visually inspected at `native-smoke/native-routing-details.png`; route preparation, dispatch and Apply used Tauri IPC after onboarding. This is current-package native runtime and rendered-state proof, not a click-only journey or real-provider Strong repair. Hosted Jev remains review-only, paid send and production Live routing remain off. Signing, installed update/retained-data acceptance, deployed hosted grant/send, task-discriminating packets and a funded held-out Jev-versus-Rules result remain required before a public Jev Routing beta. The quarantined files and Bitdefender settings stayed unchanged; no commit, push, deployment or release was made.

## September 29 — staged hosted Shadow protocol and activation boundary

The off-by-default hosted Shadow seam now serializes the reviewed coarse packet into the fixed Proxy request, validates the bound usage receipt, and exercises a Catalog/Store one-send controller with an injected fake client. A fresh exact consent review, confirmed account/workspace/recipient grant, physical Store identity and local consent are required before send. The controller rechecks the persisted Flow review, grant, Store consent revision/scope, cancellation and task state around the send. A late reply after Catalog consent revocation remains journaled but cannot become a Shadow decision or cause a second send. Rules still selects the execution profile; the tested scope is Orbit, one task, one repository domain. This is local protocol evidence only: there is no HTTP client or production dispatch caller, hosted Flow remains `review_only`, and the Proxy paid-send gate stays closed.

Independent review found the next activation issue: the current Flow packet preview accepts only an undispatched `review_only` draft, while real dispatch claims a `ready` draft and later has an active Run. The production entry therefore fails closed for a real claimed hosted Flow. Do not connect it to dispatch by weakening review-only checks. When a hosted Shadow send is authorized, add the exact claimed Flow/Run/owner validator and guarded dispatch transition together, then test the real claim and registration path with a fake client before any paid call. Link must enforce the grant and token at the remote send boundary. A funded held-out Jev-versus-Rules result, task-discriminating packets, installed/signed update evidence and the other release gates above remain open.

Current-source verification: Orchestrate library 99 passed / 7 account-dependent ignored, Planner library 35/35, hosted grant binding 9/9, and the saved hosted Flow review integration 1/1 passed. Strict affected-crate all-target Clippy and workspace Rust formatting passed. The offline benchmark contracts passed 33/33 with current Rust Store/exporter test binaries, including both real Store-to-Node bridges; this verifies instrumentation, not Jev efficacy. No paid inference, Live routing, subscription/API switch, Bitdefender setting or quarantined file changed. The prior `-20` MSI predates this source slice and is not an exact-source package for it.

## September 29 — benchmark exposure and attribution gate

The offline confirmation freeze now binds a pre-outcome `planned_advice_opportunity` boolean to every case and assignment (frozen schema v2). It requires packet variation inside that cohort, so unrelated task diversity cannot make a uniform eligible cohort look informative. The label is still a reviewed expectation, not an actual Core eligibility receipt; its predicate includes the authorized evaluated Live policy, first strong-default diagnosis, complete single-component context and both qualified profiles. Opportunity prevalence, repository spread and power still need independent preregistration before a funded screen.

The Store-trace recorder now rejects `applied` advice without a completed matching send, an ordinal-one Everyday `advice_everyday` decision, and a linked resolved admission. It emits a small routing-exposure summary for each complete assignment. The analyzer's `routing-paired-v2` report separates planned opportunities from **bundle-reported** may-sends, completed sends and first admissions marked applied, while retaining all assignments in the paired comparison. It flags a numerical win with zero reported Jev-influenced admissions. A standalone bundle can self-assert exposure, so the report keeps Store trace, review-ledger and provider authentication as explicit gaps; no Jev benefit is claimed.

Current offline benchmark verification is 35/35 passing, including the existing real Store rules/export and advisor-journal Rust-to-Node bridges with synthetic data. Three edited Node scripts passed syntax checks. A read-only review found no remaining structural trace path for a false applied first admission after the fix, and its provenance wording concern was incorporated. Rust runtime sources were unchanged in this slice; the previous full Rust suite remains the latest Rust-wide evidence. Hosted Flow is still `review_only`, the Proxy's paid-send gate remains closed, and production Live routing is disabled. Quarantined files and security settings, billing modes, commits, pushes, deployments and releases were not changed. The `-20` MSI predates this benchmark-only change and is not an exact-source package for the current tree.

## September 29 — claimed hosted Shadow candidate and local acceptance

The hosted Shadow controller now has an explicitly opted-in `routed-test-faults` fake-client entrypoint that atomically claims the exact `review_only` Flow, rederives its packet from the claimed Run and original physical Store, and sends one bound observation before ordinary Core admission. Normal Desktop dispatch still rejects `review_only`; no production HTTP client or paid send is wired. Stop and claim now share an exact Catalog order, including pre-claim Stop for hosted reviews. Dead-owner recovery accepts a verified partial profile-qualification prefix and settles prepared or possibly-sent advisor requests without retry, worker admission or Apply. A delayed fake reply after local consent revocation is discarded. Shadow advice never selects the worker; Rules still chose Strong in the tested unresolved first decision. Scope is Orbit, one task, one repository execution domain.

Verification on this combined source: hosted-focused Store/Orchestrate tests passed (including both qualification crash points, three send-journal crash points, claimed-run revocation and Stop ordering); feature-enabled library suites passed 101 Orchestrate + 57 Store with seven account-dependent ignores; the routed Flow seam passed 74 with nine intentionally ignored native-provider tests. The exact MSI-extracted host passed one offline Claude Rules Review → Apply journey and one hosted fake-client Claude Shadow Review → Apply journey. After independent read-only review found two test-proof gaps, the post-Stop test now directly asserts the hosted claim loses, and the native hosted test requires the inspected MSI executable's SHA-256; both passed on rerun. That review found no remaining P1 authority bypass. Strict affected-crate all-target Clippy, formatting and scoped diff checks passed. Desktop Svelte/CSS check reported zero errors/warnings. The benchmark suite passed 35/35 with current Rust Store/export test bridges, but has no funded Jev observations. An attempted full Rust workspace run stopped during compilation with missing cached crate artifacts; a separate broad affected-crate run had one intermittent multi-root test failure, which passed when rerun alone. These do not count as a clean workspace-wide pass.

Unsigned, uninstalled local candidate: `D:/pytxo-beta-lab/jev-routing-hosted-shadow-candidate-20260929-21`. MSI SHA-256 `A2A193EC058E07BE7D1056B5B854972B45D69DFFD667BBEE5EABF9BA0A4BF320`; release executable `B1980FF7C82EC19CC440B3A631398CF201866684DE59E74A3E77934BE885F45C`; administratively extracted executable `637D598A0D4874C41F8294DA35026A712746D2610D4856BD201E52C24790875D`; selected-source/artifact receipt `9B2CF6DEFAD49033385FDE8F09FC01BD2DCE33851859E7A5F9C5BB045154E27F`. Extraction, all packaged PE runtime imports, four embedded JS/CSS identities and empty-host refusal (exit 125, no output) passed. The test-only Claude Shadow coverage file changed after the MSI build; release runtime sources did not. This is a local experimental candidate, not a public Jev Routing beta: Link-side atomic grant/token enforcement at the actual send, a real provider-backed hosted journey, task-discriminating reviewed packets and a funded held-out Jev-versus-Rules win remain open. Signed installation/update, retained-data upgrade and public release approval also remain open. Quarantined bootstrap/helper/Cargo files and Bitdefender settings stayed unchanged; portable Rust plus the MSI-embedded host supplied local verification. Paid Jev inference and production Live routing stayed disabled; no billing-mode change, source commit, push, deployment, installation or publication occurred.

## September 29 — staged Desktop hosted HTTP bridge

Desktop now contains an unused routing-only HTTP client that can exchange a freshly verified Desktop routing session for a Link token bound to the exact hosted workspace and grant revision, then POST the Core-built bounded evaluation request to the dedicated Proxy endpoint. Its account binder compares the current reviewed packet, local Catalog consent/grant, fresh Link grant status, account and origin before constructing the client. The transport rejects mismatched workspace/revision, noncanonical or expired tokens, redirects, ambient proxies, automatic retries, oversized or malformed replies, and a receipt with the wrong request or packet identity. It sends neither worker credentials nor arbitrary task text. Normal Desktop dispatch still refuses hosted `review_only` Flows, the staged constructor has no dispatch caller, and Proxy paid-send startup remains hard off. Core still selects Rules Strong in the existing fake-client claimed-Flow test. This is local client-protocol evidence, not a real hosted Jev journey or a provider billing receipt.

The final Desktop library suite passed 68/68, including nine staged HTTP/account-binding tests. The feature-enabled claimed hosted Shadow fake-client Review → Apply fixture passed 1/1. Strict affected-crate all-target Clippy, workspace rustfmt check and scoped diff whitespace checks passed. A read-only review found no remaining P1/P2 issue in this staged client after explicit no-proxy/no-retry hardening. The two-hop delay test proves an outer two-second timeout drops the local advice future after one token request and one evaluation request; it does not prove that a remote send was cancelled or unbilled. No full Rust workspace rerun, exact-source MSI rebuild, installed upgrade, deployed hosted test or paid Jev trial was completed in this slice. The prior `-21` package predates the new release-compiled client and is no longer an exact-source package. Quarantined files and Bitdefender settings were unchanged; no paid inference, Live routing, billing-mode switch, commit, push, deployment or release occurred.
