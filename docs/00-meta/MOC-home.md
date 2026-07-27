---
title: Pytxo documentation home
slug: moc-home
status: active
tags: [moc, meta]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-27
related: [[glossary]], [[architecture-index]], [[product-vision]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[pytxo-improvement-research]], [[pytxo-architecture-research]], [[mission-loop]]
---

# Pytxo documentation home

**Pytxo** — agent hypervisor & telemetry plane ([ptyxo.com](https://ptyxo.com)).

**Thesis:** Local inspectable workflow engine for existing agent CLIs — one mission → plan → isolated waves → verify → one reviewable apply ([[mission-loop]]).

**Stack:** Rust (`portable-pty`, `tree-sitter`) · Svelte 5 Runes · Tauri v2

**Vision:** [[product-vision]] — three moats: [[signal-core]], [[blast-shield]], [[race-shield]] · Mission loop: [[mission-loop]]

**Architecture synthesis:** [[pytxo-architecture-research]] — full primary-source map of vision, tiers, moats, policy, ADRs, crates.

**Improvement program:** [[pytxo-improvement-research]] — Phases **73–74 shipped**; **Phase 77 = Mission Loop Phase 1** (primary). Phases 75–76 honesty/moat items remain secondary.

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

- [[architecture-index]] — [[three-tier-model]], [[context-diagram]], [[c4-container]]
- [[presentation-passive-telemetry]] · [[desktop-visual-system]] · [[pytxo-desktop-2-flow-voice]]
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

### Engineering (bottlenecks)

- [[adaptive-semantic-scaffolding]] · [[closed-loop-fidelity]]
- [[sparse-overlay-fs]] · [[dag-flow-engine]] · [[sqlite-wal-logging]]

### Cloud

- [[hybrid-execution]] · [[sandbox-dispatch]] · [[delta-sync]]

### Security (Sovereign Shield)

- [[permission-profile-engine]] · [[regex-sanitization]] · [[pytxo-link-signing]]

### Product

- [[product-vision]]
- [[modular-projects]] — multi-path workspaces (Antigravity-style)
- [[tiers-hobbyist-pro-max]] · [[token-arbitrage]]
- [[gtm-open-source-loop]] · [[competitive-benchmarks]]
- [[desktop-ui-improvement-backlog]] — prioritized Pytxo Desktop UI findings
- [[desktop-dangerous-ux-2026-07]] — ops-console / anti-slop Desktop UX research (0.11.0 polish)
- [[pytxo-architecture-research]] — architecture / vision / ADR / crate synthesis (2026-07-27)
- [[mission-loop]] — one mission → plan → verify → apply
- [[mission-loop-dogfood]] — Phase 1 comparison matrix scaffold
- [[pytxo-improvement-research]] — deep maturity synthesis + market landscape + ranked P0–P3 (Phases 73–74 done; 77 mission loop primary)
- [[market-ready-polish-research]] — Phase 74 Desktop supervision + marketing polish
- [[competitive-landscape-2026-07]] — primary-source competitor map + GTM compare priorities (2026-07)

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
