---
title: Pytxo presentation pass acceptance
slug: pytxo-presentation-acceptance-2026-09-14
status: active
tags: [project, desktop, website, demo, acceptance]
audience: [human, agent]
layer: presentation
created: 2026-09-14
updated: 2026-09-14
related: [[pytxo-interface-engineering]], [[pytxo-presentation-references-2026-09-14]], [[modular-project-safety-contract-2026-09-13]]
---

# Presentation acceptance — native handoff pending

Evidence directory: `target/presentation-pass-2026-09-14/` (ignored, local).
Source remains HEAD `72879702f90f2b74eece888bb117df59608f9b56` plus preserved
inherited work and this scoped diff. `starting-state.json`, `inherited.patch`,
and `before/` preserve the incoming state. `source-receipt.json` identifies
the current scope and native inputs; do not infer a clean worktree.

Native candidate: `pytxo-desktop.exe`, 37,583,360 bytes, SHA-256
`604330B13B45B013439007FA0C81BA563ADC8D8F5CBD2658AF23592F68AC8984`.
The release build passed; `native-build.log` records command/profile configuration.
Native-source manifest digest:
`12928ed070924e13c02ea521b46c2914f6a5b18a70db886b93bf34cf2555274c`.
The configured isolated main WebView profile opened into onboarding in the
September 14 continuation; the exact candidate process and empty Work surface
were natively observed. Full isolation and mission acceptance remain unproven.

## Acceptance matrix

| Surface / gate | Status | Exact evidence / boundary |
|---|---|---|
| Desktop UI/interaction readiness | BLOCKED | `desktop-final-tests.json`: 234 passed, zero skipped/flaky/failed; Svelte/CSS checks passed. Final native acceptance pending |
| Native pointer docking | BLOCKED | `sparse-drag-before-error.md` reproduces a sparse-event browser failure; new regression and existing pointer/keyboard matrix pass. Native causality remains unproved |
| Real non-empty Verify → Review → Apply | BLOCKED | Demo setup, two baseline tests, two-stage CLI dry-run, and fixture browser preflight passed. No actual agents or Apply launched |
| Website local preview | PASS | `website-build.log`, affected ESLint, and `website-final-tests.json`: 19 passed after the mobile frame correction. Final native imagery remains a launch-content gap |
| Demo capture readiness | BLOCKED | Genuine native take retained; ten-second sample decoded/inspected. Recorder start/stop needs manual help; normal-speed playback and full mission capture pending |
| Edited-video readiness | BLOCKED | 4K Remotion capture-test MP4 exists, not the final demo. `media/out/capture-proof-4k.mp4`, hash `B586CD5F4F051DFA7F030EA8E169549545337976A3D4C2F73FDDE23B8FA1C097`; editable source in `media/edit/` |
| Public release readiness | NOT TESTED | No installer, signing, clean-install, broader platform/security/release gate acceptance. No publication authorized |

The earlier mismatched native observation was superseded by the user's explicit
**desktop ready** handoff. The exact new candidate was launched; onboarding,
ready existing integrations, disposable-project selection and empty Work were
observed at 1282×802. No real agents or Apply ran. Recordly's Pytxo-only capture
target is selected, but starting Record was rejected by native hit-testing twice
after reactivation. Input is paused pending the user clicking Record and replying
**recording**. Smaller native viewport, second Windows DPI, docking and non-empty
Apply remain gaps. Active display reports 1920×1080; requested delivery is 4K,
not a claim of native 4K footage. The isolated Remotion proof project typechecks;
its render initially refused missing capture. After manual start/stop, an actual
3840×2160/30fps ten-second MP4 rendered from genuine Work → New run footage.
Full decode and contact-sheet inspection passed; normal-speed playback remains
pending. Source was 1920×1020 with a 1602×1002 app crop; 4K is upscaled delivery.
No mission, verification, Review or Apply footage has been recorded yet.

## Reference → implementation → evidence

| Reference / pattern | Actual implementation and adaptation | Evidence / dependency |
|---|---|---|
| Motion DOM mini | Desktop `evidence-motion.ts`, used by `MissionDock.svelte`: 200ms interruptible reveal, immediate hide, cleanup, OS/app reduced motion; native preview excluded | New presentation tests, `evidence-settled-browser.png`; pinned MIT Motion 13.2.0, no copied React component |
| Motion + Kokonut Smooth Tab pattern | Website `product-walkthrough.tsx`: local implementation, stable frame, selection indicator, arrow/Home/End keys, associated panels, full-size capture links | Website presentation tests/captures; MIT runtime; reference behavior adapted, source not copied |
| Progressive evidence | `RunReviewScreen.svelte`: verification stays visible, package identity disclosed, duplicate blocker removed without losing accessible description | `review-after-browser-1280x800.png`; no safety-state/backend change |
| State/causality | `MissionDock.svelte`: mount destinations before first hit-test and hit-test release | Red→green sparse browser gesture; native verification still required |
| Plain-language narrative | Hero/product/home FAQ: explicit combined verification and incomplete multi-root scope | Website marketing tests; published downloads unchanged |

Motion's standalone mini entry measures 12,060 minified bytes / 4,997 gzip bytes;
`motion-mini-metafile.json` includes no React rendering inputs. This is **not**
a measured net application-bundle delta. Both website lockfiles preserve the
existing Fumadocs Motion 12.42.2 branch; unrelated peer drift was removed.
Free official documentation was used without installing Motion+ or rewriting
global configuration. New dependency notices are preserved in root
`THIRD_PARTY_NOTICES.md` and beside the local native candidate. Distribution-wide
license packaging is still a release gate. Bklit has no defined data need; Anime.js has no exceptional
timeline requirement. Neither was installed.

## Evidence and remaining work

- `site-before/` and `site-after/`: matched 1440/390 browser viewport captures;
  before comes from the pre-existing local website build, not a native candidate.
  Use `hero-*.png` and final `walkthrough-*.png` for visual review. Long
  `product-*.png` element captures retain a sticky-header capture artifact and
  are not evidence of an actual content-overflow defect.
- `review-support-before-browser.png` and `review-after-browser-1280x800.png`:
  different Review scroll states, not a matched native before/after pair.
- `website-frame-before.json`: observed 21.25px mobile image shift on tab change;
  the final browser regression requires stable positioning across all three tabs.
- `taskboard-before-browser.png`: real fixture baseline, not Pytxo execution proof.
- [[pytxo-interface-engineering]] is now versioned; the existing local skill
  points to it. [Demo runbook](../demo/README.md) contains setup/reset and shots.

Retain the existing Setup, composer and active-work layout until native
inspection supplies the next concrete defect. Replace the old hero crop and
preview images with privacy-safe final-candidate native evidence after a genuine
mission. Native interaction quality and a complete recordable run remain
demo-quality blockers. Installer/signing/platform/security gates remain separate
release-quality blockers. Do not treat this source pass as a completed release.
