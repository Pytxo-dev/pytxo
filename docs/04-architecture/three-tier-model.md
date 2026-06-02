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

Pytxo decouples structural telemetry from process coordination using a strictly **event-driven, three-tier** layout ([[product-vision]]).

```text
┌─────────────────────────────────────────┐
│  PYTXO PRESENTATION (Svelte 5)          │
│  Reality Deck · 3D AST topology (target)│
└─────────────────┬───────────────────────┘
                  │ Tauri v2 IPC
┌─────────────────▼───────────────────────┐
│  PYTXO ORCHESTRATION (Rust core)       │
│  Signal Core · Blast Shield · Race     │
│  Shield · DAG · WAL · sanitize          │
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
2. **Orchestration** — Rust microkernel: PTYs, [[signal-core]], [[blast-shield]], [[race-shield]], [[dag-flow-engine]], [[sqlite-wal-logging]]
3. **Execution yard** — Headless CLI agents via [[mcp-hub-integration]]

Cloud offload: [[hybrid-execution]].

ADR: [[ADR-0001-three-tier-rust-svelte-tauri]].
