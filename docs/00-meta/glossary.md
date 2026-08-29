---
title: Glossary
slug: glossary
status: active
tags: [meta, glossary]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-08-29
related: [[MOC-home]], [[product-vision]], [[commit-layer]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[pytxo-commit-layer-alignment]], [[ADR-0038-epistemic-state-contract]]
---

# Glossary

Canonical product and engineering terms for Pytxo. Prefer plain language in marketing; keep codenames here and in deep docs.

| Term | Definition |
|------|------------|
| **Commit layer** | Externally enforced boundary that turns an agent proposal into an authorized, verified, and recoverable state change ([[commit-layer]]) |
| **Effect contract** | Typed state-transition commitment covering identity, resource, preconditions, allowed effect, budgets, postconditions, evidence, and recovery ([[commit-layer]]) |
| **Effect adapter** | Semantic integration implementing observe, propose, authorize, execute, verify, and compensate with declared idempotency and failure fixtures ([[commit-layer]]) |
| **Evidence DAG** | Causal record linking intent, authority, versions, calls, effects, verification, recovery, and human decisions ([[commit-layer]]) |
| **Mission loop** | Shipping repository path: one mission → proposed plan → isolated waves → verify → one reviewable Apply ([[mission-loop]]) |
| **Agent hypervisor** | Current execution-yard role: run, schedule, isolate, and meter headless coding agents on PTYs; a substrate for the commit layer, not the full product thesis ([[product-vision]]) |
| **ADE** | Agentic development environment — often UI-heavy multi-agent IDEs (e.g. BridgeSpace-style terminal walls) |
| **Signal Core** | Smarter context: `tree-sitter` skeletons on read ([[signal-core]]) |
| **Blast Shield** | Safe sandbox until you approve; disk flush on approve ([[blast-shield]]) |
| **Race Shield** | No write collisions: swarm registry + stdin buffering ([[race-shield]]) |
| **Execution yard** | Headless CLI agent processes under orchestration |
| **Pytxo Desktop** | Optional control UI with three destinations — Work, History, Setup — plus a title-bar workspace switcher and approvals overlay ([[desktop-visual-system]]); 3D AST topology is legacy-shell only; formerly Reality Deck |
| **Workspace** | User-facing name for a modular project: one or more folders under one coordinated run ([[modular-projects]]) |
| **Orchestration layer** | Rust core: PTY, DAG, Signal Core, WAL, shields |
| **Presentation layer** | Svelte + Tauri; no direct filesystem access |
| **Pytxo Cloud** | Optional hosted-sandbox and server-side cache path, available only when a non-noop dispatcher is configured; local surfaces remain **BYOK** ([[hybrid-execution]]) |
| **Pytxo Link** | P2P remote control with signing ([[pytxo-link-signing]]) |
| **BYOK** | Bring your own LLM API keys |
| **MCP hub** | Local-first Model Context Protocol router ([[mcp-hub-integration]]) |
| **Adaptive Semantic Scaffolding** | Fidelity tiers inside Signal Core ([[adaptive-semantic-scaffolding]]) |
| **Sovereign Shield** | Sanitization + cryptographic remote approvals |
| **Sparse overlay FS** | Shipping: sparse **copy-layer** (plus worktrees); kernel FUSE/ProjFS remains north star ([[sparse-overlay-fs]]) |
| **DAG flow engine** | Parallel task scheduler ([[dag-flow-engine]]) |
| **Token arbitrage** | Local billed-vs-sent profiling today; server-side context cache remains capability-gated ([[token-arbitrage]]) |
| **Blast radius** | Structural footprint of an agent’s edits on the AST graph |
| **Permission profile** | Local capability ladder (`DeepSpace` … `Supernova`); not FidelityTier or subscription tier ([[permission-profile-engine]]) |
| **DeepSpace** | Permission profile Tier 1 — air-gapped process directory (written as one word; occasionally seen as "Deep Space" in older prose) |
| **Orbit** | Permission profile Tier 2 — default engineering; CoW bubble, approve-to-flush |
| **Galaxy** | Permission profile Tier 3 — host tools + HITL for high-risk actions |
| **Supernova** | Permission profile Tier 4 — full host user privileges |
| **Execution domain** | One canonical repo root’s isolated scheduler, registry, runner, and WAL ([[execution-domains]]) |
| **Hypervisor registry** | Orchestration map of active execution domains for multi-project runs |
| **Pytxo project** | Modular workspace: named project with one or more **path roots** (folders/repos) for agents ([[modular-projects]]) |
| **Path root** | One directory on a project’s allowlist; tasks and claims resolve relative to a root |

## Epistemic states

Every state rendered on any Pytxo surface resolves to exactly one of these four.
They are the product's core vocabulary, not styling variants
([[ADR-0038-epistemic-state-contract]]).

| Term | Definition |
|------|------------|
| **Verified** | Mechanically enforced or confirmed: `status="enforced"`, `outcome="committed"`, `rollback_confirmed=true`, or a present digest. Encoded as solid fill |
| **Claimed** | Asserted but not mechanically proven: `status="advisory"`. Encoded as solid outline, no fill. Never rendered as a pass |
| **Unknown** | No evidence in this snapshot: `status="unavailable"`, `outcome="interrupted"`, `auth_state="unknown"`, `cursor_gap=true`, or a null digest. Encoded as 45-degree hatch, no hue. **A state, not a warning** |
| **Refuted** | Evidence of failure or circumvention: `status="bypassed"`, `outcome="rolled_back"`, `outcome="recovery_required"`, or a present `last_apply_error`. Encoded as solid fill |
| **Attention** | Separate channel for work needing a human decision. Carries its own hue and a count; never shares a hue with unknown |
| **Enforcement receipt** | `PermissionEnforcementReceipt`: four isolation surfaces (`workspace_isolation`, `host_filesystem_boundary`, `network`, `apply_boundary`), each with a status and the mechanism that produced it, per run and per agent. The signature component of the interface |
| **Cursor gap** | `DomainChangesPageDto.cursor_gap`: the system knows it missed events. Downgrades derived boundary checks to unknown rather than showing last-known-good |
| **Partially applied** | `rollback_confirmed=false` after a failed Apply: the working tree may be partially modified, stated plainly with a recovery step |

Back: [[MOC-home]].
