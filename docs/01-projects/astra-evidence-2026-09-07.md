---
title: ASTRA verification and artifact ledger
slug: astra-evidence-2026-09-07
status: active
tags: [verification, desktop, release, benchmarks]
audience: [human, agent]
layer: orchestration
created: 2026-09-07
updated: 2026-09-07
related: [[astra-execution-2026-09-07]], [[astra-release-proposal-2026-09-07]]
---

# Verification and artifact ledger

The native builds and local checks below were produced with uncommitted changes
on PR31's `ff0b88fb71224d579267e697261ae4d831f0cd76`; that commit alone does not
identify the built source. Their source-input identities remain authoritative
after committing the candidate. CI, clean-machine installation and public
delivery remain separate gates. The owner subsequently upgraded to GitHub Team
and authorized continuation; the organization description/website were updated.
No paid budget, security setting, release or deployment was changed by the lead.

## Completed source checks

| Check | Actual result and local log |
| --- | --- |
| Rust workspace before final receipt repair | 459 passed, `target/astra-workspace-tests-transport-final.log` |
| Final affected orchestration suite | 95 passed, including seven new receipt regressions, `target/astra-research/runtime-original-cap-orchestrate-green.log` |
| Final workspace Clippy, warnings denied | PASS, `target/astra-clippy-final.log` |
| Final CLI and MCP build | PASS, `target/astra-cli-mcp-final.log` |
| Final Rust format | PASS, `target/astra-fmt-final.log` |
| Final Windows MSI build | PASS, `target/astra-msi-final-build.log`; source-input digest `9b9079cb25fc5dc01e749c14ffaccfc67895cbce60f6bbdef12dff671de39858` |
| Desktop production browser suite | 128 passed, `target/astra-desktop-production-e2e-handoff.log` |
| Svelte/CSS checks | Zero errors/warnings, `target/astra-desktop-check-handoff-final.log` |
| Artifact inventory/version checks | 16 passed, `target/astra-release-gates-final.log` |
| Final CLI smoke | Version 1.2.2, valid status JSON and four records for the final fixture; `target/astra-cli-smoke-final.json` |
| Final website lint and production build | PASS after metadata and duplicate-heading repairs, `target/astra-web-{lint,build}-metadata-final.log` |
| Final website links and browser journeys | 161 links across 108 files; 23 browser checks, including all 45 sitemap pages without JavaScript, `target/astra-web-{links,e2e}-metadata-final.log` |
| Website crawl and local performance | Canonical/share identities, robots, sitemap, image dimensions and HTTP status checked; eight bounded timing observations, `target/astra-web-{crawl,performance}-final.json` |
| Final native Review → Apply | Three matching changed-file hashes, eight independent tests, committed journal; `target/astra-native/accepted-*.json` and `accepted-post-tests.log` |
| Native process restart | Same payload; selected committed receipt persisted, `target/astra-native/accepted-restart-history.json` |
| Windows packet | Exact MSI inventory, two baseline fixture tests, all 15 archive-entry hashes match; `target/astra-packet-*.log` and `astra-packet-archive-verification.json` |
| Final silent film | 52.000s, 1920×1080, 30fps, H.264/yuv420p, BT.709, no audio; `target/astra-demo-qa-final.log`; seven scene stills and 13 encoded transition frames inspected |

The mixed-profile receipt collision was reproduced in five candidate and two
Apply regressions. The shared runtime-ordinal resolver now retains the original
permission cap and refuses missing or inconsistent receipts. The full affected
suite and independent source review pass. Final packaging follows combined
Clippy and CLI/MCP checks and has passed; the earlier workspace count is not relabeled as a
post-repair full-workspace run.
Browser fixtures establish UI behavior; they do not establish real agent or MSI
installation outcomes. The final website, native captures, silent film render
and media QA are verified within their stated scopes.

## Native observations, including failures

| Packaged MSI / run | Observed outcome |
| --- | --- |
| `ccfdb862b9c0…` / `9d04e018-a888-46ee-af60-449a902b9adc` | Workers failed before model execution because CMD rejected the verbatim working directory. Preserved in `target/astra-native/first-build/`. |
| `ca8140d30a50…` / `11326244-4d19-4a4c-bb67-49f9493b7f27` | Three workers completed and passed their checks. Preparation correctly refused an implementation worker's out-of-scope test edit; all six baseline file hashes stayed unchanged. |
| `3e48087850e1…` / `98a3ac95-c3a7-47fa-9818-456fb88daa04` | Codex rejected split prompt arguments before model work. Two workers failed; the dependent task was blocked; six baseline hashes stayed unchanged. |
| `78f9f8b3a108…` / no native run | Build passed after the transport repair, source-input digest `ad330846…cd737`. Archived in `target/astra-native/pre-receipt-build/` and superseded by the final build containing the receipt-cap repair. |
| `dfacd548f58e…` / `55a36c03-e0f7-4b92-bec2-53df36f17541` | Workers and combined checks passed. Native diff review found stale README prose contradicting the correctly changed code. Apply was withheld; the ready package remains intact. This was an agent review decision, not automatic semantic rejection. |
| `dfacd548f58e…` / `d743c0dc-76d3-4ff8-b6a3-7d88e743eb22` | With explicit final-state documentation guidance edited into the reviewed task, all three workers and combined checks passed. Native review inspected all exact diffs and confirmed Apply. Three applied hashes match the package; eight independent tests pass; all other source bytes and the complete source inventory are preserved. |

The refusal and launch-failure observations have allowlisted JSON records in
`tooling/benchmarks/results/astra-{refused-run,transport-failure}-2026-09-07.json`.
Full local events retain the diagnosed failures. The review-withheld and
successful outcomes also have separate allowlisted JSON records. Each MSI and
its source-input snapshot remain separately named.

The accepted [native record](../../tooling/benchmarks/results/astra-native-codex-2026-09-07.json)
identifies the full MSI digest `dfacd548f58ea149d84271de50c038288f6b19894f1496dde65916a4a67d20d3`,
payload `9d99646c9ac59c4504a5f4bd7133b1bd84d3327a59a25ccb7dd75926abd9e603`,
and package `0c04ae8106a37dca0993addf572f16c0ba802ba101b0ccd453d2ff5515a0a6bf`.
Six original captures are in `docs/_attachments/astra-2026-09-07/`. An independent
read-only reviewer checked the identity chain, inventories, actual tests, prompt
override and image privacy. Both MSI and payload are Authenticode unsigned.

Recorded native start to candidate verification was 269.899 seconds. The three
CLIs reported 80,975 tokens in total; input/output split and invoice cost remain
unknown. The exact documentation prompt override is in the record and fixture.
The direct run received the original global mission without that added guidance.

## Competent direct comparison

`tooling/benchmarks/results/astra-direct-codex-2026-09-07.json` records the same
mission using Codex 0.153.4 in a separate Git worktree. It changed the requested
three files, passed eight independent tests and preserved the primary checkout.
Observed process duration was 252.172 seconds; the CLI reported 30,811 tokens.
Invoice cost and operator time were not measured. Existing hooks and background
load confound timings. The fixture and exact mission are reproducible from
`tooling/benchmarks/fixtures/astra-first-mission/`.

Do not present a timing delta, primary preservation alone, or this sample as a
Pytxo advantage. The native package/check/Apply evidence stands on its own.

## Screens and remaining local limits

Before/after browser captures are in `target/astra-ui/`. The final website and
Evidence page were inspected at 1440 and 390 pixels, with no page errors or root
horizontal overflow. Narrow Desktop plan controls were additionally inspected
after scrolling; the earlier screenshot named `plan.png` shows only its first
viewport and must not be used as proof of the whole mobile plan.

Native draft reuse restores the workspace and Codex after CLI detection settles
and requires a fresh plan. An early click while detection is pending instead
requires explicit CLI reselection. This safe but awkward edge remains deferred,
as does per-worker terminal-state persistence before all waves finish. No claim
of mixed-vendor Flow, universal Windows adapter prompt support, or OS-wide
filesystem/network isolation was added.

The portable packet is `target/astra-windows-validation-packet.zip`, SHA256
`2d259f66832a45049e89d0d52d6fd0522f0720a52ffd6fddd85cd38ce2c2d8ea`.
It contains 15 checked files, including the tested MSI and fixture-local setup
instructions. Its clean-machine result template remains `not_executed`.

The current silent film is `apps/demo-video/out/pytxo-demo-silent.mp4`, SHA256
`45aab4c6dee117d4792ac637fc181ad1445b65841359086e88d048751339317c`.
Its [provenance record](../../tooling/benchmarks/results/astra-demo-2026-09-07.json)
binds the composition, native record, poster, contact sheet and transition sheet.
No current narration was generated, and provisional caption timing is not
publishable. Older ignored renders and status files are historical artifacts.

## Final website completion audit

The initial production responses had no canonical URLs, inherited the homepage's
share identity on other pages, and omitted Evidence from the sitemap. Two browser
regressions failed against that build; robots/404 and social-image controls passed
(`target/astra-web-metadata-before.log`). Each public page now serves its own
canonical, title, description and complete Open Graph/Twitter identity. The
all-route crawl also reproduced a duplicate BYOK heading; its redundant MDX title
was removed, preserving the rendered page title. The failure is retained in
`target/astra-web-duplicate-heading-red.log`. All 23 final browser checks pass.

The 45 unique sitemap URLs returned 200 and crawlable metadata with JavaScript
disabled. Tracking parameters leave the canonical/share URL unchanged. Robots
exposes the sitemap, `/pricing` redirects to `/plans` with 307, and an unknown
page returns 404 with `noindex`. The declared social PNG returns 200 and matches
its 1600×1000 dimensions; the image was visually inspected. Actual social-network
card rendering and production indexing were not exercised. Introduction and BYOK
screens were inspected at 1440/390px with no root overflow or page errors.

Eight fresh headless Chrome contexts observed Home, Evidence, Introduction and
First mission at both widths on final build `F8ldG-vMSHyJBchf-riN2`. Unthrottled
localhost FCP/LCP observations ranged from 328 to 488ms and observed CLS from 0
to 0.00666. The window ended two seconds after load/font readiness, with a bounded
font wait. Three aborted sign-in prefetches are retained; no page errors occurred.
These single observations overlap other host/test work and do not establish
production/mobile performance, field Core Web Vitals, INP, a Lighthouse score,
or a performance improvement. Further bundle/font work is deferred until target
device or production measurements justify it.

The final native source's 352-file set and every hash still match its freeze;
the MSI, film and Windows ZIP identities also match. The separate website/docs
repairs did not change those artifacts. Recheck:
`target/astra-artifact-recheck-final.json`. The proposed commit inventory is
`target/astra-commit-inventory-final.json`; it excludes the private master brief,
preserved user captures/logs and the unused generated demo Flow copy. Nothing is
staged or published by that inventory.

## External acceptance

Earlier hosted CI was rejected before checks by the organization's Actions
allowance condition. The Team upgrade now provides 1,000 remaining included
minutes; paid overages remain blocked. The consolidated candidate CI run will
establish hosted acceptance only after all required jobs actually pass.
The clean Windows environment is unavailable on this development host. Public
version 1.2.2 is absent. The concrete owner sequence is in
[[astra-release-proposal-2026-09-07]]. Prepared protocols, null result templates,
local extraction and HTTP success never count as closure of those gates.
