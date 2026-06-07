---
title: ADR-0012 Hypervisor Shell as default UX
slug: ADR-0012-hypervisor-shell-default-ux
status: accepted
tags: [adr, tui, shell, ux]
audience: [human, agent]
layer: meta
created: 2026-06-07
related: [[ADR-0010-pty-default-execution-backend]], [[product-vision]], [[beyond-the-ade]]
---

# ADR-0012: Hypervisor Shell as default UX

## Status

Accepted (v0.2.0)

## Context

v0.1.0 shipped a passive TUI dashboard (doctor, runs, HITL). Users expected a **central command prompt** like Claude Code or Codex CLI. Pytxo is an **agent hypervisor**, not another coding chatbot.

## Decision

1. **Default `pytxo` opens the Hypervisor Shell** — board + scrollback + operator prompt.
2. **Operator input is slash commands** (`/run`, `/dry-run`, `/logs`, …) routed through **`pytxo-shell`**, shared by TUI, and later MCP/desktop.
3. **Pytxo does not embed LLM chat** in v0.2.0. User missions spawn **external ADE CLIs** (`agy`, Claude Code, Codex) via `pytxo run --cmd`.
4. **Optional NL planner** (`pytxo-planner`) is feature-gated (`PYTXO_PLANNER=1` or `[planner] enabled`) and outputs `Vec<Task>` validated by `build_plan()` before dispatch.
5. **CLI subcommands remain** for scripts (`PYTXO_NO_TUI=1`, CI, MCP).

## Consequences

- New crates: `pytxo-shell`, `pytxo-planner` (stub/heuristic).
- `RunOptions` gains runtime `tasks` and `task_cmd_template` for shell-driven runs.
- TUI splits into modules; tokio dispatches runs without blocking the UI thread.
- Non-goals for v0.2.0: multi-pane terminal grid, inline PTY mirror, built-in LLM vendor.

## Alternatives rejected

| Alternative | Why rejected |
|-------------|--------------|
| Clone Claude Code chat in ratatui | Duplicates ADEs; blurs hypervisor vs pilot |
| Planner required for v0.2.0 | Blocks ship; couples UI to model providers |
| Replace CLI with TUI-only | Breaks automation, MCP, CI |

Back: [[index]]
