---
title: Repository layout
slug: repository-layout
status: active
tags: [reference, meta]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[github-organization]], [[mvp-bootstrap]]
---

# Repository layout

## Current (monorepo transition)

Until the desktop split lands, **Pytxo-dev/pytxo** contains everything below.

| Path | Future repo | Purpose |
|------|-------------|---------|
| `crates/*` | **pytxo** | Rust control plane |
| `apps/desktop/` | **pytxo-desktop** | Tauri + Svelte Reality Deck |
| `docs/` | **pytxo** (optional **pytxo-docs** later) | Obsidian vault |
| `benchmarks/`, `scripts/` | **pytxo** | Smoke / repro scripts |
| `tests/fixtures/` | **pytxo** | Integration fixtures |
| `.github/workflows/` | Split per repo | CI |

## Crate dependency graph

```text
pytxo-core
  ├── pytxo-scheduler
  ├── pytxo-store
  ├── pytxo-sanitize
  └── pytxo-runner
        └── pytxo-orchestrate  ← CLI, MCP, desktop IPC should stop here
              ├── pytxo-cli
              └── pytxo-mcp
```

**Desktop rule:** Tauri may depend on `pytxo-core`, `pytxo-store`, and `pytxo-orchestrate` only — not `pytxo-runner` or `pytxo-scheduler` directly.

## After split

| Repo | Clone | Depends on |
|------|-------|------------|
| [pytxo](https://github.com/Pytxo-dev/pytxo) | CLI, MCP, docs | — |
| [pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop) | UI | pytxo tag `v0.1.x` (git or crates.io) |

**Version contract:** desktop `0.1.x` requires pytxo `0.1.x`.

## Local development

**Monorepo (today):**

```bash
cargo test --workspace
cd apps/desktop && npm ci && npm run check
```

**Two-repo (after split):**

```bash
git clone https://github.com/Pytxo-dev/pytxo.git
git clone https://github.com/Pytxo-dev/pytxo-desktop.git
# Run CLI from your app repo; open desktop from pytxo-desktop
```

## Cargo.lock

`Cargo.lock` is **committed** at the pytxo workspace root for reproducible CI and releases.

## Tauri `gen/`

`apps/desktop/src-tauri/gen/` is **committed** (Tauri capability/schema artifacts). Regenerate with `cargo build -p pytxo-desktop` when capabilities change.
