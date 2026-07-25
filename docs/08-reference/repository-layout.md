---
title: Repository layout
slug: repository-layout
status: active
tags: [reference, meta]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-04
related: [[github-organization]], [[mvp-bootstrap]]
---

# Repository layout

## Monorepo (current)

| Path | Purpose |
|------|---------|
| `crates/*` | Rust control plane (CLI, TUI, MCP, scheduler, runner, store, signal) |
| `packages/pytxo/` | npm installer (`npm i -g pytxo`) |
| `apps/desktop/` | Pytxo Desktop — Svelte 5 + Tauri v2 (`pytxo-desktop` crate) |
| `apps/desktop-export/` | Export / release staging slot (see README there) |
| `apps/web/` | Marketing site — Next.js static export, shadcn/ui ([pytxo.com](https://pytxo.com)) |
| `apps/web/content/docs/` | Public docs — Fumadocs MDX inside Next.js ([pytxo.com/docs](https://pytxo.com/docs/)); see [[ADR-0030-public-docs-fumadocs-next]] |
| `docs/` | Obsidian vault |
| `tooling/scripts/`, `tooling/benchmarks/` | Smoke, dev setup, competitive repro scripts |
| `tests/fixtures/` | Integration fixtures |
| `.github/workflows/` | Unified CI (Rust + desktop) |

## Crate dependency graph

```text
pytxo-core
  ├── pytxo-scheduler
  ├── pytxo-store
  ├── pytxo-sanitize
  ├── pytxo-signal
  └── pytxo-runner
        └── pytxo-orchestrate  ← CLI, MCP, desktop IPC should stop here
              ├── pytxo-cli
              └── pytxo-mcp
```

**Desktop rule:** Tauri may depend on `pytxo-core`, `pytxo-store`, and `pytxo-orchestrate` only — not `pytxo-runner` or `pytxo-scheduler` directly. Desktop uses **path dependencies** into `crates/*` (same workspace).

## GitHub org and visibility

All first-party repos live under [Pytxo-dev](https://github.com/Pytxo-dev). Public OSS is in **pytxo**; optional legacy **pytxo-desktop** mirror; planned commercial/cloud repos are documented in [[github-organization]] (not in this monorepo).

## Legacy split repo

[Pytxo-dev/pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop) remains a **public** legacy mirror. Primary development is **`apps/desktop`** in this repository.

## Local development

**Single clone:**

```bash
git clone https://github.com/Pytxo-dev/pytxo.git
cd pytxo
cargo test --workspace
./tooling/scripts/smoke.ps1   # or tooling/scripts/smoke.sh
```

**Pytxo Desktop:**

```bash
cd apps/desktop && npm ci && npm run check
cargo build -p pytxo-desktop    # from repo root
```

## Cargo.lock

`Cargo.lock` is **committed** at the workspace root (includes `apps/desktop/src-tauri`). One lockfile for control plane and desktop.

## Tauri `gen/`

In `apps/desktop/src-tauri/gen/`, generated Tauri capability/schema artifacts are **committed**. Regenerate with `cargo build -p pytxo-desktop` when capabilities change.
