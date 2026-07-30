# Plan: Pytxo Desktop readiness pass 1

## Context

This pass follows the product and architecture orientation in:

- `docs/01-projects/pytxo-architecture-research.md`
- `docs/01-projects/pytxo-improvement-research.md`
- `docs/01-projects/desktop-dangerous-ux-2026-07.md`
- `docs/05-adr/ADR-0032-desktop-2-focus-flow-primary.md`

## DO NOT TOUCH

- Orchestration policy, permission profiles, execution domains, Signal Core, Blast Shield, or Race Shield behavior.
- Desktop route names or information architecture.
- The legacy `desktop_shell_v1` shell.
- Marketing surfaces in `apps/web`.
- Existing unrelated or untracked workspace files.

## Phase 1: Restore verification truth

- [x] Align Desktop Playwright expectations with the current Desktop 2 copy and interaction states (`apps/desktop/e2e/shell.spec.ts`).
- [x] Add a planned-Flow regression check at the supported Desktop widths (`apps/desktop/e2e/shell.spec.ts`).
- [x] Keep updater behavior honest in Tauri while preventing browser-preview IPC error noise (`apps/desktop/src/components/shell/UpdateBanner.svelte`).

## Phase 2: Fix responsive Flow and build warnings

- [x] Keep the workspace, agent CLI, and Build plan controls reachable when the plan preview opens at 1280px (`apps/desktop/src/components/desktop2/desktop2-shared.css`).
- [x] Resolve the current Svelte accessibility/reactivity/unused-style warnings without changing product behavior (`AppBar.svelte`, `Sidebar.svelte`, `SettingsScreen.svelte`).

## Phase 3: Verify and record

- [x] Run Svelte checks, the production build, and Desktop Playwright coverage.
- [x] Inspect default and planned Flow at 1600x900, 1280x800, 1280x720, 1024x576, and 960x640.
- [x] Update the Desktop improvement backlog with evidence and remaining gaps (`docs/01-projects/desktop-ui-improvement-backlog.md`).

## Acceptance criteria

- Current Desktop 2 end-to-end tests use current accessible names and states.
- Planned Flow has no clipped controls or page-level horizontal overflow at supported widths.
- Browser previews do not show a false updater failure.
- `npm run check` and `npm run build` complete without Svelte warnings.
- Desktop Playwright tests pass, or any out-of-scope failure is documented with its exact command and location.

## Test commands

- `npm run check`
- `npm run build`
- `$env:PLAYWRIGHT_CHANNEL='chromium'; .\node_modules\.bin\playwright.cmd test --reporter=line --workers=1`

---

# Plan: Pytxo Desktop readiness pass 3

## Context

This pass completes the remaining Ops portion of keyboard-first operations from
`docs/01-projects/desktop-dangerous-ux-2026-07.md` without turning a shortcut
into an ambiguous process kill.

## DO NOT TOUCH

- Permission-profile policy, HITL classification, Blast Shield flush behavior, Race Shield claims, or sandbox cleanup.
- Any execution domain other than the domain explicitly associated with the selected run.
- Desktop routes, primary navigation labels, the legacy `desktop_shell_v1` shell, marketing surfaces, or unrelated workspace changes.

## Phase 1: Make stop targeting exact

- [x] Add an orchestration guard that refuses to stop when the requested run is no longer the active run for that execution domain (`crates/pytxo-orchestrate/src/lib.rs`, `crates/pytxo-orchestrate/tests/stop_exact.rs`).
- [x] Thread the explicit run ID and domain ID through the Tauri IPC and Desktop backend, while preserving the legacy selected-domain stop call (`apps/desktop/src-tauri/src/ipc.rs`, `apps/desktop/src/lib/ipc.ts`, `apps/desktop/src/lib/desktop-backend.ts`, `apps/desktop/src/lib/desktop-backend.preview.ts`).

## Phase 2: Ship confirmation-first keyboard Ops

- [x] Add a guarded Ctrl/Cmd+Shift+O shortcut that opens Ops and focuses the first active run, without intercepting editable controls or repeated key events (`apps/desktop/src/components/desktop2/DesktopShell.svelte`, `OperationsScreen.svelte`).
- [x] Let a focused active-run row open a stop confirmation with Ctrl/Cmd+Shift+Backspace; require a separate confirm action and explain that processes stop while the sandbox remains unflushed (`OperationsScreen.svelte`, `desktop2-shared.css`).
- [x] Expose pending, success, and failure feedback; update preview run, agent, and workspace state after a confirmed stop (`OperationsScreen.svelte`, `desktop-backend.preview.ts`).

## Phase 3: Verify and record

- [x] Cover exact-run refusal, shortcut guards, confirmation, cancellation, success, and compact layout behavior (`crates/pytxo-orchestrate/tests/stop_exact.rs`, `apps/desktop/e2e/shell.spec.ts`).
- [x] Inspect Ops against the production bundle at 1280x800 and 960x640, then update the Desktop readiness notes (`docs/01-projects/desktop-ui-improvement-backlog.md`, `docs/01-projects/desktop-dangerous-ux-2026-07.md`).

## Acceptance criteria

- A stale UI cannot stop a different active run in the same execution domain.
- No keyboard chord directly terminates processes; it only opens an exact-run confirmation.
- The confirmation names the workspace and run and states that the sandbox is not flushed or deleted.
- Ctrl/Cmd+Shift+O and the stop-request chord ignore editable targets and repeated key events.
- Stop success updates active-run, agent, and workspace evidence in browser preview and refreshes native snapshot evidence.
- Ops has no page-level or internal horizontal overflow at supported widths.

## Test commands

- `cargo test -p pytxo-orchestrate --test stop_exact`
- `cargo build -p pytxo-desktop`
- `npm run check`
- `npm run build`
- `$env:PLAYWRIGHT_CHANNEL='chromium'; .\node_modules\.bin\playwright.cmd test --reporter=line --workers=1`

---

# Plan: Pytxo Desktop readiness pass 2

## Context

This pass implements the open keyboard-first operations recommendation in
`docs/01-projects/desktop-dangerous-ux-2026-07.md` for the decision inbox.

## DO NOT TOUCH

- Approval policy, HITL classification, permission profiles, or Blast Shield behavior.
- Desktop routes, primary navigation labels, or the legacy `desktop_shell_v1` shell.
- Marketing surfaces or unrelated workspace changes.

## Phase 1: Define the safe decision contract

- [x] Audit the current Approvals selection, evidence, empty, success, and failure states (`apps/desktop/src/components/desktop2/CollectionScreen.svelte`, `apps/desktop/src/lib/desktop-backend.preview.ts`).
- [x] Keep single-key shortcuts navigation-only; require Ctrl/Cmd chords for approve and deny.

## Phase 2: Implement keyboard-first evidence and decisions

- [x] Add J/K selection, Ctrl/Cmd+Enter approval, and Ctrl/Cmd+Backspace denial with editable-target and key-repeat guards (`CollectionScreen.svelte`).
- [x] Map real HITL action codes to accurate decision language while keeping the raw code visible (`CollectionScreen.svelte`).
- [x] Surface requester, workspace, request time, and an honest Run Review link when matching evidence exists (`CollectionScreen.svelte`).
- [x] Give the preview enough state to verify multiple decisions without fabricating native behavior (`desktop-backend.preview.ts`).

## Phase 3: Verify and record

- [x] Add accessible-name, keyboard, decision, and evidence regressions (`apps/desktop/e2e/shell.spec.ts`).
- [x] Inspect Approvals at 1280×800 and 960×640 against the rebuilt production bundle.
- [x] Run Svelte checks, production build, all Desktop Playwright tests, and update the Desktop backlog.

## Acceptance criteria

- J/K changes the selected approval only when focus is outside editable controls.
- No destructive action is bound to an unmodified single key.
- Approve and deny chords resolve only the selected request and expose success state.
- The decision pane identifies who requested what, in which workspace, and when.
- Run evidence opens when a matching run exists; unavailable evidence is labeled honestly.
- The Approvals layout has no page-level horizontal overflow at supported widths.

## Test commands

- `npm run check`
- `npm run build`
- `$env:PLAYWRIGHT_CHANNEL='chromium'; .\node_modules\.bin\playwright.cmd test --reporter=line --workers=1`

---

# Plan: Pytxo Desktop readiness pass 4

## Context

This pass addresses the open DPI and scaling recommendation in
`docs/01-projects/desktop-dangerous-ux-2026-07.md`. The current preference
implementation already keeps OS DPI and user zoom independent, but its browser
regression proves only persistence. The Tauri window also permits 860x560 while
the verified responsive floor is 960x640. Native inspection on a Windows host
at 125% showed that plain debug or release Cargo builds without Tauri's
`custom-protocol` feature open `devUrl`; a self-contained current production
artifact requires the frontend build first and that explicit feature.

## DO NOT TOUCH

- Permission-profile policy, execution-domain selection, HITL classification,
  Blast Shield flush/cleanup, or Race Shield claims.
- Desktop routes, primary navigation labels, display-scale choices, or the
  one-release `desktop_shell_v1` rollback path.
- OS DPI multiplication. WebView2 owns monitor scaling; Pytxo user scale stays
  an independent 85/90/100/110% zoom.
- Marketing surfaces or unrelated workspace changes.

## Phase 1: Prove the scale contract

- [x] Replace the persistence-only appearance test with rendered zoom,
  reload, invalid-value, and reset assertions
  (`apps/desktop/e2e/shell.spec.ts`, `apps/desktop/src/lib/ui-prefs.svelte.ts`).
- [x] Add a 100/125/150% device-scale matrix at the native 860x560 floor,
  covering 90/100/110% user zoom, titlebar controls, page geometry, active
  work, dialogs, and internal horizontal overflow
  (`apps/desktop/e2e/dpi.spec.ts`).
- [x] Treat browser `deviceScaleFactor` as simulated coverage, and record
  physical Windows evidence separately.

## Phase 2: Align the native window with proven geometry

- [x] The strict 860x560 matrix passed at the existing native floor, so no
  CSS, breakpoint, or minimum-window change was needed.
- [x] Preserve `setZoom` for Tauri and CSS `zoom` only for browser previews;
  verify a monitor-scale change never multiplies the user preference
  (`apps/desktop/src/lib/ui-prefs.svelte.ts`).
- [x] Make the native verification sequence build the frontend before the
  native release build, so the inspected executable cannot contain a stale
  Desktop bundle (`AGENTS.md`, Desktop test commands).

## Phase 3: Verify and record

- [x] Run Svelte diagnostics, the production web build, the complete desktop
  Playwright suite, the ordered native build, and the relevant Rust gates.
- [x] Inspect the rebuilt native app at this host's real Windows 125% DPI in
  default and minimum-size layouts; verify the custom titlebar and newest Ops
  affordances are visible.
- [x] Update the readiness backlog and DPI recommendation with simulated versus
  physical evidence. Keep macOS Retina and unobserved physical Windows scale
  factors explicitly open.
- [x] Close the accumulated exact-stop lifecycle finding with an atomic
  running-only worker finalization and a genuinely dispatched blocking-run
  regression (`pytxo-store`, `pytxo-orchestrate`).

## Acceptance criteria

- Every scale choice changes rendered zoom, persists across reload, and resets
  invalid storage to 100%.
- OS device scale and user zoom remain independent; no factor is multiplied
  twice.
- The native minimum window has no page-level or critical-pane horizontal
  overflow at 90/100/110% user zoom under simulated 100/125/150% DPI.
- Titlebar controls, active-run review, approval decisions, and stop
  confirmation remain visible and keyboard reachable at the proven floor.
- The native executable inspected after the ordered build contains the current
  Desktop source, not an older `dist` tree.
- Documentation distinguishes automated scale emulation from physical device
  verification.

## Test commands

- `npm run check`
- `npm run build`
- `$env:PLAYWRIGHT_CHANNEL='chromium'; .\node_modules\.bin\playwright.cmd test e2e/dpi.spec.ts --reporter=line --workers=1`
- `$env:PLAYWRIGHT_CHANNEL='chromium'; .\node_modules\.bin\playwright.cmd test --reporter=line --workers=1`
- `npm run build:native` (from `apps/desktop`; runs the frontend build before
  `cargo build -p pytxo-desktop --release --features custom-protocol`)
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`

---

# Plan: Pytxo product-truth marketing pass 5

## Context

The canonical vision now leads with one inspectable mission becoming safe
parallel work and one verified result. The current marketing hero still leads
with agent count and renders the Pytxo logo as its only visual. The existing
Desktop attachment names are also misleading: every 1600x1000 file currently
has the same SHA-256 and shows Operations, so those aliases cannot support a
truthful Flow / Ops / Approvals story.

This is a preserve-mode marketing redesign. Keep the Chroma palette, Geist
type, navigation, routes, product names, and honest claim boundaries. The
target is clearer product evidence and a shorter path from thesis to download,
not a new brand.

## DO NOT TOUCH

- Billing, Clerk authentication, account, pricing, plans, or checkout behavior.
- Public route slugs, primary navigation labels, legal copy, or analytics
  contracts.
- Desktop orchestration policy, permission profiles, execution domains, or
  sandbox semantics while capturing assets.
- Unmeasured performance claims, invented testimonials, fake customer logos,
  generated product screenshots, or aspirational Cloud/Ultra claims.
- The Desktop product UI except for a deterministic screenshot-capture harness.

## Phase 1: Establish truthful product evidence

- [x] Add a deterministic Desktop marketing-capture path for current Flow,
  Operations, and Approvals states at desktop and compact widths
  (`apps/desktop/e2e/marketing-captures.spec.ts`,
  `apps/desktop/package.json`, `apps/web/public/product/`).
- [x] Assert each committed capture has the expected visible heading, dimensions,
  and a distinct content hash so aliases cannot silently point to the same
  screen (`apps/desktop/e2e/marketing-captures.spec.ts`,
  `apps/web/scripts/verify-product-assets.mjs`).
- [x] Replace or retire the duplicate aliases in
  `docs/_attachments/desktop-2/`; never present one Operations image as Flow,
  Approvals, Runs, or Settings.

## Phase 2: Lead with the mission loop and real UI

- [x] Rewrite the hero around the canonical mission loop and replace the logo
  panel with the current Operations capture. Keep one concise primary CTA
  intent and move the install command below the hero
  (`apps/web/src/components/site/hero.tsx`,
  `apps/web/src/components/site/install-snippet.tsx`,
  `apps/web/src/app/(marketing)/page.tsx`).
- [x] Replace the numbered install rows with a pinned, inspectable
  Plan / Run / Approve product story using the three real captures. Isolate the
  motion in one client leaf, lazy-load it, and render a complete static sequence
  under `prefers-reduced-motion`
  (`apps/web/src/components/site/product-story.tsx`,
  `apps/web/src/components/site/how-it-works.tsx`,
  `apps/web/src/app/globals.css`).
- [x] Put plain-language protections before Signal / Blast / Race codenames,
  preserve every measurement caveat, and remove decorative step numbering
  (`apps/web/src/components/site/feature-grid.tsx`,
  `apps/web/src/components/site/proof-band.tsx`).

## Phase 3: Prove hierarchy, responsiveness, and claim integrity

- [x] Add marketing Playwright coverage for desktop and mobile hero geometry,
  CTA visibility, product-image loading, no page overflow, reduced-motion
  fallback, and the Plan / Run / Approve sequence
  (`apps/web/e2e/marketing.spec.ts`, `apps/web/playwright.config.ts`,
  `apps/web/package.json`, `.github/workflows/ci.yml`).
- [x] Run a visible-copy and claim sweep across the homepage. Keep one copy
  register, remove stale internal-first language, and preserve links to the
  measured benchmark and public concepts
  (`apps/web/src/app/(marketing)/page.tsx`,
  `apps/web/src/components/site/*.tsx`).
- [x] Reconcile the "marketing shipped" research claim with the verified
  implementation and record the asset-integrity guard
  (`docs/01-projects/market-ready-polish-research.md`,
  `docs/01-projects/pytxo-improvement-research.md`).

## Acceptance criteria

- The first desktop viewport shows the mission value proposition, Download,
  and a legible current Pytxo Desktop surface. The logo is not the product
  visual.
- At 390x844, the headline, primary CTA, and beginning of the real product
  evidence are visible without horizontal overflow or wrapped CTA labels.
- Flow, Operations, and Approvals captures are current, semantically correct,
  and byte-distinct. Build verification fails when they are missing or
  duplicated.
- The primary loop reads in product language: Plan, Run in isolation, Review
  and approve. Signal / Blast / Race remain available as secondary codenames.
- Motion communicates the product sequence, uses transform/opacity only, and
  collapses to a complete static layout for reduced motion.
- No fake screenshots, invented metrics, fabricated logos, testimonial copy,
  or new dependencies without an explicit package entry.
- Homepage links, auth behavior, billing behavior, public routes, and Chroma
  tokens remain unchanged.

## Test commands

- `npm run capture:marketing` (from `apps/desktop`)
- `npm run verify:product-assets` (from `apps/web`)
- `npm run lint` (from `apps/web`)
- `npm run build` (from `apps/web`)
- `npm run e2e` (from `apps/web`)
- Browser screenshot QA at 1440x900 and 390x844, including reduced motion.
- `git diff --check`

---

# Plan: Plans claim-integrity closeout (pass 6)

## Context

Pass 5 shipped truthful homepage product evidence. A follow-on honesty audit
found the public Plans page still implied hosted sandboxes and managed
metering as generally available, while the default cloud dispatcher and
billing reconciler remain no-ops. Source edits landed; browser QA was
interrupted. This pass closes verification and residual overclaim wording.

## DO NOT TOUCH

- Checkout, Clerk, billing redirects, plan slugs, or entitlement provisioning
- Desktop orchestration, permission profiles, Blast/Race semantics
- Homepage hero / product-story redesign
- Legacy DeckWorkspace chunk split
- Physical DPI hardware matrix

## Phase 1: Lock Plans honesty in tests

- [x] Assert four plan titles, status badges, exact 2x2 desktop grid, and
  single-column mobile stack
  (`apps/web/e2e/marketing.spec.ts`, `apps/web/src/app/(marketing)/plans/page.tsx`)
- [x] Keep caveat visibility, forbidden overclaim phrases, CTA single-line,
  and overflow checks

## Phase 2: Residual claim polish

- [x] Soften Max highlight to configured-only sandbox language
  (`apps/web/src/app/(marketing)/plans/page.tsx`)
- [x] Replace topology framing with Focus / Ops / approvals
  (`apps/web/src/components/site/footer.tsx`,
  `apps/web/src/components/site/cta-section.tsx`)

## Phase 3: Visual QA and docs

- [x] Browser inspect `/plans` at 1440x900 and 390x844
- [x] Record pass 6 and update market-ready research

## Acceptance criteria

- `/plans` e2e proves four cards, status badges, visible caveats, no banned
  overclaims, single-line CTAs, no horizontal overflow
- Marketing copy no longer implies topology/3D as the Desktop primary surface
- Web lint, build, and Playwright pass; `git diff --check` clean for touched
  files

## Test commands

- `npm run lint` / `npm run build` / `npm run e2e` (from `apps/web`)
- Browser QA at 1440x900 and 390x844 on `/plans`
- `git diff --check`

---

# Plan: Demo, real-repo proof, and release closeout (pass 7)

## DO NOT TOUCH

- Paid model accounts or competitor agents
- Checkout, entitlement, Clerk, or hosted Cloud behavior
- Permission-profile policy, Race scheduling semantics, or approve-to-flush
- v0.13.0 published artifacts; this pass prepares a reviewable patch candidate

## Phase 1: Truthful product demo

- [x] Add an editable Remotion composition using real Desktop captures
  (`apps/demo-video/`)
- [x] Add scene-bounded ElevenLabs lines and a key-safe optional generator
- [x] Render and inspect all eight scenes plus the final 1080p MP4

## Phase 2: Real-repository evidence

- [x] Measure Signal across tracked production files, not the tiny fixture
  (`tooling/benchmarks/real-repo-signal.ps1`)
- [x] Measure Race/Blast control-plane behavior in a temporary real-monorepo
  snapshot (`tooling/benchmarks/real-repo-control-plane.ps1`)
- [x] Publish raw JSON with explicit non-claims

## Phase 3: Release defect and ship gates

- [x] Fix recursive sibling copy-layers found by the real workload and add
  regressions (`crates/pytxo-runner/src/{blast,overlay_projfs}.rs`)
- [x] Report effective isolation telemetry
  (`crates/pytxo-orchestrate/src/lib.rs`)
- [x] Run complete Rust, Desktop, web, Remotion, and diff gates (build, type,
  lint, Rust test, native artifact, audit, render, benchmark, and Playwright
  gates pass)

## Acceptance criteria

- Demo source renders a 63-second 1920×1080 MP4 without external credentials.
- Signal proof names the corpus and remains a byte claim.
- Control-plane proof finishes 5/5 agents, separates the declared overlap, and
  leaves the primary checkout unchanged.
- Concurrent copy-layer preparation cannot traverse `.pytxo/worktrees`.
- Public docs, benchmark evidence, and effective isolation telemetry agree.

## Test commands

- `npm ci && npm run typecheck && npm run compositions && npm run render`
  (from `apps/demo-video`)
- `powershell -File tooling/benchmarks/real-repo-signal.ps1`
- `powershell -File tooling/benchmarks/real-repo-control-plane.ps1`
- Project Rust, Desktop, and web commands from `AGENTS.md`
- `git diff --check`

---

# Plan: v1 provider readiness, first mission, and live demo (pass 8)

## Context

The v1 provider boundary and product references are recorded in
`docs/01-projects/v1-provider-auth-onboarding-research.md`. Pytxo coordinates
vendor-owned agent sessions; it does not capture consumer credentials or copy
vendor token stores. Direct inference credentials remain a separate BYOK
surface.

## DO NOT TOUCH

- Vendor password, cookie, OAuth-token, or credential-file capture.
- Permission-profile meanings, execution-domain ownership, Race scheduling, or
  Blast approve-to-flush semantics.
- Checkout, Clerk entitlement provisioning, hosted Cloud behavior, or the
  legacy Desktop shell.
- The disclosed DeepSeek key; it must not enter source, logs, fixtures,
  screenshots, command arguments, or generated artifacts.
- Existing unrelated workspace changes and v0.13.0 published artifacts.

## Phase 1: Make agent readiness honest and useful

- [x] Expand the ADE registry and PATH detection for current Codex, Claude Code,
  Cursor Agent, OpenCode, Gemini CLI, and Aider commands.
- [x] Add safe vendor-owned auth probes and fixed-command login launchers in the
  host control plane; return only redacted readiness metadata to Svelte.
- [x] Replace the Integrations binary grid with actionable installed/session
  states, recheck, official sign-in, documentation, and Use in Flow actions.
- [x] Clear inherited child environments and rebuild an allowlisted runtime
  baseline before injecting the explicitly selected provider route.

## Phase 2: Ship a real first-mission path

- [x] Add a dependency-free, testable first-mission example repository with a
  concrete risk-policy mission.
- [x] Add a native example-workspace generator that creates a local git repo
  outside the source tree without requiring global git identity.
- [x] Expose Try guided example from first-run and the Workspaces catalog, then
  take the user directly to Flow with the example selected.
- [x] Simplify Providers into a featured, status-led BYOK surface with precise
  subscription-login versus API-billing language.

## Phase 3: Record and prove the v1 story

- [x] Upgrade the Remotion composition and ElevenLabs script around real
  Integrations, Flow, Operations, and Approvals footage.
- [x] Write the Screen Studio recording runbook, reset path, shot timings,
  voice lines, and exact example mission.
- [x] Add official product-reference notes and current auth architecture to the
  documentation map.
- [x] Run focused Rust/Svelte/browser tests, native build, Computer Use QA,
  real-repo benchmarks, secret scans, and diff checks.

## Acceptance criteria

- Codex, Claude Code, Cursor Agent, and OpenCode status checks are non-billable,
  redact account identity, and do not read credential files.
- Login actions open official vendor CLI flows from a host-safe directory;
  vendor logout is not performed implicitly.
- An unrelated sentinel secret in the parent environment is absent from both
  PTY and subprocess agent children.
- The example project is created on demand, passes its baseline tests, opens as
  an Orbit workspace, and provides one copyable mission with visible path
  overlap and verification.
- Desktop remains usable at 1280x800 and 960x640; website/demo media use real
  product captures and reduced-motion-safe presentation.
- No secret-like value appears in tracked source, logs, screenshots, benchmark
  results, or demo artifacts.

## Test commands

- `cargo test -p pytxo-core -p pytxo-runner -p pytxo-desktop`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `npm run check && npm run build` (from `apps/desktop`)
- Desktop Playwright focused integration/onboarding coverage, then the full suite
- `npm run typecheck && npm run render` (from `apps/demo-video`)
- Web lint, build, asset verification, and marketing Playwright coverage
- Real-repository Signal and control-plane benchmark scripts
- `git diff --check`
