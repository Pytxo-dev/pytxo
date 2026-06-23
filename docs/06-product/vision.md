---
title: Product vision
slug: product-vision
status: active
tags: [product, vision, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-04
related: [[agent-os-vs-virtual-workspace]], [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains]], [[modular-projects]], [[reality-deck-visual-system]]
---

# Product vision

**Pytxo** ([ptyxo.com](https://ptyxo.com)) is a high-velocity, low-overhead **agent hypervisor and telemetry plane** — not a cloud-heavy virtual workspace.

## Problem

Products like BridgeSpace-style ADEs render many parallel terminal grids inside web containers. That model burns RAM and GPU, ties users to proprietary credits, and optimizes for demos instead of **systems throughput**.

## Pytxo model

Pytxo coordinates **heterogeneous headless terminal agents** (Claude Code, OpenAI Codex, Google Antigravity CLI, and similar) inside **managed background pseudo-terminals** (`portable-pty`). It does not replace your IDE; it plugs in via a local-first [[mcp-hub-integration]] and optional [[reality-deck-visual-system]].

## Productivity max (multi-project)

**Productivity max** has two layers:

1. **Many projects at once** — dispatch independent swarms on different directories (e.g. `/project1` “Fix bug” and `/project2` “Deploy theme”) without blocking the Reality Deck or leaking scheduler, registry, or log state. Today each repo is an [[execution-domains|execution domain]] with its own WAL channel and [[permission-profile-engine|permission profile]] (default **Orbit**).
2. **Modular projects** — one Pytxo **project** can include **multiple path roots** (API repo + web repo + shared protos), Antigravity-style, so a single swarm can work across folders without treating them as unrelated domains. See [[modular-projects]] (architecture; rollout in progress).

See [[ADR-0008-local-permission-profile-four-tiers]].

```text
IDE / CLI  →  MCP hub  →  Orchestration (Rust)  →  Execution yard (PTY agents)
                              ↓
                    Reality Deck (structural telemetry)
```

## Three technical moats

All future orchestration code should route through these layers — not around them.

| Moat | Codename | Function | Target |
|------|----------|----------|--------|
| **Context arbitrage** | [[signal-core]] | `tree-sitter` AST skeletons on file read (signatures, types, imports) | Up to ~60% lower agent input tokens |
| **Copy-on-write sandbox** | [[blast-shield]] | Memory-mapped virtual FS; bash/writes isolated until explicit approval | Sub-5ms rollback; disk flush on approve only |
| **Concurrency guard** | [[race-shield]] | Lock-free swarm registry + stdin buffering | No cross-agent write collisions in a monorepo |

Implementation status: see [[mvp-bootstrap]] and crate README. Moats are **partial or planned** where not yet in `crates/` (sparse kernel overlay, full Galaxy runtime HITL, 3D topology).

| Moat | Shipping today | North star gap |
|------|----------------|----------------|
| Signal Core | 5-language skeletons, closed-loop retry, fallback WAL | Broader grammars, symbol-level escalation |
| Blast Shield | Git worktrees, copy-layer overlay POC | Kernel FUSE/ProjFS sparse overlay |
| Race Shield | Registry, path claims, PTY stdin, Galaxy HITL queue | Runtime MCP gates, lock-free hot paths |

## Reality Deck

Premium **space-console** UI — obsidian void `#020205` with **teal**, **violet**, and **solar gold** accents ([[reality-deck-visual-system]]).

The Deck does **not** show walls of raw terminal text as the primary surface. It visualizes the codebase as an **interactive 3D AST dependency topology** so developers see the **structural blast radius** of agent edits live. Logs and diffs are supporting panels, not the product center.

Repo path: `apps/desktop` in [Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo).

## Pytxo Cloud

Hosted sandboxes that scale the same hypervisor model with **server-side context caching**. Strictly **BYOK** — Pytxo never becomes the LLM vendor. See [[hybrid-execution]] and [[token-arbitrage]].

## Non-goals

- Multi-pane embedded terminal walls in the product UI
- Storing provider API keys in plaintext
- Duplicating IDE editing surfaces

Back: [[MOC-home]] · Compare: [[beyond-the-ade]]
