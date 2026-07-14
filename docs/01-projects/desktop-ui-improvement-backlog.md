---
title: Pytxo Desktop UI improvement backlog
slug: desktop-ui-improvement-backlog
status: active
tags: [project, desktop, ui, presentation]
audience: [human, agent]
layer: presentation
created: 2026-07-14
updated: 2026-07-14
related: [[desktop-visual-system]], [[presentation-passive-telemetry]], [[product-vision]], [[ADR-0023-reality-deck-3d-renderer]], [[ADR-0028-desktop-product-name]]
---

# Pytxo Desktop UI improvement backlog

## Summary

Pytxo Desktop (`apps/desktop`, Svelte 5 + Tauri v2, app version 0.5.0) is a functional, mostly mature control UI that already matches its own vision docs: the 3D AST topology is the primary surface in the legacy shell, there is no multi-pane terminal wall, and the setup and workspace flows are complete. This note tracks concrete, code-referenced improvements found during a design review, ordered by priority. Items closed by the v0.5.0 Desktop 2 polish pass are marked **Resolved in v0.5.0** with their current evidence; everything else is still an open backlog item, not a changelog.

**v0.5.0 context:** the compact "Desktop 2" shell (`apps/desktop/src/components/desktop2/`) is now the default UI (`useLegacyShell` in `App.svelte` defaults to `false`); the original shell (`DeckShell.svelte` and friends, referenced throughout this note) is retained only as an opt-in rollback path. Several items below were superseded by the Desktop 2 rewrite rather than fixed in place in the legacy components.

## High priority

### 1. Title bar colors bypass the Chroma token system

**Resolved in v0.5.0.** `TitleBar.svelte` was rewritten for the v0.5.0 pass: it now renders only the brand mark and window controls, and the `.titlebar__pill` warn/live-status treatment that hardcoded `#fbbf24` / `#34d399` is gone entirely. The one remaining raw-color spot in the file (close-button hover) now uses `var(--destructive)`. No further action needed unless the pill concept is reintroduced.

### 2. Loaded entitlement data has no UI surface

**Resolved in v0.5.0 (for the default shell).** In Desktop 2, `App.svelte` now passes `tier`, `signedIn`, `cliMissing`, and `subscriptionPortalUrl` into `DesktopShell.svelte`, which surfaces real account/tier state (and a route to Account settings) via `Sidebar.svelte`'s account footer instead of a hardcoded identity. Separately, the original claim that `DeckToolbar.svelte` "has no call sites anywhere in the codebase" is out of date — it is wired into `DeckWorkspace.svelte` with a `tier` prop. The legacy shell's wallet/permission-ceiling/cloud-run-badge surfacing is still unaddressed, but that shell is now an opt-in rollback path, not the default experience.

### 3. Docs claim a 2D topology fallback that no longer exists in code

[[desktop-visual-system]] and [[ADR-0023-reality-deck-3d-renderer]] both describe a shipped 2D `TopologyPanel.svelte` sidebar fallback ("2D canvas remains sidebar fallback"). It is not present in `apps/desktop/src` — only `TopologyScene3D.svelte` exists. Decide one of:

- Reintroduce a lightweight 2D fallback (useful for large graphs or low-power / no-WebGL environments), or
- Correct [[desktop-visual-system]]'s status table and ADR-0023 to reflect 3D-only reality.

Either is fine; leaving the mismatch between docs and code is not.

## Medium priority

### 4. Title bar reimplements its own badge/pill styling

**Resolved in v0.5.0 (by removal).** The `.titlebar__pill` this item asks to consolidate no longer exists — see item 1. Still worth a look if a status-chip pattern reappears in Desktop 2: prefer one shared component over hand-rolled pills.

### 5. The 3D topology stage deserves flagship-level polish

**Still open, scope narrowed.** Desktop 2's Focus surface (`apps/desktop/src/components/desktop2/FocusScreen.svelte`) replaced the bare-canvas concern with a real-data structural graph / run-review view that does not use `TopologyScene3D.svelte` or Three.js at all — the v0.5.0 pass prioritized honest data and layout over 3D flagship polish. `TopologyScene3D.svelte` still exists, unpolished, in the legacy shell only. Decide whether the 3D stage remains a Desktop 2 goal or is fully superseded by the structural-graph approach before picking this back up.

### 6. Near-zero intentional motion anywhere in the app

**Resolved in v0.5.0 (for the default shell).** Desktop 2 now has 140–180ms transform/opacity/color transitions on clickable rows, cards, and buttons; hover/focus-visible/active/disabled states throughout `desktop2-shared.css`; and a user-controlled reduced-motion preference (`ui-prefs.svelte.ts` + `app.css`) that disables animation via `prefers-reduced-motion`-equivalent logic. The legacy shell's `LogPanel.svelte` and `DeckWorkspace.svelte` drawer were not touched and remain motion-free, consistent with that shell's rollback-only status.

### 7. No single icon library standard for Desktop

**Resolved in v0.5.0 (for the default shell).** Desktop 2 standardized on Tabler icons throughout (`@tabler/icons-svelte`), replacing ad hoc glyphs where a Tabler equivalent existed. The legacy shell's window-chrome glyphs and any of its remaining ad hoc icons were out of scope for this pass.

## Low priority / housekeeping

### 8. Duplicate ADR-0014 identifier

Two accepted ADRs both claim `ADR-0014`: `ADR-0014-multi-provider-byok-catalog` and `ADR-0014-chroma-shared-design-tokens`. Renumber the Chroma one to the next free ID and update `docs/05-adr/index.md`, which currently lists only the BYOK version.

### 9. `DeepSpace` naming in the permission-profile picker

Already reconciled in [[glossary]] and [[permission-profile-engine]] to the one-word `DeepSpace` form. Check Desktop's permission-profile picker UI against this the next time it is touched, since it is the one user-visible Desktop surface where the tier name actually renders as a label.

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

Back: [[MOC-home]]
