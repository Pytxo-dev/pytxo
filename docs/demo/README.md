---
title: Real single-repository Pytxo demo
slug: pytxo-real-demo
status: active
tags: [guide, demo, desktop, verification]
audience: [human, agent]
layer: presentation
created: 2026-09-14
updated: 2026-09-14
related: [[pytxo-interface-engineering]], [[mission-loop]]
---

# Real single-repository Pytxo demo

This runbook prepares a real mission, not simulated execution. Browser preview
fixtures are only UI tests. No completed mission or footage is claimed here.
Current candidate/evidence identities and acceptance status are in CHECKPOINT.md.

## Disposable fixture

Run `./docs/demo/setup.ps1` from the Pytxo checkout. It refuses an existing
destination and creates only `C:\pytxo-disposable-demo-2026-09-14-taskboard`, with
a sentinel, a local Git baseline/tag, and no remote. This fixture commit is not
a commit to Pytxo. No dependencies, accounts, database, or package install are
needed. `node --test` runs its tests; `node server.mjs` serves the actual board at
`http://127.0.0.1:4387`. Run these commands inside the fixture, never C:\pytxo.

Baseline: four sample tasks, add-task form, no search/status filtering, and no
blank-title validation. Data resets on reload. The mission must add the missing
behavior; do not pre-implement it or seed Pytxo's ledger.

Use this outcome in **New run**:

> Add case-insensitive search and status filtering to this local task board.
> Keep both filters composable. Show a helpful no-results message with a way to
> clear filters. Reject blank or whitespace-only task titles without adding a
> row or losing the user's input; show an accessible error and restore input
> focus. Keep the four sample tasks. Add tests for filtering, empty results,
> immutability, and title validation. Use no new dependencies or network calls.
> Change only src/model.mjs, test/model.test.mjs, src/app.js, src/index.html, and
> src/style.css. Do not change tests to bypass a failure. Verify with node --test.

The config provides two real responsibilities: model/tests, then interface.
They have disjoint ownership; the interface depends on the model. This is
manually prepared scope, not a claim of automatic AI decomposition or concurrent
execution. Inspect the actual generated plan before starting. Keep task
verification commands, approved prompts, and dependency order explicit. A CLI
dry-run is configuration evidence only, not native execution evidence.

## Native rehearsal and acceptance

1. Obtain the native computer-use handoff. Stop on Escape, a lock screen, or
   user interruption. Close the previous Pytxo window through its UI before
   launching the candidate: the single-instance guard must not silently focus
   the older executable. Never kill unrelated processes.
2. Verify the candidate hash against CHECKPOINT. Use its isolated main WebView
   profile and launch with `PYTXO_HOME=C:\pytxo-disposable-demo-2026-09-14-profile`
   and the fixture as working directory. Keep the existing installation and
   ledger untouched. Verify the opened executable and clean profile, not just
   the window title. Add only the disposable fixture through supported Setup.
3. Select an already available, authenticated CLI integration. Do not launch
   paid API jobs or change credentials. If existing allowance/integration is
   unavailable, stop execution and record that boundary. Retain Orbit defaults
   and every application-level trust/approval step; do not raise permissions
   merely to get through the demo.
4. Submit the outcome and review the actual plan. Observe real task events and
   preserve errors/retries. Confirm a non-empty combined candidate exists.
   Run **Verify candidate**, inspect the recorded command/results and exact diff,
   and explicitly approve **Apply** only after checking the fixture's full path.
   Do not Apply to C:\pytxo or another repository.
5. Record the resulting Apply state. Inspect changed files and `git diff`, rerun
   `node --test` in the fixture, and demonstrate search, composable status filters,
   no-results recovery, and blank-title refusal in its actual served app.
6. In the same exact native candidate, move a panel right → bottom → right with
   the pointer; test Escape cancellation, keyboard movement, resizing, reload
   persistence, run/project scoping, and reduced motion. Use different diagnostics
   after two inconclusive identical gestures. Record actual dimensions/DPI.
   Check 1282×802 and a smaller supported size (minimum configured 860×560).
   A second real Windows DPI is optional only when safe; browser zoom is not it.

## Capture and edit path

Use the installed Recordly recorder on Windows and Remotion for the local edit,
with FFmpeg/FFprobe for export inspection. Keep the existing `apps/demo-video` source untouched:
its historical footage cannot stand in for this candidate. No new recorder,
video-only dependencies in Desktop, cloud analysis, voice service, or upload.

Store new raw footage, `.recordly` project/telemetry, exports and QA under the
ignored `target/presentation-pass-2026-09-14/media/` directory. First prove a
10-second capture containing movement, hover, click, UI response, and one zoom.
Reopen its editable project and inspect export/cursor alignment before a long
take. Record actual physical pixels, client/window dimensions, scale, and source
fps. The requested delivery is **3840×2160 (4K)**; label upscaling honestly.
The September 14 native session's active display reported 1920×1080, not 4K.
Do not equate a 4K render canvas with native 4K source detail. Prefer reliable 60fps,
otherwise honest 30fps. Disable microphone/webcam unless deliberately needed.

The isolated edit is `target/presentation-pass-2026-09-14/media/edit/`. Its local
dependency junction reuses the existing Remotion 4.0.502 runtime; no historical
composition or lockfile was changed. `src/index.tsx` defines a 3840×2160, 30fps,
10-second capture-proof composition with one restrained camera zoom. TypeScript
passes. `node render-proof.mjs` intentionally refuses missing or uninspected footage;
this is not a completed video. Supply genuine `public/capture-proof.mp4` plus an
inspected `capture-receipt.json` containing SHA-256, candidate SHA-256, physical
width/height, source fps, duration, and native/privacy inspection flags. Then run
the script from that directory and inspect the resulting MP4 before the long take.

Native capture boundary on September 14: Recordly selected **Pytxo Desktop only**,
microphone/webcam off. Starting Record failed twice with the tool reporting that
the button point was over `msedgewebview2.exe`, not Recordly, including after fresh
activation. No input bypass or full-desktop recording was used. The user must
click Record once and hand control back with “recording” before the sample resumes.

The subsequent manual start/stop produced a genuine native take. The first
Remotion sample is now `media/out/capture-proof-4k.mp4`: 3840×2160, 30fps, ten
seconds. `media/edit/capture-receipt.json` records source hashes and the 30–40s
excerpt. The original 1920×1020 canvas contains a 1602×1002 app rectangle; the
sample crops unused black pixels and upscales, without changing clicked targets.
The full original and sidecar remain in `media/raw/`. Full decode and contact-sheet
inspection passed; normal-speed playback remains unverified. This demonstrates
only empty Work → New run, not a real mission or final demo. Start/stop controls
still require user help; keep the next complete mission in a separate fresh take.

Retain one complete genuine source run. Suggested 60–90 second edit:

| Shot | Action and reading hold | Editorial treatment |
|---|---|---|
| Outcome, 0–12s | Enter request; show selected disposable project and reviewed plan | Start in the product; no long intro |
| Work, 12–30s | Show real task responsibilities, running and completed states | Cut long computation with “edited for time” |
| Verify, 30–43s | Show exact combined candidate check and result | Hold the command/result; do not imply task checks suffice |
| Review, 43–60s | Inspect meaningful diff; optionally expand identity/evidence | One motivated zoom, readable text |
| Apply, 60–72s | Explicit approval and truthful resulting state | Keep causal order and approval visible |
| Result, 72–86s | Demonstrate task search, status, no results, title validation | Show the actual modified app |
| End, ≤4s | Pytxo name and single-repository scope | No unlicensed music or invented metric |

Use deliberate pointer movement. Smooth only if suitable separate cursor
telemetry exists; never add a second cursor or move an apparent click target.
Do not conceal unresolved failures through editing. Edited duration is not a
benchmark. Review the MP4 at normal speed, full-decode it, and inspect contact
sheet/click frames. If capture is unavailable, video remains BLOCKED, not rendered.

## Reset safely

Stop fixture agents/server and retain the candidate, raw footage, and evidence.
Run `./docs/demo/reset.ps1` first: it validates the fixed absolute root, sentinel,
Git root, and non-linked affected paths. `-ConfirmDisposableReset` then backs up
and restores only the five allowed source/test files from `demo-initial`.
It does not delete the repo, ledger, index, or untracked files. Review remaining
status manually; build a fresh plan and run. Old verification never transfers.
