---
title: Pytxo v1 live product demo runbook
slug: demo-video-shot-list
status: active
tags: [guide, marketing, release, desktop]
audience: [maintainer]
layer: guide
created: 2026-07-26
updated: 2026-07-30
related:
  - "[[release-workflow]]"
  - "[[v1-provider-auth-onboarding-research]]"
  - "[[v1-product-reference-research]]"
---

# Pytxo v1 live product demo runbook

Record a real 85–95 second product walkthrough with Screen Studio-style camera movement. The
viewer should see one continuous product story:

`agent readiness → guided repo → mission → plan → run → evidence → human decision`

The Remotion cut in `apps/demo-video` is the narrated launch asset. This runbook is for the
less-produced live recording that proves the application works.

## Capture profile

- Record at **2560×1440, 60 fps** and export 1920×1080. Use 1920×1080 only when the display
  cannot sustain the larger canvas.
- Set Windows display scaling and Pytxo Desktop scale to 100%.
- Use the Void theme with Teal accent and Comfortable density.
- Keep the pointer visible at normal size while recording. In the edit, use 1.25–1.45× camera
  zooms with 250–350 ms easing. Do not add a giant cursor halo.
- Keep every zoom anchored to the clicked control or the selected evidence panel. Hold each
  destination for at least 1.5 seconds.
- Disable desktop notifications, password-manager overlays, browser autofill, and unrelated tray
  badges. Use a clean OS account name in any visible path.
- Prefer three short matched takes over one fragile continuous take. Screen Studio on macOS is
  ideal; on Windows, capture with OBS and reproduce the same eased zooms in the editor.

## Safe rehearsal

1. Build the exact candidate being filmed:

   ```powershell
   cd C:\pytxo\apps\desktop
   npm run build:native
   ```

2. Open Integrations and verify Codex reports **ChatGPT connected**. Claude Code and Cursor may
   also be ready, but the recorded run needs only one working harness.
3. Do not configure or show a raw provider key. DeepSeek is a metered API credential, not a
   consumer login, and is not needed for this demo.
4. Go to Workspaces and choose **Open guided example**. Pytxo creates a fresh numbered Git repository
   under Documents, with passing dependency-free tests. Create a new example for every take rather
   than resetting an old agent workspace.
5. Select Codex in Flow and paste the canonical mission:

   > Add concise risk summaries for network and destructive command changes in
   > `src/risk-policy.mjs`; add regression tests in `test/risk-policy.test.mjs`; document two
   > examples in `README.md`. Keep the existing `classifyChange` API.

6. Build the plan and check that it is ready, has explicit repository-relative paths, suggests
   `npm test`, and uses the Orbit permission profile. If the plan is blocked, stop the take and fix
   the real reason; never conceal a blocked state.
7. Run one full rehearsal. Confirm the selected headless command is
   `codex exec --sandbox workspace-write`, the run reaches a terminal state, and the example tests
   still pass. Preserve that real completed run for the evidence take.

## Three recording takes

### Take A — readiness and first mission (00:00–00:28)

| Time | Action | Camera |
|---|---|---|
| 00:00–00:04 | Open on Operations. Pause before moving. | Full frame |
| 00:04–00:11 | Open Integrations. Move across Codex, Claude Code, and Cursor readiness without opening account details. | 1.28× on Agent sessions |
| 00:11–00:17 | Open Workspaces and click **Open guided example**. | Follow pointer, then settle on new workspace |
| 00:17–00:28 | Flow opens. Paste the canonical mission and choose Codex. | 1.35× on mission composer |

Voice line:

> Pytxo keeps the coding agents you already use. Their sessions stay with their vendor CLI. Start
> with your repository, or create the guided example with no API key.

### Take B — plan and execution (00:28–00:58)

| Time | Action | Camera |
|---|---|---|
| 00:28–00:35 | Click **Build plan**. Do not accelerate the first plan reveal. | Match-cut into Review |
| 00:35–00:43 | Point to task paths, dependencies, verification, execution waves, and Orbit policy. | Two restrained 1.3× focus moves |
| 00:43–00:48 | Click **Run** once. | Return to full frame |
| 00:48–00:58 | Open Operations and select the exact active run. Point to isolation, paths, approvals, and cost. | 1.25× on the summary, then detail |

Voice line:

> Flow turns the outcome into an inspectable plan before anything runs. Race Shield separates
> overlapping work into stages. Blast Shield keeps every agent away from the primary checkout.

For a long model wait, cut between two genuine states. Do not fake terminal output or show a
fabricated progress percentage.

### Take C — evidence and decision (00:58–01:30)

| Time | Action | Camera |
|---|---|---|
| 00:58–01:08 | Open the preserved completed run in Run Review. Show changed paths and verification evidence. | 1.3× on evidence |
| 01:08–01:20 | If the rehearsal produced a real pending apply/flush request, open Approvals and select it. Show requester, workspace, consequence, and latest-run evidence. | 1.35× on decision panel |
| 01:20–01:25 | Approve and apply, or deny and discard, according to the take. Click once and wait for the resolved state. | Hold on the exact action |
| 01:25–01:30 | Return to Operations or the clean completed run. | Ease back to full frame |

Voice line:

> Operations replaces terminal noise with state. Risky actions arrive with evidence and
> consequences, so you approve the result—not the promise.

Only record an approval that the real rehearsal generated. If no approval exists, end on Run
Review; do not substitute fixture data in a live-demo claim.

## Edit and release checklist

- Use hard cuts at route changes and eased zooms inside a route.
- Remove dead pointer travel, but preserve enough time to understand each state.
- Keep captions to product verbs: **Connect, Plan, Isolate, Run, Review, Approve**.
- Add the final line: **One mission. Several agents. One safe, reviewable result.**
- End on `pytxo.com/download`; do not end on a pricing tier or unshipped cloud capability.
- Watch once at 100% and once at 50% size. All key labels must remain readable at 50%.
- Scrub every frame for account emails, machine usernames, API keys, private repository names,
  terminal history, and notifications.
- Upload the final MP4 or YouTube URL and set `NEXT_PUBLIC_DEMO_VIDEO_URL`. Leave the variable
  unset until the recording passes this checklist.
