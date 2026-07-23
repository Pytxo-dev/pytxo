---
title: Pytxo Desktop UI improvement backlog
slug: desktop-ui-improvement-backlog
status: active
tags: [project, desktop, ui, presentation]
audience: [human, agent]
layer: presentation
created: 2026-07-14
updated: 2026-07-23
related: [[desktop-visual-system]], [[presentation-passive-telemetry]], [[product-vision]], [[ADR-0023-reality-deck-3d-renderer]], [[ADR-0028-desktop-product-name]], [[pytxo-improvement-research]], [[market-ready-polish-research]]
---

# Pytxo Desktop UI improvement backlog

## Summary

Pytxo Desktop (`apps/desktop`, Svelte 5 + Tauri v2, app version 0.5.0) is a functional, mostly mature control UI that already matches its own vision docs: structural Focus is the primary surface in Desktop 2, there is no multi-pane terminal wall, and the setup and workspace flows are complete. This note tracks concrete, code-referenced improvements found during a design review, ordered by priority.

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

## Medium priority

### 4. Title bar reimplements its own badge/pill styling

**Resolved in v0.5.0 (by removal).** The `.titlebar__pill` this item asks to consolidate no longer exists — see item 1. Still worth a look if a status-chip pattern reappears in Desktop 2: prefer one shared component over hand-rolled pills.

### 5. The 3D topology stage deserves flagship-level polish

**Still open, scope narrowed.** Desktop 2's Focus surface (`apps/desktop/src/components/desktop2/FocusScreen.svelte`) replaced the bare-canvas concern with a real-data structural graph / run-review view that does not use `TopologyScene3D.svelte` or Three.js at all — the v0.5.0 pass prioritized honest data and layout over 3D flagship polish. Phase 74 styled the structural list further. `TopologyScene3D.svelte` still exists, unpolished, in the legacy shell only. Decide whether the 3D stage remains a Desktop 2 goal or is fully superseded by the structural-graph approach before picking this back up.

### 6. Near-zero intentional motion anywhere in the app

**Resolved in v0.5.0 (for the default shell).** Desktop 2 now has 140–180ms transform/opacity/color transitions on clickable rows, cards, and buttons; hover/focus-visible/active/disabled states throughout `desktop2-shared.css`; and a user-controlled reduced-motion preference (`ui-prefs.svelte.ts` + `app.css`) that disables animation via `prefers-reduced-motion`-equivalent logic. The legacy shell's `LogPanel.svelte` and `DeckWorkspace.svelte` drawer were not touched and remain motion-free, consistent with that shell's rollback-only status.

### 7. No single icon library standard for Desktop

**Resolved in v0.5.0 (for the default shell).** Desktop 2 standardized on Tabler icons throughout (`@tabler/icons-svelte`), replacing ad hoc glyphs where a Tabler equivalent existed. The legacy shell's window-chrome glyphs and any of its remaining ad hoc icons were out of scope for this pass.

## Low priority / housekeeping

### 8. Duplicate ADR-0014 identifier

**Resolved in Phase 73.** Chroma tokens ADR renumbered to [[ADR-0029-chroma-shared-design-tokens]]; [[ADR-0014-multi-provider-byok-catalog]] keeps ADR-0014. Index updated.

### 9. `DeepSpace` naming in the permission-profile picker

Already reconciled in [[glossary]] and [[permission-profile-engine]] to the one-word `DeepSpace` form. Check Desktop's permission-profile picker UI against this the next time it is touched, since it is the one user-visible Desktop surface where the tier name actually renders as a label.

### 12. Prefer `--pytxo-*` / Chroma vars over scattered hex in Desktop 2 CSS

**Still open (partial).** Phase 74 added structural/density styles; a full token pass across `desktop2-shared.css` remains optional polish (A8 in [[market-ready-polish-research]]).

## Non-issues (confirmed fine, no action needed)

- No system tray implementation exists; this matches current scope, not a regression.
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

Back: [[MOC-home]]
