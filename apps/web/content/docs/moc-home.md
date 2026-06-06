---
title: Pytxo documentation home
slug: moc-home
status: active
tags: [moc, meta]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-04
related: [glossary](/docs/glossary), [architecture-index](/docs/architecture-index), [product-vision](/docs/product-vision), [permission-profile-engine](/docs/permission-profile-engine), [execution-domains](/docs/execution-domains), [modular-projects](/docs/modular-projects)
---

# Pytxo documentation home

**Pytxo** — agent hypervisor & telemetry plane ([ptyxo.com](https://ptyxo.com)).

**Thesis:** Bare-metal coordination of headless PTY agents (Claude Code, Codex, Antigravity CLI, …) with structural telemetry — not cloud-heavy multi-terminal workspaces.

**Stack:** Rust (`portable-pty`, `tree-sitter`) · Svelte 5 Runes · Tauri v2

**Vision:** [product-vision](/docs/product-vision) — three moats: [signal-core](/docs/signal-core), [blast-shield](/docs/blast-shield), [race-shield](/docs/race-shield)

---

## Start here

| Audience | Path |
|----------|------|
| Humans | [product-vision](/docs/product-vision) → [glossary](/docs/glossary) → [architecture-index](/docs/architecture-index) |
| Agents | [`AGENTS.md`](https://github.com/Pytxo-dev/pytxo/blob/main/AGENTS.md) (repo root) |
| GitHub | [github-organization](/docs/github-organization) · [repository-layout](/docs/repository-layout) · [Pytxo-dev](https://github.com/Pytxo-dev) |
| Claude Code | claude-vault-context |

---

## Maps of content

### Positioning

- [beyond-the-ade](/docs/beyond-the-ade)
- [agent-os-vs-virtual-workspace](/docs/agent-os-vs-virtual-workspace)

### Architecture

- [architecture-index](/docs/architecture-index) — [three-tier-model](/docs/three-tier-model), [context-diagram](/docs/context-diagram), [c4-container](/docs/c4-container)
- [presentation-passive-telemetry](/docs/presentation-passive-telemetry) · [reality-deck-visual-system](/docs/reality-deck-visual-system)
- [mcp-hub-integration](/docs/mcp-hub-integration)

### Context (agent code materialization)

- [signal-core](/docs/signal-core) — tree-sitter scaffolding and token arbitrage
- [context-launch-contract](/docs/context-launch-contract) — `PYTXO_CONTEXT_DIR`, `manifest.json`, agent responsibilities
- [closed-loop-fidelity](/docs/closed-loop-fidelity) — targeted retry and per-path escalation

### Technical moats

- [blast-shield](/docs/blast-shield) · [race-shield](/docs/race-shield)

### Policy and hypervisor

- [permission-profile-engine](/docs/permission-profile-engine) · [execution-domains](/docs/execution-domains) · [ADR-0008-local-permission-profile-four-tiers](/docs/adr-0008-local-permission-profile-four-tiers)

### Engineering (bottlenecks)

- [adaptive-semantic-scaffolding](/docs/adaptive-semantic-scaffolding) · [closed-loop-fidelity](/docs/closed-loop-fidelity)
- [sparse-overlay-fs](/docs/sparse-overlay-fs) · [dag-flow-engine](/docs/dag-flow-engine) · [sqlite-wal-logging](/docs/sqlite-wal-logging)

### Cloud

- [hybrid-execution](/docs/hybrid-execution) · [sandbox-dispatch](/docs/sandbox-dispatch) · [delta-sync](/docs/delta-sync)

### Security (Sovereign Shield)

- [permission-profile-engine](/docs/permission-profile-engine) · [regex-sanitization](/docs/regex-sanitization) · [pytxo-link-signing](/docs/pytxo-link-signing)

### Product

- [product-vision](/docs/product-vision)
- [modular-projects](/docs/modular-projects) — multi-path workspaces (Antigravity-style)
- [tiers-hobbyist-pro-max](/docs/tiers-hobbyist-pro-max) · [token-arbitrage](/docs/token-arbitrage)
- [gtm-open-source-loop](/docs/gtm-open-source-loop) · [competitive-benchmarks](/docs/competitive-benchmarks)

### ADRs

- [adr-index](/docs/adr-index)

### Guides

- [first-three-agent-run](/docs/first-three-agent-run)
- [pytxo-vs-claude-agent-teams](/docs/pytxo-vs-claude-agent-teams)
- [cost-and-swarm-limits](/docs/cost-and-swarm-limits)
- [multi-agent-orchestration-landscape](/docs/multi-agent-orchestration-landscape)

### Reference

- [cli-reference](/docs/cli-reference)
- [pytxo-toml](/docs/pytxo-toml)

### Ecosystem

- [mcp-router](/docs/mcp-router) · [cursor-vs-vscode-vs-neovim](/docs/cursor-vs-vscode-vs-neovim)

---

## Meta

- [style-guide](/docs/style-guide)
- [changelog-docs](/docs/changelog-docs)
- Archived import: import-v2026.2-raw
