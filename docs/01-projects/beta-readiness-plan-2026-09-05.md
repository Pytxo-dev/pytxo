---
title: Beta readiness implementation plan
slug: beta-readiness-plan-2026-09-05
status: active
tags: [project, beta, reliability, verification]
audience: [human, agent]
layer: orchestration
created: 2026-09-05
updated: 2026-09-06
related: [[mission-loop]], [[beta-core-audit-2026-09-05]], [[beta-ux-audit-2026-09-05]], [[beta-competitor-research-2026-09-05]]
---

# Beta readiness implementation plan

Baseline: `e1807cc`. Earlier release audits are historical; current source and
fresh executions decide what remains. Preserve pre-existing untracked captures,
verification outputs, desktop log, and `docs/superpowers/`.

## Phase 1: Audit and choose

- [x] Read repository instructions, checkpoint, current release evidence and vision.
- [x] Finish independent primary-source competitor, core/reliability, UX and adversarial audits.
- [x] Run the workspace baseline; reproduce current failures before fixing them.

## Phase 2: Close the delegation and evidence gaps

- [x] Make verifier processes stoppable and supervised (`crates/pytxo-runner/src/run.rs`, process registry, Stop integration tests).
- [x] Resolve candidate verification scope honestly: separate task checks from combined-candidate evidence and test incompatible individually passing workspaces (`crates/pytxo-orchestrate`, Desktop review).
- [x] Exercise mission planning through one installed harness, with an inspectable plan, independent checks and immutable review (`crates/pytxo-cli/src/mission.rs`, `tooling/benchmarks/`, `tooling/demos/`). Fix concrete failures encountered.
- [x] Align onboarding, demo, Bench and public documentation to fresh evidence; keep unknown measurements explicit.

## Phase 3: Verify and decide

- [x] Independent adversarial review of changed boundaries; reproduce negative cases.
- [x] Run required Rust, Desktop and affected web/demo gates; inspect desktop/mobile UI for visual changes.
- [x] Record raw evidence and a bounded Beta verdict, distinguishing source readiness from installer/publication and external-user validation.

## Acceptance criteria

Sep 6 checkpoint: the v3 candidate boundary, failed-check refresh recovery and
Git ancestor-discovery regressions pass focused tests. Twenty Desktop browser
checks pass at desktop/mobile widths. The first full v3 workspace run exposed
an old recovery fixture without candidate proof; the fixture now uses a real
content check and its recovery regression passes. The full rerun passed.
The subsequent actual Codex mission and native Apply passed on v3, with matched
primary hashes and four passing post-Apply tests. Evidence lives in
`tooling/benchmarks/results/beta-single-codex-2026-09-06.json`. Full workspace,
clippy, Desktop/native and web gates now pass; fresh MSI build/extraction passed.
No comparative Bench win is established.

Draft PR31 contains the core work and dependency follow-up. Fresh CI jobs were
rejected before starting because of the account payment/spending-limit condition.
Clean elevated installation and public asset verification remain outstanding.
Final independent review completed through the read-only CLI after the in-app
quota failure. Its three production defects and follow-up test-isolation finding
are fixed; full Rust gates, rebuilt native/installer artifacts and a delayed
native refresh responsiveness check passed. The public Beta
verdict is **not ready**. Dependency evidence is [[beta-dependency-audit-2026-09-06]].

Stop must not report a cancelled run while its verifier remains alive. A worker
claim or per-task check must not appear as proof of the integrated candidate.
One installed harness must complete the real planning/execution/review path or
produce a precise, actionable recorded blocker. Bench must not invent timings,
costs or comparative wins. Failed, missing and stale evidence remain explicit.

## Test commands

`cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`;
`cargo build -p pytxo-cli`; `cargo build -p pytxo-mcp`;
`cargo run -p pytxo-cli -- status --json`; `cargo fmt --all -- --check`.
Desktop: `npm run check`, `npm run build:native`, affected Playwright tests.
Web when changed: `npm run lint`, `npm run build`, affected browser tests.

## Boundaries

Local repository Beta first. No new billing, hosted coordinator, production
effect adapters, generic skill marketplace, automatic publishing, or broader
transaction claims. Runner changes apply within one execution domain and must
preserve each permission profile's existing ceiling and enforcement receipt.
