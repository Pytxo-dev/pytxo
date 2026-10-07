---
title: Flat Desktop 02 implementation
status: active
date: 2026-09-13
tags: [desktop, ux, verification]
---

# Flat Desktop 02 implementation

Matt explicitly approved `target/pytxo-flat-desktop-02/PROPOSAL.md` and its
bounded implementation sequence on September 13: "yes, I approve, let's implement."
The exact packet remains preserved. No additional design approval is needed for
that scope. Installs, spending, security/account changes, pushes, merges,
publication and deployment retain their existing gates.

## Slice 1 — onboarding and visual foundation

Implemented four visible stages: Welcome, Agent, Workspace, Ready. Optional
Pytxo terminal-tool checks remain reachable within Agent; Display is optional
from Ready. Native title-bar identity is preserved; duplicate onboarding logos
are removed. Short stages use smaller panels, actions remain anchored, and long
recent-workspace lists scroll independently. At minimum window size, necessary
body scroll remains available. Heading focus follows navigation, selected display
options expose pressed state, and Back retains the selected workspace.

Permission emblems now use flat vector silhouettes based on the approved image
generation study; original generated PNGs remain preserved. Sourced Cursor,
OpenCode and Gemini CLI marks appear beside text names in Agents. Other identities
use text rather than invented vendor marks. These are identity art, not evidence
badges. Optional CLI discovery failures are caught and leave a Desktop escape.

Initial Svelte/CSS check passed with no diagnostics; 59 affected browser tests
passed. After native inspection exposed recent-list overflow, the three targeted
onboarding tests passed, including a five-folder regression and minimum-size
110% app zoom navigation. Final source Svelte/CSS passed again. Matrices overlap;
do not sum them as unique coverage. Browser fixtures do not prove native execution.

The first native validation EXE (87686E6A…) reached Welcome, actual installed-CLI
status, Workspace and Ready through Computer Use. Discovery reported six installed
CLIs and two ready sessions. No sign-in, trust grant, paid model call, Apply or
installation was performed. That first build predates the final recent-list and
short-stage adjustments; final candidate identity/evidence is recorded under
`target/flat-02-native/`. This is a validation executable, not a frozen release.

One root-directory test invocation attempted an npm-cache Playwright fetch. It was
stopped; tests were rerun using the existing Desktop installation with
`npx --no-install`. Project package manifests/lockfiles remained unchanged.

## Slice 1b — onboarding and menu refinement

Matt then requested a fresh onboarding/menus mockup and implementation in the
actual Desktop, with mature hierarchy and automatic resizing. Generated one
six-interface board and implemented its structure using real controls. Reference,
slice diff, prior-source copies and evidence: `target/flat-02-menus/`.

Onboarding now uses a horizontal step bar and common Back/action footer. Setup
reflows against available container width and retains section selection. Workspace
switcher, editor, command menu and layout menu separate scrolling content from
fixed navigation/actions. Layout menus have grouped real views/saved layouts and
Escape dismissal. Approvals keep metadata at full height with an anchored decision
footer; the empty inbox becomes one compact message. Existing permission labels,
Chroma/state semantics, actual data, Stop/review/Apply and freshness are preserved.

Initial affected browser suite: 86 passed. Screenshot/native review revealed and
fixed approval metadata clipping, a duplicated tall empty state and absent menu
Escape. Final 21 affected tests passed; final Svelte/CSS checks have no diagnostics.
Do not add overlapping test matrices. Native/source lineage and remaining native
acceptance are recorded in `target/flat-02-menus/identity.json` and its README.
No new dependencies or external actions. This does not complete all 46 interfaces,
real central-model supervision, additional adapters or exact-installer acceptance.

## Next bounded slices

1. Bind mission supervision and ADE panels to real domain/run/task/session/control
   records. Keep worker prose separate from recorded control. Add linked folder
   runs with separate reviewed Apply/freshness/recovery receipts; no cross-root
   atomic Apply. Existing onboarding project-catalog entries also need their
   manifest-to-primary-root routing reconciled with project selection.
2. Grok Build adapter first; validate argv/cwd/output/cancel/permissions before
   claiming runtime support. Factory/Goose only with verified contracts; Grok Bot
   remains feasibility-only. No install or paid-call authority is inferred.
3. Complete the approved interface inventory and native 100/125/150% Windows
   scaling, reduced-motion and failure-state acceptance; freeze the integrated RC.
4. Clean Windows acceptance of the exact installer; matching real MP4/Bench,
   website/docs and release materials; separate explicit publication approval.

Interactive agent TUIs, nested splits, floating windows, extension marketplace,
sessions surviving app termination and a replacement IDE remain deferred.

Related: [[astra-ui-feedback-2026-09-13]], [[astra-native-finish-2026-09-10]].

## Slice 2a — responsive collections and recorded activity

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
