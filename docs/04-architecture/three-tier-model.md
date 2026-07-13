---
title: Three-tier system model
slug: three-tier-model
status: active
tags: [architecture, c4]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[presentation-passive-telemetry]], [[execution-domains]], [[mcp-hub-integration]], [[hybrid-execution]]
---

# Three-tier system model

Pytxo decouples structural telemetry from process coordination using a strictly **event-driven, three-tier** layout ([[product-vision]]).

```text
┌─────────────────────────────────────────┐
│  PYTXO PRESENTATION (Svelte 5)          │
│  Pytxo Desktop · 3D AST topology (target)│
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
2. **Orchestration** — Rust microkernel: PTYs, [[signal-core]], [[blast-shield]], [[race-shield]], [[permission-profile-engine]], [[execution-domains|execution domains]], [[dag-flow-engine]], [[sqlite-wal-logging]]
3. **Execution yard** — Headless CLI agents via [[mcp-hub-integration]]

**Multi-project hypervisor:** concurrent swarms on different repo roots are modeled as separate [[execution-domains]] inside the orchestration tier—never in the presentation layer. Policy ([[permission-profile-engine]]) and WAL routing live here only.

Cloud offload: [[hybrid-execution]].

ADR: [[ADR-0001-three-tier-rust-svelte-tauri]].
