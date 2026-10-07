---
title: Beta dependency audit
slug: beta-dependency-audit-2026-09-06
status: active
tags: [project, beta, security, dependencies]
audience: [human, agent]
layer: security
created: 2026-09-06
updated: 2026-09-06
related: [[beta-readiness-plan-2026-09-05]], [[beta-core-audit-2026-09-05]]
---

# Beta dependency audit

GitHub reported 169 open alerts on the default branch when the Beta candidate
was pushed. Those alerts describe a different dependency state from this draft.
The six critical alerts reference old `form-data` and `minimist` versions absent
from the inspected candidate lockfiles. They must not be described as six new
critical defects in this candidate, or dismissed in GitHub before the corrected
dependency state reaches its default branch.

Fresh local audits found actionable issues. The web npm lockfile reported ten
findings, including six high; Desktop reported eleven, including five high.
Compatible dependency updates removed all web findings and all Desktop high
findings. The web pnpm lockfile retained older versions independently of npm;
targeted transitive updates corrected that installation path too. Both web
lockfile audits now report zero findings. No direct application dependency
range changed in either frontend.

The demo's production audit separately found two high findings through
`ajv`/`fast-uri`. Its existing `fast-uri` override was updated from 3.1.5 to
3.1.7; the resulting full dependency audit reports zero findings. Tooling also
reports zero. These are registry observations on this date, not a guarantee
against undiscovered vulnerabilities.

Desktop retains five moderate development findings, all tracing to
[GHSA-w5hq-g745-h8pq](https://github.com/advisories/GHSA-w5hq-g745-h8pq).
The advisory concerns UUID v3/v5/v6 methods with caller-provided output buffers.
Inspected Storybook coverage and Istanbul callers use v4; jest-junit uses v1.
These are development test/reporting tools, and the production audit is clear.
The audit's proposed forced downgrade of Storybook's test runner is rejected;
no blanket cross-major UUID override or advisory suppression was introduced.
Reassess if those callers change or an upstream compatible fix becomes available.

Cargo reports zero **unignored** vulnerabilities. Existing documented exceptions
for notification `quick-xml` and `rsa` remain in `.cargo/audit.toml`; they were
not broadened. Seventeen unmaintained warnings, one GLib soundness warning and
one yanked `spin` warning remain visible. This is not a warning-free Rust audit.

Evidence summaries and lockfile hashes live in
`tooling/benchmarks/results/beta-dependencies-2026-09-06.json`. Frontend checks,
builds and the affected development-tool checks supplement the audits. The
rebuilt Desktop's 37 frontend files exactly match their prior verified hashes.
The dependency work does not establish clean elevated MSI installation or hosted
CI readiness; those gates remain separate in `RELEASE_READINESS.md`.
