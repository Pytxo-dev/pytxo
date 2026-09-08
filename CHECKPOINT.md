# CHECKPOINT — ASTRA product improvement — 2026-09-08

## Current state — final MSI host rehearsal, film and website passed; guest stopped

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
