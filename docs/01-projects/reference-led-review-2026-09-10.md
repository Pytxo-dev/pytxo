---
title: Public reference research and review hierarchy
slug: reference-led-review-2026-09-10
status: active
tags: [project, desktop, ux, research, verification]
audience: [human, agent]
layer: presentation
created: 2026-09-10
updated: 2026-09-10
related: [[astra-desktop-menus-2026-09-09]], [[desktop-visual-system]], [[ADR-0038-epistemic-state-contract]], [[ADR-0039-evidence-ledger-visual-contract]]
---

# Public reference research and review hierarchy

Matt approved a public-source research workflow and a bounded Work → Review → Apply pilot. Preserve the current Chroma Aperture identity, Work/History/Setup navigation and backend authority. The separate Focus/Control proposal is not part of this increment.

## Observed problem

The initial browser inspection placed Prepared changes about 1,046 pixels down a 720-pixel viewport. Review led with two metadata panels, separating the actual changed content from the decision. Work placed Review package below a long receipt. Before editing, deterministic production-preview captures were saved in both themes at 1280×800, 960×800 and 390×844. These are labelled browser fixtures, not native execution.

## Evidence and transfer

| Reference | Observation / documentation | Adaptation and rejected transfer |
| --- | --- | --- |
| [VS Code review](https://code.visualstudio.com/docs/agents/run/review-code-edits) | Visually inspected official screenshot places file selection beside diff content; docs explain review and integration. | Bring prepared changes into the first viewport. Do not import an editor shell, branding, inline agent feedback or worktree-integration semantics. |
| [GitHub Actions](https://docs.github.com/en/actions/how-tos/monitor-workflows/use-the-visualization-graph) | Official screenshot shows job state and dependencies; docs describe run → job → log investigation. | Keep concise state/context near actions, with detailed evidence accessible. Retain Pytxo's wave ledger rather than adding a graph. |
| [Zed Agent Panel](https://zed.dev/docs/ai/agent-panel) | Documented changed-file summary and Review Changes action leading to multi-buffer review; runtime not exercised. | Put the review entry beside the focused run. Do not add hunk acceptance or copy Zed's execution model. |
| [Linear Triage](https://linear.app/docs/triage) | Documented decision queue with explicit review actions and ways to defer/request information; authenticated UI untested. | Place existing decisions and recovery warnings ahead of metadata. Do not import issue statuses, triage automation or Linear's visual identity. |

Sources were checked on 2026-09-10. Screen observations establish visible composition; documented transitions do not establish exercised competitor behavior. The reusable source matrix also covers design systems, GitHub, Figma Community, Product Hunt, official videos and galleries, including access limitations. No percentage of Mobbin's value replaced is claimed.

## Changes and constraints

Work uses the same review-availability predicate as History, with Review package beside Stop. Opening review remains distinct from Apply authority. The commit-boundary panel leads with existing decisions, preparation failures and recovery warnings while preserving all four enforcement surfaces.

Run Review puts prepared changes first and supporting evidence beside them, stacking evidence afterward at narrow widths. A decision summary groups run/workspace identity, package digest, eligibility, combined verification and the primary action. Full base/date, ownership, check commands, exclusions, binary bytes, digests and attempt history remain available. Lengthy enforcement explanations use a disclosure; their statuses and mechanisms remain visible. Mobile paths and ownership occupy their own row rather than competing with Inspect.

No Rust, IPC, persistence, permission or Apply-confirmation contract changes are included. Current UI tokens are reused; this pass does not restore older density/radius values from historical notes. Existing dirty work is the baseline and remains preserved.

## Verification

The hierarchy regressions failed before repair. Final verification: `npm run check` passed with zero Svelte errors/warnings and CSS lint clean; `npm run build` passed; the complete production-preview suite passed 170 tests with one worker; Storybook rebuilt and passed 42 stories across three suites. Independent screenshot/source review found no material in-scope issues after ownership/digest wrapping and enlarged-text collision repairs. Matching screenshots and measurements are in the task's `outputs/pytxo-review-comparison.html` report. At 1280×800 first changed content moved from y≈1182 to y≈543, and Work Review from y≈888 to y≈182; the offscreen-target scroll lower bound becomes zero. No human timing study is claimed. Tests cover the existing state matrix, exact content, delayed responses, cross-workspace selection and guarded confirmation, plus the new viewport/path checks. The skill also receives structural validation, helper tests and an independent non-Pytxo transfer exercise.

Browser results establish presentation only. Previously recorded native results belong to their named builds; this new frontend requires separate native verification when control resumes. No release, deployment or new model run is performed by this increment.
