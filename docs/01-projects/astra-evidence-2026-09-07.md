---
title: ASTRA verification and artifact ledger
slug: astra-evidence-2026-09-07
status: active
tags: [verification, desktop, release, benchmarks]
audience: [human, agent]
layer: orchestration
created: 2026-09-07
updated: 2026-09-08
related: [[astra-execution-2026-09-07]], [[astra-release-proposal-2026-09-07]]
---

# Verification and artifact ledger

## Current candidate — September 8, 02:55 UTC

MSI `cf8db2e0b41ceaaf678d3c74226b62dc62589f2a68880ec8ba49d42088dbd911`
contains the static-runtime repair and actionable guided-example Git preflight.
Packaged EXE: `d75be6b767164e681b278c717551b84f62baf6987467751f2e98a39538c72db5`.
All 354 build inputs match manifest digest
`c721a49414e348dc93d7801c9d36fefdd445a75d44dd3ecaa6487824265e38f0`.
MSI and EXE remain Authenticode unsigned. Actual MSI extraction/import guard
passed for its one EXE, with no forbidden runtime imports.

`npm run check` passed with zero Svelte errors/warnings and CSS lint clean.
Four `ipc_install` native tests passed with static CRT and distribution/voice
features, including missing Git without partial files and actual Git success.
Four Playwright journeys passed: missing-Git retry/Skip at 1440 and 390, plus
existing guided-example journeys. Both error-state screenshots were inspected.
Desktop all-target Clippy passed with warnings denied. `npm run build:msi`
completed successfully (native 4m33s; frontend 38.03s).

Logs, frozen inputs, extracted executable and reports are under
`target/astra-onboarding-prereqs-20260908/`. Its final 16-file packet ZIP hash is
`47517631b8602ad671ec3cade5ff3530de2615648f7a0b2d1b008c79627bac46`;
all archive entries and both read-only transfer discs were compared byte for
byte. This candidate's guest installation and guest mission remain pending.
The preceding 1f47 MSI's successful guest welcome below remains narrower,
historical evidence. No additional Actions run or publication occurred here.

The final MSI's separate host rehearsal completed at 02:27:47 UTC. Run
`267ba929-200f-48af-b5bc-6735e5810ae4` used Codex 0.153.4/Astra/high, three tasks,
two waves and at most two workers. Three combined checks passed. All six primary
hashes stayed unchanged before Apply and after Cancel. Three exact diffs were
reviewed; confirmation committed package
`349ff5f7a504516497aa60f04887bf969dd4215d92f84701d54f8ca40f5bc93c` with attempt
`43939199-b4d3-4a3b-bf67-d58b7201e194` at 02:29:55. Three changed hashes matched
the frozen manifest; the other three stayed unchanged. Actual post-Apply
processes passed 11 repository and 26 independent tests, zero failed or skipped, and
the receipt survived native restart. The independent checks had five expected
baseline failures. Bounded evidence review found no material discrepancy.

Sanitized result: `tooling/benchmarks/results/astra-final-native-2026-09-08.json`;
raw private evidence: `target/astra-final-native/`. Six native element captures
are in `docs/_attachments/astra-2026-09-08/`. Workspace catalog setup used native
IPC; the OS folder picker was not tested. Permission, draft reuse, mission,
review, Cancel and Apply used actual native UI controls.

The rebuilt silent master is 52.000 seconds, 1920×1080 at 30fps,
H.264/yuv420p/BT.709, SHA256
`9c427fc2ae0ab8acd6303b5b0379c432b5dc4b136dcec1a968c47ca59a0015e4`.
Typecheck, original asset hashes, transcript/cues, format and black-frame checks
passed. Seven scene stills and 13 encoded transition frames were inspected;
continuous playback was not watched. No narration service was used.
`tooling/benchmarks/results/astra-final-demo-2026-09-08.json` binds the source,
captures and QA artifacts. The old master was hash-archived before replacement.

The website's current-candidate link resolves to this final native record.
Production build, lint and 163 internal links across 108 source files passed.
All three affected production browser tests passed in 17.7 seconds, including
artifact identity at 1440 and 390 pixels. Both full-page screenshots were
inspected; the current follow-up text and hash are readable at both widths.
Logs and captures: `target/astra-final-native/web-*`.

## Clean Windows finding — September 7, 23:53 UTC

**REPRODUCED:** MSI `614d44e2d47e…` installed with exit 0, but its Start-menu
launch failed because `MSVCP140.dll` was missing. The fresh guest had no Visual
C++ redistributable added. WebView2 was present after installation. The frozen
payload directly imports 95 symbols from that DLL. Screenshot:
`target/astra-clean-vm-20260907/screens/guest-pytxo-first-start.png`.
Guest installation logs remain on its preserved disk; before/after inventory
JSON and inspected installer-result screenshot are retained on the host.

The preceding static-runtime replacement uses opt-in `.cargo/windows-msvc.toml` with an explicit
target through `npm run build:msi`. Its 354-input manifest digest is
`3e57d96e802b903bfab4ae73a6ce822c15b52159b1602b38351f3a733f6039ae`.
MSI SHA256: `1f47f34d673b552ae8ddebab222ad49989ef8d94f6032b94027e970ab980219f`;
payload SHA256: `44106e772f38ef4a1ba8123ec97128c091691b4592f6e0e6eb5bfc942d80d430`.
Both are Authenticode unsigned. Build and actual MSI import checks passed;
four native libraries use static CRTs, 41 release-tooling tests and 12 native
voice contract tests passed. This does not establish speech transcription.
Logs, source manifest and extracted-payload report are retained in
`target/astra-crt-native/scoped-policy/`. An intermediate 353-input build is
historical; the final policy preserves ordinary Cargo host defaults.

The failed candidate was removed normally. Exported guest inventory confirms
its executable and registry entry absent and all six checked VC++ runtime DLLs
still absent. The replacement installed with exit 0 on September 8 at
00:56:41 UTC. Start-menu launch produced a native white window at 00:57:35;
rendered UI was not yet verified. At 01:06 UTC the host-resource guard stopped
only the owned guest after two low-memory samples. Its disk and installer logs
are preserved for continuation. This is not yet clean-machine acceptance.
Original MSI, native mission, film and Bench records below remain evidence of
their named earlier bytes. Hosted CI remains green for `f64a0b0`, not this repair.

Continuation at 01:09:52 UTC retained the installed replacement. Start-menu
launch at 01:12:36 rendered the welcome screen at 01:13:13. Continue with Desktop
proceeded without the separate Pytxo CLI; all five absent agent CLIs reported
not installed. Exported post-launch inventory confirms installed EXE `44106e…`,
all six checked VC++ runtime DLLs absent, and no developer commands present.
Inventory SHA256 `ced16382b5c00a579a1e8f0d5d6faffac1813eec567daf9a091affee2b446c30`.
Actual MSI log and result were exported and verified; log SHA256
`e5be4579ffb259e1d85cbd35bf153aa3175e847f34a482f13536a5c35052d87d`.
`target/astra-crt-native/scoped-policy/guest-install-and-launch.json` records
the narrow verified scope. This establishes the missing-runtime repair.

Before Git installation, Try the guided example produced `[io] program not found`.
A subsequent source correction probes Git before creating files and names the
missing prerequisite with recovery guidance. Its native no-partial-files and
two-viewport UI retry/skip regressions passed in the current candidate section.
Do not relabel the `1f47…` bytes as containing those later changes.
Verified official Git/Node installers completed with observed exit 0. Fresh
terminal versions: Git2.55.0.windows.5, Node24.20.0, npm11.19.0. Codex0.153.4 was
installed to match the earlier rehearsal. Its official device-login page reports
device-code authorization disabled. Approval for temporarily enabling it and
restoring the setting is pending; no security setting or host credential changed.

The preceding native builds and checks were produced with uncommitted changes
on PR31's `ff0b88fb71224d579267e697261ae4d831f0cd76`. The CI correction below was
built from `2964d5ad2ad677dec014d32ae227635de8dfea5b` plus uncommitted repairs.
Build-time HEAD alone does not identify either artifact; source-input identities
remain authoritative after committing. CI, clean-machine installation and public
delivery remain separate gates. The owner subsequently upgraded to GitHub Team
and authorized continuation; the organization description/website were updated.
No paid budget, security setting, release or deployment was changed by the lead.

## CI correction and current native proof

First executed CI run `34100785788` completed: eight jobs passed and four failed.
Both Unix Rust jobs exposed a registry lock held while Stop waited for child
reaping; Windows Rust 1.98.1 exposed a new Clippy lint in a UTF-16 test helper;
the npm version check still parsed superseded public-binary wording. All three
causes were repaired. Cancellation is persisted before process termination;
only captured process identities are removed after it. Identity guards and
failure evidence remain. Cheap npm metadata checks now precede Rust compilation.

Fresh local checks on the correction: **468 workspace tests, 21 release tooling
tests, workspace Clippy and formatting pass**. The real repository version check
passes at 1.2.2. `cargo audit` passes with the existing 19 allowed warnings;
no exception or check was removed. Logs: `target/astra-ci-{workspace,clippy,audit}.log`
(workspace/Clippy have `-green` suffix), `astra-ci-release-tests-green.log` and
`astra-ci-version-green.log`. Independent runtime and workflow reviews passed.

The rebuilt MSI is `614d44e2d47eeeea2e62c0689b3146183daea30816f4d4341be0333d7b8948b3`,
payload `2d0124917fad349f340cc210780de8dfee5162d2694eeabb15e7b54a71d30b5f`;
both are Authenticode unsigned. Its 352-input source digest is
`d54be69068c39cf8b14ce58bbad84a77a274ae878a7874d796c9da2644db8e4a`.
MSI build log: `target/astra-ci-msi-build.log`.

The separate [current-build record](../../tooling/benchmarks/results/astra-ci-native-2026-09-07.json)
binds run `53728815-a282-44f8-8be7-05f224e91223` and package
`c01b2215f9f083565e56043c8147e483cfcffc12666bfdd893c83ef468bb3756`.
It adds `credentials/` to the previously applied synthetic baseline, with an
explicit documentation-task override observed in worker input. Three workers
and three combined checks pass. Native exact-diff review and explicit Apply
produced three matching hashes and preserved the other three source files.
Eleven fixture tests and 26 separately authored acceptance tests pass; five
acceptance cases failed before the change. The committed receipt survived restart.
Computer Use prepared and ran the mission; inconsistent native-helper state led
to a same-payload restart and normal WebView review/Apply clicks. The actual
worker headers report Astra/xhigh. This is a different task and baseline from
the direct comparison; no timing or cost advantage is inferred.

`target/astra-ci-native/acceptance-review.md` independently checks the MSI,
payload, package, journal, baseline/post-state and tests. Development-host
extraction remains narrower than clean installation. The earlier records and
captures below remain unchanged. The current film is 52.000 seconds, 1080p/30fps,
H.264/yuv420p/BT.709 with no audio; SHA256
`761b377c71a15eed5a59ec29592f36919d711736e08122cfc6a24c11a044a2ee`.
Seven composition stills and 13 encoded transition frames were inspected.
[Current film provenance](../../tooling/benchmarks/results/astra-ci-demo-2026-09-07.json)
binds the new captures and record. Full continuous playback was not watched.

Current packet: `target/astra-ci-windows-validation-packet.zip`, 16 files, SHA256
`42845a33b2e7044787843b7a63c51cf77b1e5132c1a38f95311ff3d292e10503`.
Every ZIP entry matches its allowlisted name, size and hash; all six baseline
files match the earlier applied commit. Eight baseline tests pass; the portable
acceptance verifier reproduces 21 pass/five fail. Its source hash differs only
because the import is portable. The template remains `not_executed`.
Run `34107810014` on `402c5991cb064341bf240d2d28e3c46c424b9e13` subsequently
finished with eleven passing jobs. Windows Rust failed only two ten-second
transport fixture deadlines; the log cannot distinguish startup delay from a
hang. All five transport tests passed unchanged locally. The narrow correction
adds Windows-only elapsed diagnostics and owned-run cleanup, with a 60-second
startup/non-Stop allowance. Post-Stop settlement remains ten seconds, below the
probe's natural 30-second exit. Timeouts still fail and all assertions remain.
Fresh affected checks: **95 orchestration tests and Clippy pass**; independent
review found no actionable issue. Logs: `target/astra-ci-windows-orchestrate-green.log`
and `target/astra-ci-windows-orchestrate-clippy.log`.

Final hosted run `34112393479` passed **all twelve jobs** on
`f64a0b092b02af917fea52180a4e76458e2f1934`, completed at 11:13:50 UTC on
September 7. This includes the Windows workspace, all five transport lifecycle
tests, audit and feature checks. Records: `target/astra-ci-34112393479-final-run.json`,
`target/astra-ci-34112393479-final-jobs.json` and the retained Windows log.
This closes hosted CI for that source; it does not establish clean installation
or publication.

This later correction changes only the existing `#[cfg(test)]` module in
`flow.rs`. The complete non-test prefix matches after explicit newline
normalization; all 351 other frozen native input hashes match. Retained MSI,
film and packet hashes remain unchanged. Their original 352-input snapshot and
native observation remain authoritative: these artifacts were **not rebuilt
from the later test-only commit**, and no binary reproducibility claim is made.
The separate local provenance record is `target/astra-ci-test-only-provenance.json`.
Clean installation remains unexecuted and must identify the exact intended
publication artifact if an authorized release build produces different bytes.
The user subsequently authorized assistant-owned VM setup. The disposable
Windows 10 LTSC guest is being prepared locally: WHPX firmware execution, private
QMP control, the 16-file packet ISO roundtrip and the complete Windows media
SHA256 are verified. The monitored resource gate passed and Windows setup began
at 22:39:41 UTC, with actual setup screenshots retained. A continuation reached
the fresh desktop. The offline inventory verifies Windows 10 LTSC 19044.1288,
no Pytxo/WebView2/Git/Node/agent CLI, enabled Defender protection, and the exact
candidate MSI hash. Its retained JSON SHA256 is
`e2351f1ebbf0b33740f7bbb4ae2b1aa47487ee5430a3196dc8f5d699ba69498a`.
Evaluation activation is not verified: the offline inventory reports status 5
and zero remaining grace. Normal online activation, servicing and complete
installer/native acceptance remain pending. Current
provisioning evidence is in `target/astra-clean-vm-20260907/PROVISIONING.md`.
Keep creation-time gate states inside the unchanged packet/media records intact.

The current website rebuild and lint pass; 162 internal links across 108 files
were checked. All three affected production browser tests pass. The current
candidate section and its JSON were additionally exercised at 1440/390px;
displayed hashes/counts match, with no page errors or root horizontal overflow.
Screens: `target/astra-ci-native/site-evidence-{1440,390}.png`. All 23 website
browser checks passed again on `402c599` in the second hosted run.

## Preceding source checks

| Check | Actual result and local log |
| --- | --- |
| Rust workspace before final receipt repair | 459 passed, `target/astra-workspace-tests-transport-final.log` |
| Final affected orchestration suite | 95 passed, including seven new receipt regressions, `target/astra-research/runtime-original-cap-orchestrate-green.log` |
| Final workspace Clippy, warnings denied | PASS, `target/astra-clippy-final.log` |
| Final CLI and MCP build | PASS, `target/astra-cli-mcp-final.log` |
| Final Rust format | PASS, `target/astra-fmt-final.log` |
| Final Windows MSI build | PASS, `target/astra-msi-final-build.log`; source-input digest `9b9079cb25fc5dc01e749c14ffaccfc67895cbce60f6bbdef12dff671de39858` |
| Desktop production browser suite | 128 passed, `target/astra-desktop-production-e2e-handoff.log` |
| Svelte/CSS checks | Zero errors/warnings, `target/astra-desktop-check-handoff-final.log` |
| Artifact inventory/version checks | 16 passed, `target/astra-release-gates-final.log` |
| Final CLI smoke | Version 1.2.2, valid status JSON and four records for the final fixture; `target/astra-cli-smoke-final.json` |
| Final website lint and production build | PASS after metadata and duplicate-heading repairs, `target/astra-web-{lint,build}-metadata-final.log` |
| Final website links and browser journeys | 161 links across 108 files; 23 browser checks, including all 45 sitemap pages without JavaScript, `target/astra-web-{links,e2e}-metadata-final.log` |
| Website crawl and local performance | Canonical/share identities, robots, sitemap, image dimensions and HTTP status checked; eight bounded timing observations, `target/astra-web-{crawl,performance}-final.json` |
| Final native Review → Apply | Three matching changed-file hashes, eight independent tests, committed journal; `target/astra-native/accepted-*.json` and `accepted-post-tests.log` |
| Native process restart | Same payload; selected committed receipt persisted, `target/astra-native/accepted-restart-history.json` |
| Windows packet | Exact MSI inventory, two baseline fixture tests, all 15 archive-entry hashes match; `target/astra-packet-*.log` and `astra-packet-archive-verification.json` |
| Final silent film | 52.000s, 1920×1080, 30fps, H.264/yuv420p, BT.709, no audio; `target/astra-demo-qa-final.log`; seven scene stills and 13 encoded transition frames inspected |

The mixed-profile receipt collision was reproduced in five candidate and two
Apply regressions. The shared runtime-ordinal resolver now retains the original
permission cap and refuses missing or inconsistent receipts. The full affected
suite and independent source review pass. Final packaging follows combined
Clippy and CLI/MCP checks and has passed; the earlier workspace count is not relabeled as a
post-repair full-workspace run.
Browser fixtures establish UI behavior; they do not establish real agent or MSI
installation outcomes. The final website, native captures, silent film render
and media QA are verified within their stated scopes.

## Native observations, including failures

| Packaged MSI / run | Observed outcome |
| --- | --- |
| `ccfdb862b9c0…` / `9d04e018-a888-46ee-af60-449a902b9adc` | Workers failed before model execution because CMD rejected the verbatim working directory. Preserved in `target/astra-native/first-build/`. |
| `ca8140d30a50…` / `11326244-4d19-4a4c-bb67-49f9493b7f27` | Three workers completed and passed their checks. Preparation correctly refused an implementation worker's out-of-scope test edit; all six baseline file hashes stayed unchanged. |
| `3e48087850e1…` / `98a3ac95-c3a7-47fa-9818-456fb88daa04` | Codex rejected split prompt arguments before model work. Two workers failed; the dependent task was blocked; six baseline hashes stayed unchanged. |
| `78f9f8b3a108…` / no native run | Build passed after the transport repair, source-input digest `ad330846…cd737`. Archived in `target/astra-native/pre-receipt-build/` and superseded by the final build containing the receipt-cap repair. |
| `dfacd548f58e…` / `55a36c03-e0f7-4b92-bec2-53df36f17541` | Workers and combined checks passed. Native diff review found stale README prose contradicting the correctly changed code. Apply was withheld; the ready package remains intact. This was an agent review decision, not automatic semantic rejection. |
| `dfacd548f58e…` / `d743c0dc-76d3-4ff8-b6a3-7d88e743eb22` | With explicit final-state documentation guidance edited into the reviewed task, all three workers and combined checks passed. Native review inspected all exact diffs and confirmed Apply. Three applied hashes match the package; eight independent tests pass; all other source bytes and the complete source inventory are preserved. |

The refusal and launch-failure observations have allowlisted JSON records in
`tooling/benchmarks/results/astra-{refused-run,transport-failure}-2026-09-07.json`.
Full local events retain the diagnosed failures. The review-withheld and
successful outcomes also have separate allowlisted JSON records. Each MSI and
its source-input snapshot remain separately named.

The accepted [native record](../../tooling/benchmarks/results/astra-native-codex-2026-09-07.json)
identifies the full MSI digest `dfacd548f58ea149d84271de50c038288f6b19894f1496dde65916a4a67d20d3`,
payload `9d99646c9ac59c4504a5f4bd7133b1bd84d3327a59a25ccb7dd75926abd9e603`,
and package `0c04ae8106a37dca0993addf572f16c0ba802ba101b0ccd453d2ff5515a0a6bf`.
Six original captures are in `docs/_attachments/astra-2026-09-07/`. An independent
read-only reviewer checked the identity chain, inventories, actual tests, prompt
override and image privacy. Both MSI and payload are Authenticode unsigned.

Recorded native start to candidate verification was 269.899 seconds. The three
CLIs reported 80,975 tokens in total; input/output split and invoice cost remain
unknown. The exact documentation prompt override is in the record and fixture.
The direct run received the original global mission without that added guidance.

## Competent direct comparison

`tooling/benchmarks/results/astra-direct-codex-2026-09-07.json` records the same
mission using Codex 0.153.4 in a separate Git worktree. It changed the requested
three files, passed eight independent tests and preserved the primary checkout.
Observed process duration was 252.172 seconds; the CLI reported 30,811 tokens.
Invoice cost and operator time were not measured. Existing hooks and background
load confound timings. The fixture and exact mission are reproducible from
`tooling/benchmarks/fixtures/astra-first-mission/`.

Do not present a timing delta, primary preservation alone, or this sample as a
Pytxo advantage. The native package/check/Apply evidence stands on its own.

## Screens and remaining local limits

Before/after browser captures are in `target/astra-ui/`. The final website and
Evidence page were inspected at 1440 and 390 pixels, with no page errors or root
horizontal overflow. Narrow Desktop plan controls were additionally inspected
after scrolling; the earlier screenshot named `plan.png` shows only its first
viewport and must not be used as proof of the whole mobile plan.

Native draft reuse restores the workspace and Codex after CLI detection settles
and requires a fresh plan. An early click while detection is pending instead
requires explicit CLI reselection. This safe but awkward edge remains deferred,
as does per-worker terminal-state persistence before all waves finish. No claim
of mixed-vendor Flow, universal Windows adapter prompt support, or OS-wide
filesystem/network isolation was added.

The portable packet is `target/astra-windows-validation-packet.zip`, SHA256
`2d259f66832a45049e89d0d52d6fd0522f0720a52ffd6fddd85cd38ce2c2d8ea`.
It contains 15 checked files, including the tested MSI and fixture-local setup
instructions. Its clean-machine result template remains `not_executed`.

The preceding silent film is preserved in `apps/demo-video/out/archive-2026-09-07-pre-ci/`, SHA256
`45aab4c6dee117d4792ac637fc181ad1445b65841359086e88d048751339317c`.
Its [provenance record](../../tooling/benchmarks/results/astra-demo-2026-09-07.json)
binds the composition, native record, poster, contact sheet and transition sheet.
No current narration was generated, and provisional caption timing is not
publishable. Older ignored renders and status files are historical artifacts.

## Final website completion audit

The initial production responses had no canonical URLs, inherited the homepage's
share identity on other pages, and omitted Evidence from the sitemap. Two browser
regressions failed against that build; robots/404 and social-image controls passed
(`target/astra-web-metadata-before.log`). Each public page now serves its own
canonical, title, description and complete Open Graph/Twitter identity. The
all-route crawl also reproduced a duplicate BYOK heading; its redundant MDX title
was removed, preserving the rendered page title. The failure is retained in
`target/astra-web-duplicate-heading-red.log`. All 23 final browser checks pass.

The 45 unique sitemap URLs returned 200 and crawlable metadata with JavaScript
disabled. Tracking parameters leave the canonical/share URL unchanged. Robots
exposes the sitemap, `/pricing` redirects to `/plans` with 307, and an unknown
page returns 404 with `noindex`. The declared social PNG returns 200 and matches
its 1600×1000 dimensions; the image was visually inspected. Actual social-network
card rendering and production indexing were not exercised. Introduction and BYOK
screens were inspected at 1440/390px with no root overflow or page errors.

Eight fresh headless Chrome contexts observed Home, Evidence, Introduction and
First mission at both widths on final build `F8ldG-vMSHyJBchf-riN2`. Unthrottled
localhost FCP/LCP observations ranged from 328 to 488ms and observed CLS from 0
to 0.00666. The window ended two seconds after load/font readiness, with a bounded
font wait. Three aborted sign-in prefetches are retained; no page errors occurred.
These single observations overlap other host/test work and do not establish
production/mobile performance, field Core Web Vitals, INP, a Lighthouse score,
or a performance improvement. Further bundle/font work is deferred until target
device or production measurements justify it.

Before the CI repair, the preceding native source's 352-file set and every hash
matched its freeze; MSI, film and Windows ZIP identities also matched. The later
Stop repair changed native source and required the new build above. Historical recheck:
`target/astra-artifact-recheck-final.json`. The proposed commit inventory is
`target/astra-commit-inventory-final.json`; it excludes the private master brief,
preserved user captures/logs and the unused generated demo Flow copy. Nothing is
staged or published by that inventory.

## External acceptance

Earlier hosted CI was rejected before checks by the organization's Actions
allowance condition. The Team upgrade showed 1,000 remaining included minutes
at 08:25 UTC, before the first executed run; paid overages remain blocked.
Run `34100785788` completed eight passing/four failing jobs. The consolidated
correction establishes hosted acceptance only after all required jobs pass.
A disposable Windows guest is now provisioned locally; its activation, servicing
and Pytxo acceptance are incomplete as recorded above. Public version 1.2.2 is
absent. The concrete owner sequence is in
[[astra-release-proposal-2026-09-07]]. Prepared protocols, null result templates,
local extraction and HTTP success never count as closure of those gates.
