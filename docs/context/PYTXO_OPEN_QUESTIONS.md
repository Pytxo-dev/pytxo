---
title: Pytxo open questions
slug: pytxo-open-questions
status: active
tags: [context, audit, pytxo]
audience: [human, agent]
layer: meta
created: 2026-09-16
updated: 2026-09-16
related: ["[[PYTXO_CURRENT_STATE]]", "[[PYTXO_DECISIONS]]", "[[PYTXO_HISTORY]]"]
---

# Pytxo open questions

Snapshot 2026-09-16. Questions are not roadmap commitments. The single recommended engineering mission is defined in [[PYTXO_CURRENT_STATE]]; the remaining items stay separate.

## Repository-answerable

| Question | Current evidence / bounded way to answer |
|---|---|
| Does exact-build native Review preserve the repaired A1 boundary? | September 16: legitimate stale A → refreshed B reproduced before repair. Required reviewed digest now rejects A in Core; two-client browser tests pass. Native/package proof remains BLOCKED. |
| Does a stale decision remain invalid when target bytes stay the same but base/check evidence changes? | Automated regression now passes for unchanged targets with fresh verification evidence. Preserve this coverage; demonstrate the exact installed candidate before broader claims. |
| Is the current whole source represented by any one release artifact and acceptance run? | No such proof found. Existing MSI/EXE receipts have different snapshots. Compare exact manifests before runtime acceptance. |
| Are required recipe exclusions too broad for meaningful freshness? | Candidate compares included base inventory; inspect realistic dependency-bearing tasks without weakening the check just to reduce friction. |
| Are Stop, cancellation and journal recovery sound across all exposed backends/platforms? | Source and older evidence exist; focused audit did not rerun the full platform/lifecycle suite. Cloud verifier has an explicit unsupported cancellation contract. |
| What can a malicious same-user child tamper with? | Host filesystem is advisory; store/envelopes are not independently attested. Threat model and boundary review needed before stronger hostile-agent claims. |
| Which recognized harnesses can complete a real task with current auth, permissions and cancellation? | Registry readiness is not runtime proof. Use bounded per-version tests; Qwen currently detection-only. |
| Does any complete routing/RuntimeAdapter/CapacityPool/plugin implementation exist elsewhere? | Bounded search of Core/runner/planner/CLI/Desktop/services found contracts/registries, not these full systems. Avoid broad new implementation until a concrete need. |
| Are old CLI/legacy/manual integration surfaces consistent with the reviewed-run contract? | Desktop golden path was inspected; exhaustive reachability/security audit of every lower-level public API was not performed. |
| Does migration 008 preserve legacy/admin/multi-provider grants and transaction ordering on Postgres? | SQL and memory tests exist; disposable DB execution and concurrency/failure injection remain outstanding. |
| Does Web preserve signatures through the actual network stack? | Raw arrayBuffer ingress is present; Link route tests bypass Next.js → network → Postgres. |
| Which release docs are superseded by later source changes? | This pack identifies major layers; old docs deliberately preserved. Use dated snapshot identities, not the most optimistic paragraph. |

## Product-decision required

These genuinely require Matt's direction; none is needed merely to finish the audit.

- **Minimum Beta scope:** Is a Pytxo plugin API/manager actually a Beta gate, or can the existing golden path ship without it? H2 calls it a foundation; current code does not deliver it.
- **Official initial support contract:** Which platforms and harnesses are promised, and is Desktop the complete blessed journey or is CLI-only Review/Apply parity required?
- **Commercial launch scope:** Which capabilities are paid at the initial release, what plans exist, and does Dodo remain optional alongside Paddle? Source tier enums/caps are not a price/product launch decision.
- **Public emphasis:** Which sentence and first-session demonstration best represent useful delegation plus trust? Existing evidence does not select a unique winning headline.

Do not ask for another decision on established fundamentals: deterministic authority, exact-state intent, single-worker usefulness, honest enforcement, native-sandbox complement, preserved Paddle, and no unapproved publication remain intact.

## User-validation required

Code/reasoning cannot answer:

1. Does Pytxo uniquely prevent meaningful unsafe/out-of-scope changes beyond native permissions, branches, PR review, CI and existing orchestration?
2. Does it reduce total active supervision/coordination/review time, including install, plan repair, false blocks, retries and recovery?
3. Can independent users complete useful real work, understand limitations, and voluntarily choose Pytxo for task two?
4. Is the one-harness/one-worker experience valuable enough to justify another tool?
5. How much review friction is acceptable, and do users understand that host/network surfaces can be advisory?
6. Do users need operational topology, or do lists/diffs/waiting reasons already answer their questions?
7. Which user segment has a recurring pain that existing native workflows do not solve well?

Combine H1's incremental-catch/overhead design with H2/H3 completion/return behavior. Proposed cohort sizes and thresholds are learning aids, not proof or commitments. Retain failed/abandoned tasks and allow users to choose their normal workflow.

## External/provider validation required

- Fresh PR31/hosted CI/public-release state. Local remote refs and dated successful jobs do not prove today's status. Do not revive the old blanket CI billing blocker without a current response.
- Current-artifact clean Windows installation, supported scaling, dependency-bearing mission, Review/Apply and restart/recovery. Development-host administrative extraction is insufficient.
- Dodo account/brand approval, exact catalog, test checkout, signed lifecycle sequence and delivery compatibility. H4 confirms site verification context, not specifically completed Pytxo approval.
- Existing MBCZ/Paddle checkout compatibility and public product state in the separate mbcz-site repository; not audited here.
- Provider/model availability, pricing, quotas, terms and real shared capacity. Historical DeepSeek version/economic claims and free-harness claims remain unverified.
- Harness-native sandbox/permission behavior per supported version and OS.
- Current publication clearance for bundled third-party assets: the September 15 provenance/checkpoint records an Antigravity icon approval-or-omit gate. The provider terms question is separate.
- Privacy/data-egress behavior of a complete local-model workflow. A keyless localhost endpoint alone does not prove offline execution.
- Current service deployment/security posture, signing/distribution and downloaded-asset verification.

No credentials, test purchases, provider changes, migrations, publications or external messages were authorized by the historical handoff prompts or this audit.

## Long-term research questions

- Should Pytxo mediate consequential effects beyond repository state, and what independent post-state evidence/compensation can it honestly provide?
- When does semantic advice improve decomposition enough to justify cost, latency, privacy and new failure modes?
- Can routing learn useful local competence without confounding model, harness, task difficulty, retries and human rescue?
- Where does useful-work scaling flatten, and which shared resources need explicit reservations beyond path claims?
- Can ACP or another existing protocol satisfy a named adapter need without duplicating a harness?
- What plugin containment/API boundary is feasible across supported platforms without expanding the trusted kernel?
- Does semantic topology reduce decision time enough to justify maintenance and screen space?
- Do enterprise policy, marketplaces or hosted fleets solve a demonstrated buyer problem beyond the current repository path?

Unanswered is not blocked implementation work. Do not turn this list into an automatic backlog.
