---
title: ADR-0010 PTY default execution backend
slug: adr-0010-pty-default-execution-backend
status: accepted
tags: [adr, orchestration, pty]
audience: [human, agent]
layer: orchestration
created: 2026-06-04
updated: 2026-06-04
adr_id: ADR-0010
related: [ADR-0001-three-tier-rust-svelte-tauri](/docs/adr-0001-three-tier-rust-svelte-tauri), [race-shield](/docs/race-shield), [product-vision](/docs/product-vision)
---

# ADR-0010: PTY as default execution backend

## Status

Accepted

## Context

Product vision and ADR-0001 describe headless agents in **managed pseudo-terminals** (`portable-pty`). Phase 1–2 shipped **piped subprocess** execution (`std::process::Command` with stdout/stderr pipes), which does not expose a TTY to interactive CLIs and leaves Race Shield `StdinBuffer` disconnected from child I/O.

## Decision

1. **Default** agent execution uses **`portable-pty`** in `pytxo-runner` (`execution_backend = "pty"` in `pytxo.toml`).
2. **Fallback** `execution_backend = "subprocess"` retains piped subprocess behavior for CI, minimal smoke, and environments where PTY spawn fails.
3. **Race Shield** stdin: during PTY sessions, a pump thread drains `SwarmRegistry` stdin queues into the PTY master writer.
4. **Child environment** is built through a shared `ChildLaunchEnv` helper so Ultra `ManagedTransport`, Signal context paths, and permission sanitization apply identically to PTY and subprocess backends.
5. `pytxo doctor` includes a PTY smoke check (echo in a TTY).

## Consequences

**Positive**

- Aligns runtime with hypervisor positioning and [pytxo-link-signing](/docs/pytxo-link-signing) (stdin unlock on approved streams).
- Enables interactive CLI agents without changing the three-tier IPC model.

**Negative**

- PTY output multiplexes stdout/stderr on one stream unless split by the shell; WAL events use a single `stdout` kind for PTY lines (stderr kind reserved for subprocess backend).
- Windows ConPTY adds platform-specific failure modes; subprocess fallback remains required.

## Supersedes

None.
