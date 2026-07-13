---
title: Glossary
slug: glossary
status: active
tags: [meta, glossary]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-09
related: [[MOC-home]], [[product-vision]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]]
---

# Glossary

Canonical product and engineering terms for Pytxo. Prefer plain language in marketing; keep codenames here and in deep docs.

| Term | Definition |
|------|------------|
| **Agent hypervisor** | Pytxo’s role: run, schedule, isolate, and meter headless coding agents on PTYs ([[product-vision]]) |
| **ADE** | Agentic development environment — often UI-heavy multi-agent IDEs (e.g. BridgeSpace-style terminal walls) |
| **Signal Core** | Smarter context: `tree-sitter` skeletons on read ([[signal-core]]) |
| **Blast Shield** | Safe sandbox until you approve; disk flush on approve ([[blast-shield]]) |
| **Race Shield** | No write collisions: swarm registry + stdin buffering ([[race-shield]]) |
| **Execution yard** | Headless CLI agent processes under orchestration |
| **Pytxo Desktop** | Optional control UI: 3D AST topology + telemetry ([[desktop-visual-system]]); formerly Reality Deck |
| **Workspace** | User-facing name for a modular project: one or more folders under one coordinated run ([[modular-projects]]) |
| **Orchestration layer** | Rust core: PTY, DAG, Signal Core, WAL, shields |
| **Presentation layer** | Svelte + Tauri; no direct filesystem access |
| **Pytxo Cloud** | Hosted sandboxes, server-side context cache, **BYOK** ([[hybrid-execution]]) |
| **Pytxo Link** | P2P remote control with signing ([[pytxo-link-signing]]) |
| **BYOK** | Bring your own LLM API keys |
| **MCP hub** | Local-first Model Context Protocol router ([[mcp-hub-integration]]) |
| **Adaptive Semantic Scaffolding** | Fidelity tiers inside Signal Core ([[adaptive-semantic-scaffolding]]) |
| **Sovereign Shield** | Sanitization + cryptographic remote approvals |
| **Sparse overlay FS** | FUSE/ProjFS virtual workspace ([[sparse-overlay-fs]]) |
| **DAG flow engine** | Parallel task scheduler ([[dag-flow-engine]]) |
| **Token arbitrage** | Pro-tier cloud context cache ([[token-arbitrage]]) |
| **Blast radius** | Structural footprint of an agent’s edits on the AST graph |
| **Permission profile** | Local capability ladder (`DeepSpace` … `Supernova`); not FidelityTier or subscription tier ([[permission-profile-engine]]) |
| **Deep Space** | Permission profile Tier 1 — air-gapped process directory |
| **Orbit** | Permission profile Tier 2 — default engineering; CoW bubble, approve-to-flush |
| **Galaxy** | Permission profile Tier 3 — host tools + HITL for high-risk actions |
| **Supernova** | Permission profile Tier 4 — full host user privileges |
| **Execution domain** | One canonical repo root’s isolated scheduler, registry, runner, and WAL ([[execution-domains]]) |
| **Hypervisor registry** | Orchestration map of active execution domains for multi-project runs |
| **Pytxo project** | Modular workspace: named project with one or more **path roots** (folders/repos) for agents ([[modular-projects]]) |
| **Path root** | One directory on a project’s allowlist; tasks and claims resolve relative to a root |

Back: [[MOC-home]].
