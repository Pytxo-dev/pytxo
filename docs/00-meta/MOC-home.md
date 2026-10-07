---
title: Pytxo documentation home
slug: moc-home
status: active
tags: [moc, meta]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-09-22
related: [[MOC-home]], [[architecture-index]], [[product-vision]], [[commit-layer]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[pytxo-improvement-research]], [[pytxo-architecture-research]], [[mission-loop]], [[pytxo-commit-layer-alignment]], [[v1-1-architecture]], [[pytxo-v1-1-trustworthy-mission-control]], [[chroma-aperture-identity]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0035-desktop-2-quiet-instrument-ia]], [[ADR-0036-effect-contract-commit-boundary]], [[ADR-0037-chroma-aperture-visual-contract]], [[ADR-0038-epistemic-state-contract]], [[ADR-0039-evidence-ledger-visual-contract]], [[ADR-0040-mbcz-merchant-of-record-boundary]], [[dodo-mor-integration]]
---

# Pytxo documentation home

Approved Desktop implementation: [[astra-flat-desktop-02]] — flat identities,
four-stage onboarding and the remaining mission/workflow verification sequence.

Current Desktop feedback: [[astra-ui-feedback-2026-09-13]] — dock dragging,
folder grouping, compact review/setup controls and evidence boundaries.
Current reassessment: [[desktop-interaction-audit-2026-09-13]] defines the
truthful core-loop repair; [[modular-project-safety-contract-2026-09-13]]
separates folder grouping and execution primitives from safe multi-root Apply.
Current interface-tool research: [[pytxo-interface-upstream-research-2026-09-13]]
records the verified Motion, Anime.js, Kokonut UI, Bklit UI, and developer-tool
references behind the reusable Pytxo Desktop interface workflow.
Website design proposal: [[website-ascii-motion-and-clarity-plan-2026-09-24]]
records the product-first ASCII motion and content-reduction plan; no site code is changed.
Versioned implementation contract: [[pytxo-interface-engineering]]; current
presentation references: [[pytxo-presentation-references-2026-09-14]].
Local source/capture status: [[pytxo-presentation-acceptance-2026-09-14]];
repeatable single-repository scenario: [real demo runbook](../demo/README.md).
Harness identity source and release constraints:
[[pytxo-final-three-harness-logo-research-2026-09-15]].

**Pytxo** — agent hypervisor for delegated work, with a reviewed repository
Apply boundary shipping first ([pytxo.com](https://pytxo.com)).

**Thesis:** Let any agent propose; Pytxo controls what may become real, verifies
the resulting state, and records an honest recovery path ([[product-vision]],
[[commit-layer]]). The current proof is one mission → plan → isolated waves →
verify → one reviewable repository Apply ([[mission-loop]]).

**Stack:** Rust (`portable-pty`, `tree-sitter`) · Svelte 5 Runes · Tauri v2

**Vision:** [[product-vision]] · Commit boundary: [[commit-layer]] · Current
controls: [[signal-core]], [[blast-shield]], [[race-shield]] · Repository proof:
[[mission-loop]]

**Architecture snapshot:** [[pytxo-architecture-research]] — July 2026 map of
tiers, controls, policy, ADRs, and crates. Use [[product-vision]] for the current
product thesis.

**Programs:** [[pytxo-v1-1-trustworthy-mission-control]] documents the shipping
repository contract. [[pytxo-commit-layer-alignment]] gates the proposed
production-effect expansion. [[pytxo-improvement-research]] remains the earlier
research ledger.

**Current improvement cycle:** [[astra-execution-2026-09-07]] records scoped
research decisions, implementation ownership, acceptance evidence, and release gates.
[[astra-local-preview-boundary-2026-09-13]] prepares the isolated local-preview renderer decision; it is not enabled.
[[astra-evidence-2026-09-07]] retains actual checks and unsuccessful native attempts.
[[astra-desktop-polish-2026-09-08]] records the current layout details and appearance-persistence repair.
[[astra-desktop-menus-2026-09-09]] records the subsequent menu, sidebar, draft and keyboard workflow improvements.
[[reference-led-review-2026-09-10]] records public workflow references and the verified Work → Review → Apply hierarchy pilot.
[[astra-native-finish-2026-09-10]] records its real native mission, ownership/Git repairs, current package and remaining acceptance gates.
[[astra-release-proposal-2026-09-07]] prepares the owner actions and publication
sequence without authorizing external writes.

---

**GitHub:** [[github-organization]] · [[repository-layout]] · [Pytxo-dev](https://github.com/Pytxo-dev)  
**Discord:** [discord.gg/AUFRPFjSYv](https://discord.gg/AUFRPFjSYv)

---

## Start here

| Audience | Path |
|----------|------|
| Humans | [[product-vision]] → [[glossary]] → [[architecture-index]] |
| Agents | [`AGENTS.md`](../../AGENTS.md) (repo root) |
| GitHub | [[github-organization]] · [[repository-layout]] · [Pytxo-dev](https://github.com/Pytxo-dev) |
| Discord | [discord.gg/AUFRPFjSYv](https://discord.gg/AUFRPFjSYv) |
| Claude Code | [[claude-vault-context]] |

---

## Maps of content

### Positioning

- [[beyond-the-ade]]
- [[agent-os-vs-virtual-workspace]]

### Architecture

- [[2026-09-22-jev-routing-design]] — revised Jev proposal with exact profiles, attempt state machine, handoff/service contracts and rollout; design only
- [[2026-09-22-jev-routing-stress-review]] — source-backed integration findings and required versus preserved changes
- [[2026-09-22-jev-routing-benchmark]] — frozen rules-versus-Jev comparison, quality/cost gates and power limits; not yet run
- [[experimental-claude-proposal-route]] — gated local Desktop route for one reviewed Claude subscription proposal; no Jev send
- [[architecture-index]] — [[three-tier-model]], [[context-diagram]], [[c4-container]]
- [[commit-layer]] — effect contracts, prepare/commit, independent verification, evidence, and recovery
- [[v1-1-architecture]] — immutable review bytes, execution-domain serialization, journaled Apply, and exact limits
- [[presentation-passive-telemetry]] · [[desktop-visual-system]] · [[chroma-aperture-identity]] · [[pytxo-desktop-2-flow-voice]]
- [[mcp-hub-integration]]

### Context (agent code materialization)

- [[signal-core]] — tree-sitter scaffolding and token arbitrage
- [[context-launch-contract]] — `PYTXO_CONTEXT_DIR`, `manifest.json`, agent responsibilities
- [[closed-loop-fidelity]] — targeted retry and per-path escalation
- [[mission-loop]] — one mission → plan → verify → one reviewable apply

### Technical moats

- [[blast-shield]] · [[race-shield]]

### Policy and hypervisor

- [[permission-profile-engine]] · [[execution-domains]] · [[ADR-0008-local-permission-profile-four-tiers]]
- [[ADR-0034-immutable-review-package-and-durable-apply]]: immutable reviewed bytes and process-crash reconciliation for one repository root

### Engineering (bottlenecks)

- [[adaptive-semantic-scaffolding]] · [[closed-loop-fidelity]]
- [[sparse-overlay-fs]] · [[dag-flow-engine]] · [[sqlite-wal-logging]]

### Cloud

- [[hybrid-execution]] · [[sandbox-dispatch]] · [[delta-sync]]

### Security (Sovereign Shield)

- [[permission-profile-engine]] · [[regex-sanitization]] · [[pytxo-link-signing]]

### Product

- [[product-vision]]
- [[positioning]] — October 2026 market, wedge, message hierarchy, product language and allowed claims
- [[pytxo-commit-layer-alignment]] — three-phase falsification, shadow, and gated-production plan
- [[modular-projects]] — multi-path workspaces (Antigravity-style)
- [[tiers-hobbyist-pro-max]] · [[token-arbitrage]]
- [[dodo-mor-integration]] — Dodo MoR, MBCZ business account, Pytxo brand, and provider-neutral Link migration
- [[gtm-open-source-loop]] · [[competitive-benchmarks]]
- [[desktop-ui-improvement-backlog]] — prioritized Pytxo Desktop UI findings
- [[desktop-dangerous-ux-2026-07]] — ops-console / anti-slop Desktop UX research (0.11.0 polish)
- [[pytxo-architecture-research]] — architecture / vision / ADR / crate synthesis (2026-07-27)
- [[mission-loop]] — one mission → plan → verify → apply
- [[mission-loop-dogfood]] — Phase 1 comparison matrix scaffold
- [[pytxo-improvement-research]] — deep maturity synthesis + market landscape + ranked P0–P3 (Phases 73–74 done; 77 mission loop primary)
- [[market-ready-polish-research]] — Phase 74 Desktop supervision + marketing polish
- [[v0.13-first-run-research]] — v0.13.0 impact, live-site docs drift, first-run next steps (2026-07-30)
- [[v0.13-demo-benchmark-readiness]] — Remotion demo, voiceover, real-repo pins, and isolation recursion fix (2026-07-30)
- [[v1-provider-auth-onboarding-research]] — official vendor auth boundaries, child-environment blocker, and v1 integration acceptance criteria (2026-07-30)
- [[v1-product-reference-research]] — 18 official product references and the v1 Desktop, website, and demo design decisions (2026-07-30)
- [[pytxo-v1-1-trustworthy-mission-control]]: v1.1 release thesis, implemented trust contract, and remaining priorities
- [[ADR-0035-desktop-2-quiet-instrument-ia]] — Desktop 2 Ops / Missions / Approvals quiet instrument
- [[ADR-0036-effect-contract-commit-boundary]] — proposed effect-contract expansion beyond repository Apply
- [[ADR-0037-chroma-aperture-visual-contract]] — shared website and Desktop presentation contract
- [[competitive-landscape-2026-07]] — primary-source competitor map + GTM compare priorities (2026-07)
- [[beta-competitor-research-2026-09-05]] — fresh competitor evidence, single-harness value, and testable Beta differentiation
- [[beta-ux-audit-2026-09-05]] — single-harness onboarding, task evidence, and remaining Beta proof gaps
- [[beta-core-audit-2026-09-05]] — verifier Stop ownership, durable cancellation, and remaining core proof gaps
- [[beta-dependency-audit-2026-09-06]] — current lockfile fixes and retained dependency warnings

### ADRs

- [[adr-index]]

### Guides

- [[first-three-agent-run]]
- [[first-mission]]
- [[demo-video-shot-list]]
- [[release-workflow]]
- [[pytxo-vs-claude-agent-teams]]
- [[pytxo-vs-github-copilot-app]]
- [[pytxo-vs-ade-virtual-workspace]]
- [[pytxo-vs-cursor-cloud-agents]]
- [[pytxo-vs-warp-oz]]
- [[pytxo-vs-devin-amp]]
- [[cost-and-swarm-limits]]
- [[multi-agent-orchestration-landscape]]

### Reference

- [[cli-reference]]
- [[pytxo-toml]]

### Ecosystem

- [[mcp-router]] · [[cursor-vs-vscode-vs-neovim]]

---

## Meta

- [[style-guide]]
- [[changelog-docs]]
- Archived import: [[import-v2026.2-raw]]
