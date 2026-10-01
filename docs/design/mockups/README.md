---
title: Pytxo visual proposals — workbench and control boundary
date: 2026-09-16
status: approved-visual-intent
---

# Visual proposals, not implementation evidence

Generated with Codex's built-in ImageGen and inspected visually. The image-generation pass changed no frontend or backend implementation. The subsequent approved implementation is tracked in RELEASE_READINESS. Every screen is an illustrative fixture, not an observed native run, verified result, accepted release or public screenshot. The source baseline is branch `codex/beta-candidate-verification`, HEAD `72879702f90f2b74eece888bb117df59608f9b56` plus inherited dirty work recorded in the September 16 checkpoint.

The user approved the recommended standalone targets on September 16 for local Desktop and website implementation. This approves visual intent only; Core contracts remain authoritative and publication requires separate authorization. The hypervisor treatment supersedes the initial conventional workbench treatment.

## September 18 — implemented workbench craft pass

The latest local frontend pass keeps the approved glyph, execution topology,
connected task inspector, visible Review candidate map and spectral repository
boundary. It consolidates them into a more deliberate operating surface:

- Work places the mission, execution state and exact-run actions in one compact
  command area. The graph, selected task and inspector use distinct slate depth;
  active, selected and focused states retain separate meanings.
- Review treats candidate relationships as context for one bordered code
  workspace. Prepared-file selection, exact candidate, recorded verification and
  the Apply boundary remain visible without competing with the comparison.
- History uses a settled recorded-state surface. Candidate evidence and Apply
  records remain visibly separate where correspondence is unavailable.

Browser fixture captures are under `target/laptop-workbench-20260918/` with the
`craft-final-` prefix at 1600×1000, 1280×720 and 860×760. They are rendered UI
evidence, not native WebView, packaged mission or Apply evidence. The mockups
remain intent references; the current components are now the implementation
target for further acceptance.

## Approved follow-up: spatial Work, visible candidate relationships in Review

The September 17 correction keeps the task title, selectable file rows, repository
context and neutral exact-content comparison. On wide available content (900px+),
Review shows a compact candidate map by default: actual prepared files converge
on the exact candidate, recorded checks lead to the repository Apply boundary.
These are candidate relationships, not code dependencies. The explicit Focus on
code preference persists. Narrow or dock-constrained content defaults to the
collapsed summary; an explicitly opened map becomes a keyboard-accessible list
layout. Large inventories show three files plus access to the full existing list.
No extra authority or inferred telemetry is introduced. This supersedes the
previous default-collapsed candidate-map treatment, not the approved visual world.

## Recommended visual targets

| Surface | Target | Operator question |
|---|---|---|
| New work | `desktop-new-work-final.png` | What am I asking for, in which repository, with which checks? |
| Execution | `desktop-execution-final.png` | What is active, what waits, and which dependency causes the wait? |
| Review / Apply | **`desktop-review-hypervisor-final.png`** | What is proposed, what evidence exists, and what can cross into this repository? |
| History / recovery | `desktop-history-final.png` | What boundary event was recorded, and is its outcome known? |
| Website | **`website-home-final.png`** | Why use Pytxo, what does its workbench do, and where do I start? |
| Motion | `motion-storyboard.png` | Which recorded state change does a transition explain? |

Open PNGs directly at full size. Do not use the smaller website reproduction as the Desktop specification. These independent fixtures illustrate different states; they are not a continuous, recorded mission.

## Composition comparison (manual Impeccable-informed image review)

This is a manual visual comparison, not a dual-agent Impeccable critique run, DOM detector result, accessibility audit or usability experiment.

| Option | Hierarchy and density | Usability and feasibility | Distinctiveness and fit | Verdict |
|---|---|---|---|---|
| Review A — `desktop-review-a.png` | Changes and evidence read horizontally; large empty code area in this probe. | File list and exact Before/After are close to existing components. Fixed bottom decision is easy to find. | Calm Chroma fit, but conventional. | Selected as the initial structural foundation. |
| Review B — `desktop-review-b.png` | Persistent evidence rail competes with code; stacked Before/After needs more vertical scanning. | Rail consumes narrow-window width; file tabs scale poorly to many changes. | Serious but less distinctive and less efficient. | Retained as a rejected alternate. |
| Refined workbench — `desktop-review-final.png` | More compact header and clearer evidence; read-only destination. | Most direct translation from current exact-content reader. | Useful fallback, insufficient hypervisor identity for latest direction. | Superseded as the primary visual target; retain for responsive simplification. |
| Hypervisor Review — `desktop-review-hypervisor-final.png` | Proposed files converge into one candidate; code and verification remain left of the aperture; destination is visibly separate. | Feasible with existing data and simple DOM/SVG. Wide-only destination bay must collapse in narrow views. | Strongest Pytxo-specific moment without decorative topology. | Recommended. |
| Website A — `website-home-a.png` | Split hero has an explanatory diagram plus a large product view. | Readable, but repeats the same boundary story twice. | Stronger than old marketing; still diagram-led. | Alternate. |
| Website final — `website-home-final.png` | Large statement followed immediately by product-led presentation. | Existing walkthrough tabs can carry it; Review tab is correctly selected. | Lets the signature product interaction sell the product. | Recommended. |

## The diagrammatic system

- **Execution bay:** task nodes and orthogonal edges reflect approved tasks and recorded dependencies. The active-worker bracket marks an actual worker/task association. Geometry is dependency order, not elapsed time or percentage. Never draw activity from a timer.
- **Candidate bay:** task outputs converge into a single prepared manifest. Future stages are labeled as possible/pending until backend evidence establishes their state. Do not invent preparation sub-stages where only one status exists.
- **Chroma Aperture:** a narrow branded separator identifies the reviewed repository Apply boundary. Brand spectrum is not a success scale or progress signal. A broken connector means no confirmed integration; a decorative opening in a line is never authority.
- **Canonical repository bay:** a read-only destination and prepared impact paths. It is not a complete filesystem inventory, semantic dependency map, immutable Git commit, or proof that external actors left the repository untouched.
- **History traces:** recorded candidate/Apply/outcome states shown as compact boundary events. Unknown, interrupted, unapplied and discarded are distinct. Only show events actually persisted; never manufacture chronology from a final status.

The ordinary UI remains ordinary: navigation, text fields, search, buttons, file lists, disclosure and exact content. Signature Pytxo language belongs in dependency relationships, candidate convergence and the Apply boundary, not every control.

## Interaction and motion intent

1. Selecting a task highlights its direct recorded prerequisites and updates the existing task inspector. The full accessible list remains available; nodes must be keyboard operable.
2. A one-worker bracket may relocate over about 180 ms after a recorded task/worker transition. It is not a moving progress indicator. Missing or ambiguous worker identity means no bracket.
3. Candidate file tokens may settle into one manifest over about 200 ms after that manifest is available. Do not animate invented assembly progress.
4. Opening Review, opening confirmation, or changing a selection never moves anything across the aperture.
5. After explicit authorization and backend-confirmed Apply success, a short 220 ms acknowledgement may connect the candidate to its recorded destination. It is retrospective feedback, never an optimistic write preview. Failure or uncertainty retains an unresolved boundary and the appropriate recovery message.
6. Reduced motion updates directly. Labels, focus, errors and controls update immediately and never wait for choreography. No loops or ambient pulses.

## Directly implementable versus conceptual

**Directly implementable:** existing shell, typography/token roles, contextual actions, exact content navigation, evidence summaries, read-only destination, task/dependency layout from existing plan data, candidate-path convergence from a prepared manifest, and unknown/recovery presentation through current state contracts. Use bounded DOM/SVG paths; no WebGL, 3D AST or new graph subsystem.

**Needs simplification/validation:** fitting arbitrary large DAGs, selecting a stable subset without hiding blockers, aligning a worker bracket with recorded task identities, derived waiting reasons, wide Review destination bay, and per-attempt history traces. If the required persisted fact is absent, show unavailable or omit that visual element. Do not broaden Core to match the artwork.

At narrow widths, collapse the destination bay to a labeled boundary strip, retain readable code, and present stages/dependencies as a structured list. Large graphs should expose the active task and direct blockers with an explicit route to the complete task list; never pretend the partial graph is complete. Native DPI, keyboard, contrast, reduced motion and performance remain untested by these images.

## Do not copy these ImageGen details literally

- Generated logo, glyphs, font metrics, line weights, code and line numbers are illustrative. Use existing vetted assets, installed fonts and exact decoded content.
- The example repository, tasks, commands, files and states are fixtures. Do not hardcode them or invent check outcomes. The Review examples deliberately lack combined verification and disable Apply.
- Repeated file names in the convergence diagram, navigator and destination are a compositional probe; compress duplication when the diagram no longer adds information.
- Cyan denotes focus/activity in these proposals, not a new epistemic state. Reconcile it with the existing theme/accent contract; unknown stays neutral and worker completion does not become green verification.
- The website's miniature app is ImageGen's reproduction, not an exact screenshot. Its compressed footer must not override the standalone Review target. Replace it with a real capture of the accepted build before publication.
- The website headline is scoped by its nearby repository-change explanation. It does not authorize universal side-effect, sandbox, network or production-system claims.
- Fixture banners belong to these design artifacts, not the eventual customer UI. Conversely, previews must retain honest provenance until real accepted screenshots replace them.
- History's candidate caption naming one selected path does not mean the entire candidate contains only that path. Implement counts and selection separately.
- The static motion storyboard illustrates conditional transitions only; no animation has been implemented or rendered.

## Generation provenance and prompt briefs

Built-in `image_gen` was used, not a CLI/API fallback. Full generation and correction prompts are in this conversation's tool calls. The following condensed prompt briefs describe the final set:

- **Shared:** Chroma Aperture, flat near-black surfaces, off-white Sora-like UI type and IBM Plex Mono evidence, fine separators, compact shell, explicitly labeled fixtures; no invented metrics, telemetry, verification or enforcement. Existing current-source fixture captures supplied the initial style reference.
- **Composer:** selected repository/Codex/one-worker context; bounded request; command inputs explicitly not results; Build plan; secondary guidance.
- **Execution:** approved task dependency graph, one active-worker bracket, waiting reason, candidate pending stages, separate primary repository bay; no fake progress.
- **Review:** preserve changed content → verification → attention → decision; literal path convergence into one speculative candidate; exact Before/After; missing evidence; disabled Apply; read-only destination beyond the aperture. Corrected clipped code and the initially misleading decision subtitle.
- **History:** recorded boundary traces and interrupted Apply with unknown repository outcome; reconciliation action; no rollback claim. Corrected generated confusion between test commands, worker execution and Apply-attempt metadata.
- **Website:** product-led statement and large Review preview, supported-path CTA, explicit repository boundary limitation. Corrected wrong selected tab and generated overbroad write claims.
- **Motion:** four conditional story frames: bracket handoff, manifest formation, Review remains speculative, acknowledgement only after confirmed Apply. No runtime proof.

Images containing unintended green completion marks, overbroad repository claims or misleading Apply metadata were rejected and regenerated. Earlier rejected generations remain in Codex's generated-image directory, not in the recommended set.

## Handoff boundary

Local frontend implementation is explicitly authorized by the user's September 16 instruction. These artifacts do not authorize state-machine changes, public uploads, commits or deployment. Protect Review identity, stale invalidation, backend eligibility, bounded content loading and conservative recovery.

## Aperture Orb exploration — awaiting user selection

Two ImageGen concept boards based on the current Work capture and September 17
expanded Review capture (not older design mockups):

- `aperture-orb-a-split-shell-v1.png`: opaque graphite split shell, oblique spectral seam; recommended for clear silhouette, restraint and direct relationship to the Apply boundary.
- `aperture-orb-b-optical-core-v1.png`: recessed optical iris and titanium housing; stronger mechanical depth but more camera-like and more expensive to render.

Each board shows Work, compact Review and website placement. These are visual
proposals with fixture content, not implemented UI, motion, performance results or
native evidence. Neither treatment has been selected. No renderer or production
code was changed. Stop for selection before the isolated Three.js prototype.

Impeccable critique: A preserves the operational composition and stays recognizable
at small sizes. B places the Review object correctly beside candidate identity,
but its extra Work instrument block competes with actual task relationships. A's
website staging is too theatrical; B's upright product demonstration is the better
website composition. Both preserve the visible prepared-file convergence map.

Do not copy generated drift: A places Review's orb beside the mission title; the
prototype should instead use a settled 24–28px treatment beside candidate identity.
Omit A's stone/laptop staging, gradient headline and invented slogans. Omit B's
extra named instrument block, screws at small sizes, and generated “isolated” copy;
this visual analogy makes no VM-level isolation claim. Generated code, paths,
labels and glyphs must never replace actual product data. Keep required-check
success distinct from confirmed Apply, as the boards' fixture labels indicate.

After selection: use the existing Three.js dependency for one local fixture
prototype, with static/reduced-motion fallback and measured lifecycle/performance.
Production integration needs separate approval. No dependencies installed, uploads,
purchases, commits or publication occurred during this concept pass.

## Aperture Core — software-native 2.5D correction

The user selected A's silhouette/concept while explicitly rejecting its photoreal
rendering and B's optical-hardware direction. The next concept is
`aperture-core-2p5d-v2.png`: stylized matte charcoal lobes, one oblique spectral
seam, broad soft shading and a clean contour. No metal grain, lens housing,
photographic staging or physical-device analogy. This rendering is a proposal
awaiting confirmation, not an implemented asset or motion prototype.

Recommended direction: 64px calm Work object beside mission context; settled
24–28px Review object beside candidate identity; roughly 180px website object
with a slightly stronger localized aura. Preserve the actual logo and copy:
the board's invented slogans, altered logo and overscaled Work orb are image
generation drift, not requirements. The board places the Review object correctly
and retains file convergence, checks, destination and no-confirmed-Apply wording.
All product states still come from existing records; no motion has been tested.

This supersedes both prior boards' physical material treatment. A remains the
conceptual base; B is not the recommended alternative. No frontend changes,
new dependencies, prototype, publication or performance claims in this pass.


## September 17 — typographic Aperture / Work inspection study

**The glossy split-shell Aperture Core is superseded**, including the 2.5D
material treatment. Keep the earlier images as design history; do not recolor or
integrate them. The current proposal is a character-only spectral volume with a
diagonal negative-space aperture. Hollow halo is the quieter alternative.
Neither treatment is selected for production yet.

Local preview: `glyph-work-preview/index.html` (serve repository root, then open
`/docs/design/mockups/glyph-work-preview/`; `?view=studies` compares both marks).
Only this isolated HTML/CSS/JS preview and the handoff were authored for this
pass. No production imports, IPC, backend edits, dependencies or publication.
Existing navigation names and execution relationships guide the concept; disabled
navigation/review actions deliberately do not pretend to be production controls.

- `volume-motion.webm`, `halo-motion.webm`: same 120px footprint and background;
  labeled running → waiting → checking → ready fixtures. Ready is not applied.
- `work-wide.png`: 1600×1000 recommended Work composition; connected task
  selection and one Output / Files / Evidence inspector.
- `work-narrow.png`: 860px adaptation, inspector follows the canvas. Below 650px
  the dependency layout becomes an ordered list. Selecting a task scrolls its
  inspector into view on narrow layouts; this interaction is a proposal.
- `studies.png`: side-by-side static comparison; `work-static.png`: reduced-motion
  capture (before the final small typography/hover correction).
- `evidence.json`: measured timing and 11 local interaction/lifecycle checks.

Recommendation: volume. Its negative-space aperture is more distinctive; the
halo risks looking like a spinner. Canvas 2D draws fixed upright monospace glyphs;
only coherent brightness/color fields and aperture angle evolve. No underlying
sphere image, WebGL, shader, random text stream, telemetry or progress inference.
At this footprint individual glyphs are texture, not text to read. Larger website
and compact Review treatments are intentionally outside this pass.

Measured on Windows, Ryzen 7 7735HS, headless Chrome 152.0.7977.83: three 2.2s
runs with and without the volume. Active drawing: 63–64 draws/run (~29/s), median
2.1ms and p95 2.7–2.8ms CPU draw time. Baseline: zero glyph draws. Both had ~7ms
median animation-frame callback spacing and no observed long tasks. These short
runs do not measure display FPS, GPU cost, battery use or native WebView behavior.
Waiting, reduced motion, offscreen pause and unmount/pagehide cleanup passed;
hidden-document pause is implemented but was not independently exercised.

Impeccable: improved surface separation, visible selected-node brackets, readable
operational labels, one contextual inspector, and explicit unknown/unprepared
states. Detector's remaining repeated task-title warning is intentional: node,
prerequisite reference and selected-task inspector must identify the same task.
Final preview retains fixture-only state controls. Mobbin search was blocked by
its paid-plan gate; no references were retrieved or reused. Native Windows/DPI,
real runtime, large-graph fallback and production acceptance remain NOT RUN.

Next gate: Matt selects the typographic treatment and Work composition. Do not
integrate into production or redesign Review/website without that approval.


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

## September 18 — outcome browser and Work finishing (implemented locally)

The selected hypervisor direction remains unchanged. History is now an
outcome-first browser: compact all-repository navigation, a larger selected-run
workspace, separate execution/integration snapshot lanes, candidate-bound checks,
applied-file manifests and record-dependent next actions. It is deliberately
static, with explicit timeline gaps rather than reconstructed agent activity.
Technical receipts remain under Technical details. Narrow layouts use a
list-to-detail flow with focus-restoring Back navigation.

The bounded presentation follow-up removes the default fallback-title explanation,
consolidates the unapplied message, and moves record/provenance caveats into
contextual disclosures. Request, destination, execution, integration outcome and
the available next action now lead. Files and checks remain directly inspectable;
recovery warnings remain visible.

Work retains the spectral glyph, visible graph and connected inspector. Saved
request and task prose now leads where available; IDs remain secondary. Active
brackets, selected surfaces and keyboard outlines are independent. Header copy
is consolidated; long inspector descriptions use their own bounded row, with
full text available in Source details. No new glyph or backend semantics.

An approval is promoted inside Work only when its recorded domain and run match
the focused run. Global approval count remains available in the shell but does not
create a run-level blocked state. The exact-run decision, Review and Stop form one
action hierarchy. Selected queued work uses a separate surface while the actual
running task keeps the worker bracket and recorded relationship highlight.

Local matched before/after and state fixture captures are in
`../../../target/presentation-refinement-20260918/`. Use
`after-history-applied-void-1600.png`, `after-work-decision-void-1600.png`, and
`after-work-selected-queued-void-1600.png` as presentation references, not proof
of native execution. Verification and limitations are recorded in
`../../../RELEASE_READINESS.md` (September 18). Native WebView, DPI and real-run
acceptance remain NOT RUN. No publication or release actions were performed.


## September 18 — laptop-first refinement (locally implemented)

Keep the existing hypervisor direction. Work uses its connected inspector beside
recorded tasks at laptop width, or directly below tasks before candidate details
when narrow. Inspect selected task is an explicit keyboard destination. The
glyph, recorded flowlines and active/selected/focus distinctions are unchanged.
Inspection tools use the existing disclosure on both Work and Review.

Review keeps its map visible by default. A compact relationship band and ready-only
decision row recover comparison space: 247.5px visible at 1280x720 with substantial
fixture code. Stale, refusal and recovery explanations stay visible. History uses
available width, separates candidate evidence from unlinked integration records,
and exposes prepared versus applied inventories distinctly.

Use `target/laptop-workbench-20260918/final-work-1280.png`,
`accepted-review-ready-long.png`, and `final-history-1600.png` in that directory
as current browser-fixture references. Matched `before-*` and `final-*` captures
cover 1600x1000, 1280x720 and 860x760; additional long-code stale/missing/focused
Review captures preserve state distinctions. These are not native acceptance or
marketing proof. See RELEASE_READINESS for test results and remaining native/DPI
checks. No new visual concept, backend authority or publication was introduced.
