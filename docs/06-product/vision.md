---
title: Product vision
slug: product-vision
status: active
tags: [product, vision, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-23
related: [[agent-os-vs-virtual-workspace]], [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[desktop-visual-system]], [[competitive-benchmarks]], [[pytxo-improvement-research]], [[beyond-the-ade]], [[pytxo-vs-github-copilot-app]]
---

# Product vision

**Pytxo** ([ptyxo.com](https://ptyxo.com)) runs and coordinates the coding agents you already use — Claude Code, Codex, Antigravity CLI, and similar — in the background on your machine. It is a local **agent hypervisor**: schedule work, keep agents from colliding, show what they changed, and let you approve merges. It is **not** another ADE with walls of terminals, and **not** a single-vendor agent desktop (see [[pytxo-vs-github-copilot-app]], [[beyond-the-ade]]).

Maturity and honesty program: [[pytxo-improvement-research]].

## Problem

Products like BridgeSpace-style ADEs render many parallel terminal grids inside heavy desktop or web shells. That model burns RAM and GPU, often ties users to proprietary credits, and optimizes for demos instead of **systems throughput**.

Vendor control centers (e.g. GitHub Copilot app) solve supervision with worktrees and sandboxes inside one ecosystem. Pytxo’s job is different: **cross-CLI local orchestration** for the agents you already run.

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
| **Smarter context** | [[signal-core]] | `tree-sitter` AST skeletons on file read (signatures, types, imports) | Aspirational ~60% lower input tokens on body-heavy files; measure with `signal-reduction` ([[competitive-benchmarks]]) |
| **Safe sandbox until you approve** | [[blast-shield]] | Worktree or sparse copy-layer isolation until explicit approval | Flush on approve; kernel ProjFS/FUSE remains north star |
| **No write collisions** | [[race-shield]] | Locked swarm registry (`RwLock`/`Mutex`) + stdin buffering | No cross-agent write collisions; lock-free shards only after profiling |

Implementation status: see [[mvp-bootstrap]] and crate README. Moats are **partial** where kernel ProjFS/FUSE virtualization and full Galaxy syscall hooks remain north star (sparse copy-layer overlay default shipped Phase 69).

| Moat | Shipping today | North star gap |
|------|----------------|----------------|
| Signal Core | 8+ language skeletons, closed-loop retry, fallback WAL | Broader grammars, symbol-level escalation; verified large-file savings |
| Blast Shield | Git worktrees + sparse copy-layer default via `prefer_kernel_overlay` (Phase 69) | Full kernel ProjFS provider + FUSE default on all platforms |
| Race Shield | Registry, path claims, PTY stdin, Galaxy HITL queue + runtime MCP/stdin gates (Phase 64/70); separate stdin lock | Lock-free / path-prefix shards after profiling |

## Pytxo Desktop

Optional control UI — obsidian void `#020205` with **teal**, **violet**, and **solar gold** accents ([[desktop-visual-system]]). Formerly called Reality Deck.

Desktop does **not** show walls of raw terminal text as the primary surface. **Desktop 2** (default) centers a **structural Focus graph** (Signal Core nodes/edges) plus Ops, Flow, Approvals, and Run Review. Interactive **3D** AST topology (`TopologyScene3D.svelte`) remains available only in the **legacy shell** (`desktop_shell_v1=true`). Logs and diffs are supporting panels, not the product center.

Repo path: `apps/desktop` in [Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).

## Pytxo Cloud

Hosted sandboxes that scale the same hypervisor model with **server-side context caching** — **capability-gated** until a non-noop cloud dispatcher is configured ([[hybrid-execution]]). Strictly **BYOK** on supported surfaces — Pytxo never becomes the LLM vendor. See [[token-arbitrage]] and [[pytxo-improvement-research]].

## Non-goals

- Multi-pane embedded terminal walls in the product UI
- Storing provider API keys in plaintext
- Duplicating IDE editing surfaces
- Claiming unique multi-agent / unique sandbox / invented worktrees vs 2026 vendor products

Back: [[MOC-home]] · Compare: [[beyond-the-ade]] · [[pytxo-vs-claude-agent-teams]] · [[pytxo-vs-github-copilot-app]]