---
title: Beta workflow UX and proof audit
slug: beta-ux-audit-2026-09-05
status: active
tags: [beta, desktop, onboarding, evidence]
audience: [human, agent]
layer: presentation
created: 2026-09-05
updated: 2026-09-05
related: [[release-audit-ux-demo]], [[first-mission]], [[competitive-benchmarks]]
---

# Beta workflow UX and proof audit

## Evidence baseline

The August 31 [[release-audit-ux-demo]] is historical. Current source already
implements ready-agent selection, stale-plan revocation, causal run/domain
identity, partial snapshots, and exact Apply confirmation. Root
`RELEASE_READINESS.md` records September 5 native v1.2.2 success and stale-path
refusal with package digests, run IDs, executable hashes, and captures. These
are recorded release evidence, not fresh executions by this audit.

The release still declares NOT READY: elevated normal MSI installation and
hosted publication remain incomplete. Administrative MSI extraction is not
a clean installation. A successful deterministic adapter does not establish
the natural-language planner or a real vendor CLI workflow.

## Highest-leverage gaps

1. **Single-harness onboarding contradicts the intended user.** Public
   `apps/web/content/docs/getting-started/first-mission.mdx` explicitly directs
   single-agent users away. Replace it with the concrete value of isolated
   work, independent task checks, exact review, and Apply using one CLI.
   Multiple instances and vendors are optional.
2. **The default mission example cannot reliably plan.**
   `crates/pytxo-planner/src/lib.rs` infers literal existing path mentions;
   unscoped requests fail safely. The documented vague API request supplies
   none. Use an explicit-path first mission and explain current scope
   inference. Automatic skill selection, visual repo understanding, and a
   general intelligence layer are not implemented by this planner.
3. **No checks is a dead end for plugin work.** Current suggestions cover
   root Cargo, npm test, pytest, and Go. Gradle is absent. Desktop blocks an
   unchecked plan but suggests changing the mission; the local planner does
   not parse verification requirements from prose. Tell users to define
   `verify` on manifest tasks and rebuild. Backend reviewed-plan saving copies
   prompt edits only; adding a frontend command editor would silently discard
   changes and needs deliberate backend authority design first.
4. **Evidence needs an explicit scope.** Flow's displayed commands are
   per-task checks, not combined-candidate verification. Label that scope.
   An absent cost estimate must not display “Local” and imply a free vendor
   invocation. Use “Not estimated.”
5. **Vendor login cannot refresh during onboarding.**
   `SetupStepAgents.svelte` probes once, opens vendor login, then has no
   recheck. Add a check-again action and a ready count consistent with Flow,
   including CLIs whose authentication is not applicable.

## Demo, Bench, and telemetry

Root `DEMO.md` truthfully discloses its deterministic adapter and exercises
the actual boundary. Preserve it as control-boundary proof. Add a separate
one-harness mission rehearsal that records mission, approved plan, adapter
version, run/package IDs, task checks, Apply, and post-state checks. Neither
browser preview fixtures nor the deterministic adapter prove model quality.

All inspected `tooling/benchmarks/mission-loop/*/*/NOTES.md` are empty
measurement forms. Existing Signal byte reduction and isolated echo timings
are bounded control-plane measurements. Do not call them task-success,
supervision savings, or a competitor win. A small repeatable one-harness
Bench record should precede a broad scorecard. Parent owns that runner.

No Beta product analytics or opt-in diagnostic export was found in Desktop;
the existing run ledger is local operational telemetry. Keep that distinction
explicit. A local evidence report is a smaller first step than another
network service or unsolicited analytics collection.

## Verification

Desktop: `npm run check`, targeted Playwright shell/Flow checks, production
build, and desktop/390px browser captures for edited onboarding. Public docs:
`npm run check:links` and production Next build. Browser fixtures establish
presentation behavior only; parent owns native and real-harness evidence.

Back: [[MOC-home]] · [[release-audit-ux-demo]]
