---
title: Pytxo Desktop UI improvement backlog
slug: desktop-ui-improvement-backlog
status: active
tags: [project, desktop, ui, presentation]
audience: [human, agent]
layer: presentation
created: 2026-07-14
updated: 2026-07-29
related: [[desktop-visual-system]], [[presentation-passive-telemetry]], [[product-vision]], [[ADR-0023-reality-deck-3d-renderer]], [[ADR-0028-desktop-product-name]], [[pytxo-improvement-research]], [[market-ready-polish-research]], [[desktop-dangerous-ux-2026-07]]
---

# Pytxo Desktop UI improvement backlog

## Summary

Pytxo Desktop (`apps/desktop`, Svelte 5 + Tauri v2) is a functional, mostly mature control UI that already matches its own vision docs: structural Focus is the primary surface in Desktop 2, there is no multi-pane terminal wall, and the setup and workspace flows are complete. This note tracks concrete, code-referenced improvements found during a design review, ordered by priority.

**v0.11.0 (2026-07-26):** install-trust + dangerous UX polish — CLI install without PowerShell spawn storm, NSIS logo branding (`com.pytxo.desktop`), SetupWizard anti-slop, Ops fingerprint/cost freshness + DPI zoom re-apply, eyebrow/chroma-edge cutback, tray “needs you” tooltip. Research: [[desktop-dangerous-ux-2026-07]].

**Readiness pass 1 (2026-07-28):** restored Desktop 2 verification truth, removed false updater noise from browser previews, made preview approval resolution stateful, repaired planned-Flow controls at split-pane widths, and cleared all Svelte diagnostics. Production build and 37 Playwright scenarios pass across 960×640, 1024×576, 1280×720, 1280×800, and 1600×900.

**Readiness pass 2 (2026-07-28):** made the decision inbox match the real Galaxy/Blast action model, added guarded keyboard operations, exposed exact request metadata and honest latest-run evidence, and repaired the compact Approvals composition. Production build and 42 Playwright scenarios pass across the same five widths.

**Readiness pass 3 (2026-07-28):** completed keyboard-first Ops with exact-run process control. Ctrl/Cmd+Shift+O focuses active work; the stop chord opens review instead of terminating directly; confirmation names the run, workspace, and execution domain and preserves the unflushed sandbox. The orchestration boundary refuses stale run IDs and persists a cancelled run status after a successful stop. A 2026-07-29 accumulated review then hardened that terminal transition: ordinary worker completion now updates only a still-running row, so a killed worker cannot overwrite `cancelled` while it unwinds. The production Tauri and web builds, Svelte diagnostics, full Rust workspace tests and clippy, and all 43 Playwright scenarios pass. Production inspection covers Void and Light at 1280x800 plus compact Ops at 960x640.

**Readiness pass 4 (2026-07-29):** the simulated 860x560 matrix passed for device scales 100/125/150% and user zoom 90/100/110%, with 53 full Desktop Playwright scenarios passing. No CSS, breakpoint, or native minimum-window change was needed at the existing floor. Physical Windows 125% inspection passed, including the native 860x560 logical floor. The inspected artifact was the ordered `npm run build:native` release executable with Tauri `custom-protocol`; plain debug or release Cargo builds without that feature can load `devUrl` rather than the embedded current bundle. Physical Windows 100%/150% and macOS Retina remain open.

**v0.5.0 context:** the compact "Desktop 2" shell (`apps/desktop/src/components/desktop2/`) is the default UI (`useLegacyShell` in `App.svelte` defaults to `false`); the original shell is retained only as an opt-in rollback path.

**Phase 74 (2026-07-23):** market-ready supervision polish shipped — live snapshot poll, Approvals selection/refresh, active domain/recents, Ops→Run Review, thin Fleet panel, Integrations/Settings honesty, Focus structural list CSS. See [[market-ready-polish-research]].

## High priority

### 1. Title bar colors bypass the Chroma token system

**Resolved in v0.5.0.** `TitleBar.svelte` was rewritten for the v0.5.0 pass: it now renders only the brand mark and window controls, and the `.titlebar__pill` warn/live-status treatment that hardcoded `#fbbf24` / `#34d399` is gone entirely. The one remaining raw-color spot in the file (close-button hover) now uses `var(--destructive)`. No further action needed unless the pill concept is reintroduced.

### 2. Loaded entitlement data has no UI surface

**Resolved in v0.5.0 (for the default shell).** In Desktop 2, `App.svelte` now passes `tier`, `signedIn`, `cliMissing`, and `subscriptionPortalUrl` into `DesktopShell.svelte`, which surfaces real account/tier state (and a route to Account settings) via `Sidebar.svelte`'s account footer instead of a hardcoded identity. Separately, the original claim that `DeckToolbar.svelte` "has no call sites anywhere in the codebase" is out of date — it is wired into `DeckWorkspace.svelte` with a `tier` prop. The legacy shell's wallet/permission-ceiling/cloud-run-badge surfacing is still unaddressed, but that shell is now an opt-in rollback path, not the default experience.

### 3. Docs claim a 2D topology fallback that no longer exists in code

**Resolved in Phase 73 (docs corrected).** [[desktop-visual-system]] and [[ADR-0023-reality-deck-3d-renderer]] honesty addendum now state Desktop 2 structural Focus is default, 3D is legacy-shell only, and `TopologyPanel.svelte` is not shipped. No reintroduction planned unless a future ADR requires it.

### 10. Live Ops and Approvals supervision gaps

**Resolved in Phase 74.** Desktop 2 mounts a ~1s `loadSnapshot()` poll; Ops Live badge reflects refresh/error; Approvals rows are selectable with detail pane follow; approve/deny reloads snapshot (not local-only resolved list); Ops run rows open Focus Run Review; active domain from workspace/recents drives AppBar + Flow `domain_id`.

### 11. Integrations / Settings stubs and unused fleets

**Resolved in Phase 74.** MCP Integrations show CLI/`pytxo-mcp` + Cursor setup truth; Cloud stays honest local-only; Settings Coming soon entries removed (Appearance / Voice / Privacy / Account remain); thin Fleet panel on Ops; Focus `structural-list` styled; density preference wired in CSS.

### 13. Planned Flow controls clip when the review pane opens

**Resolved in readiness pass 1 (2026-07-28).** The workspace, agent CLI, and Build plan controls now wrap within the composer instead of creating an unreachable horizontal strip. A Playwright regression builds a plan and verifies both page and action-row overflow at every supported test width. Direct production-bundle inspection confirmed zero horizontal overflow at 1280×800 and 960×640.

### 14. Browser previews report false native failures and do not resolve approvals

**Resolved in readiness pass 1 (2026-07-28).** `UpdateBanner.svelte` now checks for the Tauri runtime before calling updater IPC, so a browser/Storybook preview cannot claim an updater failure caused only by missing native APIs. The preview backend now removes approved or denied requests from its instance snapshot, keeping the success message, inbox count, and empty state consistent.

### 16. Every HITL request is mislabeled as a Blast Shield flush

**Resolved in readiness pass 2 (2026-07-28).** The runner emits distinct action codes for sandbox flushes, filesystem operations, Git mutations, network access, process commands, and MCP calls. Desktop now maps those codes to accurate titles, categories, consequences, and decision labels while preserving the raw action code in the evidence pane. Unknown future codes fall back to neutral approval language instead of silently inheriting “Approve & flush.”

### 18. Desktop stop control cannot prove which run it will terminate

**Resolved in readiness pass 3 (2026-07-28; terminal-state hardening 2026-07-29).** The previous native stop IPC resolved whichever execution domain happened to be selected and accepted no expected run ID, so exposing it as a global shortcut would have made stale supervision state dangerous. Desktop now passes the explicit domain and run. `pytxo-orchestrate::stop_exact` refuses the operation if that run is no longer active in the domain, leaves the newer active marker intact, and records `cancelled` after a matching stop. Worker finalization is an atomic transition from `running`, which preserves that cancellation after the killed process returns. Ops exposes a labeled Stop control and Ctrl/Cmd+Shift+Backspace only while an active-run row is focused; both open the same confirmation. No shortcut terminates directly, and the dialog states that the sandbox is neither flushed nor deleted.

## Medium priority

### 4. Title bar reimplements its own badge/pill styling

**Resolved in v0.5.0 (by removal).** The `.titlebar__pill` this item asks to consolidate no longer exists — see item 1. Still worth a look if a status-chip pattern reappears in Desktop 2: prefer one shared component over hand-rolled pills.

### 5. The 3D topology stage deserves flagship-level polish

**Still open, scope narrowed.** Desktop 2's Focus surface (`apps/desktop/src/components/desktop2/FocusScreen.svelte`) replaced the bare-canvas concern with a real-data structural graph / run-review view that does not use `TopologyScene3D.svelte` or Three.js at all — the v0.5.0 pass prioritized honest data and layout over 3D flagship polish. Phase 74 styled the structural list further. `TopologyScene3D.svelte` still exists, unpolished, in the legacy shell only. Decide whether the 3D stage remains a Desktop 2 goal or is fully superseded by the structural-graph approach before picking this back up.

### 6. Near-zero intentional motion anywhere in the app

**Resolved in v0.5.0 (for the default shell).** Desktop 2 now has 140–180ms transform/opacity/color transitions on clickable rows, cards, and buttons; hover/focus-visible/active/disabled states throughout `desktop2-shared.css`; and a user-controlled reduced-motion preference (`ui-prefs.svelte.ts` + `app.css`) that disables animation via `prefers-reduced-motion`-equivalent logic. The legacy shell's `LogPanel.svelte` and `DeckWorkspace.svelte` drawer were not touched and remain motion-free, consistent with that shell's rollback-only status.

### 7. No single icon library standard for Desktop

**Resolved in v0.5.0 (for the default shell).** Desktop 2 standardized on Tabler icons throughout (`@tabler/icons-svelte`), replacing ad hoc glyphs where a Tabler equivalent existed. The legacy shell's window-chrome glyphs and any of its remaining ad hoc icons were out of scope for this pass.

### 17. Decision inbox lacks keyboard operations and request evidence

**Resolved for Approvals in readiness pass 2 (2026-07-28).** J/K changes selection; Ctrl/Cmd+Enter approves; Ctrl/Cmd+Backspace denies. Destructive decisions require modifier chords and ignore editable targets or repeated key events. The selected request now shows requester, workspace, timestamp, exact action code, and the latest matching workspace run with an explicit warning that HITL records do not currently carry a run ID. At 960px the inbox and detail switch to one column instead of clipping internally.

## Low priority / housekeeping

### 8. Duplicate ADR-0014 identifier

**Resolved in Phase 73.** Chroma tokens ADR renumbered to [[ADR-0029-chroma-shared-design-tokens]]; [[ADR-0014-multi-provider-byok-catalog]] keeps ADR-0014. Index updated.

### 9. `DeepSpace` naming in the permission-profile picker

Already reconciled in [[glossary]] and [[permission-profile-engine]] to the one-word `DeepSpace` form. Check Desktop's permission-profile picker UI against this the next time it is touched, since it is the one user-visible Desktop surface where the tier name actually renders as a label.

### 12. Prefer `--pytxo-*` / Chroma vars over scattered hex in Desktop 2 CSS

**Resolved in v0.11.0 (partial → default chrome).** AppBar live-dot and chroma-edge decoration now use accent tokens; rainbow `--pytxo-edge` gradients softened to single-accent. Residual hex in dense list CSS remains optional cleanup.

### 15. Legacy rollback shell exceeds the production chunk advisory

**Open, low priority.** The 2026-07-28 production build reports `DeckWorkspace` at 555.45 kB minified (142.63 kB gzip), above Vite's 500 kB advisory. Desktop 2 remains clean and functional, so code-splitting the rollback-only shell should be weighed against retiring it after its compatibility window rather than mixed into product-screen polish.

## Non-issues (confirmed fine, no action needed)

- No system tray implementation exists; close-to-tray ships in Desktop 0.8.0 (`tray-icon` + hide on close).
- `WorkspaceHome.svelte` already has a considered empty state ("No workspaces yet. Open a folder to start a run.").
- The setup wizard flow (`SetupWizard.svelte` and its steps) is complete and has end-to-end coverage (`e2e/shell.spec.ts`).

## Related

- [[desktop-visual-system]]
- [[presentation-passive-telemetry]]
- [[product-vision]]
- [[ADR-0023-reality-deck-3d-renderer]]
- [[ADR-0028-desktop-product-name]]
- [[pytxo-improvement-research]]
- [[market-ready-polish-research]] — Phase 74 Desktop + marketing polish
- [[desktop-dangerous-ux-2026-07]] — 0.11.0 ops-console / anti-slop research

Back: [[MOC-home]]
