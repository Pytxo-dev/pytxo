---
title: Release audit — Desktop, web, and demo UX
slug: release-audit-ux-demo
status: active
tags: [release, desktop, web, demo, ux]
audience: [human, agent]
layer: presentation
created: 2026-08-31
updated: 2026-08-31
related: [[vision]], [[commit-layer]], [[pytxo-desktop-2-flow-voice]], [[demo-video-shot-list]]
---

# Release audit — Desktop, web, and demo UX

## Decision

**Do not publish the current launch video or call the Desktop demo-ready.** The current application has a coherent Work / History / Setup shell and unusually strong review/recovery presentation, but the launch cut mixes two generations of Desktop, the browser release gate uses preview fixtures, and several native data paths can mis-scope or silently omit evidence. The application release can proceed only after the P1 trust issues below are fixed and a packaged, real-repository smoke run is recorded.

Status terms used here: **verified** means inspected or executed in this audit; **implemented** means present in source but not exercised against a packaged native runtime; **unverified** means no release evidence was found.

## Findings

### P0 — publication blocker

1. **The 52-second launch asset is internally stale.** The canonical shell has exactly Work / History / Setup (`docs/04-architecture/desktop-visual-system.md:46-50`), but `flow-plan-1920x1080.png` and both `run-review-*-1920x1080.png` visibly use the retired six-destination shell while `work-1920x1080.png` uses the current shell. The cut consumes all four (`apps/demo-video/src/PytxoLaunchDemo.tsx:181,238,251,322-338`), calls its final scene “Operations” (`:334,469`), and its narration repeats that name (`apps/demo-video/VOICEOVER.md:8`). The capture job only refreshes Work at 1920×1080 (`apps/desktop/e2e/marketing-captures.spec.ts:159-174`); the media validator checks existence/dimensions, not IA labels or source revision (`apps/demo-video/scripts/validate-assets.mjs:17-20`). Therefore `npm run validate:assets` passes a visually contradictory video. Narrated delivery is also blocked in this checkout: the required continuous narration, licensed music, and certificate described in `apps/demo-video/README.md:44-54` are absent, so only silent validation could run. Recapture every 1920 scene from one commit/current route, generate a manifest containing source SHA and route, reject retired labels, rewrite Flow/Operations narration to New run/Work/History, and regenerate the poster, silent master, captions, and narrated master.

### P1 — release gate

1. **Workspace scope and run scope can disagree.** Native snapshot loading aggregates runs from every domain (`apps/desktop/src-tauri/src/ipc.rs:1088-1119`). Work then uses all `snapshot.runs` (`apps/desktop/src/components/desktop2/WorkActive.svelte:48-57`) while fetching the focused review with the title-bar `activeDomainId` (`:69-80`). History likewise sorts every run (`apps/desktop/src/components/desktop2/HistoryScreen.svelte:32-33`). Selecting a workspace changes the label/domain but reloads the same aggregate snapshot (`apps/desktop/src/components/desktop2/DesktopShell.svelte:397-410`). Filter run/agent/history state by active domain, or explicitly present a cross-workspace view and always carry the run's own domain ID into review/recovery. Add a two-domain native test that proves no run is shown under the wrong workspace.

2. **A partial native snapshot is presented as successful truth.** Config, store, and run-list errors are silently skipped per domain (`apps/desktop/src-tauri/src/ipc.rs:1091-1104`), after which the IPC returns `Ok` (`:1144-1150`). Only a total IPC failure becomes `snapshot.error` (`apps/desktop/src/lib/desktop-backend.ts:67-93`). Missing runs can therefore look like a verified empty workspace. Return per-domain diagnostics and render the affected scope as unknown/partial; never green-light a snapshot that omitted a domain.

3. **New run does not expose the plan it claims the user reviews.** `FlowPlan` includes permission, isolation intent, execution backend, ADE availability, warnings, blockers, and verification commands (`apps/desktop/src/lib/types.ts:302`), but the plan screen renders tasks/waves, estimate, and a collision count only (`apps/desktop/src/components/desktop2/FlowScreen.svelte:371-399`). A blocked plan gives no actionable reason. Worse, `buildPlan` does not clear the old plan before a failed refresh (`:81-90`), so the previous ready plan remains dispatchable. Clear/mark the plan stale before preview, bind it to a mission/domain/agent digest, show all safety fields and exact blocker messages, and test “valid plan → edit mission → preview fails → Run stays disabled.”

4. **First-run agent selection ignores detected readiness.** Flow defaults to Cursor (`apps/desktop/src/components/desktop2/FlowScreen.svelte:24`) and offers a fixed selector (`:351-357`), although onboarding probes installed/signed-in agents. Auto-select the first ready agent, explain unavailable agents inline, and persist an explicit choice. A Codex-only fresh machine should reach a ready plan without knowing to change the default.

5. **The repository-boundary promise is false for Supernova.** Marketing and Desktop state that nothing reaches the repository without Apply (`apps/web/src/components/site/hero.tsx:21-25`; `apps/web/src/components/site/boundary-section.tsx:107-110`; `apps/desktop/src/components/desktop2/BoundaryPanel.svelte:168-172`), while the reference correctly says Supernova writes directly/unrestricted (`apps/web/content/docs/reference/permission-tiers.mdx:25-28`; `apps/web/content/docs/developers/architecture.mdx:48`). Scope this claim to Orbit/Galaxy and render profile-specific boundary copy. Also replace “only irreversible action” (`apps/web/src/components/site/product-section.tsx:20-23`): reviewed Apply is journaled/recoverable and is not the only consequential action.

6. **Approval evidence is guessed, not causal.** HITL DTOs carry no run ID (`apps/desktop/src-tauri/src/ipc.rs:1121-1131`). The overlay selects the latest run sharing a repository and admits it is doing so (`apps/desktop/src/components/desktop2/ApprovalsInbox.svelte:35-40,181-184`). This can attach an unrelated profile/receipt to a risky decision. Add run, agent, and proposed-effect identifiers to the request/ledger, or remove “Run evidence” until an exact join exists.

7. **The release gate verifies fixtures, not the packaged boundary.** Browser/Storybook/Playwright deliberately switch to `PreviewDesktopBackend` outside Tauri (`apps/desktop/src/lib/desktop-backend.ts:153-160`); that backend contains fabricated signed-in agents, runs, receipts, plans, and Apply outcomes (`apps/desktop/src/lib/desktop-backend.preview.ts:41,133-153,317-350`). This is appropriate for visual tests but cannot certify native IPC, PTY execution, or repository mutation. Add one packaged Windows E2E that creates the real guided repo, dispatches a deterministic local adapter through the native runner, loads its immutable review package, applies it, checks the resulting Git bytes/tests, and exercises stale-head/recovery refusal.

8. **Public release identity is inconsistent.** Code and release notes are 1.2.0 (`Cargo.toml:25`; `apps/web/src/lib/site.ts:3`; `distribution/release-notes/v1.2.0.md:1`), while the root README advertises install v0.3.0 and status v0.1.0 (`README.md:23,74`) and `CHECKPOINT.md:1-4` says v1.0.0. Update public entry points before announcing 1.2.0.

### P2 — important polish and dead ends

- Poll failures after first load are swallowed and the last snapshot remains (`apps/desktop/src/components/desktop2/DesktopShell.svelte:327-371`). Preserve cached content, but mark it stale/offline and expose Retry.
- A wave of failed/stopped agents is counted as “complete” (`apps/desktop/src/lib/epistemic.ts:140-156`; `apps/desktop/src/components/desktop2/WorkActive.svelte:209-213`). Say “reported/settled” and show verified versus failed counts.
- Work exposes Stop for any selected run, including completed runs (`apps/desktop/src/components/desktop2/WorkActive.svelte:48-55,219-221`), and the boundary exposes Review package for any run (`apps/desktop/src/components/desktop2/BoundaryPanel.svelte:168-172`). Gate these actions by native capability/prepared state and explain disabled reasons.
- Onboarding says dispatch works without the CLI (`apps/desktop/src/components/setup/SetupStepCli.svelte:27-32`) while Download says CLI is required (`apps/web/src/app/(marketing)/download/page.tsx:47-52`). The selected-folder confirmation also calls every arbitrary folder a “Local git example” (`apps/desktop/src/components/setup/SetupStepWorkspace.svelte:100-102`) and refers to a retired home screen (`:90-94`). Use one dependency statement and track whether the guided example was actually created.
- The website labels its screenshot “regenerated ... on every build” (`apps/web/src/components/site/product-section.tsx:62-65`), but build only synchronizes checked-in captures. Say “captured from the app” and publish capture SHA/date.

### P3 — consistency debt

- Desktop still renders a chroma ribbon (`apps/desktop/src/components/shell/TitleBar.svelte:85`) although the visual contract reserves the spectrum for the website hero (`docs/04-architecture/desktop-visual-system.md:86`).
- Story names and internal guidance retain retired Operations/Workspaces/Missions labels (`apps/desktop/src/components/desktop2/DesktopShell.stories.ts:40-48`; `docs/07-guides/first-mission.md:53`; `apps/web/content/docs/concepts/modular-projects.mdx:42`). Rename them so capture/test vocabulary cannot regress the IA.

## What is already credible

- **Verified:** Desktop type/style checks, 100 browser release tests, and 19 native Desktop library tests passed. Browser coverage includes 960×640 through 1600×900, keyboard operation, reduced motion, exact-content review, stale Apply, and recovery states; native tests cover exact package bytes, recovery audit persistence, version/auth probes, and creation of the committed example. Web lint, 153-link validation, 27-source/10-marketing capture parity, and a production Next build passed. Demo typecheck, 52-second composition discovery, silent-asset validation, audio-QA unit tests, and transcript/cue validation passed.
- **Implemented:** packaged Tauri uses native IPC rather than preview data; Run Review has exact lazy-loaded before/after bytes, explicit loading/error/unavailable states, durable Apply attempts, stale-head refusal, discard confirmation, and recovery reconciliation. The setup wizard creates a unique, committed, dependency-free Git example (`apps/desktop/src-tauri/src/ipc_install.rs:366-424`) with a native test (`:460-479`).
- **Unverified:** a packaged first launch; actual agent detection/auth; a real PTY run; native immutable-package generation, Apply, stale checkout, crash recovery, and multi-domain switching; installer/download integrity; external website deployment; and a published narrated video. The runbook asks for `NEXT_PUBLIC_DEMO_VIDEO_URL` (`docs/07-guides/demo-video-shot-list.md:130`), but no web source consumes it.

## Deterministic 3-minute commit-boundary demo

Use a clean Windows VM, packaged Desktop/CLI 1.2.0, Orbit, one explicitly signed-in agent, and a newly created `approval-risk-demo-N`. Record source SHA, installer SHA, agent/CLI versions, domain ID, run ID, base revision, and final revision.

| Time | Real action and proof |
|---|---|
| 0:00–0:25 | Finish first-run Agent check and create the guided example. Run its baseline `npm test`; show that no API key is required for the repo itself. |
| 0:25–0:55 | Work → New run; auto-selected ready agent; paste the checked-in mission from `examples/pytxo-first-mission/README.md`; build the native plan. |
| 0:55–1:20 | Review path owners/waves, Orbit ceiling, worktree/PTY intent, ADE command/readiness, verification commands, warnings, blockers, and cost provenance. Dispatch the persisted plan. |
| 1:20–1:55 | Work shows the real run ledger, per-agent path ownership, snapshot age, and enforcement receipt. Use an edit or time cut, never fixture state presented as live. |
| 1:55–2:35 | History → exact run → Run Review. Open one addition/modification/deletion, base revision, plan digest, verification result, receipt, and recovery promise. |
| 2:35–3:00 | Apply reviewed changes. Show committed attempt ID, final revision, clean recovery state, `git diff HEAD^`, and passing `npm test`. End in History on that exact run. |

Needed tooling: (1) a first-party deterministic local demo adapter that makes the checked-in expected patch through the normal runner—never the browser preview backend; (2) `tooling/demos/release-demo.ps1` to create a unique example, verify clean baseline, capture versions/SHAs/IDs, run post-Apply assertions, and emit a non-destructive cleanup path; (3) a native packaged E2E using that adapter; (4) current-route 1920 capture automation plus an asset manifest and retired-label validator; and (5) an optional exact run-linked Galaxy approval fixture only after finding P1.6 is fixed.

## Verification commands

```powershell
cd apps/desktop
npm run check
npm run e2e:release
npm run build:native

cd ../..
cargo test -p pytxo-desktop --lib

cd apps/web
npm run lint
npm run check:links
npm run verify:product-assets
npx next build

cd ../demo-video
npm run typecheck
npm run compositions
npm run validate:assets
npm run render:silent
npm run validate:silent
# Only after licensed narration/music/certificate exist:
npm run validate:assets -- narrated
npm run render:narrated
npm run validate:narrated
```

Release evidence must also include the packaged real-repo script above; green browser fixture tests alone are not sufficient.
