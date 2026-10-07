---
title: Experimental Claude proposal route
slug: experimental-claude-proposal-route
status: active
tags: [routing, desktop, experimental]
audience: [human, agent]
layer: orchestration
created: 2026-09-28
updated: 2026-09-29
related: ["[[2026-09-22-jev-routing-design]]", "[[2026-09-22-jev-routing-benchmark]]"]
---

# Experimental Claude proposal route

This is a Windows-only, locally gated **Rules** route for one reviewed repository file. It chooses between Claude subscription Haiku (everyday) and Sonnet (strong). It does not call Jev, send a hosted routing packet, use an API billing account, or enable Live routing. The normal Desktop Codex flow remains the default. The experiment exists to qualify the profile, attempt, verification, and Review → Apply path before evaluating Jev advice.

Set these variables in the Desktop process environment before starting it:

```powershell
$env:PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL = '1'
$env:PYTXO_ROUTED_CLAUDE_EXE = 'C:\path\to\claude.exe'
$env:PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME = $env:USERPROFILE
$env:PYTXO_ROUTED_CLAUDE_PROBE_ROOT = 'D:\existing-separate-probe-directory'
```

The executable and probe directory must exist. The probe directory must be absolute, separate from the selected account home, and suitable for disposable native tests. The selected Claude CLI must use a signed-in `claude.ai` subscription. Pytxo rechecks that claim at dispatch; choosing this route is not a provider login or a billing guarantee. No API key is passed to the worker. The first explicit experimental preview creates a one-slot local capacity pool keyed by a hash of the canonical selected account-home path. It never enlarges or re-enables a disabled existing pool.

This pilot requires the included checkout files to match the reviewed Git snapshot exactly. A clean checkout transformed by `core.autocrlf` or another checkout filter, or one containing an included Git-ignored file, is rejected before account probes. Use a separate, byte-identical checkout for the experiment; ordinary Git cleanliness alone is insufficient for this route's current exact-byte Review and Apply contract.

In **Work → New work**, explicitly choose **Claude proposal route · subscription**, describe one change to one tracked file, and add a verification command. Build and inspect the plan. The task prompt is frozen in the reviewed route; change the request and rebuild if it is wrong. The review expires one hour after preview. Run performs live account and model qualification before worker launch. Both profiles receive positive and cancellation probes, which can consume subscription quota and several minutes even if no candidate results. The worker returns a no-tools JSON file proposal; Pytxo alone writes the claimed file into its isolated candidate, runs the frozen check, and prepares existing Review → Apply. Review the exact bytes before Apply.

The separate, default-off `PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_REPAIR=1` flag permits **at most one** clean Sonnet retry after a Haiku attempt whose frozen check fails with intact, retained evidence. The reviewed plan shows this two-call limit before Run. Strong-first tasks, ambiguous failures, changed checker views, and Stop do not trigger repair. The second attempt starts from the reviewed base; it does not inherit failed output. Both calls may consume subscription quota, and Pytxo cannot report an exact subscription cost.

The clean retry receives the first failed check's ID, exit code and exact owned receipt digest. Pytxo rechecks that retained receipt and the failed predecessor before binding and launching the second prompt. It does not forward checker stdout or stderr, which may contain secrets, and it does not import failed file bytes or check-suggested commands. A diagnostic-text handoff would need a separate, explicit disclosure design.

After Run starts, **Stop starting run** records an exact durable Stop request while qualification or execution is pending. The same control is available from a saved request whose startup is still unresolved after returning to Work. A request is not a claim of completed termination: wait for the saved request and run ledger to settle, and use recovery if native ownership remains uncertain. Stop uses the original Store location recorded at dispatch even if `data_dir` changes during startup.

The route is limited to Orbit, a single execution domain, subprocess worktree isolation, one task, one claimed file, and one worker at a time. It allows one attempt by default or two under the separately reviewed repair gate. Requested model identity is recorded; immutable provider-side model identity and per-attempt subscription cost are unknown. Stop and recovery still use the routed attempt ledger. Removing the experiment flags disables new launches, including previously reviewed plans. A browser-preview fixture and disposable native test are development evidence; neither substitutes for a signed installer or a funded Jev-versus-rules benchmark.
