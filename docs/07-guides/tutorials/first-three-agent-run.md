---
title: First three-agent run
slug: first-three-agent-run
status: active
tags: [tutorial, guides]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[cli-reference]], [[pytxo-toml]], [[ADR-0005-worktree-isolation-for-mvp]]
---

# First three-agent run

## Prerequisites

- Rust toolchain (see [`rust-toolchain.toml`](../../../rust-toolchain.toml))
- Git 2.20+
- A git repository (your project root)

## Build

```bash
cargo build -p pytxo-cli
```

## Initialize

```bash
cargo run -p pytxo-cli -- init
```

Creates `.pytxo/worktrees`, `.pytxo/data`, and adds `.pytxo/` to `.gitignore`.

## Configure tasks

Copy [`pytxo.toml.example`](../../../pytxo.toml.example) to `pytxo.toml`. Tasks `task-b` and `task-c` both touch `package.json` — Pytxo schedules them in **separate waves**.

## Dry-run (preflight)

```bash
cargo run -p pytxo-cli -- run --config pytxo.toml --dry-run
```

Inspect JSON: `waves` array and `conflicts` list.

## Run

```bash
cargo run -p pytxo-cli -- run --config pytxo.toml --cmd "echo hello-from-agent"
```

## Inspect telemetry

```bash
cargo run -p pytxo-cli -- status
cargo run -p pytxo-cli -- logs --agent <run_uuid>:agent-0 --tail 20
```

Use agent IDs printed under `status`.

## Without a config file

```bash
cargo run -p pytxo-cli -- run --agents 3 --cmd "echo pytxo"
```

Uses synthetic disjoint paths (`src/agent-0.ts`, etc.) — all can run in one wave.

## Benchmark script

From repo root after build:

```bash
./benchmarks/three-agent-preflight.sh
# Windows:
./benchmarks/three-agent-preflight.ps1
```
