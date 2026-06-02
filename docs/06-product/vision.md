---
title: Product vision
slug: product-vision
status: active
tags: [product, vision, architecture]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[agent-os-vs-virtual-workspace]], [[signal-core]], [[blast-shield]], [[race-shield]], [[reality-deck-visual-system]]
---

# Product vision

**Pytxo** ([ptyxo.com](https://ptyxo.com)) is a high-velocity, low-overhead **agent hypervisor and telemetry plane** — not a cloud-heavy virtual workspace.

## Problem

Products like BridgeSpace-style ADEs render many parallel terminal grids inside web containers. That model burns RAM and GPU, ties users to proprietary credits, and optimizes for demos instead of **systems throughput**.

## Pytxo model

Pytxo coordinates **heterogeneous headless terminal agents** (Claude Code, OpenAI Codex, Google Antigravity CLI, and similar) inside **managed background pseudo-terminals** (`portable-pty`). It does not replace your IDE; it plugs in via a local-first [[mcp-hub-integration]] and optional [[reality-deck-visual-system]].

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

Implementation status: see [[mvp-bootstrap]] and crate README. Moats are **partial or planned** where not yet in `crates/`.

## Reality Deck

Premium **space-console** UI — obsidian void `#020205` with **teal**, **violet**, and **solar gold** accents ([[reality-deck-visual-system]]).

The Deck does **not** show walls of raw terminal text as the primary surface. It visualizes the codebase as an **interactive 3D AST dependency topology** so developers see the **structural blast radius** of agent edits live. Logs and diffs are supporting panels, not the product center.

Repo: [Pytxo-dev/pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop).

## Pytxo Cloud

Hosted sandboxes that scale the same hypervisor model with **server-side context caching**. Strictly **BYOK** — Pytxo never becomes the LLM vendor. See [[hybrid-execution]] and [[token-arbitrage]].

## Non-goals

- Multi-pane embedded terminal walls in the product UI
- Storing provider API keys in plaintext
- Duplicating IDE editing surfaces

Back: [[MOC-home]] · Compare: [[beyond-the-ade]]
