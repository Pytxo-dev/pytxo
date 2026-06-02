---
title: Three-tier system model
slug: three-tier-model
status: active
tags: [architecture, c4]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[presentation-passive-telemetry]], [[mcp-hub-integration]], [[hybrid-execution]]
---

# Three-tier system model

Pytxo decouples UI telemetry from process coordination using a strictly **event-driven, three-tier** layout.

```text
┌─────────────────────────────────────────┐
│  PYTXO PRESENTATION (Svelte 5)          │
│  Runes · Monaco diff · xterm.js         │
└─────────────────┬───────────────────────┘
                  │ Tauri v2 IPC
┌─────────────────▼───────────────────────┐
│  PYTXO ORCHESTRATION (Rust core)       │
│  Token compressor · Flow engine · WAL   │
└─────────────────┬───────────────────────┘
        ┌─────────┴─────────┐
        │ Unix sockets /   │  P2P WebRTC / TLS
        ▼                  ▼
┌───────────────┐  ┌──────────────────┐
│ LOCAL         │  │ PYTXO CLOUD      │
│ EXECUTION     │  │ SANDBOX          │
│ YARD          │  │                  │
└───────────────┘  └──────────────────┘
```

## Tiers

1. **Presentation** — [[presentation-passive-telemetry]]
2. **Orchestration** — Rust microkernel: spawn/throttle PTYs, isolation, scaffolding, [[sqlite-wal-logging]]
3. **Execution yard** — Headless CLI agents via [[mcp-hub-integration]]

Cloud offload: [[hybrid-execution]].

ADR: [[ADR-0001-three-tier-rust-svelte-tauri]].
