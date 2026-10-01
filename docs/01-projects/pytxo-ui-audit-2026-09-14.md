---
title: Desktop readability and scroll audit
status: active
tags: [desktop, ux, audit]
created: 2026-09-14
---

# Desktop readability and scroll audit

Method: independent assessments by `/root/design_audit` and `/root/technical_audit`, synthesized before native acceptance. Design assessment used the frozen before-source and a labeled historical browser screenshot. Technical assessment used source and one detector pass. Neither assessment is a usability study or accessibility certification.

## Starting assessment

Nielsen heuristic scores, each out of four: system status 3; real-world match 3; user control 3; consistency 2; error prevention 3; recognition 3; efficiency 3; minimalist design 3; recovery 3; help 2. Total **28/40**, provisional.

Technical scores: accessibility 3; performance 3; responsive 3; theming 2; integrity 3. Total **14/20**, provisional. No measured performance or screen-reader certification is implied.

Pytxo has a recognizable, useful structure: Work, History, Setup; explicit Review and Apply; contextual evidence; keyboard alternatives. The weak points are inconsistent scale and the difficulty of locating recovery actions below long content. Expert users face changing target sizes; low-vision users face small explanations; first-time users can see failure before discovering its recovery action.

## Findings and changes

| Priority | Finding | Disposition |
|---|---|---|
| P1 | Provider variable normal/hover colors yield 3.64:1 / 1.32:1 against Light's white panel | Replaced hardcoded colors with theme text-soft/strong tokens; 12px text |
| P2 | Setup sticky rail derives height from window while an ancestor also scrolls | Available-height layout with sibling rail/body scrolling; section switch resets the body |
| P2 | Main History filters are 30px and inspection tools 28px | 40px main controls; 13px labels |
| P2 | History request/metadata are 12/11px; disabled run reasons 10px | 14/12px History text; 12px reasons with clearer contrast |
| P2 | Setup body capped at 760px despite wide available area | Inner content uses a 960px reading-width cap; the scroll viewport spans the full pane so fullscreen chrome stays at the window edge |
| P2 | Recovery actions can follow a long ledger | Open; retain as follow-up requiring failure-state interaction validation |
| P2 | Long History can expose multiple possible scroll owners | Source risk only; no new wheel failure demonstrated, left unchanged |

Five detector `side-tab` warnings were semantic state notices, not decorative violations. No blanket suppression or notice removal. Raw evidence: `target/ui-audit-2026-09-14/detector-b.json`; independent technical report: `assessment-b.md` there. No browser detector overlay was injected; production-preview screenshots and existing journey checks supplied rendered evidence. No critique-only server was started.

## Validation and lineage

HEAD remains `72879702f90f2b74eece888bb117df59608f9b56`; source is dirty and uncommitted. Frozen copies of the five initially touched source files are under `target/ui-audit-2026-09-14/before/`. The regression in `apps/desktop/e2e/setup-scroll-ownership.spec.ts` exercises wheel scrolling, stationary navigation, bottom permission controls, section reset, overflow, and edge-aligned scroll geometry at 1920x1020, 1280x800, and 860x560. Browser fixtures represent test data, not real agent execution.

62 selected production-preview checks passed (shell, motion/scroll, presentation, Setup scroll ownership). After the final provider-color change, the two Setup journey checks passed again; these are overlapping checks, not 64 unique tests. Svelte and style checks passed before that final two-color edit. Screenshots of Setup, History and Work at both sizes were captured; Setup and History were visually inspected. Exact source manifest, candidate hash, final check and native status are recorded in the checkpoint and local source receipt.

Questions skipped: the user already authorized audit, critique and polish and specified scrolling/readability priorities. No new aesthetic or scope decision was needed. Chroma Aperture, existing drafts, isolation, evidence scoping and backend Apply authority remain unchanged. No new dependency, commit or publication.

## Harness and identity follow-up

The user-supplied native screenshot exposed a visually heavy default-looking Setup thumb and white identity tiles. The final source keeps one Setup scroll owner, narrows its native scrollbar to 10px with a transparent track and a 4px alpha-mixed thumb, moves content nearer the gutter, and removes the hardcoded white identity tile. Cursor and Gemini retain their alpha-channel favicons. OpenCode now uses its official transparent square SVG pinned to upstream commit `df23b7f9488a38e6f8064a0739d4f8cde86d7cfb`; the opaque favicon is no longer rendered.

The registry now recognizes 14 harnesses. Five new harnesses are dispatch-ready from reviewed vendor commands: Grok Build, Factory Droid, Cline CLI, Goose, and Kimi Code CLI. Qwen Code is detection-only: it appears truthfully when found, but Desktop Flow, reviewed dispatch, CLI defaults, and the interactive shell do not make it runnable until Qwen approval modes map to Pytxo permission profiles. Amp remains deferred because its documented Windows route is WSL-only. Grok Bot is not represented as a local harness because xAI documents it as a separate cloud teammate; Grok Build is the actual local CLI.

This is registry, command-construction, native detection, and UI evidence—not proof that every added executable is installed, authenticated, or completes a real Pytxo mission. The exact vendor review is in [[pytxo-harness-catalog-research-2026-09-14]].

Final combined production-preview acceptance passed **85/85** with zero skipped, unexpected, or flaky results before the fullscreen correction. After the user reproduced the remaining fullscreen geometry defect, the focused Setup matrix passed **3/3** at 1920x1020, 1280x800, and 860x560. Svelte/style checks and the frontend production build passed. The corrected release/custom-protocol candidate is `target/ui-audit-2026-09-14/pytxo-desktop-fullscreen-scroll-final.exe`, SHA-256 `71CA3697FBCBFAC18E742E361F3842CCE1661426C3D1DBB8058950373498CF79`; source-manifest digest `44ade2b5c8e655aeb1fccca4fb44bd46a0ff3b8b21b1f2c6b45dcb168523620e`. That exact candidate was launched and maximized to a 1536x816 logical native capture. The scrollbar was visually flush with the far-right window edge, and a pointer-wheel action scrolled only the Setup content while both navigation rails remained stationary.

The website's pre-existing registry table and FAQ were updated to the same 14-entry model rather than leaving the public claim at eight tools. Its production build generated 62 static pages, affected ESLint passed, and the final marketing matrix passed **15/15** after an isolated navigation-timeout diagnosis.
