---
title: Jev Routing v1 Git handoff
slug: jev-routing-git-handoff-2026-09-30
status: active
tags: [routing, handoff, beta]
audience: [human, agent]
layer: orchestration
created: 2026-09-30
updated: 2026-09-30
related: ["[[2026-09-22-jev-routing-contracts]]", "[[2026-09-22-jev-routing-benchmark]]", "[[ADR-0041-advisory-coordinator-and-routing-boundary]]"]
---

# Jev Routing v1 Git handoff

This commit is an **experimental local foundation**, not activation of hosted Jev or a public beta release. It adds the execution-profile/account/readiness contracts, deterministic eligibility, durable route decisions and attempts, bounded Rules/Shadow advisor seams, reviewed handoffs, exact attempt ownership and recovery, and benchmark recording/analysis tools. The existing DAG and Core's permission, spend, verification, candidate and Apply authority remain in place. The initial choice is only eligible Everyday versus eligible Strong; repair is a separate recorded attempt under Core rules.

Desktop exposes the local experimental review and attempt record. A hosted packet can be inspected before a workspace grant. The account bridge uses a routing-only credential and separate hosted grants; ordinary account sign-in remains a separate path. A guarded HTTP client exists, but normal Flow dispatch does not call it. The hosted review is `review_only`, and the Proxy's paid-send gate is hard closed. Default routing mode is disabled; Rules/Shadow require experimental configuration. Existing subscription/API/local billing choices are not silently converted.

**Source and artifact identity:** the scoped commit deliberately excludes concurrent Desktop presentation/updater work, marketing changes, and Dodo commerce files still present in the working checkout. The locally built MSI in `D:\pytxo-beta-lab\jev-routing-hosted-http-candidate-20260929-22` came from that broader dirty checkout, so it is **not** a package of this commit. It is unsigned and uninstalled. Its extracted executable passed one embedded-host routing fixture before disappearing during the extended Windows launch tests; that is limited local package evidence, not a clean-install or real-traffic result. The original Bitdefender-quarantined bootstrap/Cargo/helper files stayed quarantined, and no security setting was changed. The portable Cargo executable also disappeared during that run.

**Scoped verification:** a clean checkout of the staged source passed `cargo check --locked` for Desktop, CLI, Link, and Proxy (all targets); Rust tests for Core, Desktop, Link, Proxy, Orchestrate, Planner, CLI, and the Runner unit suite; 242 Store/Scheduler tests run directly from the exact compiled test binaries; `cargo fmt --all -- --check`; Desktop `npm run check`; web typecheck and production build; seven focused routing browser tests; three desktop-bridge tests; and 33 benchmark-tool tests. The 73-case routed Flow seam, seven local execution fixtures, and 15 Apply cases passed. Live-provider cases and two benchmark exporter integrations remain skipped. The extended Windows `owned_launch` integration suite is **not green**: its exact-argv case failed after the hash-pinned extracted host disappeared, and the suite stopped before completion. The earlier, broader dirty checkout passed Orchestrate/Desktop Rust tests and produced the local MSI above; those results do not prove the scoped commit's package identity. There has been no real hosted Jev run or paired efficacy trial.

**Git continuation:** start from this commit in a clean checkout and inspect `git show --stat`. Keep the other dirty checkout work separate. Before any Link deployment, reconcile the migration lineage: this commit has Routing migrations 009–019 while an unrelated Dodo 008 migration was untracked in the source checkout. Check applied production versions and the eventual merge order rather than assuming a gap is safe. Rebuild and verify a signed installer from the exact release commit; repeat clean install, upgrade, updater, and native Work → Review → Apply acceptance with the final artifact.

**Activation gate:** keep hosted sends and Live advice off until recipient and redacted-packet review, account/workspace grant and revocation races, real-service admission/accounting tests, source-identical package verification, and an authorized funded benchmark are complete. The benchmark protocol requires frozen R0 rules and a paired RJ trial, including advisor overhead and all failed attempts, with accepted-task quality and cost bounds; no measured Jev advantage exists yet. A fake local advisor or recorded fixture cannot establish that claim. If the benchmark does not pass, retain Rules routing and the replaceable advisor boundary.
