---
title: GitHub organization
slug: github-organization
status: active
tags: [reference, gtm]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[gtm-open-source-loop]], [[mvp-bootstrap]], [[repository-layout]]
---

# GitHub organization

**Canonical org:** [github.com/Pytxo-dev](https://github.com/Pytxo-dev)

Use this organization for all first-party Pytxo repositories, releases, and CI — not personal forks as the source of truth.

## Repositories

| Repo | URL | Contents |
|------|-----|----------|
| **pytxo** | https://github.com/Pytxo-dev/pytxo | `crates/*`, `docs/`, `benchmarks/`, `scripts/`, Rust CI |
| **pytxo-desktop** | https://github.com/Pytxo-dev/pytxo-desktop | Svelte 5 + Tauri Reality Deck; pins `pytxo` git tag |

Optional later: **pytxo-docs** if the vault needs its own release cycle.

See [[repository-layout]] for ownership boundaries and version contract (`desktop 0.1.x` ↔ `pytxo 0.1.x`).

## Conventions

- **Issues & PRs:** Open against the repo that contains the code.
- **Cargo metadata:** `repository = "https://github.com/Pytxo-dev/pytxo"` at workspace root.
- **Fork workflow:** Fork from `Pytxo-dev/*`, PR back to `Pytxo-dev/*`.
- **User projects:** Developers keep their own app repos; Pytxo runs *in* those repos via `pytxo run --repo`. Only Pytxo *product* code lives under Pytxo-dev.

## Local clone

```bash
git clone https://github.com/Pytxo-dev/pytxo.git
cd pytxo
cargo run -p pytxo-cli -- doctor
cargo test --workspace
```

Reality Deck:

```bash
git clone https://github.com/Pytxo-dev/pytxo-desktop.git
cd pytxo-desktop
npm ci && npm run check
```

Requires a Git repo with at least one commit before `pytxo run` (worktrees need `HEAD`). See [[first-three-agent-run]].
