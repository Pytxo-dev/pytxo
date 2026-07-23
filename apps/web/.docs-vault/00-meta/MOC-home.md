---
title: Pytxo documentation home
slug: moc-home
status: active
tags: [moc, meta]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-23
related: [[glossary]], [[architecture-index]], [[product-vision]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[pytxo-improvement-research]]
---

# Pytxo documentation home

**Pytxo** — agent hypervisor & telemetry plane ([ptyxo.com](https://ptyxo.com)).

**Thesis:** Bare-metal coordination of headless PTY agents (Claude Code, Codex, Antigravity CLI, …) with structural telemetry — not cloud-heavy multi-terminal workspaces.

**Stack:** Rust (`portable-pty`, `tree-sitter`) · Svelte 5 Runes · Tauri v2

**Vision:** [[product-vision]] — three moats: [[signal-core]], [[blast-shield]], [[race-shield]]

**Improvement program:** [[pytxo-improvement-research]] — Phase **73 shipped** (proof pins); Phases **74–76** next (Desktop supervision, moat depth, commercial gates).

---

## Start here

| Audience | Path |
|----------|------|
| Humans | [[product-vision]] → [[glossary]] → [[architecture-index]] |
| Agents | [`AGENTS.md`](../../AGENTS.md) (repo root) |
| GitHub | [[github-organization]] · [[repository-layout]] · [Pytxo-dev](https://github.com/Pytxo-dev) |
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
- [[pytxo-improvement-research]] — deep maturity synthesis + market landscape + ranked P0–P3 (Phase 73 done; 74 next)

### ADRs

- [[adr-index]]

### Guides

- [[first-three-agent-run]]
- [[pytxo-vs-claude-agent-teams]]
- [[pytxo-vs-github-copilot-app]]
- [[pytxo-vs-ade-virtual-workspace]]
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
