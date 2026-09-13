---
title: Desktop polish and appearance persistence
slug: astra-desktop-polish-2026-09-08
status: active
tags: [project, desktop, ux, verification]
audience: [human, agent]
layer: presentation
created: 2026-09-08
updated: 2026-09-08
related: [[astra-execution-2026-09-07]], [[astra-evidence-2026-09-07]]
---

# Desktop polish and appearance persistence

Matt requested Cursor/Codex-level polish, custom scrollbars, and a cleaner layout.
The implementation improves the existing Work, History and Setup structure. The
separate Focus/Control proposal still needs major-redesign approval. Neither a
successful browser test nor a visual reference establishes equivalent usability.

## Implemented decisions

| Before | After |
| --- | --- |
| Shell hover/active surfaces, radii and timings varied. | `app.css` supplies neutral surface states, 6px controls, 10px panels, 40px targets and 140ms interruptible transitions. Shared panel styling follows the token. |
| Scrollbar styling mixed standard and WebKit rules. | `deck-scroll.css` styles native tracks and rounded thumbs throughout Desktop, with native wheel/drag behavior and forced-colors fallback. Standard non-auto values no longer override the WebKit styling. |
| Content height and sidebar resizing caused reachability/motion problems. | `DesktopShell` contains vertical scrolling, reserves its gutter and removes the grid-width animation. Real loading placeholders follow the workspace layout and honor reduced motion. |
| Search, navigation, workspace menu and footer competed for space. | `Sidebar` uses one-line search, stable shortcut sizing, 40px navigation, a bottom footer and quieter branding. `AppBar` uses a 52px row, larger controls and bounded wrapping/scrolling for long workspace menus. |
| Run identity, progress and status squeezed onto one line. | `WorkActive` separates identity/state/Stop from wave and isolation details. Long IDs shorten visually while retaining their full title. Stacked `StateChip` details wrap. |
| Empty Work repeated unknown receipts and implied approval protection for every profile. | A single introduction and New run action explain scope, permissions and evidence. `BoundaryPanel` omits redundant no-run rows; actual unknown/advisory receipts remain visible whenever a run exists. |
| Task and history columns competed at narrow widths. | `RunLedger` and `HistoryScreen` retain every column, wrap state labels, use 44px rows and narrower minimums. Task-local agent names retain complete IDs in accessible labels and titles. History filters expose selection. |
| Review controls and surfaces depended on dark-only values and conflicting shared styles. | `RunReviewScreen` uses semantic surfaces/text, 12px evidence text, working responsive panels and 40px controls. The Back and Discard variants override shared rules explicitly. Exact package identity, eligibility and confirmation focus behavior remain intact. |
| Destructive confirmation used insufficient white-on-red contrast. | Refuted text and an outlined tinted surface meet the measured 4.5:1 text-contrast threshold in both themes. Apply retains its existing coral action cue. |
| Appearance controls had tiny hit areas and legacy CSS overrides. | `SettingsScreen` uses readable theme previews, 40px segments, 44×40 switch targets, neutral selected states, named custom-color input and visible keyboard focus. Scale/density expose pressed state; existing persisted accent keys remain compatible. |
| Default accent and keyboard badges inherited dark-only styling. | Neutral focus follows the active theme's strong text token. Keyboard badges use semantic surfaces, text and borders. Custom accent does not change evidence colors. |
| Startup persisted Void before reading the saved theme; another write could undo Match system. | `App` resolves appearance before child initialization. Only the legacy shell owns its reactive theme writer. Manual selection and opposite-OS startup now survive reload. |

## Verification and limits

The first focused run passed 26/28: one actual startup bug and one test-environment
problem. Headless Chromium suppressed scrollbar hit targets with its default
`--hide-scrollbars` argument; disabling that argument reproduced successful mouse
dragging without changing app scrolling. The corrected production suite passed
141/141; the separate development legacy-shell test passed 1/1. Svelte reported
zero errors/warnings and CSS lint passed. The final keyboard/switch styling passed
7/7 focused tests against the exact MSI frontend. All 41 rebuilt stories passed;
loading, empty, error and offline captures are retained. `CHECKPOINT.md` and
`target/astra-polish-20260908/` contain exact logs and screenshots.

An independent source reviewer checked the theme lifecycle, authority copy,
destructive contrast and cascade repairs. Browser fixtures exercise presentation
and interaction; they do not establish native agent execution.

The earlier c77 executable, 475 MSI, native mission and 58-second MP4 are preserved
checkpoint evidence. They do not attest these newer source bytes. Host-profile
paths are now visibly masked in the separately reviewed checkpoint film; its
provenance is `tooling/benchmarks/results/astra-aperture-demo-2026-09-09.json`.
The new MSI (`e7752cf4…`) built successfully and passed packaged dependency checks;
all 356 frozen input files still match. Its extracted executable is `7abe2097…`.
New native proof, exact-source CI, clean-Windows installation and public-download
verification remain separate gates. No Actions run or publication was triggered.
