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

Pytxo Desktop (`apps/desktop`, Svelte 5 + Tauri v2, app version 0.3.5) is a functional, mostly mature control UI that already matches its own vision docs: the 3D AST topology is the primary surface, there is no multi-pane terminal wall, and the setup and workspace flows are complete. This note tracks concrete, code-referenced improvements found during a design review, ordered by priority. Nothing here is implemented yet — treat this as a project backlog, not a changelog.

## High priority

### 1. Title bar colors bypass the Chroma token system

`apps/desktop/src/components/shell/TitleBar.svelte` hardcodes four raw hex colors (`#fbbf24` warn pill, `#34d399` live-status dot, `#c42b1c` close-button hover) instead of the shared `packages/chroma` tokens (`--brand-gold`, `--brand-cyan`, `--destructive`) that every other panel consumes. Effect: these spots do not repaint when the user switches between the `void` / `light` / `terminal` / `nebula` deck themes ([[desktop-visual-system]]), while the rest of the shell does. Fix: replace with `var(--brand-gold)`, `var(--brand-cyan)`, and `var(--destructive)`.

### 2. Loaded entitlement data has no UI surface

`App.svelte` fetches `walletMicrocredits`, `permissionCeiling`, `subscriptionPortalUrl`, `cloudRunBadge`, and `maxAgents`, but only `tier` renders, as a plain text pill in `TitleBar.svelte`. `apps/desktop/src/components/shell/DeckToolbar.svelte` exists specifically to display this data and has no call sites anywhere in the codebase. Today a user cannot see their wallet balance, permission ceiling, or cloud run status anywhere in the app. Fix: wire `DeckToolbar` into `DeckShell.svelte` or `TitleBar.svelte`, or delete it and design a replacement — either way, stop silently dropping this data.

### 3. Docs claim a 2D topology fallback that no longer exists in code

[[desktop-visual-system]] and [[ADR-0023-reality-deck-3d-renderer]] both describe a shipped 2D `TopologyPanel.svelte` sidebar fallback ("2D canvas remains sidebar fallback"). It is not present in `apps/desktop/src` — only `TopologyScene3D.svelte` exists. Decide one of:

- Reintroduce a lightweight 2D fallback (useful for large graphs or low-power / no-WebGL environments), or
- Correct [[desktop-visual-system]]'s status table and ADR-0023 to reflect 3D-only reality.

Either is fine; leaving the mismatch between docs and code is not.

## Medium priority

### 4. Title bar reimplements its own badge/pill styling

`TitleBar.svelte`'s `.titlebar__pill` is a hand-rolled pill component, while `ActivitySidebar.svelte` and other panels use the shared shadcn-svelte `Badge` (`$lib/components/ui/badge`). Consolidate on one status-chip component so tier, demo-mode, CLI-missing, and future entitlement badges look and behave consistently.

### 5. The 3D topology stage deserves flagship-level polish

The topology view (`TopologyScene3D.svelte`) is the product's actual differentiator (per [[product-vision]], it is the reason Desktop exists instead of a terminal wall), but currently reads as a bare Three.js canvas. Concrete upgrades:

- A cohesive on-canvas legend/HUD for node and edge color meaning (file / edited / symbol), instead of relying on users to infer it.
- Subtle depth cues (fog or vignette) so the graph reads as a coherent space rather than floating primitives.
- Confirm the Fit/Reset camera controls read as discoverable controls, not plain unstyled text buttons.

### 6. Near-zero intentional motion anywhere in the app

No transitions were found beyond instant state swaps (panel show/hide, tab switches). Add restrained, motivated motion only where it reinforces the product story:

- Smooth expand/collapse for the telemetry log panel (`LogPanel.svelte`) and the workspace-settings drawer in `DeckWorkspace.svelte`.
- A brief highlight pulse on a topology node when an agent just edited it — this directly reinforces the "live blast radius" narrative from [[product-vision]] instead of being decoration.
- Respect `prefers-reduced-motion` throughout.

### 7. No single icon library standard for Desktop

Native window-chrome glyphs for minimize / maximize / close (`—`, `□`, `❐`, `×` in `TitleBar.svelte`) are fine as an OS convention. Audit the rest of the panels (`ActivitySidebar`, `InspectorPanel`, `ProjectPathPanel`) for ad hoc icon usage and standardize on one library, consistent within Desktop (it does not need to match the website's `lucide-react` choice, but it must be singular within this app).

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
