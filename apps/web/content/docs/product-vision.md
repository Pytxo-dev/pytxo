---
title: Product vision
slug: product-vision
status: active
tags: [product, vision, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-09
related: [[agent-os-vs-virtual-workspace]], [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[desktop-visual-system]]
---

# Product vision

**Pytxo** ([ptyxo.com](https://ptyxo.com)) runs and coordinates the coding agents you already use — Claude Code, Codex, Antigravity CLI, and similar — in the background on your machine. It is a local **agent hypervisor**: schedule work, keep agents from colliding, show what they changed, and let you approve merges. It is **not** another ADE with walls of terminals.

## Problem

Products like BridgeSpace-style ADEs render many parallel terminal grids inside heavy desktop or web shells. That model burns RAM and GPU, often ties users to proprietary credits, and optimizes for demos instead of **systems throughput**.

## Pytxo model

Pytxo coordinates **headless terminal agents** inside **managed background pseudo-terminals** (`portable-pty`). It does not replace your IDE; it plugs in via a local-first [[mcp-hub-integration]] and optional [[desktop-visual-system|Pytxo Desktop]].

## Workspaces (multi-project)

**Productivity max** has two layers:

1. **Many projects at once** — dispatch independent swarms on different directories (e.g. `/project1` “Fix bug” and `/project2` “Deploy theme”) without blocking Desktop or leaking scheduler, registry, or log state. Today each repo is an [[execution-domains|execution domain]] with its own WAL channel and [[permission-profile-engine|permission profile]] (default **Orbit**).
2. **Workspaces (modular projects)** — one Pytxo **Workspace** can include **multiple path roots** (API repo + web repo + shared protos), so a single swarm can work across folders without treating them as unrelated domains. See [[modular-projects]].

See [[ADR-0008-local-permission-profile-four-tiers]].

```text
IDE / CLI  →  MCP hub  →  Orchestration (Rust)  →  Execution yard (PTY agents)
                              ↓
                    Pytxo Desktop (structural telemetry)
```

## Three technical moats

All future orchestration code should route through these layers — not around them. Plain language first; codenames for deep docs.

| What it does | Codename | Function | Target |
|--------------|----------|----------|--------|
| **Smarter context** | [[signal-core]] | `tree-sitter` AST skeletons on file read (signatures, types, imports) | Up to ~60% lower agent input tokens |
| **Safe sandbox until you approve** | [[blast-shield]] | Memory-mapped virtual FS; bash/writes isolated until explicit approval | Sub-5ms rollback; disk flush on approve only |
| **No write collisions** | [[race-shield]] | Lock-free swarm registry + stdin buffering | No cross-agent write collisions in a monorepo |

Implementation status: see [[mvp-bootstrap]] and crate README. Moats are **partial or planned** where not yet in `crates/` (kernel sparse overlay production default, full Galaxy runtime syscall hooks).

| Moat | Shipping today | North star gap |
|------|----------------|----------------|
| Signal Core | 5-language skeletons, closed-loop retry, fallback WAL | Broader grammars, symbol-level escalation |
| Blast Shield | Git worktrees, copy-layer + `prefer_kernel_overlay` (Phase 62) | Kernel FUSE/ProjFS default on all platforms |
| Race Shield | Registry, path claims, PTY stdin, Galaxy HITL queue + runtime MCP gates (Phase 64) | Lock-free hot paths |

## Pytxo Desktop

Optional control UI — obsidian void `#020205` with **teal**, **violet**, and **solar gold** accents ([[desktop-visual-system]]). Formerly called Reality Deck.

Desktop does **not** show walls of raw terminal text as the primary surface. It visualizes the codebase as an **interactive 3D AST dependency topology** so developers see the **structural blast radius** of agent edits live. Logs and diffs are supporting panels, not the product center.

Repo path: `apps/desktop` in [Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).

## Pytxo Cloud

Hosted sandboxes that scale the same hypervisor model with **server-side context caching**. Strictly **BYOK** — Pytxo never becomes the LLM vendor. See [[hybrid-execution]] and [[token-arbitrage]].

## Non-goals

- Multi-pane embedded terminal walls in the product UI
- Storing provider API keys in plaintext
- Duplicating IDE editing surfaces

Back: [[MOC-home]] · Compare: [[beyond-the-ade]]
