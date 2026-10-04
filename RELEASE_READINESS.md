# Pytxo v1.2.2 release readiness

## October 4 (later) - new Desktop UI, real DPI, in-app update proof

**Decision (Matt): the beta ships without Windows Authenticode signing.** The
install guide explains the SmartScreen prompt; in-app updates stay
signature-verified.

Desktop changes: compact Work header (title and state on one line, inspection
tools folded to an icon), two-pane onboarding with a step rail, and the
approved "live instrument" Work pass: a run status bar (working/done/failed/
waiting from recorded agent states, time since start while running, review
state), activity rails on working agents (activity, never progress), and
handoff flow on canvas edges only when a completed task feeds a running one.
All motion stops under OS or in-app reduced motion. Apply, verification and
stale gates are unchanged.

**Final head `cc0baa8`, candidate build 37186466601, acceptance run
37186468707, all on GitHub-hosted Windows: passed.** MSI SHA-256
`4290F365BCFE39ABEB77B541F5B114C4903ABB1822A6BEC9394354B8B51F8DED`
(Authenticode: NotSigned, by decision).

- Clean install and mixed-agent journey as a standard user through UI
  Automation: changed files equal reviewed files, `npm test` passed after Apply,
  7/7 files byte-identical to the recorded 2 October workers (stand-in replay).
- **Real Windows display scaling** set live through DisplayConfig: 150%
  (window DPI 144) and 175% (window DPI 168) passed. 200% is not offered by the
  hosted display (maximum 175% at 1920x1080), so it remains a real-hardware check.
- Retained-data upgrade from public v1.2.1: one 1.2.2 install, no data lost,
  onboarding not shown again.
- **In-app update from public v1.2.1 to this candidate: passed.** v1.2.1's own
  updater fetched the feed, downloaded the candidate MSI, accepted its
  production updater signature with v1.2.1's embedded key, installed it, and
  relaunched as 1.2.2 with onboarding retained. The feed was a local HTTPS
  stand-in reached through the runner's hosts file and a throwaway trusted
  certificate; the real GitHub feed URL was not exercised, and Desktop ran as the
  runner's administrator, so UAC consent was not exercised.

Not established: live vendor agents on this artifact (stand-ins replay the 2
October run; that run is the live multi-vendor evidence), true 200% DPI, UAC
consent during update, publication. PytxoFilm v2 (camera push-in) is rendered
privately for Matt's review.

## October 4 - cloud Windows acceptance, upgrade proof and PytxoFilm

All checks ran on GitHub-hosted Windows runners (`desktop-acceptance.yml`,
manual); nothing ran on a developer machine. Artifact: the unsigned candidate
MSI from build run 37114184513 (source `926cc1f`), SHA-256
`22BF155A600B20E7935CD931E2883129ECAEAFB555687ED451282852E695E6C7`.
Later commits change only acceptance tooling, the film and the UI font stack
(Satoshi removed so every machine renders the bundled Sora), so a fresh
candidate build must repeat these runs before release.

- **Clean install and mixed-agent journey (run 37169676357): passed.** Installed
  per machine; Desktop ran as a standard-user account, driven through Windows UI
  Automation with the real pointer and keyboard. Onboarding, real folder dialog,
  six-task plan across Codex, Claude Code, Cursor Agent, OpenCode and Antigravity
  stand-ins, 6/6 workers completed, Review, stale Apply refused after an added
  file, Refresh, Apply. Afterwards git status matched the reviewed files exactly,
  the fixture's `npm test` passed 10/10, and the applied task board ran in Edge
  (Spanish switch, new task). Stand-ins replayed the recorded 2 October workers'
  output and files: 7/7 applied files byte-identical to that run.
- **Layout at 150% and 200% WebView scale: passed** (controls inside the window,
  no horizontal page scroll). This is `--force-device-scale-factor`, not Windows
  display scaling; gate 3 below stays open for true per-monitor DPI.
- **Retained-data upgrade from public v1.2.1 (run 37179034172): passed.** One
  install of 1.2.2 afterwards, no data files lost, and onboarding finished in
  v1.2.1 was not shown again (WebView storage retained).
- Why UI Automation: WebView2 153 on hosted runners starts no DevTools server for
  this app from any channel (environment, HKCU or HKLM policy, elevated or
  standard user), so CDP-driven acceptance is not possible there.

**Final head `7de9dc8` (candidate build 37179601528, acceptance run
37179603560): all passed** on its own MSI, SHA-256
`90006BAD28309BEB4E973DF9F4E180C30A9478C12E8D77EEB3688F7B8747A486`: clean
install, journey (changed files = reviewed files, tests passed, 7/7 replayed),
150%/200% layout, and the v1.2.1 retained-data upgrade.

PytxoFilm (`apps/demo-video/src/film`, rendered by `demo-motion.yml`): 51.5 s,
1080p60, CC0 audio. The interface is recreated in motion graphics from the
recorded 2 October ledger; the end card states this and that time is
compressed. Footage of the final build exists in the acceptance artifacts
(native screen recording with the real pointer) for a later edit.

Still open: Authenticode/updater signing and an installed updater
download/install/restart proof; true Windows 150%/200% DPI; real vendor agents
(not stand-ins) on the final artifact; acceptance on a fresh build of the final
head; privacy/brand review of the film and publication. None of these were
attempted here.

## October 3 - Codex continuation, silent film and native safety acceptance

**Decision: NOT READY for public Beta.** Continued Claude's latest implementation
in `C:/pytxo/.claude/worktrees/nav-inventory-prototype-2e7ea3`, starting at
Claude's `43cd5e6c`. Matt approved committing the candidate to private
`Pytxo-dev/pytxo` as `2ntt/pytxo-beta-candidate-20261003` and opening an unmerged
PR for CI. That authorization includes the 41 accumulated local commits, not
only this UI patch; experimental routing and service migration source remain
included but are not activated or deployed. No main merge, deployment, tag or
release is authorized. Local chats, audit logs and temporary capture tests are
excluded. The older dirty root checkout was preserved. This section supersedes
older readiness summaries, not their historical evidence.

### Changes and verification

- Fleet cards now keep vendor names, paths and useful output readable at the
  supported widths. Settled workers drain their final events, long tails continue
  past 2,000 records, failed reads retry, and switching runs cannot mix output.
  Counts distinguish configured checks, completed tasks and no-change workers.
- Review's line diff preserves CRLF and final-newline differences. A file whose
  bytes changed can no longer appear unchanged because of newline normalization.
- List view shows the saved task request, retaining technical IDs in inspection.
  Readable output groups records and conservatively repairs corroborated ConPTY
  wraps; Raw text retains the original payloads. Cursor movement alone never
  authorizes removing a character. Escape strings remain suppressed.
- Release notes now describe the current multi-agent candidate and its limits;
  `distribution/release-notes/v1.2.2.md` is the publication payload.
- Accessibility review corrected the focused-run landmark and selected Review
  file-path contrast. The missing-project story now targets the current Project
  picker, and Storybook normalizes mixed Windows path separators.
- Locked `devalue` moved from 5.8.1 to compatible 5.9.4; compatible tooling fixes
  cover axios, brace-expansion, fast-uri and joi. Clean `npm ci` confirms installed
  versions. Desktop and Web production audits report zero vulnerabilities.
  Desktop's full audit still reports 10 development-only findings: 5 high through
  the braces/glob chain and 5 moderate through uuid/Storybook coverage tooling.
  No forced major upgrades or audit suppressions were introduced.
- The private candidate workflow runs only for this exact branch/repository,
  builds updater-signed artifacts with read-only repository permissions, and
  checks the MSI-extracted executable against the current frontend. Its evidence
  records both installer and payload hashes. It does not publish a release or
  update feed. Vercel automatic deployment is disabled for this candidate branch.

| Local check | Evidence |
|-------------|----------|
| Rust format and workspace Clippy, warnings denied | PASS |
| `cargo test --locked --workspace -j2`, pinned attempt host, isolated state | PASS: 1,114 tests, 16 ignored at patched TLS head `69501ac`; subsequent platform repairs require their affected checks and hosted CI |
| Desktop Svelte and CSS checks | PASS: 0 errors, 0 warnings |
| Desktop full production-preview suite | PASS: 413/413 after clean dependency install and final accessibility/output fixes |
| Storybook build, interactions and accessibility | PASS: 42/42 after fixing Windows discovery, landmark semantics and Review contrast |
| Full development component suite | PASS: 6/6; final run-switch lock, readable List and raw-output behavior included |
| Marketing captures and dense-canvas performance | PASS: 28 captures; 1 performance scenario |
| Web lint, links, assets, build and browser tests | PASS: 172 links, 30 Desktop assets, 64 pages, 28/28 browser tests |
| Release inventory and version tests | PASS: 27/27, including the candidate no-deployment boundary |
| Silent film source, evidence and master validation | PASS: 56 s, 1920x1080, 60 fps, H.264/BT.709, no audio; full decode and normal-speed browser playback |

Detailed local logs are under `apps/desktop/audit-shots/`; the patched workspace
run is `astra-candidate-rust-final-tests.log`. Strict workspace Clippy passed at
`c45c756`; the subsequent preview-policy regression run passed 5/5.

### Native evidence

Run `313e35f2-a0ae-45c5-b048-fe73e811f721` used the guided example, Codex, Orbit,
one worker at a time, and three ordered tasks. All workers exited 0; task and
combined checks passed. Adding an unrelated `operator-note.txt` made the first
package (`324c6c935fc08dbf7a86bdb6a863292f5c3a2e785155eb523582bcc83f783e82`)
refuse as stale. Refresh produced
`de8349e40c18b3e7aa305887e0489f3b64bb00d3c800f06c971e5d4546cec17b`.
Apply wrote exactly README.md, `src/risk-policy.mjs` and
`test/risk-policy.test.mjs`; the operator file survived. All eight post-state
inventory hashes match the verified candidate; post-Apply tests passed 11/11.
History records Completed and Apply recorded.

Evidence: `D:/pytxo-native-acceptance/astra-20261003-final/native-receipt.json`,
plus onboarding, plan, running, stale-refusal, applied and History PNGs. This run
used unsigned MSI
`F1D211C3F5D36B7D990C9312A30EE7E8ADB67C6452AD3706F4FEA23B33AC519C`, EXE
`C8E9D0AEBF26B773C253402691263E18671D6EB39597DF6249113EE7201CFAFF`.
It was an administrative extraction, not an installed upgrade, at actual Windows
125% / 120 DPI and a 1280x800 logical viewport. This run predates the later
List/Output refinements. Host-filesystem and network isolation remain advisory.

### Last native-tested artifact

Local MSI: `D:/pytxo-native-acceptance/astra-20261003-refined/Pytxo Desktop_1.2.2_x64_en-US.msi`.
SHA-256 `75B64A70914A7AE266C92767A16203324F6B17294758AA502BFC28F3DAF44FF9`.
Payload EXE SHA-256
`A5FCFFD1283E6902F946053A3176EFBD0A6F1BD926385D8868AFA2872ED2A617`.
Both are unsigned. `npm run build:msi` passed and verified all four current
hashed JS/CSS assets in the executable; the 452 selected build-input hashes
remained unchanged. `source-manifest.json` sits beside the staged MSI.

The exact extracted payload reopened a copy of the previous isolated state.
Native History retained the completed run and Apply record; List displayed the
saved requests; readable output reconstructed `risk is found.` while Raw text
still contained the original cursor escape and repeated character. The 1280x800
logical viewport at actual 125% DPI had no horizontal overflow. Native window
inspection and screenshots passed. See `ui-smoke-receipt.json`,
`list-and-output.png` and `list-full.png` in that evidence directory.
This is retained-state **extracted-payload** proof, not installed-upgrade proof
or a fresh agent dispatch on the rebuilt artifact.

The MSI runtime audit also passed: its one packaged PE has no unprovided
versioned MSVC runtime imports (`runtime-audit/result.json`). This artifact
predates the later dependency, accessibility and CI-boundary changes. Do not
transfer its native evidence to the new frozen candidate. The private hosted
build must establish its own artifact identity; installed updater acceptance
remains separate even when a `.sig` exists.

### Silent film

`apps/demo-video/out/pytxo-beta-silent.mp4`, SHA-256
`82B8894859B9253A673FF1135F30DF1C91536C8E7A54C0DB53485E42D589C931`.
Editable source is `apps/demo-video/src/beta/`. The film uses large type,
restrained movement and the recorded October 2 fleet. Its on-screen disclosure
says edited sequence / native stills. Four concurrent workers followed by two
ordered tasks are shown accurately, including the two no-change workers.
It is not a fresh fleet run, continuous capture, a performance benchmark or a
published-release announcement. Silence is Matt's explicit October 3 choice.

### Private CI follow-up

Private draft PR [#32](https://github.com/Pytxo-dev/pytxo/pull/32) was opened at
`d2c7702`, with main unchanged at `9139a908`. Its first hosted run exposed the
same unused Windows/test-only import on Linux and macOS. The import now matches
the callers' `cfg` conditions; no runtime path, permission profile, execution
domain or feature default changed. Scope is compilation of the existing Orbit
local qualification code within one repository's execution domain, not a new
execution authorization.

The next macOS pass exposed unused preview geometry outside the Windows renderer.
Bounds validation now runs in the authorized UI-thread IPC closure before backend
selection, preserving Windows hide-on-invalid and stale-revision behavior. Other
platforms still return the same unsupported-preview error. No lint suppression or
test relaxation was added; independent review found no boundary change.

CI then exposed three Windows-only consent test imports; their imports now share
the tests' existing platform guard. Once macOS reached execution, stale supervisor
recovery failed because a missing process was indistinguishable from a failed
`ps` query. The non-Linux Unix fallback now confirms absence using a signal-zero
probe and accepts only `ESRCH`; permission errors remain uncertainty. Checked PID
conversion cannot accidentally select a process group. New native tests cover
live, out-of-range and reaped PIDs plus permission-error classification. Scope is
the existing owner-identity primitive for all profiles, exercised by Orbit
single-repository recovery; no authority or execution domain is expanded.

Later hosted execution exposed unreaped Unix children blocking Stop and a
fast-command registration race when termination was conflated with identity.
Creation-token capture now retains unreaped Unix identities; Stop and recovery
separately require liveness. Only zombie state confirms termination on macOS/BSD;
Linux also requires exactly one remaining thread in the same `/proc` snapshot.
Darwin's halted/unclassified states remain possibly live, never confirmed absent.
Windows behavior and Store recovery guards are unchanged. Local parser and
dependency-outcome tests pass; native Unix execution remains a hosted-CI gate.
The shell smoke now drains dry-run stdout, preventing early grep termination
from breaking the CLI pipe without suppressing producer or assertion failures.

A fresh Rust audit also identified RUSTSEC-2026-0285. The lock now uses rustls
0.23.45 and its compatible webpki 0.103.15 dependency. Audit reports zero blocking
vulnerabilities under the unchanged documented exceptions in `.cargo/audit.toml`;
unmaintained/unsound/yanked warnings remain visible. The patched local workspace
rerun passed 1,114 tests with 16 ignored; match subsequent affected checks and
hosted results to the PR head before acceptance. The demo's fast-uri 3.1.8 and
js-yaml 4.3.2 patches produce a clean full npm audit; source and evidence validation
pass. The exported film's bytes are unchanged. These patches require a new
candidate build; the first PR head and all older MSIs lack this new Rust lock.

Independent review of eight decoded final film frames, current source, props and
asset hashes found no visible private identifiers, unsupported counts or important
text overlap. This does not grant publication or vendor approval. The default-main
Dependabot inventory is not the candidate audit: all 12 critical default alerts
are fixed/removed in candidate source. Retired `apps/docs` still has 26 listed
high-severity alerts and is explicitly outside deployment; it must not be revived
or deployed without remediation.

### Remaining public-release gates

1. Final-artifact mixed-CLI rehearsal. The new native onboarding reported five
   agents ready, including OpenCode, but readiness is not proof of successful
   edits. Claude quota is exhausted per Matt; no credentials or accounts were
   changed to bypass it. Six tasks across five vendors in 4 -> 1 -> 1 steps must
   not be described as six vendors or six simultaneous successful editors.
2. Signed candidate, installed updater download/install/restart proof, and clean
   install plus retained-data upgrade of that exact artifact.
3. Actual native 150%/200% DPI acceptance. Browser viewport/text-zoom checks and
   the 125% native run do not discharge this gate.
4. Hosted CI on the frozen candidate. Private branch/PR and CI are authorized;
   their run results must be verified before treating this gate as passed.
5. Matching final-build launch footage and independent privacy/brand review.
   The silent historical-stills film is a local delivery, not that footage gate.
6. Exact publication/deployment approval. The approved private source PR and
   private CI artifacts do not authorize a merge, deployment, tag or release.

## October 3 — positioning, Desktop and website polish, terminal quick start

**Decision: release candidate pending native acceptance (below) and the
publication gates.** Local commits only; nothing pushed.

Matt's direction (2026-10-02): use the Antigravity logo; make Desktop, website
and positioning release-ready after studying competitors.

- **Positioning** (`docs/06-product/positioning.md`, sources checked 2026-10-02):
  running agents side by side is now common (Conductor, Superset, Orca,
  BridgeMind, vendor apps), so Pytxo leads with one request split across agents,
  checks Pytxo runs itself, and an exact, stale-safe Apply. README, website,
  docs and onboarding follow it.
- **Desktop:** Review opens on a line diff (Myers, computed from the same exact
  bytes; edit cap falls back to Before & after) with the writing agent per file;
  internal terms removed from Review, History, Approvals, onboarding and New
  work; "wave" is "step" everywhere; team picker above Build plan; compact
  waiting fleet cards; small plans fit the canvas; decision action no longer a
  second primary; native caption buttons; compact agent rows with logos.
- **Website:** new homepage (outcome-first hero with the Antigravity mark, how it
  works with current captures, a category comparison, the recorded Oct 2 run as
  proof with its limits, requirements, practical FAQ); evidence page adds the
  fleet record; new docs guide "One job, several agents"; setup/install/
  troubleshooting no longer assume one Codex worker; Conductor and BridgeMind
  comparisons refreshed; release copy flips on `CANDIDATE_PUBLISHED`.
- **Terminal shell:** opens on a quick start; plain status; corrected help (the
  planner-flag note was wrong).
- **Antigravity mark:** public use per Matt; no Google approval on file
  (`apps/desktop/public/ade/PROVENANCE.md`).

Checks on the final source: see the native acceptance and gate table below.

## October 2 — native six-agent fleet run, five vendors, Review → stale → Apply

**Decision: NOT READY for public Beta.** The mixed-CLI fleet works end to end
natively; the run exposed defects now fixed in source but not yet in a rebuilt
MSI, and two external blockers remain. Local commits only; nothing pushed.

Native run (MSI `0D20D3B8EB5EFB8D5C28A8060F6EE55E0CE9289106FEFD4FB5E3095CDCCECF29`,
EXE `40BB45D26175882C184DFF2B7E676320D240CFB9BDAA0EEE810A6D700B1171D4`,
unsigned; extracted payload, isolated `PYTXO_HOME`/trust store/WebView2, CDP;
folder chosen through the real OS dialog): `docs/demo/fleet` fixture, one
request, plan 6 tasks in 4 → 1 → 1 waves across OpenAI Codex, Claude Code,
Cursor Agent, OpenCode, Antigravity and Codex. Four workers ran concurrently on
the fleet board. All six exited 0 with task checks passing (14 m 28 s); combined
checks passed; Review listed 7 files, each named with the CLI that prepared it
(Codex 3, Cursor Agent 2, Claude Code 2). An unrelated `operator-note.txt`
made Apply refuse as stale; after removing it and refreshing (digest
`2c6b6cac…957b`), Apply wrote exactly the 7 paths, the ledger records
`applied`, and the fixture's `npm test` passed 10/10 (baseline 3). Evidence and
ledger export: `D:/pytxo-native-acceptance/fleet-20261002b`.

Defects found in that run and fixed since (tests cover each):

- A finished worker read as running until the whole plan returned; workers now
  settle when they exit (`agent-exit`).
- OpenCode and Antigravity exited 0 without editing anything. OpenCode could
  not find its OpenRouter model because workers do not receive host env keys;
  Desktop called it connected anyway. Readiness now counts only stored
  credentials. Antigravity's headless mode auto-denied a command and gave up;
  the task handoff now tells workers Pytxo runs the checks, so they edit
  directly. The fleet board marks a passing worker that prepared no files as
  "No changes".
- Onboarding said to start with Codex and listed Gemini CLI instead of
  Antigravity; a picked folder could not be changed.
- Fleet cards truncated vendor names at native width.
- A stale refusal repeated the raw runner error under the stale status.

Also fixed after the run: recorded worker output was merged into one line by
every reader (workers record one event per line) and stored twice per worker.

Rebuilt candidate: MSI `F65B09E5025BFD2B63ED217E8F1983E3E33FDC00574EB1BF2C546E42FB27F3ED`,
EXE `FB5A6461FAED6706BA90D5970CE064BC77A1BCDA7E06B5E5E4FC6D3CB1036D36` (unsigned),
from `23b77ce`. On it, the scripted capture (`docs/demo/fleet/capture`) drove
onboarding through the real folder dialog to a Ready 6-task / 3-step plan with
continuous screencast footage (`D:/pytxo-native-acceptance/fleet-20261002d`);
no agents ran. The new onboarding copy, Beta agent list and folder change were
confirmed natively. On a cold first launch the OpenCode readiness probe can hit
its 10 s limit and read "Session status unavailable — recheck"; Check again
recovers.

A draft of the fleet film renders from the ledger (`apps/demo-video`,
`PytxoFleetFilm`, 52 s); it uses the run's native stills until the final run is
captured.

Blockers before the film run: store the OpenRouter key in OpenCode
(`opencode auth login`; the OpenCode Zen credential has no funds and its free
tier refuses outside OpenCode), then rebuild the MSI and repeat this run.
Antigravity's public mark still needs Google's compatibility-use approval or
removal (see `apps/desktop/public/ade/PROVENANCE.md`).

## October 1 — consolidated source, full local gates, native Codex Review → Apply

**Decision: NOT READY for public Beta; locally verified candidate.** Source is
branch `mbcz/pytxo-beta-release-4f320c`: `1c01532` imports the Beta work that
existed only uncommitted in the main checkout (unchanged), then this session's
fixes. Local commits only; nothing pushed, released or published.

Final candidate: MSI `C751A8025941FA8D6E1E63903CB89182279008DDFA87CF75817C789986FAA7BF`,
payload EXE `2E4C908DBF3063C7947984CC017FF06A816339F233A3600836E527BB52C2213B`
(unsigned). Its native smoke (fresh onboarding → guided example → 3-step Ready
plan, word-boundary saved title, clean example `git status`) passed. The full
mission below ran on an earlier build (`2024CA88…`) that differs only by the
saved-title and stale-review copy and the example `.gitignore`.

Native mission (extracted payload, isolated `PYTXO_HOME`/trust store/WebView2,
CDP, Codex CLI 0.159.3 after the host upgrade): guided example, Orbit,
`projfs-sparse-copy-v2`, one worker, three sequential tasks. All three exited 0
with task checks passing; combined checks passed; Review showed exactly
README.md, `src/risk-policy.mjs`, `test/risk-policy.test.mjs` (digest
`7f0f908e…`). An unrelated untracked `operator-note.txt` added after
preparation made Apply refuse as stale (whole-project freshness, by design) with
nothing written. Refresh produced `c504395c…`; Apply then committed exactly the
three paths, the operator file survived, the fixture's `node --test` passed
40/40 (baseline 2), and History shows Completed / Apply recorded with one
committed attempt. Evidence: `D:/pytxo-native-acceptance/run-20261001-d`.

Additional fixes from that run: saved-request titles end on a whole word with
"…" (were cut mid-word, e.g. "…changes i"); the stale message no longer claims
"affected checkout paths changed" when any project file changed.

Fixes made and covered:

- Planner: the guided example's own first mission failed planning because
  `` `README.md`. `` kept a backtick after the sentence period was stripped.
  Trailing prose punctuation is now trimmed in any order (regression test
  reproduces the shipped mission); the error no longer prints a double period.
- Failed runs now name the failed task and the agent's last reported error via
  a new `agent_failure_hint` IPC (ConPTY wrap-aware, bounded to 240 chars,
  advisory only). Previously Work said only "Open details", and Details did not
  contain the cause.
- A starting run no longer shows "Checks could not be loaded: … no
  review/apply contract".
- Review: blocked-Apply reason shown once beside Apply (was duplicated);
  file-row status badges aligned; sentence-case digest disclosure; one-line
  "Prepared files" label. Work: empty-state heading no longer squeezed into the
  glyph column. Setup rail fits "Agents & permissions". Sidebar account label
  fits. Shortcut hints show `Ctrl` on Windows instead of `Ctrl/⌘`. Onboarding
  Ready step no longer says "choose a project" after one was chosen. "New run"
  copy updated to "New work" in the guided example and first-mission guide.
- Clippy (Rust 1.98 `chunks_exact_to_as_chunks`) and rustfmt failures fixed;
  routing-attempts e2e selects History rows by `data-run-id`.
- `build-windows-desktop.mjs` honors `CARGO_TARGET_DIR`.

Verification (all exit 0 unless noted):

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | 1072 pass; 30 runner/orchestrate failures were the unset `PYTXO_TEST_ATTEMPT_HOST` pin. With CI's pinned helper: runner+orchestrate 503/503 |
| Planner / desktop-native (`pytxo-desktop`) | 36/36, 74/74 |
| Desktop `npm run check` (svelte-check + stylelint) | 0 errors, 0 warnings |
| Desktop production-preview e2e | 394/394 on the final source (includes new failure-cause/title specs); dev suite 1/1 |
| Desktop `@performance` | 4/4 idle; one 53 ms long task only under concurrent build load |
| Web lint, 167 links, 27 product assets, production build (63 pages) | PASS |
| Web e2e (system Chrome) | 32/32 (sitemap spec passed on idle retry) |
| `npm run build:msi` | PASS; final MSI `C751A802…A7BF`; unsigned |

Failure path (earlier build, Codex CLI 0.144.4): the host `~/.codex` model
`gpt-6.1-sol` was rejected for the ChatGPT account; Work showed "mission-0
failed. The agent reported: The 'gpt-6.1-sol' model is not supported when using
Codex with a ChatGPT account." No package was prepared. Upgrading the CLI to
0.159.3 resolved it without changing the Codex config.

Remaining gates: hosted CI on this branch (push not yet authorized), signed
build, clean install/upgrade of the final artifact, multi-DPI native pass,
matching media, publication approval. The Jev routing experiment is still in
the source (see September 24 note). The guided example now ignores `.pytxo/`.

## September 24 — beta release cut audit

**Decision: NOT READY for public Beta.** The approved first release path remains
Windows Desktop, Codex, one repository, Orbit, and one worker. The broader Jev
routing work is experimental and is not required to prove that path.

The exact R8 MSI (`D:/pytxo-beta-lab/cockpit-beta-candidate-r8-20260921/`,
SHA-256 `D7CA3875D477525C264070B75C8AD4BD032323425917A89545C081215701A58F`)
has meaningful local evidence: 358/358 Desktop functional tests, 11/11 dense
performance executions, clean elevated Windows installation, retained-data
upgrade, and a real Codex Work → Review → Apply run with 7/7 post-Apply tests.
The R8 candidate receipt, not the older R8 paragraph below, is authoritative
for those later September 22 installation and mission observations. The MSI
hash was rechecked September 24.

R8 is unsigned. Its signed in-app forward download/install/restart, native
150%/200% acceptance at the declared logical minimum, hosted candidate build,
and matching continuous launch footage remain unverified. R8 is also not a
frozen source release: the checkout has extensive uncommitted work, and three
files named in R8's source receipt (`FlowScreen.svelte`,
`SetupStepWelcome.svelte`, `WorkActive.svelte`) now have different hashes.
Do not attach R8 runtime receipts to a rebuilt binary or to the current source.

Read-only GitHub inspection on September 24 found draft PR #31 still at
`eb5f73d` with its last green CI run from September 8. The public latest
release is still v1.2.1. `.github/workflows/desktop-candidate.yml` remains
local and undispatched; its Tauri signing secrets are configured by name,
but neither their contents nor a signed artifact have been verified. The
current release inventory/version tests passed 26/26, candidate/published
version parity passed, and `actionlint` passed for the build-only workflow.
The normal local `cargo --version` command currently fails because the stable
toolchain's Cargo component is unavailable, so no full current-source Rust
suite was claimed in this audit. These are static checks, not hosted or native
update acceptance.

Next release-critical sequence: isolate and freeze the approved beta source
without the unfinished routing experiment; run hosted CI and a private signed
candidate build on that exact source; verify its installed forward update and
remaining supported-size native journey; capture matching real footage; then
review the exact publication payload and obtain release approval. A public
release requires independent download and updater-manifest verification.

## September 21 — R3 motion-design storyboard

The exact R3 beta candidate now has a separate 24-second Remotion review
storyboard. It uses four hash-locked native stills from the verified disposable
Codex Work → Review → Apply mission, two bounded masks over the disposable host
path that share the screenshot transform, a persistent still-storyboard disclosure,
and 12-frame scene dissolves.
Asset identity/dimensions, TypeScript, composition discovery, H.264/yuv420p/BT.709
metadata, full decode, contact-sheet inspection, boundary-frame inspection and a
complete 1× local agent playback pass.
Output SHA-256 is
`d57c73c2ecfaa9e31abd05c10da1bbd058e845b2cd527866a62490211aa9d683`;
receipt is `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/r3-storyboard-receipt.json`.
The earlier `d8f81a68…` output was superseded after source review found transform-
relative privacy-mask drift.

This closes motion-direction preparation only. The film contains no continuous
R3 footage or pointer motion, is sourced at 1282×802, and is not a launch master.
Fresh privacy-reviewed 3840×2160 native capture, normal-speed human playback and
publication review remain open. Candidate receipt SHA-256 after binding this
artifact is `1655a1b0a02b087c487a896d4b5df6644103ad80ad6272262f06881f62637001`.

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

## September 20 — native diagnostic Work / Review / Apply PASS

Registry diagnostics build (MSI B1E2608F…9B05FE52, extracted executable
A16D5FE4…06A694D9) completed a real Codex Orbit run with one worker, one test
file and npm test verification. Native exact Review and Apply completed for
run 2c585a7f-3e5f-4ba7-9521-3ca48045b8ab. All seven destination inventory
digests match the prepared candidate; independent destination tests pass 5/5.
Evidence: target/beta-regression-repair-20260919/native-diagnostic-apply-receipt.json.
Full runner suite passes 151 tests; build snapshot has zero subsequent drift.

This advances native workflow evidence but does not close beta readiness.
The earlier Windows sharing violation did not recur; its cause is still open,
and error-context changes are not a fix. Native stale/refreshed review, recovery,
DPI, clean installation and installed updater acceptance remain unverified.
No publication, installation or public update-feed mutation occurred.

## September 20 — native acceptance resumed

**Real task acceptance FAILED:** run `2aa33320-721a-4687-8f97-9715b9d9a28f`
executed Codex and produced three new tests. Independent isolated-workspace
`node --test` passes 5/5; original repository remains unchanged. Pytxo's own
verification recorded `io: The process cannot access the file because it is
being used by another process. (os error 32)`. Review correctly remains
unavailable without a prepared candidate. Exact failing I/O operation still
requires diagnosis. The agent also encountered a distinct sandbox spawn EPERM.

Native-control blocker below is superseded: Computer Use now targets the final
MSI-extracted executable. Native Setup reports v1.2.2 and a manual update check
visibly completes with no newer version. This is launch/no-update-path evidence,
not installed upgrade or clean-machine acceptance. Codex readiness and a single
Orbit task plan are observed; real execution/Review/Apply acceptance is underway.
No installer or public feed changes were made. Release remains NOT READY.

## September 19 — final local candidate packaged; native acceptance blocked

Final MSI build session 58021 completed exit 0 (release compile 4m56s).
All 477 final snapshot entries match after build. Staged installer-only inventory
and SHA256 verification PASS. Artifact and receipt:

- `target/beta-regression-repair-20260919/final-installer/pytxo-desktop-windows-x64.msi`
- `target/beta-regression-repair-20260919/final-candidate-receipt.json`
- MSI SHA256: `10CEECD4C04739A1206C693EDA4DF8446ED9BFD2707D64FD2E83D6DA479EE547`
- Extracted EXE SHA256: `F70A21A82FB9A03961DC58FCDA35277CE4FBFB0DC1E64EA8EA9B2F661FDB4BC4`

Read-only final MSI and executable inspection both report 1.2.2. MSI retains
ALLUSERS=1 and is Authenticode unsigned. This is not updater-signature evidence.

| Acceptance requirement | Current evidence / verdict |
| --- | --- |
| Desktop workflow/UI and failure states | 307 browser tests PASS; relevant screenshots inspected; fixture scope |
| Fresh public interface previews | 25 capture checks PASS; source/docs/site parity PASS; 15 affected website tests PASS |
| Website/docs/SEO | Production build and 23-test full browser suite PASS, 192 internal links PASS; local only |
| Rust behavior and static checks | Locked workspace tests PASS; subsequent formatting-only repair; final fmt and all-target workspace Clippy PASS |
| Exact Windows package | Final MSI build/inventory/hashes and zero source drift PASS; dirty source snapshot, no commit |
| Native real-agent job and reviewed Apply | NOT RUN on final installed artifact |
| Native stale refusal, interrupted Apply/recovery, DPI | NOT RUN on final installed artifact |
| Clean Windows install and installed upgrade | NOT RUN; signing/feed and actual UAC/upgrade outcome unverified |
| Real demonstration / pilot evidence | Still depends on accepted native workflow; generated/fixture images are not substitutes |

The same native-control limitation has persisted across more than three goal
continuations. Native apps are unavailable through this session's computer-use
surface; no permitted native-control tool was found. The request for native
access or guided manual observations remains unanswered. Now that independent
source/browser/package work is complete, further acceptance requires that
external prerequisite. Goal is blocked, not complete; no publication is implied.
Resume from `NATIVE_ACCEPTANCE.md` in the evidence folder when access/evidence is
available. Do not restart UI redesign or replay passing suites without a change.

## September 19 — full workspace checks passed; final package rebuilding

`cargo test --workspace --locked -j 2` completed exit 0 (session 91088 terminal).
Then repaired only rustfmt wrapping in three ipc_voice updater guards and one
store test block. `cargo fmt --all -- --check` now passes. Post-format
`cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings` passed
(1m09s, session 92974 terminal). Logs are in the current evidence folder.

Captured `candidate-final-source-before.json` (477 files; broad snapshot includes
tests/captures as well as production inputs) and started the final MSI rebuild,
session 58021, `candidate-final-msi-build.log`. It is not accepted until build
completion, new MSI/payload hashes and post-build source comparison are recorded.
Earlier MSI preserved as `candidate-before-format.msi`. Its recorded hash must
not be attributed to the new output path once the rebuild replaces that file.

## September 19 — refreshed captures and broader verification

Capture generator updated to current Approvals and Agents & permissions headings.
After the initial obsolete Agents heading stopped the serial run, the final run
passed 25/25 (52.4s), refreshing 27 Desktop/docs captures, 10 website images and
the applicable demo stills. Browser fixture labeling remains; these are not
recorded native missions. Asset parity and 192 internal links PASS. Review at
960x640, Approvals and Setup captures inspected.

Added a hit-test assertion that the keyboard-focused walkthrough tab is not
covered by the sticky header, with desktop/mobile viewport captures. All four
presentation checks passed; viewport inspection confirms the earlier overlap
was caused by element screenshot framing. After captures finished, the affected
website marketing/presentation set passed 15/15 (1.2m). Hashes/receipts/logs are
in `target/beta-regression-repair-20260919/`.

The original broad candidate snapshot now differs only in capture images and
the capture test. Runtime source remains unchanged from the MSI. Full locked
workspace tests are running (session 91088). `cargo fmt --all -- --check` failed
on line wrapping at ipc_voice.rs:204/348/377 and store.rs:1109. Apply formatting
after the current test run, then bind the final source/package identity; do not
waive this failure. Full log: `workspace-format.log` in the evidence folder.

`NATIVE_ACCEPTANCE.md` in that folder identifies the MSI/payload and the remaining
actual Windows acceptance rows. Native desktop controls remain unavailable;
requested either native access or a guided manual check while independent work
continues. The candidate is not accepted and no publication occurred.

## September 19 — Windows candidate MSI built and identified

`npm run build:msi` completed exit 0 with `voice-whisper`, the explicit Windows
target and distribution CRT configuration (release compile 6m32s). Artifact:
`target/x86_64-pc-windows-msvc/release/bundle/msi/Pytxo Desktop_1.2.2_x64_en-US.msi`
(14,688,256 bytes), SHA256
`3F99F192BE637E62B532A1A9038FF36B758FD478C61C4530AD3CCC9430547209`.
All 477 snapshotted source hashes matched after the build. The checkout remains
dirty at HEAD `72879702f90f2b74eece888bb117df59608f9b56`.

Read-only MSI inspection reports ProductVersion 1.2.2 and ALLUSERS=1
(per-machine). Both MSI and executable are Authenticode unsigned. No installation
or updater signing was performed. Extracted payload version is 1.2.2; SHA256
`53F3EA0D45F8F3F19E53EEEE15DB332DB6207CF9EB9CC8CF89D4F335A298C2CC`.
It differs from the loose release executable at exactly three bytes: packaged
`MSI` versus loose `UNK` bundle marker. Native/updater acceptance must identify
the packaged payload, not silently substitute the loose binary. Evidence and
an extracted executable are under `target/beta-regression-repair-20260919/`.

Website production build passed. Initial browser suite: 19 PASS / 4 FAIL.
It exposed keyboard navigation using the selected tab rather than the focused
tab as its origin; fixed to use the event-owning tab index. Updated obsolete
walkthrough accessible names and homepage structure expectations, retaining
focus, selection, image positioning, reduced-motion and overflow checks.
Fresh post-fix production build passed; full browser rerun passed **23/23 (1.4m)**,
including docs/search, mobile/desktop navigation and crawlable SEO metadata.
Receipt and inspected walkthrough captures are preserved in the evidence folder.
Visual review also confirms the embedded Desktop captures still show the earlier
`Your coding task` layout. Asset parity verifies consistency, not freshness:
refresh the product captures from the now-tested Desktop before launch acceptance.
The desktop element capture also includes the sticky site header over its top;
use a viewport capture to assess actual tab-focus visibility before changing UI.

## September 19 — website/update guide follow-up

Desktop setup now describes the current task-first, movable-panel UI and Review
access from Work/History. Updater instructions distinguish check/download/install/
restart errors and running-version confirmation, describe the activity preflight,
and explicitly retain pending installed-upgrade acceptance. Troubleshooting links
to that section and uses the current Agents & permissions label.

192 internal links PASS; MDX generation PASS; 27 source captures, 27 docs images
and 10 marketing captures passed dimensions/distinctness/parity checks; docs
source check found 45 MDX pages. Fresh website production build is running;
these checks alone do not establish browser rendering or a deployed site.
MSI build is also still running. Interim 477-input source comparison found no
drift; final post-build comparison remains required.

## September 19 — combined browser regressions resolved

This supersedes the 286 PASS / 21 FAIL result below. The subsequent full gate,
`npx playwright test --config playwright.config.ts --grep-invert '@marketing-capture' --workers=2 --reporter=line`,
completed with **307 PASS (3.3m)**. The runner receipt is preserved in
`target/beta-regression-repair-20260919/combined-last-run.json`.

MissionDock now reveals run commands after moving/resizing docks, and WorkActive
responds to its actual pane width through container queries. This fixes Stop
visibility with a long request and bottom/right docks, including after reload.
The existing full-visibility assertions were retained. Other affected tests now
open the current inspection/worker/details disclosures and assert the current
task-first labels and beta admission rules; authority, identity and scroll checks
remain in place. The final focused set passed 24 tests; Svelte reported zero
errors/warnings and CSS lint passed. The repaired screenshot was inspected.

Release-version consistency passed for candidate 1.2.2 versus declared public
1.2.1; that script does not verify public availability. These are browser and
source checks, not packaged Windows acceptance. Combined Rust verification,
source-bound packaging, real-agent Apply/recovery, DPI and installed-upgrade
acceptance remain outstanding. Generated mockups are proposals, not runtime proof.

Follow-up combined Rust gate: `cargo test -p pytxo-core -p pytxo-store
-p pytxo-orchestrate -p pytxo-desktop --locked -j 2` completed with exit 0.
Full log: `target/beta-regression-repair-20260919/combined-rust-tests.log`.
This includes integration tests, but not every workspace crate or the packaged
voice feature set. Release inventory/version verifier tests also passed 22/22.
The candidate source manifest hashes 477 Git-known Desktop/Rust/Chroma inputs
at HEAD `72879702f90f2b74eece888bb117df59608f9b56`, including dirty files; it is
not a clean-source commit. `npm run build:msi` has started with two Cargo jobs;
result remains pending in `candidate-msi-build.log` in the same evidence folder.

## September 19 — launch-path audit, docs and combined regression gate

The only non-Flow managed-run IPC found is `dispatch_run_cmd`, used by the
development-only legacy Deck. Its UI was already hidden from production; the
native endpoint now refuses builds without debug assertions before reading
configuration or dispatching. Packaged rejection still requires release/native
acceptance. Standalone Flow uses the same scoped backend as the main shell.
Fleet commands exposed here list/status existing work; they do not launch it.
Manual terminals and external CLI commands remain outside this Desktop beta
admission policy. Existing Review/Apply/recovery access is preserved.

Desktop setup, first mission, Desktop concepts and Review/Apply docs now mark
the 1.2.2 scope as an unpublished candidate, distinguish it from general CLI
capabilities, explain failed detection/sign-in recovery, and separate runtime
approvals from reviewed repository Apply. No download version was promoted.
Docs validation: 191 internal links across 114 web source files PASS;
`npx fumadocs-mdx` PASS. Native `cargo clippy -p pytxo-desktop --lib -- -D warnings`
PASS; scoped diff check clean. No website deployment or native installation.

**Combined browser gate is failing:** `npm run e2e:release` ran 307 tests using
the configured four workers: 286 PASS, 21 FAIL (6.2m). The attempted npm worker/
reporter overrides were ignored; no process was restarted during this run.
Evidence is retained in `target/beta-combined-browser-20260919/`.

Failure work queue:
- Agent identity, epistemic evidence, local preview and workspace-fit tests reach
  controls before opening Worker records / Inspection tools; inspect and update
  journeys while retaining identity, evidence, renderer-lifecycle and scroll checks.
- First-use/navigation tests assert obsolete generic headings/status placement
  or hidden verification commands; reconcile against the approved task-first UI.
- Shell onboarding copy and alternate-CLI dispatch assumptions need alignment
  with the implemented beta gate, without weakening authority assertions.
- Long mission plus bottom dock reports Stop only 70% in the viewport: investigate
  and repair the layout, retaining the full-visibility assertion.
- History scroll tests reach candidate details while their disclosure is closed;
  preserve long/short scroll-ownership assertions when adjusting the journey.

These failures are not waived as test drift. Finish triage, repair actual defects,
verify the affected journeys and rerun the combined gate before packaging a
candidate. Packaged/native/installed-upgrade acceptance remains separately open.

## September 19 — Desktop Flow beta admission

Desktop IPC now calls scoped orchestration preview/dispatch entry points.
New-work Flow admits Codex, one worker, local PTY, and Orbit for both the domain
and every resolved worker profile. Existing single-repository/root and trust
checks still apply. Unsupported choices produce plan blockers and are checked
again from the execution config snapshot before the draft is claimed or any
run reserved. General CLI Flow remains available under its existing policies;
saved configurations are not rewritten. Additional-agent selections remain
inspectable but cannot start a Desktop beta run.

Scope: this is Desktop Flow admission, not a universal sandbox restriction or
removal of other adapters. It does not constrain separately invoked CLI commands,
manual terminals or legacy non-Flow entry points. Windows remains the candidate
acceptance target; this policy does not assert that a package has been accepted.

Verification:
- `cargo test -p pytxo-orchestrate --test flow -- --nocapture`: 16 PASS. Final
  `desktop_beta` filtered rerun: 3 PASS after improving dispatch error wording.
  Tests cover supported preview, unsupported profiles/backends, preserved config,
  and rejection of general ready plans with the draft still unclaimed.
- `cargo test -p pytxo-orchestrate --lib desktop_beta -- --nocapture`: 1 PASS,
  covering per-worker profile overrides and absent/non-Codex adapter selections.
- `cargo clippy -p pytxo-desktop --lib -- -D warnings`: PASS, including IPC wiring.
- `npx playwright test e2e/beta-admission.spec.ts e2e/beta-workflow.spec.ts
  e2e/menu-workflow.spec.ts --workers=1 --reporter=line`: 26 PASS (57.2s).
  Final admission capture rerun: 1 PASS (24.2s). Selecting Codex alone cannot
  authorize an old blocked plan; rebuilding is required. Blocker/action screenshot
  inspected. `npm run check`: zero Svelte errors/warnings; CSS lint PASS.
- Evidence: `target/beta-admission-20260919/` (runner receipts, source hashes,
  screenshot). Initial test fixture misspelled `deep_space`; corrected to the
  actual serialized profile before the successful full Flow run.

Remaining: audit non-Flow Desktop launch exposure, align beta docs with the
admission contract, freeze/build the combined candidate and perform packaged
real-job, Apply/recovery, native DPI and installed-upgrade acceptance.

## September 19 — agent setup failure and recovery

Reproduced an onboarding bug: after a successful detection, a failed recheck
kept the previous connected-session label and ready Continue action. The new
regression failed on the stale `ChatGPT connected` label before repair.
`SetupStepAgents.svelte` now withdraws prior detection data when rechecking,
disables progression during detection/sign-in launch, and offers an explicit
`Set up agents later` path when readiness is absent. Codex remains first in
the short onboarding list. Failed detection does not claim zero installed agents.

Preview-only fixtures now cover missing agents, Codex signed out, unknown
session, detection failure and sign-in launch failure. Tests verify launch
success alone does not promote readiness; successful recheck is required before
Setup offers Use in new work. No native authentication behavior was changed.

Verification from `apps/desktop`:
- `npx playwright test e2e/agent-setup-recovery.spec.ts e2e/flat-onboarding.spec.ts
  e2e/beta-first-use.spec.ts --workers=1 --reporter=line`: 10 PASS (35.5s).
- Final recovery suite after adding the compact skip-journey check and captures:
  `npx playwright test e2e/agent-setup-recovery.spec.ts --workers=1
  --reporter=line`: 5 PASS (23.3s).
- `npm run check`: zero Svelte errors/warnings; CSS lint PASS. Scoped diff check
  clean. Error-state and 860x560 no-agent screenshots inspected; footer actions
  remain visible while the agent list scrolls. Receipts, captures and source
  hashes retained in `target/agent-setup-recovery-20260919/`.

This proves browser behavior with controlled backend failures, not real vendor
login, PATH discovery or native Windows acceptance. Full candidate verification,
packaging, actual job/Apply/recovery and installed update gates remain open.

## September 19 — beta first-use exposure verified in browser

Setup leads with Codex and separates installation from vendor-owned sign-in.
Additional agents and editor/cloud integrations expand on demand. The current
default permission profile stays visible while its alternatives are disclosed
explicitly. Existing non-default preferences are preserved. New work groups
Codex as the beta starting point and retains deliberate alternate-agent choices;
this is focused presentation, not a backend Codex-only enforcement gate.

Verification on the current worktree:
- `npx playwright test e2e/beta-first-use.spec.ts e2e/harness-catalog.spec.ts
  e2e/setup-scroll-ownership.spec.ts e2e/menu-workflow.spec.ts --workers=1
  --reporter=line`: 23 PASS (1.0m). Covers Setup-to-New-work selection, retained
  Galaxy preference, all 14 catalog entries, dark/light controls and stationary
  Setup navigation at 1920/1280/860 widths.
- `npx playwright test e2e/shell.spec.ts --grep 'Agents reports' --workers=1
  --reporter=line`: 1 PASS (28.9s), preserving vendor-session distinctions.
- `npm run check`: zero Svelte errors/warnings; CSS lint PASS.
- Inspected fresh 1280x800 and 860x800 screenshots. Codex status and its primary
  task action remain in view; long sections scroll within the content pane.
  Screenshots, source hashes and runner receipts: `target/beta-first-use-20260919/`.

The initial five browser failures were test-navigation/setup defects: an exact
display-name mismatch, localStorage access on about:blank and two attempts to
reach a now-collapsed agent without expanding its disclosure. Corrections retain
the original behavior/layout assertions. Native DPI, missing-agent/sign-in-error
first use, real job lifecycle, MSI/UAC and installed upgrade remain unaccepted.
No installer, product commit, deployment or publication occurred.

## September 19 — cooperative exclusive upgrade handoff

Supersedes the earlier observation-only limitation for participating current-source
processes in the same user catalog. `UpgradeGuard` uses shared/exclusive OS file
locks at the catalog home's `.pytxo/upgrade.lock`. Blocking and detached runs,
fleets, Flow dispatch, reviewed Apply, review refresh/discard/recovery and legacy
workspace commit participate. Detached run supervisors retain the shared lease
after dispatch returns. All permission profiles and registered domains retain
their existing authority; this adds upgrade admission, not agent capabilities.

Desktop acquires an exclusive native handoff after download, rechecks persisted
and in-process activity under that lock, and retains ownership through install
and relaunch. A per-attempt token prevents stale release of a different handoff;
only the main window can begin/finish it. Failure releases admission. Terminal
creation and voice start/resume/finish participate, closing their local check/start
races. OS exit releases ownership without deleting the durable lock file.

Verification:
- Core upgrade-guard tests PASS, including real child-process shared/exclusive
  contention and abrupt process exit bypassing Rust Drop.
- Orchestration `upgrade_admission`, `dispatch_lifecycle`, `fleet_run` and
  `flow_concurrency` tests PASS. A final targeted dispatch rerun verifies the
  detached lease is still held immediately after return. Upgrade refusal covers
  Flow and run dispatch plus repository mutation entry points before effects.
- Desktop library suite: 44 PASS. Final `cargo clippy -p pytxo-desktop --lib --
  -D warnings`: PASS after Flow integration.
- Updater Playwright/controller suite: 9 PASS (23.5s), including exclusive
  handoff refusal, ownership during install/relaunch and release on failure.
  Svelte/CSS checks PASS with zero errors/warnings.

These are source/native-library and browser/controller checks, not installed-app
acceptance. Older binaries that do not implement this cooperative lock, processes
using another user/catalog home, and arbitrary external commands are outside its
admission contract; retained preflight can detect their recorded activity but
cannot prevent them starting later. Native updater UI, actual MSI/UAC, old-to-new
upgrade and packaged real-job acceptance remain open. No product commit, release,
installer invocation or deployment occurred; Git activity in test logs is confined
to temporary fixture repositories.

## September 19 — update preflight and next-launch version receipt

The shared updater now invokes native preflight before downloading, immediately
before installing, and before relaunch. `update_safety.rs` checks every registered
domain with an unbounded store query for starting/running runs and preparation,
Apply or recovery-required states. Missing/corrupt/incompatible domain databases
refuse the operation rather than creating a new empty database. Fleet activity,
open workspace terminals and recording/paused/transcribing voice sessions also
refuse upgrade. This applies across registered execution domains and permission
profiles without changing run or Apply authority.

A local update-request receipt is written before installer handoff. A new
controller launch compares the running binary version with that requested
version independently of feed availability. Matching version reports that this
launch runs the requested version; a mismatch keeps completion unconfirmed.
Storage failure refuses installer handoff. Neither download Finished events nor
installer return values are treated as proof that the upgrade completed.

Correction to the earlier retry note: inspected native updater 2.10.1
`commands.rs` retains the downloaded byte resource when installation fails and
closes it only on successful return. Installer/preflight retries now reuse those
verified bytes; failed downloads retry download. This avoids orphaning the prior
SDK resource. A fresh preflight still precedes every attempt.

Current verification:
- `npm --prefix apps/desktop run check`: PASS, zero Svelte errors/warnings and
  CSS lint PASS.
- `npx playwright test e2e/update-controller.spec.ts --workers=1 --reporter=line`
  from Desktop: final 8 PASS (21.9s). These use injected lifecycle adapters plus the
  existing browser Setup fixture, not an actual native update.
- `cargo test -p pytxo-store --lib`: 25 PASS, including read-only observation,
  preparation/Apply/recovery states and fleet lifecycle counts.
- `cargo test -p pytxo-desktop --lib`: 44 PASS, including missing/corrupt database
  refusal and an older active run beyond the normal history window.
- `cargo clippy -p pytxo-desktop --lib -- -D warnings`: PASS.

The preflight is an observation, **not an exclusive upgrade lock**. A separate
process (or another command) can start work after it; race-free handoff remains
open and must not be claimed complete. Native progress/error rendering, MSI/UAC,
real upgrade/next-launch receipt and full beta/package acceptance remain unproven.
The receipt confirms a version string, not artifact authenticity or source hash;
package signature enforcement remains the native updater's responsibility.
No installer, commit, release or deployment was performed.

## September 19 — shared updater lifecycle, partial reliability repair

Banner and Setup now share one controller and one controls component. Updates
live in General and Settings search follows that location. Check errors are
visible rather than classified as benign; install errors remain visible beside
the available version. Native version comes from `getVersion`; a null channel
response says no newer version is available, not that this is the latest release.
Checks have a 30-second timeout and downloads a 120-second timeout. Overlapping
operations are refused, download byte progress is exposed, and replaced update
resources are closed. Download completion events do not authorize installation
until the download promise succeeds. Installer failure triggers fresh download
on retry; restart failure retries only restart. No success/version promotion is
inferred from installer handoff. Manual download is offered on error.

Verification: `npm --prefix apps/desktop run check` PASS, zero Svelte warnings or
errors and CSS lint PASS. From `apps/desktop`, `npx playwright test
e2e/update-controller.spec.ts e2e/menu-workflow.spec.ts --workers=1
--reporter=line` passed 19 tests in 40.9 seconds. After a screenshot-led padding
correction exposed a global Settings flex rule, the updater container was isolated
from that rule and a compact-height/in-viewport regression was added. The final
four updater tests passed (21.6 seconds) and static checks passed again.
General Settings screenshot inspected and copied to
`target/updater-lifecycle-20260919/`. Pure controller failure tests use injected
adapters; browser verification covers discovery/search and unavailable-native UI.
They do not prove native download, UAC, MSI installation or upgrade completion.

Still required: authoritative active-work protection before install/restart,
next-launch reconciliation of the requested version, rendered native error and
progress acceptance, and installed 0.9.0/public 1.2.1/current candidate upgrade
testing. The bounded run-history snapshot is not sufficient proof that no work
is active. No native installer was invoked, and no release was published.

## September 19 — approval authority and contextual drawer

Legacy `blast.flush` requests now lead to Review rather than offering generic
Approve and apply. The generic approval shortcut refuses this request type;
opening Review leaves the request unresolved and cannot authorize Apply. Denial
no longer promises workspace deletion. Other runtime decisions retain their
existing approval path. Backend candidate identity and Apply checks are unchanged.

Approvals use a right-side drawer with a compact request list, independently
scrolling evidence and pinned decisions. The empty inbox remains compact.

Final combined verification:
- PASS: `npm --prefix apps/desktop run check`, zero Svelte errors/warnings and
  CSS lint passed.
- PASS: from `apps/desktop`, `npx playwright test e2e/approval-authority.spec.ts
  e2e/responsive-menus.spec.ts e2e/dpi.spec.ts e2e/work-glyph.spec.ts
  e2e/shell.spec.ts --grep 'approval|menus adapt|glyph|DPI' --workers=1
  --reporter=line`: 24 passed in 52.1 seconds.
- The first wider run exposed obsolete hidden-control assumptions. DPI tests
  now open Worker records and responsive menu tests open Inspection tools before
  interacting; existing geometry, focus and decision assertions are retained.
- Inspected browser captures at 1440x900 and 860x560, including scrolled evidence
  and pinned actions. Evidence copied to `target/approval-drawer-20260919/`.

This proves fixture/browser behavior only, including simulated device scale;
it does not prove native Windows DPI, packaged execution or installer behavior.
Updater reliability, broader beta acceptance and release gates remain open.
No commit, publication or deployment occurred.

## September 18 — workbench craft consolidation

Local frontend-only refinement on `codex/beta-candidate-verification`, HEAD
`72879702f90f2b74eece888bb117df59608f9b56`, preserving the inherited dirty tree.
No backend, IPC, schema, authorization, dependency, glyph-renderer, commit or
publication change was made.

Work now presents mission context and exact-run controls as one command area and
starts the execution surface about 70px earlier at the 1280×720 browser fixture.
The recorded graph, selected task and contextual inspector have clearer surface
depth; active-worker brackets, selection and keyboard focus remain independent.
Review now reads as one candidate/code/decision workspace while keeping the
wide-layout candidate map visible. History presents its accurate recorded trace
as a settled outcome surface while retaining the explicit candidate-to-Apply
identity limitation.

Evidence:
- **PASS:** `npm --prefix apps/desktop run check` after the final source and test
  changes; zero Svelte errors/warnings and CSS lint passed.
- **PASS with bounded reruns:** the selected 129-case Desktop batch initially
  passed 123 and exposed six obsolete test assumptions. Four tests now open the
  intentionally collapsed Inspection tools before operating the existing dock;
  one focuses the real heading target rather than the adjacent glyph; one tests
  scrollbar movement relative to available overflow. Final targeted reruns were
  presentation 5/5, shell focus/Stop 1/1 and scrollbar/chrome 1/1. Every selected
  case therefore has current passing evidence, although the 129 were not rerun
  together after test-only corrections.
- **PASS:** one required Impeccable detector run over the six changed frontend
  targets returned `[]`.
- **PASS — browser fixture only:** matching reduced-motion captures at 1600×1000,
  1280×720 and 860×760 are in `target/laptop-workbench-20260918/` with the
  `craft-final-` prefix. At 1280×720 the Work inspector begins at y=281.875; the
  expanded Review candidate map is 109px high and the 59px decision surface stays
  pinned below the comparison.
- **NOT RUN:** native Tauri WebView, Windows 100/150/200% scaling, packaged app,
  real runtime events and a genuine Review → Apply mission. Browser fixtures do
  not close those gates.

## September 18 — laptop-first workbench refinement

Local frontend-only follow-up on `codex/beta-candidate-verification`, HEAD
`72879702f90f2b74eece888bb117df59608f9b56`. Baseline was 255 dirty entries;
inherited work was preserved. No backend, IPC, authorization, schema, dependency,
glyph-renderer, commit or publication changes in this pass.

Work now uses the existing collapsible inspection toolbar, a tighter request
header and less node metadata. Active-worker brackets, selected surfaces and
keyboard focus remain separate. The connected inspector stays beside the graph
when content width permits; below 900px it follows tasks before candidate and
repository sections. Inspect selected task focuses and reveals that inspector.
Task IDs remain in Source details and paths in Files. Light-theme inverted action
text inherits its button foreground instead of the global strong-text colour.

Review retains the default-visible candidate map. Its relationship band and
ready-only decision area are shorter. At 1280x720 the substantial-content fixture
shows 247.5px of immediately visible comparison with a 58px decision bar.
Refusal, stale and recovery reasons remain visible; the compact eligibility
disclosure applies only when no Apply-disabled reason exists. No handlers,
identity binding or prepared-content loading logic were changed.

History uses available width and compact execution snapshots. Candidate/check
snapshots and integration records are separate; unavailable candidate-to-Apply
correspondence is visible beside them. Applied file count comes from the applied
manifest. Candidate selection exposes its own inventory rather than presenting
applied files as candidate evidence. Existing async scope and action refresh
safeguards remain intact.

Evidence in `target/laptop-workbench-20260918/`:
- PASS: final `npm --prefix apps/desktop run check`, zero Svelte errors/warnings;
  CSS lint passed (`check.log`).
- PASS: 79/79 regression cases covering History, glyph, topology, Review identity,
  refusal/recovery, scroll, themes and six new laptop/contrast/inspection checks
  (`tests.log`).
- Final History inventory change: 30/31 follow-up cases passed; the remaining
  dock test assumed the toolbar remained open after reload. Its disclosure was
  explicitly opened before exercising the unchanged layout assertions. See
  `dock-confirmation.log`: all 11/11 docking cases passed after that correction.
  Counts overlap; this is not one combined final invocation.
- The four previously outstanding tests were re-evaluated, not weakened: wheel
  tests now scroll to lower evidence with substantial code, rather than assuming
  the compact file navigator must be offscreen; evidence spacing is measured from
  its opened disclosure; voice size checks open the existing Voice input disclosure.
- Impeccable detector returned no findings on the principal changed surfaces.
  The detector is not accessibility acceptance. Computed contrast regressions pass
  in both themes. `git diff --check` passed with existing line-ending warnings.
- Matched before/final fixture captures: Work, Review and History at 1600x1000,
  1280x720 and 860x760 (Light). Additional long-code ready, stale, missing-check
  and focused Review captures are explicitly browser fixtures. Dock occupancy,
  keyboard, reduced motion, unknown/recovery and delayed selection are covered by
  the affected tests. No claims of real agent execution follow from these images.
- NOT RUN: native WebView, Windows DPI, packaged real-task acceptance. No release
  readiness upgrade or publication authorization is implied.


## September 18 — outcome-first History and bounded Work finishing

Local frontend implementation only. Baseline: `codex/beta-candidate-verification`,
HEAD `72879702f90f2b74eece888bb117df59608f9b56`. The current dirty inventory is
255 entries and includes inherited product work plus this bounded frontend slice.
The earlier inventory and selected-file hashes remain in
`target/history-work-20260918/git-before.txt` and `baseline.json`. Inherited work
was preserved; no reset, commit or external action occurred.

History now uses a searchable all-repository run navigator and a larger selected
workspace. Saved request titles lead; an honest repository fallback is used without
putting its derivation in the default reading path. Destination, execution,
integration outcome and one available next action lead the detail. Execution
snapshots, prepared candidates, candidate-bound checks, applied manifests and
integration attempts remain separate. The compact recorded-state trace explicitly
identifies itself as snapshots rather than a complete timeline. Files and checks
are directly inspectable; record limits, raw identities and provenance remain under
disclosures. Recovery warnings remain visible. The spectral boundary indicates
confirmed Apply only; no confirmed Apply does not establish an unchanged repository.

Actions are record-dependent: Review prepared changes, View applied result,
Inspect recovery or View run details. Candidate/result navigation refreshes the
record and carries the execution domain. No History Apply, automatic retry,
rollback or replay was added. Selection uses domain plus run identity; old async
responses cannot populate a new selection. At constrained content widths,
History becomes list-to-detail with search retained and Back restoring row focus.

Work changes are limited to saved request/task prose (IDs remain accessible),
separate active-worker brackets / selected surfaces / keyboard focus, and a
compact header without repeated approval instructions. Only an approval whose
domain and run identity match the focused run becomes its primary Decision needed
action; a repository-wide approval count no longer implies that this run is
blocked. Execution state, Review and Stop remain visible. Full recorded task prose
remains in Source details. The glyph renderer and Review screen hashes still match
the saved baseline. No Core, IPC, authorization, dependency or rendering-system
changes were made.

Evidence:
- PASS: `npm --prefix apps/desktop run check` — zero Svelte errors/warnings and
  CSS lint PASS.
- PASS: 52/52 focused Playwright checks across History outcomes, epistemic state,
  glyph behavior and execution topology after the presentation refinement.
- PASS: all 16 directly impacted compatibility cases passed in bounded follow-up
  reruns after current labels/selectors were corrected (11, then 4, then 1); this
  was not one combined 16-case invocation.
- BROADER DIAGNOSTIC: 137/157 passed before those expectation repairs. Four
  remaining failures were not counted as pass and are outside this Work/History
  slice: two Review-scroll preconditions, one Review-support spacing threshold and
  one intentionally hidden voice-selector expectation.
- PASS, browser fixtures: empty/two/42-run histories, saved/missing titles,
  multiple repositories, stopped candidate, unapplied/applied/recovery/rollback,
  unavailable evidence, delayed-response selection, narrow back/focus, queued vs
  active selection, saved task prose, long output, Stop and Review navigation.
- Rendered and inspected Chrome captures in
  `target/presentation-refinement-20260918/`: matched wide before/after views plus
  sparse, applied, recovery, unavailable, exact-run decision, foreign-approval,
  selected-queued and 860px Light fixtures. Captures are fixture evidence, not real
  agent executions or installer acceptance.
- Impeccable Operate guidance applied to outcome/action hierarchy, progressive
  technical disclosure, width-driven composition and non-colour selection cues.
  Its detector returned no findings on History, Work and ExecutionMap after the
  status treatment was normalized.
- PASS: `git diff --check` (line-ending conversion warnings only).
- NOT RUN: native WebView, laptop DPI/scaling, packaged app and real database/run
  acceptance. No release readiness is inferred from browser fixtures.


## September 17 — spectral glyph and connected Work integration

The spectral glyph volume is selected and locally integrated. The halo and glossy
orb remain historical proposals, not production targets. Work preserves the real
shell/navigation, dependency graph, Stop, approvals and candidate-to-Review flow.
Its selected task opens one contextual inspector using the existing scoped,
bounded `DockInspection` event reader. Output, Events, Files and Evidence retain
recorded semantics: planned claims and prepared contributions are separately
labeled; absent or ambiguous worker records remain unavailable. The existing
worker ledger still opens detachable docks when deliberately selected.

`ApertureGlyph.svelte` uses a 96px character-only volume, fewer/larger upright
characters and a wider diagonal negative-space aperture. SVG is the settled,
reduced-motion and Canvas-failure fallback. Canvas 2D is used only for active
motion. No dependency was installed. `workActivity` is a read-only presentation
projection: pending decisions, failed/stopped, unknown or unavailable snapshots
settle. Ready for review does not imply Apply. No independently observable live
verifier state was added or inferred. App and OS reduced motion, visibility,
intersection, font readiness, theme changes and teardown are handled.

Task nodes are tighter, selected recorded edges stronger, and the repeated Work
destination inventory is removed (exact candidate files remain in Review).
Sora is the loaded UI face in browser evidence; headings/prose share the existing
Pytxo UI stack, while paths/output/glyphs retain IBM Plex Mono. Narrow selection
scrolls the single inspector into view; dense plans retain the existing list
fallback. No diagram subsystem or backend contract changes.

Artifacts: `work-glyph-integrated.webm` (25.5s real frontend/browser-fixture
interaction, not native execution); `work-glyph-before.png`,
`work-glyph-integrated.png`, `work-glyph-narrow.png`, and
`work-glyph-narrow-inspection.png`. Debug instrumentation exists only in
`target/glyph-integration-20260917/`, not production. The previous standalone
preview and its fixture controls are not imported by the app.

Evidence: static check PASS, zero errors/warnings, CSS lint PASS; 43 distinct
selected Playwright tests PASS (41 first batch + 2 locator-only reruns). The two
dock tests now target retained ledger/dock controls instead of the old task-node
opens-dock interaction; their safety/layout assertions are retained. Impeccable
scan found no findings on the three targeted components. Browser capture checked
wide/narrow and settled/active/stopped inspection; existing tests cover light,
390px, 200% text zoom, docks, scoped selection and Review states.

Performance: headless Chrome 152.0.7977.83 on Ryzen 7 7735HS, dev frontend with
recording enabled; three 2.2s samples per condition. Rendering disabled: zero glyph
draws. Active: 68–69 callbacks/sample (~30/s), median 0.4–0.5ms, p95 0.7–0.8ms.
No observed long tasks in either condition. Measurements overlapped a test/build
process and are short CPU callback samples, not a native/GPU/power benchmark.
Reduced motion, offscreen pause, synthetic visibilitychange, context-failure SVG
fallback and route cleanup PASS. Native WebView, laptop DPI, actual hidden-window
behavior, GPU/battery cost and packaged/runtime acceptance remain NOT RUN.
Review component, IPC and backend interface hashes match the pre-edit baseline.
No commits, deployments, publication, purchases or production data actions.

Follow-up baseline comparison (`performance-no-glyph.json`) hides the entire
mark while preserving its layout space, not just its canvas. With no recording
or concurrent build, three further 2.2s samples produced zero baseline draws;
active samples each recorded 70 callbacks, 0.4ms median and 0.7ms p95, with no
long tasks in either condition. This isolates the visual's CPU callback cost;
it still does not establish native frame pacing or battery impact.



## September 17 — visible spatial Review correction

The latest user instruction supersedes the default-collapsed map below. Review now defaults to visible candidate relationships at 900px+ available content width, including dock occupancy. Real prepared files converge on the exact candidate; selection highlights its connection and opens the existing comparison. Three file nodes plus Browse all files bound diagram density without losing the full inventory. Recorded verification opens existing checks; destination details expose the actual read-only run target. Focus on code persists in local storage; narrow content defaults to the compact summary and can explicitly open an accessible list layout. Header spacing and empty inspection-toolbar space were reduced. No successful integration animation was introduced.

Evidence in `target/review-spatial-correction-20260917/`: before-wide and after ready/missing/stale/focused/narrow PNGs, static check, regression logs and receipt. Static PASS (0 errors/warnings, CSS lint); 58 distinct affected tests PASS across the final batch and two locator-only reruns (56 + 2). Tests cover map actions, preference across reload, dock occupancy, keyboard, reduced motion, exact content, stale digest, recovery and existing confirmation behavior. Impeccable layout detector returned no findings. Apply handlers, loadReview and selectPreparedFile are unchanged; protected Core/Tauri/IPC hashes unchanged. Browser fixture captures are not native or release acceptance. Native laptop/DPI and packaged acceptance remain NOT RUN. No publication, commits, website edits or backend changes in this correction.

## September 17 — Review finishing pass

Kept the focused comparison composition. Consolidated verification into one compact summary and moved eligibility into the Apply decision area; removed the repeated verification pipeline. Added an explicit Show candidate map disclosure with recorded files, exact candidate, checks and destination. Increased code/file-label readability, exposed the full Apply destination, retained one contextual Back to Work action and corrected command pluralization. History remains available through main navigation. Exact-content panes remain neutral; no inferred line-change highlighting.

Evidence: `target/spatial-workbench-20260917/finishing/`. Static checks PASS (0 errors/warnings; CSS lint). Focused regression batch 64 PASS; final affected visual/capture batch 11 PASS; final expanded-map/long-content confirmation 2 PASS. These overlap, not 77 distinct tests. Coverage includes stale identity, refresh, recovery, keyboard focus, reduced motion, narrow/light layouts and text enlargement. New substantial-file browser fixture and synthetic long destination stress are explicitly not native execution evidence. Product-asset parity PASS; Review/Apply captures refreshed locally. Impeccable detector completed without findings on the changed Review component.

Core/Tauri/IPC protected hashes and the reviewed Apply request/handler are unchanged. Only the browser preview gained an opt-in substantial-content fixture. No inherited files removed, no staged changes, commits or publication. Native laptop readability, Windows DPI, packaged execution and genuine mission footage remain NOT RUN. This is UI acceptance, not release acceptance.

## September 17 — spatial workbench refinement follow-up

The user's follow-up adopts Work as an interactive system map and Review as its
focused reading mode. This supersedes the tall Review destination rail and
four-column summary in the earlier local implementation. No Core, Tauri, IPC,
commerce or runtime behavior changed. No release action occurred.

- **PASS — Desktop source checks:** Svelte/type and CSS checks, zero errors/warnings.
- **PASS — 79 distinct targeted Desktop tests/capture cases across final batches.**
  The main confirmation had 72 passes and six old reference-capture heading failures;
  the corrected hierarchy/topology batch passed all 31, including one new context
  isolation test. Tests retain stale-client refusal, exact content/chunk loading,
  Apply/recovery, verification failure, docks, keyboard operation, 150% text sizing,
  narrow layouts, themes and reduced motion. An earlier real task-selection bug
  found by the new round-trip regression was corrected before confirmation.
- **PASS — visual inspection:** actual current browser renders at wide/narrow
  widths and light theme; Work/Review marketing captures refreshed at 1600x1000,
  1280x800 and 960x640. The existing capture flow also records the fixture Apply
  outcome. Neutral Before/After is an exact-content comparison, not computed line
  highlighting. No synthetic changed-line coloring was added.
- **PASS — Impeccable detector:** no reported findings on ExecutionMap,
  RunReviewScreen and MissionDock. Manual visual inspection remains separate evidence.
- **PASS — website tests:** 15 marketing/docs Playwright checks against the local
  built site with refreshed product images.
- **PASS — asset parity/internal links:** 27 Desktop/27 docs/10 marketing assets;
  191 internal links. Website source remains the prior built editorial composition;
  this slice refreshes its real UI fixture images rather than inventing footage.
- **BLOCKED / NOT RUN — native Windows, installed artifact, real-agent mission,
  accepted success demonstration and clean-machine acceptance.** The website
  previews do not establish these facts. No fabricated sequential success demo.

Primary verification commands (Desktop):

```text
npx playwright test e2e/execution-topology.spec.ts e2e/review-depth.spec.ts e2e/review-hierarchy.spec.ts e2e/mission-dock.spec.ts e2e/beta-workflow.spec.ts e2e/marketing-captures.spec.ts --grep-invert '@marketing-capture.*(approvals|setup|integrations|workspaces|history|flow)|1920x1080' --workers=2
npx playwright test e2e/review-hierarchy.spec.ts e2e/execution-topology.spec.ts --workers=2
```

The source receipt is `target/spatial-workbench-20260917/source-checkpoint.json`;
it is not a release or build-input manifest. Preserve inherited work and use the
existing release plan's native acceptance gate next.

## September 17 — approved hypervisor UI source checkpoint

**Local implementation and browser evidence; not a frozen release candidate.**
Branch `codex/beta-candidate-verification`, HEAD
`72879702f90f2b74eece888bb117df59608f9b56`, plus inherited and new dirty work.
The approved mockups are visual references. Core contracts remain authoritative.

### Changes and evidence limits

- Work: recorded dependency flowlines, actual-worker brackets, candidate/check
  stages and a separate repository destination. Missing/ambiguous worker evidence
  stays unknown. Large/dense/narrow graphs preserve all tasks in a list.
- Review: changes/evidence/attention/decision hierarchy, file navigation alongside
  frozen Before/After, prepared-file convergence, speculative/canonical separation
  and persistent decision area. In-flight Apply is unconfirmed, not successful or
  a recovery failure. Existing digest binding, confirmation, stale invalidation,
  bounded reads and recovery actions remain authoritative.
- New work: repository/agent before the task composer, explicit check disclosure,
  plan stages and retained readiness/input validation. Voice is secondary and its
  existing behavior is retained. History adds recorded boundary events and
  domain-bound review detail without claiming a live filesystem inspection.
- Website: repository-scoped hypervisor message, current UI fixture imagery,
  Execution / Review & Apply / Recorded outcome, supported Beta requirements,
  first mission, limitations and support. Published download versions are unchanged.
- Impeccable: context/new-work/Operate/craft-floor guidance, shared semantic
  tokens, restrained aperture, truthful states, reduced motion and manual rendered
  review. Detector reports two intentional side-border findings: selected-file
  navigation and the inherited recovery notice. No formal dual-agent critique or
  comprehensive accessibility certification is claimed.

### Checks actually run for this visual slice

| Check | Result | Evidence scope |
|---|---|---|
| Desktop `npm run check` | PASS — zero Svelte errors/warnings; CSS lint passes | Final frontend source |
| Desktop final affected Playwright batch | PASS — 109 tests | Menu, shell, Review depth, topology/state visualization and 12 main-surface capture cases; production preview |
| Earlier expanded Desktop batch | 128 PASS / 8 FAIL, then corrected tests pass in final batch | Eight failures used Voice controls without opening the newly approved disclosure; recording/cancel assertions retained |
| Beta workflow, mission dock, responsive menus, flat onboarding, Review state suites | PASS in the expanded batch | Includes remaining dock width, keyboard resizing, 200% text zoom, readiness and check authority; not native DPI |
| Web `npm run lint` and `npx next build` | PASS | Final web source; 63 routes generated |
| Web `npx playwright test e2e/marketing.spec.ts e2e/docs.spec.ts` | PASS — 15 tests | 1440/390px homepage, keyboard tabs, repository limits, docs/search and existing release links |
| Web `npm run verify:product-assets` | PASS | 27 Desktop / 27 docs / 10 marketing images: dimensions, distinct content and byte parity; does not certify every image is from this slice |
| Web `npm run check:links` | PASS — 191 links in 114 files | Internal source links |
| Native/package/clean Windows, real Codex mission, installed Apply/recovery, Windows 100/150/200% DPI | BLOCKED / NOT RUN | Native controls unavailable; browser fixtures do not replace these gates |
| Rust suites / hosted CI / Storybook / full release E2E suite | NOT RUN in this visual slice | Core/IPC source unchanged from preserved baseline; historical passes remain historical |

Exact final Desktop command (from `apps/desktop`):

```text
npx playwright test e2e/menu-workflow.spec.ts e2e/shell.spec.ts e2e/review-depth.spec.ts e2e/execution-topology.spec.ts e2e/marketing-captures.spec.ts --grep-invert '@marketing-capture.*(approvals|setup|integrations|workspaces)|1920x1080' --workers=2
```

An earlier broad capture attempt hit an existing `integrations` heading expectation
(`Agents`) that does not match the current screen; that unrelated route is not
claimed accepted. The final capture set deliberately covers the four redesigned
surfaces at 1600x1000, 1280x800 and 960x640. Review capture cases also render the
recorded Apply outcome. Dark/light, stale/recovery/unavailable states and narrow
execution were inspected; website desktop/mobile captures were inspected.

### Preservation, provenance and next gate

`target/hypervisor-ui-20260916/inherited-baseline.json` holds 262 initial dirty or
untracked file hashes. The source receipt and rendered QA references are in the
same local evidence directory. No inherited files are missing; changes outside the
visual slice were preserved. Core, Tauri, IPC, backend interfaces and preview state
providers retain their baseline hashes. Review's script differs only by the new
presentation import and selected-file derivation; its authority/read handlers did
not change. A transient encoding-write failure was restored to the exact original
SHA256 before the intended Review edits were reapplied.

This is a source/capture checkpoint, not a complete build-input or installer
manifest. No version bump, installer, commit, staging, push, publication,
deployment, commerce change or launch action occurred. UI is ready for native
acceptance; Public Beta is not release-approved. Follow RELEASE_PLAN for the
remaining supported-path, packaged lifecycle and release gates.

## September 16 — reviewed-identity source checkpoint (not a release candidate)

Branch `codex/beta-candidate-verification`; HEAD
`72879702f90f2b74eece888bb117df59608f9b56`. The inherited dirty tree is preserved;
HEAD alone does not identify these changes. No version bump, commit, package,
release, deploy, provider/account change or public post occurred.

### Implementation and proof boundaries

- **PASS — red/green Core reproduction:** a reviewed A, legitimate primary drift,
  stale refusal and refresh to B formerly allowed old client A to Apply B. It now
  returns typed `StaleReview` before a new Apply claim/write. Fresh B applies.
  Additional coverage keeps target bytes identical but refreshes check evidence,
  rejects empty/wrong identities without a journal, and permits only one concurrent
  Apply. Existing domain, receipt, drift and recovery checks remain.
- **PASS — source authority trace:** required digest flows through frontend,
  backend interface, IPC and Core under the existing domain mutation lease.
  Confirmation pins the displayed digest; events invalidate it; missing events
  still fail at the backend. Refusals use existing events and do not mark B failed.
  Recovery may reconcile prior authorized work before the new-claim comparison.
- **PASS — relevant evidence scope:** package digest includes candidate and base
  inventories and verification evidence. Production `save_run_contract_with_status` occurs at
  initial dispatch for a new run ID; refresh reuses its plan and enforcement
  envelope. No legitimate same-run mutation path for those separately stored
  authority fields was found. Revisit this contract before adding one.
- **PASS — partial planning repair:** run-specific checks go through new preview;
  edited checks cannot silently pass prompt-only save. `max_workers` is persisted
  and enforced at dispatch, even after a committed config increase. Missing legacy
  worker authority requires repreview. Default new Desktop requests use one worker;
  all approved tasks remain. Configuration and explicit agent choices are preserved.
- **PASS — bounded Impeccable pass:** actual detector ran on FlowScreen and
  RunReviewScreen with no reported findings. Browser fixture screenshots at 1440px
  and 390px were inspected, a compact check-editor/stale-state correction was made,
  and confirmation screenshots were inspected once. This is not full design,
  theme, native DPI or accessibility acceptance.
- **BLOCKED — native/package proof:** native app control is unavailable in this
  session. No current-source packaged mission, installed Review/Apply, native DPI,
  clean-Windows acceptance or exact-artifact release proof.
- **NOT RUN — remaining Beta scope:** complete supported-surface gating,
  comprehensive first-use/lifecycle matrix, substantial Desktop/website redesign,
  hosted CI, RC freeze, pilot, matching demo/gallery and Product Hunt preparation.

### Commands actually run

| Check | Result | Scope |
|---|---|---|
| `cargo test -p pytxo-desktop --lib --locked --offline -j 1` | PASS — 42 tests | Includes structured stale-review IPC refusal and existing recovery adapters; no native UI proof |
| `cargo test -p pytxo-orchestrate --test run_apply --test candidate_run --test flow --test flow_concurrency --test stop_exact --locked --offline -j 1 -- --test-threads=1` | PASS — 34 tests | Final source; 15 Apply, 1 candidate, 13 Flow, 1 concurrency, 4 Stop |
| `cargo test -p pytxo-orchestrate --lib --locked --offline -j 1 -- --test-threads=1` | PASS — 39 tests | Includes fault-injected interrupted Apply, conservative reconciliation, offline local defaults and runtime caps |
| `cargo test -p pytxo-runner --test candidate_verification_scope -p pytxo-planner --locked --offline -- --test-threads=1` | PASS — 8 tests | Only the named runner integration target ran; planner was tested separately |
| `cargo test -p pytxo-planner --locked --offline -- --test-threads=1` | PASS — 17 tests | Local planning, task conservation, explicit cloud opt-in |
| Installed Chrome, `review-depth`, `review-state`, `beta-workflow`, `astra-workflow`, one worker | PASS — 53 tests | Production-preview fixtures; includes two-client notification and missed-notification paths |
| Same browser, affected `beta-workflow` and `astra-workflow` after final UI correction | PASS — 20 tests | Fresh-preview check edits, one-worker scope, stale state, onboarding, error paths |
| `npm run check` in apps/desktop | PASS | Zero Svelte errors/warnings; CSS lint passed |
| `cargo check -p pytxo-cli --locked --offline -j 1` | PASS | Companion CLI compiles with extended Flow input; no runtime parity claim |
| `git diff --check` | PASS | No whitespace errors; inherited CRLF notices are not failures |

Earlier attempts are not hidden: a malformed initial fixture comparison included
preimages that were supposed to change; after correction the intended old-client
regression failed before repair. The check-edit regression likewise failed before
repair. A config-change concurrency fixture initially hit the independent dirty
input guard; committing the fixture-only config change tested legitimate drift.
Playwright's uncached Chromium channel was BLOCKED (no browser installed); rerun
used existing Chrome without downloading tools. A combined Rust invocation hit
metadata/internal-compiler failures and exposed an introduced undeclared `anyhow`
type in the IPC test helper; that helper was corrected without adding a dependency,
and verification was split into package-specific runs. Do not count failed attempts
as passing evidence or infer a clean full-workspace build from selected suites.

RELEASE_PLAN and CHECKPOINT contain the remaining approved work and stop boundary.

Preservation check: all 223 untouched inherited entries match their pre-work
SHA256. The other 17 inherited entries were intentionally edited in this slice;
11 previously clean tracked files were also changed (28 touched files total).
No baseline file is missing. Final status is 126 modified tracked files and 125
untracked files, no staged changes; HEAD/worktree selection unchanged. A local
hash receipt is at `target/beta-trust-20260916/source-checkpoint.json`; it identifies
this edit slice, not all consumed build inputs and not a frozen release candidate.

## Current source — native recorded agent inspection

Final local EXE 89408CAE supersedes 3305D97E and D228FD91; native Task & receipt
with recorded workspace/unknown custom launcher observed. Fresh browser/Rust/
Svelte/CSS/native build checks pass. Native known ADE/live-session, Windows scale
and installer gates remain open. No RC/MSI/publication; see CHECKPOINT and
target/flat-02-identity/README.md for precise lineage.


## Current source — workspace fit and activity

Local source and native EXE D228FD91 supersede the prior menu candidate.
Browser checks and native build passed; native inspection was stopped by physical
Escape before capture. This is an uninspected local executable, not an installer
or frozen RC. See CHECKPOINT and target/flat-02-supervision/README.md.


## Current source — Flat Desktop 02 menu refinement

Latest UI source and final isolated native validation EXE supersede the foundation
capture below. `target/flat-02-menus/pytxo-menus-final.exe`: SHA256
99CB1D323DE2D5E19D44055C70AF9C169B2C59BE4ACDC94DA89A24A5347E87B0.
Final 21 affected browser checks and Svelte/CSS/native build passed. Final native
Appearance, compact empty inbox and layout Escape are observed; broader same-slice
captures have explicit earlier-build lineage. See its README/identity record.
This is not a new installer or frozen RC. Actual Windows scaling/minimum native
size, native pending decisions and whole-product/release gates remain open.
No hosted Actions, publishing, pushes or account/security changes in this slice.


## Current source — Flat Desktop 02 foundation

Approved onboarding/flat-art implementation now supersedes the older UI candidate.
Final isolated validation EXE is 6FEF8413…; native onboarding/list-scroll/app-zoom
captures and full identity are in `target/flat-02-native/`. No new MSI or frozen
release candidate. The broader approved mission/workflow scope and exact-installer,
clean Windows, final media/Bench and publication gates remain open. See CHECKPOINT
and [[astra-flat-desktop-02]] for fresh versus historical evidence.


## Current source — September 13 UI feedback supersedes the MSI below

The latest authorized local UI/folder-grouping changes are described in
[[astra-ui-feedback-2026-09-13]] and the current CHECKPOINT. Previous MSI
322489A0… does not contain them. A separate configured native validation EXE
D54CE7FB… was built and launched; it is not the default-config release installer.
Desktop automated checks pass. Native inspection remains incomplete because the
capture showed competing Roblox Studio despite returning Pytxo's accessibility
tree. Do not count that capture as visual acceptance. Clean Windows installation,
fresh final-build demo/Bench, matching release materials and explicit publication
approval remain open. No paid/hosted Actions were triggered for this UI pass.

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

Fresh native mission acceptance is now complete within the recorded debug-build
scope: three successful tasks; Cancel unchanged; deliberate checkout drift refused
before Apply; refreshed package verified; three exact after-images applied; one
committed receipt and unchanged bytes after same-build restart. Fourteen fixture
tests and twenty-seven independent assertions pass after Apply. Refresh replaced
the package identity because verification timestamps changed, with identical file
bytes. Native captures 43–47 and fresh-mission snapshots establish the result.
This does not close Stop settlement, Windows scaling, terminal input, default-profile
RC/installer, clean Windows, matching media or publication/download gates.

## September 13 — native mission workflow repairs; candidate not frozen

Latest native validation build: 3693D25BE5B5D7C14D845AA5616F2007D50E39137D28AC672960FEBF0F94EFCC,
explicit copied main profile, verified by process evidence. Native tests exposed
dirty-checkout startup failure, cramped New run after dock focus, stale parallel
documentation and Review filenames crushed beside the right dock. These are fixed
within the current approval scope. Final affected checks: 38 UI, 16 planner,
15 orchestration tests passed; Svelte/CSS and strict library Clippy passed; native
build passed. Evidence: target/astra-native-dock-20260913/combined-review-record.md.

The first fresh completed package (0c7e2acf) passed code checks but was rejected
for stale documentation, then native Apply was cancelled with primary bytes
unchanged and zero attempts. The corrected three-stage native mission is
dc873e30-4b14-4780-a1ee-b45ec1c8dad6; its actual final result is recorded in the
latest checkpoint and fresh-mission snapshots, never inferred from build success.
No source commit/push, CI run, installer, release, deployment or new MP4. Only the
disposable fixture's prior accepted bytes were committed as its next test baseline.
Full Windows scaling, Stop settlement, terminal input, frozen RC, exact installer,
clean-Windows, matching media and publication/download gates remain open.

## September 13 — native preview recovery repair accepted within its test scope

Latest validation EXE: 1740B643FC405E1BC83572CCDAC607B0A2FBB4C8C7ECCEAB25F90E4B969FD2E9,
explicit copied-profile configuration, not final/default-config installer. Native
redirect rejection, Retry recovery, successful page interaction, old renderer
exit and backward Tab handoff were observed (captures 24–26). This supersedes
the preceding EXE identity; earlier evidence retains its recorded build scope.

Final affected suite: 14 passed (46.2s); Svelte/CSS clean; native build passed.
Independent source review findings were fixed and rechecked. Retry waits for
closure; stale sync/close/hide completions cannot overwrite a replacement view.
No new Rust/security/dependency change. Full keyboard/failure acceptance, Windows
scaling, fresh native lifecycle/Apply, terminal input, frozen RC/installer, clean
Windows, matching MP4 and publication/download gates remain open. See CHECKPOINT.md
and target/astra-native-dock-20260913/preview-recovery-review.md.

## September 13 — final preview validation build inspected

Supersedes the in-progress positive-preview status below. Validation EXE
1AB48645874466E26DA2251F119F28224522D3F9F1ADE1FF1F0418D0E2CB26A7 runs with
the verified copied profile. Actual 1282 x 802 captures 18/19 show the repaired
bottom viewport and interactive local test page; F6 -> Tab -> Enter returned
to Pytxo and paused that page (capture 20). Prior DD1EF49B captures/probe logs
establish bounded storage, request, popup, frame and menu behavior, not a general
network-sandbox certificate. Build-specific evidence is in
target/astra-native-dock-20260913/README.md.

Final affected preview/dock tests: 13 passed; Svelte/CSS clean; native build
passed. Earlier combined checks in this slice: 78 UI and 35 native tests passed,
strict library Clippy passed. Actual Windows scaling, complete native keyboard/
failure/download/redirect checks, fresh run/Stop/review/Apply and terminal typing
remain open. No installer/frozen candidate, final MP4, clean-Windows, CI or public
download gate is newly closed; default-config release acceptance is separate.

## September 13 — local preview integrated; native acceptance in progress (preceding checkpoint)

Supersedes the policy-only entry below. The raw renderer, dock, visibility barriers,
keyboard return and block/load reporting are now local implementation. Fresh 78
integrated UI tests, four final preview contracts, 35 Desktop native tests and
strict library Clippy passed; Svelte/CSS checks passed. Actual native capture 14
confirms shared-storage refusal on EXE 8b77c28d. Positive isolated preview behavior
is not yet accepted. The ba39 relative-profile attempt used the default profile
and was stopped; a narrowly conditional main-profile fix is now tested/reviewed.
See CHECKPOINT.md and the target/astra-native-dock-20260913 evidence record.

No frozen candidate, MSI, clean-Windows acceptance, final MP4, CI or publication
gate is newly satisfied. Release and publication approvals retain their scope.

## September 13 — preview policy only; renderer not yet integrated

The local URL/caller policy passes four final native library tests and Desktop
clippy with warnings denied. An independent read-only review rejected the ordinary
Tauri-child/ACL-only isolation route and found a blob: origin bypass in the new
policy, reproduced and fixed. The selected raw Wry route still requires actual
renderer/dock integration and native boundary acceptance. See the current
CHECKPOINT.md and docs/01-projects/astra-local-preview-boundary-2026-09-13.md.
No renderer, dependency manifest, CSP or capability changes were activated; the
running 38825c1f output executable and all release gates retain their prior state.

## September 13 — readable agent output follow-up

Current local debug EXE SHA256 is
38825c1fa9b1691dc80d15ddda2709fb54ce900037a21f5ef93f5c50537c4c05. Plain recorded
output and the original raw-event option were exercised in the actual native app
against historical real records. Fresh 13 tests and Svelte/CSS checks passed;
native build passed. Evidence: target/astra-native-dock-20260913/readable-*.log,
readable-native-launch.json and screenshots 12/13. This supersedes the preceding
debug executable only. No installer, fresh dispatch/Apply, clean Windows, final
video or publication gate is newly satisfied. The manual terminal handoff remains
pending; do not repeat it while independent authorized work can proceed.

## September 13 — native docking recheck completed; release acceptance still open

Desktop control resumed explicitly. The current debug custom-protocol EXE is
51f458a1a341a54cf4ecdd1b63900432ebabd1de4addffe0d93c5599ce27ddde. Native inspection
found and repaired mission/output density, dialog placement and a saved-height
Stop clipping case. Final native laptop captures and build/test identities are in
target/astra-native-dock-20260913/README.md. Fresh checks: 39 affected browser tests,
then 9 final docking tests after the last fix; Svelte/CSS clean; native build passed.

The UI was exercised against historical real run/package records. New dispatch,
Stop/Cancel/drift/Apply/restart acceptance and interactive terminal typing remain
open, as do actual Windows scaling and isolated local preview integration. Computer
Use cannot operate the shell; a manual terminal-input check is the next acceptance
action. Prior native backend tests are historical evidence. No MSI was rebuilt or
accepted, no final MP4 was captured, and no CI or publishing operation was triggered.
The older build/pause entries below are superseded only as described here; the
five-stage completion sequence and external approval gates remain unchanged.

## September 12 — final background UI regression slice, native rebuild pending

The latest local UI fixes pass 104 affected browser tests plus Svelte/CSS checks.
Current native Rust also passes Desktop clippy with warnings denied. Source hashes,
logs and inspected browser fixture screenshots are recorded under
`target/astra-mission-dock-20260912/`; see CHECKPOINT.md for exact coverage.
This fixes Focus/hide, zoom-aware sidebar/mission geometry, real compact/focused
resizing and stale package metadata. It does not change Apply authority.

Desktop control remains paused after the user's Escape. The running bbb25acd debug
executable predates these latest UI changes. A fresh native rebuild/inspection,
isolated local-preview integration and the remaining candidate/installer/demo/
publication gates remain open. No new installer or native acceptance is claimed.


## September 12 approved docking implementation — local, not a frozen release

The later explicit user approval supersedes the pending-design statement in the
reconciliation below. Mission/right-bottom docking, independent inspection,
workspace terminals and saved wide/narrow layouts are locally implemented.
Fresh checks: 128 affected browser tests before the last narrow-layout change;
6 final docking tests; 29 final native library tests; Svelte/CSS checks; and the
existing post-review workspace-drift rejection test passed. Desktop clippy passed
before the last one-line fail-closed correction. The final debug custom-protocol
executable built successfully; identity and logs are in
`target/astra-mission-dock-20260912/`. This is not an MSI or frozen release candidate.

The user stopped Computer Use with physical Escape during final window discovery.
Final native inspection is incomplete; preceding slice screenshots and historical
Cancel/Apply/restart evidence must not be attributed to this executable. Isolated
local previews remain unimplemented. Final native/scaling acceptance, integration
freeze, matching installer/clean-Windows checks, demo and public download gates
remain open. No CI run or publishing action was dispatched for this change.


## September 12 reconciliation — not yet an accepted final candidate

The September 6 amendment and older verdicts below are historical. Current source
identity is PR31 HEAD `eb5f73d9dc21ffbcc6e51306db071fddbe0b8d78` plus the preserved
uncommitted ASTRA work. Fresh GitHub reads show PR31 open/draft with 12 successful
jobs in CI run `34181828835`. Those checks cover the committed head only. The old
claim that billing prevents all CI is superseded; current-source CI is unexecuted,
and the remaining included allowance is unknown. The current usage response has
zero net Actions charges and no remaining-balance field; the $0 stopping budget
is unchanged. No workflow or spending probe was run.

Fresh SHA256 checks match the existing local MSI
`9676d38f5749b7ba81a2799c56c423753840fc43e15874de5351a5868a678ea1`
and extracted executable
`e0eeb950c53010a1f1834cff3b0c2dae93397e03d062f82d5c3f7499463090b2`
under `target/astra-native-finish-20260910/`. All 361 frozen inputs match their
manifest; two current inputs have line-ending-only differences, with identical
normalized text (see CHECKPOINT). This is fresh identity checking, not a rerun
of packaging or behavioral tests. Preserve the raw-byte distinction.

The preceding 6acc build has recorded native Cancel/Apply/restart and 11 project
tests plus 26 independent checks. The e0ee launch is recorded, contradicting old
no-launch prose, but its full native workflow and ownership repair remain
unaccepted. Existing 65 UI, 193 Rust and 21 integration passes belong to their
September 10 executions; none were rerun today. Clean-Windows packet preparation
does not establish installation or guest acceptance. Binaries remain unsigned
according to the saved inspection, not a new signature check.

The public release mirror still reports v1.2.1 as Latest (fresh API observation);
no public v1.2.2 download/install acceptance exists. The new layout/docking packet
is a proposal awaiting explicit approval, not implemented or runtime evidence.
Use the five-checkpoint sequence at the top of RELEASE_PLAN.md. Preserve the
existing PR-only CI authorization within its included-allowance boundary; merge,
release, deployment, spending and security changes retain their separate gates.

## Sep 6 Beta source candidate amendment

The Beta changes in draft [PR31](https://github.com/Pytxo-dev/pytxo/pull/31),
starting with `f21acfb` on baseline `e1807cc`, supersede the older
task-check-only evidence below. The full Rust workspace suite, clippy with
warnings denied, CLI/MCP builds, Desktop check/native build, 20 targeted browser
checks and web link/lint/build gates passed. Six refined workflow tests also
passed using History to select the actual completed preview run at both widths.
The subsequent Cursor inconclusive-probe fix passed five focused tests,
Desktop clippy and a fresh native build.

The last audit found and fixed silent live-event loss: SQLite's agent row now
exists before its first event, and event write failures produce an explicit
evidence-gap failure instead of an Apply-ready run. A fault-injected database
regression and normal-event persistence checks pass, as do dispatch/Stop tests.
Final workspace and clippy gates now pass after this correction. The final MSI
also built and extracted successfully; its hashes are recorded in
`tooling/benchmarks/results/beta-final-msi-2026-09-06.json`. The native rehearsal
hashes below identify the earlier v3 candidate, not this final rebuild.

Combined verification now binds a version 3 package to the included base and
candidate inventories and the executed recipe. Individually passing tasks with
a failing combined result are rejected. Stale input drift requires fresh checks;
refresh preserves frozen effects and current operator files. Failed checks can
recover through explicit successful refresh. Git discovery cannot ascend from
the source-only candidate into the primary repository. Git-history-dependent
checks currently fail in that snapshot, and excluded dependencies are not attested.

The real Codex 0.153.4 mission `cb21abee-360a-42a8-8f3d-9166828bd66c` passed the
one-worker natural-language path and produced a verified three-file package.
The built Tauri app displayed its exact contents and accepted explicit Apply
confirmation. Journal `f286acee-0b33-4d33-8844-261e4605aadf` committed; all three
primary hashes matched frozen targets and four post-Apply tests passed.
Evidence: `tooling/benchmarks/results/beta-single-codex-2026-09-06.json` and
`docs/_attachments/beta-2026-09-06/`. This is a raw native executable rehearsal,
not a fresh MSI-install proof or an external-user reliability measurement.

**Not ready for public Beta.** Fresh hosted CI run `34033572511` was rejected
before its jobs started: GitHub explicitly reports failed account payments or
an insufficient spending limit. Clean elevated MSI installation and independent
public download/install verification remain unproved. Final independent review
subsequently completed through the read-only Codex CLI after the in-app reviewer
hit quota. Its three production findings are fixed: native refresh no longer
blocks the event thread, refresh cannot clear missing-event evidence, and
approval decisions require a successful audit write for the originating actor.
Follow-up review found no additional production defects; its test-isolation
finding is fixed. A forced audit-write failure regression confirms approval
stays pending, then succeeds with a recorded decision after the write recovers.

After these corrections, the full workspace suite, clippy with warnings denied,
debug/release CLI and MCP builds, status JSON and native Desktop build passed.
The workspace run also exposed a Windows socket race in the Link HTTP mock;
the fixture now restores blocking reads and consumes complete requests before
responding. Focused and full-suite checks passed afterward. The rebuilt MSI
extracted with exit 0 and matches the built executable except its three-byte
Tauri bundle marker; hashes are in
`tooling/benchmarks/results/beta-review-msi-2026-09-06.json`.
The rebuilt Desktop also completed an Orbit refresh with a deliberately delayed
20-second check. Its native window handled Maximize and returned the new size
1.437 seconds after refresh started, while verification was still pending;
refresh then produced a newly verified package. The three candidate effects
remained outside the primary checkout. This is test automation evidence, not a
human usability study; see `beta-review-native-2026-09-06.json` in the same folder.
The following Sep 5 assessment is historical and does not certify this candidate.

Fresh v3 MSI packaging also succeeded before the final event correction. Its
administrative image extracted with exit 0, reported 1.2.2, reopened the completed
run and displayed the corrected Cursor status. All executable bytes matched the
built payload except Tauri's three-byte `UNK` → `MSI` bundle marker. Evidence is
`tooling/benchmarks/results/beta-msi-2026-09-06.json`. Clean elevated installation
is still unverified (the current process is not elevated). GitHub currently lists
v1.2.1 as latest public; v1.2.2 has not been published.

The rebuilt CLI completed deterministic run
`044cdc97-7bae-48ba-8ced-51fdd2f69b2e` with verification-boundary, command, output
and success events persisted and no evidence gaps. A separate direct-Codex
baseline completed the same three-file application change as the real Pytxo
mission, with four independently rerun tests passing. The two-run case study in
`tooling/benchmarks/README.md` does not establish a speed or quality advantage.

The dependency follow-up patches both web lockfiles, Desktop development tools
and the demo's `fast-uri` override. Full web (npm and pnpm), demo and tooling
audits report zero findings. Desktop production is clear; five moderate findings
remain in development tools through a UUID advisory whose affected methods are
not used by the inspected callers. Cargo's existing exceptions and warnings are
unchanged. See `docs/01-projects/beta-dependency-audit-2026-09-06.md` and the
hashed summary in `tooling/benchmarks/results/beta-dependencies-2026-09-06.json`.
Desktop check/native build, Storybook build and all 41 Storybook browser tests
pass after these updates. All 37 rebuilt Desktop frontend files are byte-for-byte
identical to the prior verified assets. Web frozen install/lint/build and demo
typecheck/composition bundling also pass, along with all 17 web production browser
tests and the web link check.

The edited film has now been recut around the real single-worker v3 evidence.
Its 52-second silent 1080p/30fps master passed format, duration, source-capture
hash and publishing-text checks; final scene and transition frames were visually
inspected. It explicitly identifies the edit and automation-operated native UI
rehearsal. There is no simulated click, human-user study or elapsed-time claim.
Evidence: `tooling/benchmarks/results/beta-demo-film-2026-09-06.json` and
`docs/_attachments/beta-2026-09-06/beta-demo-film-contact-sheet.png`. Narrated
output remains unavailable pending new approved audio and its music certificate.

**Assessment date:** 2026-09-05
**Branch:** `codex/release-1.2.2-integrity`
**Baseline HEAD:** `9139a908e05cf796578c094d485ee45baf50ba0d` (`origin/main`)
**Local candidate tag:** `local-v1.2.2-rc1` (annotated, unsigned, not pushed)
**Recommendation:** **NOT READY — clean/elevated MSI installation and hosted publication remain blocked**
**Publication state:** v1.2.2 passes the complete local code/runtime gate and most protected CI jobs; a normal Program Files install, runner allocation, fresh-tag publication, and post-publication verification remain required

## Executive assessment

Pytxo v1.2.2 is a corrective release-integrity patch over the fully tested
v1.2.1 product. The v1.2.2 Windows executable exercised the actual commit
boundary, not only preview UI: a deterministic worker ran in a persisted ProjFS-backed
overlay, independent verification passed, Desktop displayed the exact stored
package and receipt, an explicit human confirmation gated Apply, unrelated
operator drift survived, the primary checkout passed post-state tests, and
History recorded the committed attempt. The corrective patch primarily changes
version and publication surfaces and includes one small UI truthfulness fix;
the runtime claim was re-exercised on the exact v1.2.2 native build.

A separate rehearsal through the current MSI payload changed an affected checkout path after review.
Pytxo refused Apply, did not partially apply the other reviewed files, did not
show success, and recorded a failed attempt. This is the most important
adversarial proof for the product thesis: an agent's claim and a prepared package
do not become a commit when the preconditions are no longer true.

The fresh patch is not ready to claim until protected CI completes, the corrected
workflow publishes a new v1.2.2 tag, and independently downloaded public assets
match the private build and checksum manifests. Protected run `33827029442`
passed its Rust, Desktop frontend, npm wrapper, service, smoke, and benchmark
jobs. Its web audit request received `503 Service Unavailable` from npm; GitHub
then refused the retry and native Desktop runner because the organization has a
failed payment or exhausted Actions spending limit. This external gate also
prevents the release workflow from starting. The repository has a Tauri
updater-signing key, but no Windows Authenticode certificate or local Git signing
key is configured. Narrated media also remains conditional on licensed audio.

## What changed

### Architecture and product boundary

- Folder trust is an out-of-band ceiling across global, per-agent, project-root,
  stdin, MCP, and offline-entitlement paths.
- Cloud repository/cache/delta egress requires dual consent, a hash-bound
  manifest, protected-path and credential-content checks, no-follow file reads,
  and a separate opt-in before transport failure may fall back locally. Policy
  denial never falls back.
- Verifiers have a separate enforcement receipt, minimal environment, bounded
  output, profile/network/HITL gates, process-tree timeout, and fail-closed cloud
  behavior.
- Failed or verification-failed tasks cannot unlock dependents; failed durable
  runs return failure rather than a false success.
- Detached dispatch persists `starting` plus exact active ownership before it
  returns. Concurrent dispatch cannot replace another live owner, and crashed
  ownership is reconciled.
- Fleet nodes run concurrently within approved waves and unfinished rows settle
  failed/skipped on error.
- Apply packages remain one execution domain and one repository root. Stored
  blobs, path preimages, mutation leases, journal reconciliation, and recovery
  states are preserved rather than broadened into a universal transaction claim.

### Desktop and UX

- The v1.2 shell is Work, History, and Setup, with workspace switching in the
  title bar and approvals as an overlay.
- Run and agent rows carry their owning execution domain. Review, recovery, and
  History no longer substitute the currently selected workspace domain.
- Completed runs retain their agents/waves, and Desktop prefers the persisted
  isolation receipt over recomputed configuration intent.
- Partial native snapshots expose per-domain diagnostics rather than rendering
  omitted data as a verified empty state.
- Approval records carry causal domain/run/agent/request/action identity; the UI
  refuses recency-based substitute evidence.
- Apply now opens a focus-contained, Escape-cancellable confirmation naming the
  exact path count and package digest.
- Dark-theme semantic colors, compact layout, empty/loading/error/stale/success
  states, and Storybook catalog navigation were reconciled with the current IA.

### Reliability

- PTY process identity is persisted at spawn with Windows creation identity.
  Stop rejects PID reuse, kills descendants, confirms exit, and preserves the
  `cancelled` terminal state.
- Agent futures and wave siblings are supervised; lifecycle errors become
  durable failed results; Race/MCP cleanup is explicit; cloud teardown is RAII.
- SQLite upgrades are transactional and schema-verified before version advance.
- The final audit found a Windows-only race in the Link HTTP test harness:
  nonblocking mode could leak from the listener to accepted sockets under
  parallel load. Accepted sockets now restore blocking mode, read the complete
  request, send a complete response, half-close, and wait for client EOF.
- Ratatui is upgraded to 0.30.2 and resolves to patched `lru 0.18.3`; optional
  Linux FUSE resolves to patched `fuser 0.16.0`. The declared Rust minimum is
  now 1.88.

### Security

- Paddle entitlement changes require a configured secret, fresh signature,
  allowlisted price identity, durable event replay record, and persisted
  subscription/customer/user ownership. Duplicate delivery is idempotent.
- The duplicate web entitlement provisioner was removed in favor of a
  fail-closed Link proxy.
- Hosted Link and cloud-sandbox processes reject unauthenticated public binds
  and reject auth-enabled startup without credentials.
- Reusable Clerk bearer tokens are no longer placed in Desktop custom-protocol
  URLs. The unavailable v1.2 account-return flow is stated honestly.
- DeepSpace raw/high-fidelity MCP reads are denied/capped, and MCP error
  envelopes are sanitized without converting errors into success.
- CLI/npm installers require exact asset identity, SHA-256, embedded version,
  and atomic replacement. Release Actions are pinned to immutable revisions.

### Release publication integrity

- The v1.2.1 hosted workflow passed, but a later Desktop mirror stage inherited
  obsolete tracked root `dist/` files and replaced the public Windows CLI and
  five-platform checksum manifest. Those assets were restored and independently
  rehashed, but that tag's artifact history is now mutable.
- v1.2.2 removes and ignores repository-root release staging, empties every
  staging directory after checkout, and requires exact non-empty inventories
  immediately before private, public CLI, and public Desktop publication.
- Signed Desktop releases require one updater asset plus `latest.json`;
  unsigned releases require neither. Unit tests cover both valid cases, the
  original stale-file collision, empty assets, nested directories, and missing
  signed updater evidence.
- Version parity covers all 15 versioned Rust lock entries plus Desktop/demo
  lockfiles, Tauri, npm, web download metadata, installer copy, README, public
  changelog, and versioned release notes. These tests run in reusable CI.
- The release icon pipeline no longer uses abandoned `to-ico` or its legacy
  request/image dependency chain. A bounded PNG-in-ICO encoder is unit-tested;
  both generators pass and a clean tooling audit reports zero vulnerabilities.

## Runtime evidence

### Successful commit boundary

- Workspace: `target/release-demo/commit-boundary-20260905-154715-929`
- Run: `6b2bd73d-cb87-409b-b874-8023dd7f210b`
- Base commit: `61133324a2933e0a4cd15ca2d0ff9f481686a5b8`
- Package: `2230d886bd2c5faf55cd95f6180ce4c7a43e2939b27351e7c70e4096a3523915`
- Work showed `1 of 1 settled · 1 passed`, the exact agent, Orbit, and
  `overlay · projfs-sparse-copy-v2` from the persisted receipt.
- Review contained exactly `README.md`, `src/risk-policy.mjs`, and
  `test/risk-policy.test.mjs`.
- The confirmation dialog appeared before mutation and named three paths plus
  the exact digest.
- Unrelated `operator-note.txt` remained untracked after Apply.
- The three reviewed paths were applied; primary-checkout tests passed 3/3.
- History showed Completed / Applied with one committed Apply attempt.
- Captures: `docs/_attachments/release-v1.2.2/final-native-work.png`,
  `final-native-review.png`, `final-native-apply-confirm.png`,
  `final-native-applied.png`, and `final-native-history.png`.

### Stale affected-path refusal

- Workspace: `target/release-demo/commit-boundary-20260905-154945-856`
- Run: `75bbb2f3-6c64-4051-8eeb-6c9ae953667a`
- Base commit: `5d732ace774324870271f4796e07de718524b6e1`
- Package: `13612346e51e834e9a244ab2c56af1be735283ac12fcf0a96378053911031767`
- After review, the operator changed affected path `src/risk-policy.mjs`.
- Confirmed Apply returned “Review is stale”; README and the test file were not
  applied, and only the operator edit remained.
- Baseline tests still passed 2/2.
- The contract stored `apply_status = stale`, `recovery_state = source_drift`,
  no `applied_at`, and no success receipt. History showed Completed / Apply failed.
- Captures: `docs/_attachments/release-v1.2.2/final-native-stale-refusal.png` and
  `final-native-stale-history.png`.

## Verification commands and results

### Repository and Rust — VERIFIED

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p pytxo-cli --release
cargo build -p pytxo-mcp --release
cargo run -p pytxo-cli -- status --json
cargo run -p pytxo-cli -- --version
git diff --check
```

All commands exited 0 on the final code candidate. The workspace test includes
all unit, integration, process, and doc-test targets: 69 core, 50 runner, 32
change-set, 9 Apply, 4 dispatch-lifecycle, 3 dependency-outcome, 3 Stop, 22
Desktop-native, 22 Link, and 6 cloud-sandbox tests among the passing targets.
The rebuilt CLI reports `pytxo 1.2.2`; status returned valid JSON with the
persisted domain isolation mechanism.

The v1.2.2 corrective tree was verified on 2026-09-04 with formatting, the
complete workspace test suite, warnings-denied clippy, optimized CLI and MCP
builds, and the expanded release-version and inventory contracts. All exited 0.

The repaired 69-test core suite additionally passed 10 consecutive parallel
runs. The exact Windows descendant Stop test and PTY environment isolation test
passed after rejecting a regressing `portable-pty 0.9` upgrade.

### Desktop — VERIFIED (patch scope)

```powershell
cd apps/desktop
npm run check
npm run build:native
npm run e2e:release
npm run storybook:build
npm run storybook:test:ci
cargo tauri build --target x86_64-pc-windows-msvc --bundles msi --features voice-whisper
cargo test -p pytxo-desktop
```

- On v1.2.2, Svelte reported 0 errors/warnings, CSS lint passed, the frontend
  transformed 6,366 modules, the optimized Windows native build passed, and all
  22 Desktop-native tests passed in the workspace run.
- The MSI is 10,428,416 bytes, reports product/version `Pytxo Desktop 1.2.2`,
  and has SHA-256 `F22C2CCA7FFABE43332C31AE90EE20A2EC5B12ABA465E89E2C05DF92B11512E9`.
  WiX extraction found the embedded 35,608,064-byte `pytxo-desktop.exe`, version
  1.2.2, with SHA-256
  `1AFCAEC51F827B666C558F7B80B60E43C6AF96531C032746349A71243082A384`.
  Both correctly report `NotSigned`; Authenticode is not claimed.
- Windows Installer administrative-image installation exited 0, and the exact
  extracted payload launched and completed the fresh success/refusal rehearsals.
  A normal quiet upgrade from the existing machine-managed 0.9.0 install was
  rolled back with Windows Installer error 1730 because this session was not
  elevated; a clean/elevated Program Files install remains unverified.
- The final receipt aggregation bug found during adversarial review is fixed:
  two enforced plus two advisory surfaces now render “Partly advisory only,”
  not “Enforcement not fully reported.” The new rendered regression failed
  before the fix and the complete 13-test epistemic-state suite then passed.
- The release-audit UI correction is covered by the production-preview browser
  suite (105/105), development-only legacy-deck suite (1/1), Storybook
  production build and interaction/accessibility suite (41/41), and the fresh
  v1.2.2 native success and stale-refusal captures.

### Website and public docs — VERIFIED

```powershell
cd apps/web
pnpm install --frozen-lockfile
pnpm run lint
pnpm run verify:product-assets
pnpm run check:links
pnpm run build
pnpm run e2e -- e2e/marketing.spec.ts --workers=1 --reporter=line
pnpm audit --prod --audit-level=high
```

On v1.2.2, the frozen pnpm install, lint, 27 product-asset references, 153
internal links, and the 58-page production build passed after updating public
version and Windows-only Desktop copy. A fresh production dependency audit
examined 483 dependencies and reported zero vulnerabilities at every severity.
The unchanged product pages retain the v1.2.1 17-test browser evidence.

### Demo — VERIFIED SILENT / BLOCKED NARRATED

The v1.2.2 typecheck, composition list, asset validator, audio parser tests, and
publishing validator passed. The unchanged v1.2.1 demo clean install, production
audit, still/poster generation, silent render, silent validation, and transition
sheet remain valid. The master is
52.000 seconds, 1920×1080, 30 fps, BT.709, H.264. The transition sheet was
visually reviewed and uses one coherent current Work/Run Review shell.
The canonical recording recipe now administratively extracts the current MSI
and launches that payload, not the raw build-tree executable; a fresh recipe
test exited 0 and resolved the expected 1.2.2 payload digest
`1AFCAEC51F827B666C558F7B80B60E43C6AF96531C032746349A71243082A384`.

Narrated publication is **BLOCKED** until approved/licensed audio is supplied
and `validate:narrated` passes. The release may claim only the silent master.

### npm package and staged install — PARTIALLY VERIFIED

```powershell
cd packages/pytxo
npm test
npm pack
```

- v1.2.2 installer/platform/checksum/atomic tests: 4/4 passed against the rebuilt
  release CLI, including embedded-version and checksum refusal.
- v1.2.2 dry-pack: exactly six intended files; zero runtime dependencies.
- The exact final tarball installed into fresh prefix
  `target/final-package-install-20260902-155602-514` through the production
  postinstall path.
- Downloaded CLI SHA-256:
  `aab81760200de903cb06c35f3a5e324bdfbf241ee1e5fa6a882c131b09517bda`.
- The original exact-package rehearsal reported `pytxo 1.2.0`; after discovering
  that tag was already public, the retargeted v1.2.1 wrapper tests, dry-pack,
  source parity, optimized binary build, and live `pytxo 1.2.1` check passed.
- A clean install from the public `pytxo@1.2.2` package remains required before
  this section can become fully verified.

### Security and artifact audit — VERIFIED WITH ACCEPTED WARNINGS

- `cargo audit`: 0 unignored vulnerabilities; 17 unmaintained informational
  warnings, 1 Linux-only GLib soundness warning, and 1 unenabled yanked `spin`
  dependency warning.
- Ratatui/LRU and optional FUSE soundness advisories discovered during the audit
  were removed by dependency upgrades.
- Desktop, web, and demo production dependency audits report zero known
  vulnerabilities.
- The npm wrapper has no runtime dependencies and therefore no lockfile audit
  surface; its exact tarball and downloaded binary were inspected instead.
- Tracked high-confidence secret scan matched only synthetic sanitizer/upload
  fixtures in test-bearing source files. No tracked absolute developer path was
  found. User-owned untracked captures/logs/notes are not part of a clean release
  checkout and were preserved.
- Version parity verifies 15 first-party Cargo lock entries plus Rust, Desktop
  and demo package/lock metadata, Tauri, npm, web downloads, the PowerShell
  installer, README, public changelog, and release notes at v1.2.2.

## Demo readiness

`DEMO.md` is the canonical 2–4 minute sequence. `GROK_DEMO_BRIEF.md` adds a
75-second launch treatment, exact narration, generated bridge prompts, truthful
capture rules, five gallery frames, social crops, and failure fallbacks. The
canonical walkthrough includes the first-20-second
product promise, exact commands and clicks, expected evidence, a controlled
stale-path insert, fallback behavior, narration, recording checklist, suggested
shots, and release/social descriptions. `prepare.ps1` creates a deterministic
fixture plus isolated `PYTXO_HOME` and WebView2 user-data stores so personal
workspaces and recents cannot leak into a take.

## Deployment and release readiness

- The canonical workflow gates publication on full CI, version parity,
  protected-main ancestry, exact five-platform CLI artifacts, exact one-platform
  Windows Desktop artifacts, checksums, and required mirror credentials.
- CLI and Desktop artifacts are prepared and independently inventory-checked
  before either GitHub release job can begin. The public mirror contains both
  inventories, and npm waits for that combined mirror.
- The bypass Desktop publisher was removed; every referenced third-party Action
  is pinned to an immutable revision.
- npm postinstall and direct installers fail hard on missing asset, checksum, or
  embedded-version mismatch and replace atomically.
- Updater metadata requires exactly the declared Windows platform. The repository
  has a Tauri updater-signing secret; Windows Authenticode credentials are absent.
- v1.2.1 subsequently passed protected CI (12/12 jobs) and the complete release
  workflow. v1.2.2 must repeat those hosted gates through the corrected workflow
  before it replaces the repaired-but-mutable v1.2.1 artifact history.

## Merchant-of-record architecture

- Dodo is the intended Merchant of Record/legal seller; MBCZ is the verified
  business-account owner and payout beneficiary; Pytxo is the customer-facing
  brand. The legal/KYC layer remains outside this repository.
- Proposed `ADR-0040` and `docs/01-projects/dodo-mor-integration.md` define a
  provider-neutral Dodo-to-Link adapter behind a raw Pytxo-domain proxy, with
  replay, ordering, grant aggregation, allowlist, and migration requirements.
- This is **IMPLEMENTED as documentation** and **UNVERIFIED as runtime**. v1.2.2
  does not claim a Dodo checkout or webhook cutover; the existing Paddle adapter
  remains authoritative until test-mode and dual-run evidence exist.

## Known limitations and accepted risks

- Reviewed Apply covers Orbit/Galaxy, one execution domain, one repository root,
  and whole prepared paths—not cross-root transactions or partial-file approval.
- The Apply journal supports process-crash reconciliation, not claimed power-loss
  ACID or cross-filesystem atomicity.
- Isolation is platform-dependent; receipts expose enforced, advisory,
  unavailable, or bypassed mechanisms. Supernova is intentionally host-direct.
- Desktop v1.2 is Windows-first. The workflow rejects unexpected macOS/Linux
  Desktop artifacts; older installers must not be relabeled as v1.2.
- Regex sanitization is defense in depth, not an authorization boundary or proof
  that arbitrary output is secret-free.
- Link, cloud, and proxy production health endpoints passed after the v1.2.1
  deployment. A live production PostgreSQL migration and authenticated sandbox
  sync-to-exec transaction remain unverified and outside the default local v1
  claim.
- `portable-pty 0.8.1` retains an unmaintained `serial` dependency. Its maintained
  0.9 upgrade caused a reproducible PTY lifecycle hang and was rejected. There is
  no RustSec vulnerability advisory for `serial`.
- Tauri's Linux-only GTK3 graph carries an old GLib iterator soundness advisory
  and unmaintained GTK3 warnings; Linux Desktop is not shipped in v1.2.
- Windows notification support pins `quick-xml 0.37.5`, with two ignored XML DoS
  advisories. The parser receives only Pytxo-generated notification XML, not
  untrusted remote XML. The ignored RSA timing advisory is absent from the
  Windows graph and is not used for Pytxo key handling.
- A narrated master is unavailable until approved/licensed audio is provided.
- GitHub Releases and npm cannot participate in one transaction. Publication is
  sequenced only after the complete candidate is validated, but a later service
  can still fail after an earlier upload; the independent post-publication audit
  remains mandatory.

## Current Work cockpit candidate — R6 command ownership and semantic zoom

R6 supersedes R5 as the current local candidate. It preserves the recorded graph, scale-aware topology tiles and Work → Review → Apply authority boundary. New Work now presents one command per phase: the request composer owns `Build plan`, while the reviewed plan owns `Run`. The redundant second `Build plan` action beside `Run` is gone. The redundant Work commit rail remains absent, and the first-use promise remains `Review every change` with exact-change language tied to the save decision.

The exact MSI at `D:/pytxo-beta-lab/cockpit-beta-candidate-r6-20260921` has SHA256 `D2950419F3F843B5D615E46F0986544650F5B57DF48426795B9D1FADDDA8183A`; its administratively extracted executable has SHA256 `0B9BCACEAF1F5A04867CBDB86DDF4DBCAE7DD9956D1970D3EE227F498CDA02FA`. Extraction, runtime-import policy and four embedded frontend identities pass. Svelte reports no errors or warnings, CSS lint passes, the corrected command-ownership regression passes at 1920×1080, 1280×720, 860×760 and 390×760, the definitive Desktop suite passes 355/355 serially, and the dense interaction performance fixture passes 1/1. The planned-flow interaction is production-preview browser evidence; exact-package native launch is verified separately.

Exact native evidence at Windows 125 percent scaling/window DPI 120 covers fresh first use and the dense Work canvas. A fresh Pytxo home plus fresh WebView profile renders onboarding without inherited workspaces and keeps the revised copy and full `Get started` action visible. The dense lab profile renders all 24 workers/eight waves at 39 percent Fit with readable IDs, dependency edges, minimap and no duplicate rail. A settled 20.009-second root-process sample observed no CPU-time increment at the timer resolution, with 40.832 MB working set and 11.383 MB private memory; it is not a summed WebView-tree measurement.

Candidate receipt: `D:/pytxo-beta-lab/cockpit-beta-candidate-r6-20260921/candidate-receipt.json`, SHA256 `225D72B36C7AD971B58379AFF2BDC8F5DDAB94A81710D8C837BCF691C2C92855`. State receipt: `D:/pytxo-beta-lab/state-compat-r6-20260921/state-compatibility-receipt.json`, SHA256 `0B3049F5BF1A1516C65E3330BA394F9DC211A25D6C0DFEF106A006332074A757`. Exact-R6 retained-data clone and installer-upgrade proof remain open.

This promotes the command-ownership correction to verified local candidate status. It does not close clean/elevated MSI installation, retained-data upgrade, native Windows 100/150/200 percent scaling beyond the verified 125 percent host, a genuinely newer signed forward update, stable-manifest/release publication, continuous pointer-visible R6 footage, native 4K capture or human playback review. VirtualBox 7.2.18 is installed, but the prepared clean Windows VM still terminates before boot with hardening error `-5619`; Windows Sandbox and Hyper-V Manager are unavailable on this Windows 11 Home host. No security setting was changed. No dispatch, Apply, install, commit, push, deployment or publication was performed for R6.

## Current R6 cockpit motion proof

The local `PytxoR6CockpitProof` composition now presents the exact R6 clean-first-use and dense-canvas native stills in a 12-second Chroma Aperture review film. It visibly preserves the 24-worker/eight-wave Fit view, minimap, explicit `Review changes` entry and removed duplicate bottom rail. The manifest locks both image hashes, R6 MSI/executable identities, candidate/state receipts and the 125 percent / 120 DPI source condition. It also forces continuous-footage, pointer-motion and native-4K claims to remain false.

The retained 1920×1080 master at `D:/pytxo-beta-lab/cockpit-beta-candidate-r6-20260921/pytxo-r6-cockpit-proof.mp4` has SHA256 `E47E651C88AC32A68040B124151D3018CB675D3986BA5F21525BF98E4AA87FE8`. Manifest validation, TypeScript, composition discovery, full master validation, full decode and contact-sheet inspection pass. The matching 3840×2160 delivery raster has SHA256 `29CB57C8B47CBB2543C5ED7B4F1F6DAB7A692558E0E508138E92ECF857F08F5C`; its full decode and original-raster midpoint inspection pass. The vector overlays are 4K, while the embedded 1602×1002 native frames remain upscaled. Updated receipt SHA256 is `069F978C2593F5C718920C3297B34FEC15CAA4AE5BE920206C4EEEAACB36A292`. This is a motion proof, not a launch master: continuous native interaction, pointer evidence, real Review/Apply footage, native 4K source capture, privacy review and normal-speed human playback remain open.

## Current Desktop candidate — R7 explicit updater handoff

R7 supersedes R6 as the current local package candidate while retaining the exact R6 Work/New Work UI source identities. The duplicate Work commit rail remains removed, the recorded-worker canvas and minimap remain the primary cockpit, and `Review changes` remains the single route to exact-byte review. The updater now separates safe preparation from the Windows process handoff: `Download update` verifies bytes while Desktop remains open, then `Install and restart` explains that Pytxo closes and Windows requests administrator permission. Ready and restart-required states prevent a replacement check from orphaning the verified updater resource. No Rust, orchestration, permission or Apply behavior changed.

The exact R7 MSI at `D:/pytxo-beta-lab/cockpit-beta-candidate-r7-20260921` has SHA256 `76D5F98ED0FB7541763F07C331835F713029C547BA8839A4A5D829E6B3D4AA13`; its administratively extracted executable has SHA256 `9303857141D6A3727BDF2D20B54181CCED87335DCEC9FC20EB51F5E68BF4076A`. Extraction, runtime-import policy and four embedded frontend identities pass. Svelte reports 0 errors and 0 warnings, CSS lint passes, focused updater tests pass 12/12, the definitive Desktop suite passes 357/357 serially, and the dense interaction performance fixture passes 1/1.

Exact native fresh-first-use acceptance passed at Windows 125 percent scaling/system DPI 120 with an isolated Pytxo home and WebView2 profile. The responsive extracted executable rendered the full welcome card and fixed `Get started` action in a DPI-aware 1938×1038 capture, then closed normally. Candidate receipt: `D:/pytxo-beta-lab/cockpit-beta-candidate-r7-20260921/candidate-receipt.json`, SHA256 `790A65382C183053F1E0721CD20CD30BD6AD491E1016699F9E7841C4B47E6E15`.

The public installed v0.9.0 → v1.2.1 experiment remains incomplete. The prior signed bytes and Desktop-to-installer handoff are recorded, but the secure prompt is no longer visible and the installed executable remains v0.9.0. Current observation receipt SHA256 is `711EFAC0D8BFA69E1DBA464CA6B1DE498D984F547158A583C7AD126FC078D086`. That evidence does not establish whether elevation was cancelled or the installer failed. R7's new ready-to-install state is source/controller/browser verified; exact native acceptance needs a genuinely newer signed update.

R7 remains a local beta candidate, not a releasable beta. Clean/elevated MSI installation, retained-data upgrade, native 100/150/200 percent scaling, a real coding-agent Work → Review → Apply mission, hosted release gates, publication and final authentic launch footage remain open. No install, commit, push, deployment or publication was performed for R7.

## Current Desktop candidate — R8 Work focus and updater hierarchy

R8 supersedes R7 as the current local package candidate. The duplicate Work commit rail remains removed, the recorded-worker canvas and minimap remain the primary cockpit, and `Review changes` remains the single path into exact-byte review. On wide Work layouts, a selected worker now opens in a lightweight overlaid focus panel so the 24-node canvas does not relayout. Raw output still opens as an independent bottom dock, and the summary clears that dock instead of covering its controls. Compact and short windows retain the shared tabbed inspection surface.

The updater now presents one primary action for each lifecycle stage. The redundant check action disappears once a version or next action is known; download and install remain separate, semantic error styling is restored, and manual recovery no longer competes with the primary action. Ready and restart-required states continue protecting the verified updater resource. No Rust, orchestration, permission or Apply behavior changed.

The exact R8 MSI at `D:/pytxo-beta-lab/cockpit-beta-candidate-r8-20260921` has SHA256 `D7CA3875D477525C264070B75C8AD4BD032323425917A89545C081215701A58F`; its administratively extracted executable has SHA256 `0811CC5BDF8443F5D735BB29EB6E0A9E3F1B131E0EF534263822455F50DAF073`. Extraction, runtime-import policy and four embedded frontend identities pass. Svelte reports 0 errors and 0 warnings, CSS lint passes, focused updater tests pass 13/13, focused native motion tests pass 3/3, and the definitive functional Desktop suite passes 358/358 serially. The exact-final dense interaction fixture passes once in the release run and 10/10 under repetition, with no browser main-thread task above 50 ms.

Exact native fresh-first-use acceptance passed at Windows 125 percent scaling/system DPI 120 with isolated Pytxo and WebView2 profiles. The responsive extracted executable rendered the complete welcome card and fixed `Get started` action in a 1618×1010 DPI-aware window-only capture, then closed normally. Its main process measured 32.93 MB and 0.47 percent of one core during the retained 10.045-second sample. Candidate receipt SHA256 is `30E5E2BD19AC59D0DF1A096A85056F88062A7001D44C210E326AE95DD24450F6`; artifact manifest SHA256 is `929D72A2A9E44B9CB99B06575F052F3B998A8272202AE5032065745D5B7B9E28`.

R8 is a verified local package candidate, not a releasable beta. Its MSI and executable are unsigned, so no host installation or upgrade was attempted. Clean/elevated installation, retained-data upgrade, a genuinely newer signed forward update, native 100/150/200 percent scaling, a current-candidate real coding-agent Work → Review → Apply mission, hosted release gates, publication and authentic launch footage remain open. No install, dispatch, Apply, commit, push, deployment or publication was performed for R8.

## Final recommendation

**NOT READY.**

The local Windows-first product claim, main success path, critical stale-path
failure, UI behavior, MSI administrative image/payload, tests, builds, dependency posture, and
silent demo are supported by current v1.2.2 evidence. The corrective source also
passes its full local patch-scope gate, while the allocated protected CI jobs passed. It remains
not ready until a clean/elevated MSI install passes, GitHub Actions runner allocation is restored, the web/native
Desktop jobs pass, the fresh tag publishes through the corrected workflow, and
independent downloads prove
the exact private/public inventories, checksums, embedded versions, updater
signature, and npm install. If those checks pass, the appropriate final rating
is **READY WITH KNOWN RISKS**, limited by absent Authenticode/Git signing,
licensed narration, and the documented runtime boundaries above.
