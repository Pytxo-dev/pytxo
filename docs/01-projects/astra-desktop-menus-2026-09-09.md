---
title: Desktop menu and workflow refinement
slug: astra-desktop-menus-2026-09-09
status: active
tags: [project, desktop, ux, verification]
audience: [human, agent]
layer: presentation
created: 2026-09-09
updated: 2026-09-10
related: [[astra-desktop-polish-2026-09-08]], [[astra-evidence-2026-09-07]]
---

Matt reopened the local UI scope on September 9: inspect every menu, improve the
sidebar, and fix UI and UX problems. This pass retains Work, History, Setup and
the reviewed Apply boundary. The proposed Focus/Control redesign remains gated.
The previous e775 installer and c77 native film are historical evidence, not
acceptance evidence for these new sources.

## Decisions and changes

| Before | After |
| --- | --- |
| New run required returning to Work; unfinished input disappeared on navigation. | Persistent sidebar New run / Continue draft. Window-local mission, source, workspace and agent survive navigation. Plans and dispatch authority never carry over. |
| Workspace changes could retarget retained input. | The mounted composer owns its selected workspace. A missing original folder requires explicit selection; fresh planning remains mandatory. |
| Collapsed navigation lost accessible names; signed-out account chrome dominated the footer. | Named icon controls, active run count, clearer recent-workspace label and compact Local Core footer linking to full account status. |
| Settings were an undifferentiated list; search only found section names. | Three groups, control-name keyword search, descriptions, result counts and recoverable empty search. Appearance owns density. |
| Nested headings and padding competed; long catalogs scrolled the settings navigation away. | One clear section heading, visible description, aligned embedded catalogs, sticky settings rail and scroll reset on section change. Narrow navigation consumes less height. |
| Command search had dark-only styling and mixed navigation/actions. | Themed grouped commands, direct access to all nine settings sections, combobox/listbox semantics, disabled reasons, keyboard selection and bounded scrolling. |
| Command Stop could target another workspace and bypass the existing confirmation. | Commands use the selected workspace. Stop opens the exact-run confirmation with Keep running focused. |
| Workspace switcher lacked search and predictable keyboard/focus handling. | Search, selected checkmark, arrow selection, Escape, focus return and suppression of background shortcuts. |
| Workspace table actions and column headings clipped at ordinary window widths. | Sized action and count columns, complete Open/Settings controls, readable headings and horizontal overflow only when needed. |
| Workspace Settings was an overlay without native modal isolation. | Native modal dialog, keyboard focus containment, Escape/return, readable theme tokens, larger Close target and pressed profile state. |
| Use in mission discarded the clicked agent. | The explicit CLI choice reaches the composer. An unavailable explicit choice cannot silently become another agent. |
| Agent actions clipped long connection labels; Docs icons wrapped beneath text. | Responsive agent rows, wrapping action groups, inline icons and content-sized 40px action targets. |
| Light-theme selects, secondary actions, profile cards and workspace search retained dark values. | Existing semantic input, surface, text and focus tokens apply consistently. Providers paragraphs share the card inset. Current settings section has a persistent indicator. |
| History detail could retain a filtered-out run or show review actions without a selection. | Detail follows visible results, empty detail has no review action, and opening a cross-workspace review updates workspace identity. |
| Voice capture preference was disconnected from the composer. | Both, hold and click modes are honored. Keyboard hold cancels on lost button/window focus; late startup after unmount cancels the session. |
| Cloud-consent overlay lacked modal keyboard behavior. | Themed native dialog with initial Cancel focus, Tab containment, Escape and focus return. Existing consent/storage semantics remain unchanged. |
| Onboarding offered “Later” for missing CLIs and claimed readiness too broadly. | Install-guide links, one continuation action and “Desktop setup complete” with the remaining run prerequisites stated. |
| Keyboard settings used retired Operations/mission language. | Labels describe current Work and command behavior. Workspace-default controls expose selected state. |
| A restored draft's generic message overwrote the missing-workspace explanation. | The specific recovery instruction survives mounting; planning stays disabled until a workspace is selected. |
| Icon barrel imports pulled thousands of unused modules into builds. | The same 47 icons use their public direct imports, without a dependency change. The local production build transformed 264 modules in 8.32 seconds, versus approximately 66 seconds immediately before the change. This is an observed local comparison, not a CI timing guarantee. |
| Native CLI readiness checks briefly opened terminal windows over the app. | Background checks suppress their Windows console. The explicit vendor sign-in action keeps its intentional terminal. The repaired package still needs native re-observation. |

## Evidence and boundaries

Root owns implementation. Independent menu/workflow and visual specialists
identified concrete defects; a separate adversarial review found the draft-scope
and keyboard-recording bugs. Both were repaired and re-reviewed without remaining
blockers in that bounded scope. Review is not a substitute for runtime evidence.

Validation is local. Browser fixtures exercise UI behavior without sending model
requests or changing vendor accounts. They do not prove microphone hardware,
native dispatch, clean Windows installation or public distribution. No GitHub
Actions, normal-index staging, push, publication, native computer input or password change
occurred in the initial browser pass. Existing PR31 CI authorization and the $0 spending cap
remain intact.

Final local test and build identities are recorded in CHECKPOINT and the private
`target/astra-ux-20260909/` evidence directory. Earlier failed runs are retained
alongside the corrected final evidence.

The final combined source passed 154 production-preview browser tests (3.1
minutes), all 42 rebuilt Storybook checks (32.4 seconds), Svelte checking with zero
errors/warnings, and CSS lint. The browser run saved 64 screenshots. The missing
workspace story first reproduced the misleading generic notice, then passed with
the specific recovery message and disabled planning intact. Review also verified
all 84 direct icon imports against the installed package exports.

Storybook transformed 272 modules and built in 1 minute 39 seconds; the preceding
successful barrel-import build transformed 6,377 modules in 6 minutes 46 seconds.
Its existing large-chunk advisory remains. These timings are local observations;
the production app itself built in 8.32 seconds with 264 modules. No hosted CI
minutes were consumed to obtain these measurements.

The menu-stage Windows MSI build and extracted runtime-dependency check also passed.
MSI SHA256: `3bd609e083fa61897f87db904f0f99e8c71bd738b6b2048ebfad0e6049f9ae4f`.
Packaged EXE: `fcd039e122a2c46c05e9c0e97ff9a036e6f4fc9e876cd005ae6e7960b9692b05`.
All 359 frozen source inputs matched after building. The installer is unsigned;
runtime checks were still pending at that packaging checkpoint. The private
`target/astra-ux-20260909/verification.json` indexes exact artifacts and screenshots.

## September 10 native follow-up

Current boundary: after the f968 acceptance below, denser video review found a
netsh startup window. Its read-only receipt query now suppresses console creation;
a detached-parent regression failed before the flag and passed after it. All 77
runner tests and strict Clippy passed. MSI 7d30141b… / payload 8910d153… built and
passed dependency inspection, with only network_isolation.rs changed. Physical
Escape stopped native control before the new payload launched. Its native retest
and smooth demo remain pending explicit resumption. The 94-second f968 film is
finished with the netsh defect visible and labeled; its native correctness record
remains specific to that earlier build.

### Preserved f968 acceptance

The 073 native pass exposed verification-terminal interruption, draft restoration
after successful dispatch and inherited plan scroll in the live view. Windows
verification now uses CREATE_NO_WINDOW alongside its existing process-group
flags, including its timeout/cancel taskkill helper. Profile, environment,
output, cancellation and Apply semantics remain unchanged. The separate durable
Stop helper was outside this repair. Scope: existing local verification in one
execution domain, including the fixture's Orbit profile.

The dispatched composer uses an instance-local consumed flag so Svelte teardown
cannot restore the stale draft. Dispatch returns the overview to the top. Tasks
without recorded terminal results say “Awaiting result.” Native f968 testing
confirmed all three UI behaviors, then completed the three-task/two-wave mission,
Cancel, exact Apply and restart. All 11 project tests and 26 independent checks
passed. Review confirmed all six primary files unchanged through Cancel and
exactly the three reviewed writes after Apply.

At that source checkpoint: 157/157 browser tests, 76/76 runner tests, strict runner
Clippy, zero Svelte errors/warnings, CSS lint and formatting. Sixty-four browser
screenshots are archived. The preceding 42 story checks were not rerun here.
The controlled console test passed before and after the fix; native video QA is
the distinct visual check. Build/runtime inspection passed, and all 359 inputs
matched their frozen copies at that checkpoint. MSI `0c2d70e0…` / payload
`f968d665…` are unsigned.

The f968 acceptance record is `tooling/benchmarks/results/astra-native-menus-2026-09-10.json`.
The 900-second raw native take includes Cancel and Apply; its private 94-second
Remotion edit is complete with the startup netsh defect visible and labeled.
Restart is a later native still. Clean Windows, current-source
CI and public-download gates remain open. No Actions or publication occurred.

## Preserved native readiness repair, before resumption

After Matt explicitly invoked Computer, the fcd payload completed fresh host
onboarding, selected a disposable fixture and opened Work and New run. Existing
Claude/Codex sessions reported ready. Background probes visibly opened console
windows; `ipc_meta.rs` now applies CREATE_NO_WINDOW to those probes only. Explicit
vendor sign-in retains its separate terminal launch. No runner, orchestration,
permission-profile or execution-domain contract changed.

The final Windows regression runs a controlled child through the production cmd
wrapper and checks GetConsoleWindow, stdout/stderr and successful/nonzero exit.
All 26 Desktop library tests, strict Clippy and formatting pass; independent review
found no remaining issue. The first wrapper-fixture attempt had a quoting error;
that failed log remains beside the corrected final run. The repaired MSI build and
packaged dependency inspection pass. Current MSI is `26ef95bc…`, payload `073a1a30…`,
359-input manifest `8d42c8b5…`; full hashes are in CHECKPOINT and private
`target/astra-ux-20260909/native-probe-verification.json`. Only ipc_meta.rs changed
among native build inputs; the frontend rebuilt with the same JS/CSS assets.

Physical Escape stopped native interaction before draft entry. No mission or
Apply occurred; all six fixture hashes remain at baseline. Recordly HUD targeting
failed twice. Fallback clips establish readable static Work framing only: no
timestamped concurrent action proves motion capture. Preserve those clips as
diagnostics, not final-build demo evidence. The repaired payload is unlaunched;
resume native checks only after explicit resumption, then verify recording follows
a known UI transition before the full run. Clean installation, current-source CI
and public download remain separate open gates.

## Local review

Open `http://127.0.0.1:5174/#/work`. This is the production frontend with preview
fixtures, so the example runs and account statuses are sample data. Start a New
run, write an outcome, visit Setup and return with Continue draft. In Setup, use
search to find a control such as density, then check Appearance in both themes.
Use Ctrl+K and the workspace switcher with the keyboard; Stop opens the existing
confirmation. These are useful acceptance journeys for the UI without claiming a
real agent run.
