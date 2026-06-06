# ADR-0001: Three-tier Rust / Svelte / Tauri stack

## Status

Accepted

## Context

Pytxo must orchestrate many headless CLI agents with low latency, strong isolation, and optional desktop telemetry—without becoming a full IDE.

## Decision

Adopt a **three-tier** architecture:

1. **Presentation** — Svelte 5 (Runes) + Tauri v2; IPC-only, no direct FS.
2. **Orchestration** — Rust (tokio, portable-pty, tree-sitter).
3. **Execution yard** — Child CLI processes + local-first MCP router.

## Consequences

**Positive**

- Clear security boundary at IPC layer.
- Rust suitable for PTY, overlay FS, and sanitization hot paths.

**Negative**

- Two language runtimes to ship and test.
- Tauri IPC schema becomes a versioning contract.

## Links

- [three-tier-model](/docs/three-tier-model)
- Supersedes: none
