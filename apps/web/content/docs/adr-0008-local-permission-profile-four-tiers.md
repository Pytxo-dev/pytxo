---
title: ADR-0008 Local permission profile (four tiers)
slug: adr-0008-local-permission-profile-four-tiers
status: accepted
tags: [adr, security, orchestration]
audience: [human, agent]
layer: security
created: 2026-06-02
updated: 2026-06-02
adr_id: ADR-0008
related: [permission-profile-engine](/docs/permission-profile-engine), [execution-domains](/docs/execution-domains), [ADR-0001-three-tier-rust-svelte-tauri](/docs/adr-0001-three-tier-rust-svelte-tauri), [blast-shield](/docs/blast-shield)
---

# ADR-0008: Local permission profile (four tiers)

## Status

Accepted

## Context

Pytxo is expanding from a single-repository shell runner into a **multi-project agent hypervisor**. Users will dispatch independent agent swarms across different repo roots concurrently (e.g. `/project1` and `/project2`) without UI blocking or cross-project state leakage.

Agents require deep local system access in some workflows (Docker, local databases, production deploy scripts) but must be constrainable for untrusted or read-only tasks. Today, isolation is expressed only via `IsolationMode` (worktree vs overlay) and scheduling `paths`—not a unified capability model. The term **tier** is already used for architecture layers, Signal **FidelityTier**, and subscription billing; we need a distinct, canonical name for local trust levels.

ADR-0001 requires the Svelte/Tauri presentation layer to remain a passive telemetry skin with **no direct filesystem access**. Policy and enforcement must live in orchestration and runner crates.

## Decision

1. Introduce **`PermissionProfile`** as the single orchestration-side local capability ladder with four variants (codenames):
   - **DeepSpace** (Tier 1) — air-gapped process directory
   - **Orbit** (Tier 2) — default engineering; workspace-scoped read, CoW writes, approve-to-flush
   - **Galaxy** (Tier 3) — interactive host tools with Human-in-the-Loop (HITL) for high-risk actions
   - **Supernova** (Tier 4) — full host user privileges

2. Default for engineering workflows: **`Orbit`**.

3. **`IsolationMode`** (`worktree` | `overlay`) remains a **mechanism** subordinate to profile:
   - Orbit requires a CoW-capable backend (worktree MVP; overlay north star).
   - DeepSpace may forbid physical flush entirely.
   - Supernova may bypass Blast Shield isolation.

4. Policy enforcement is implemented **only** in `pytxo-orchestrate`, `pytxo-runner`, and future `pytxo-core::moat::permission`—never in the presentation layer.

5. Multi-project concurrency is modeled as **`ExecutionDomain`** instances under a **`HypervisorRegistry`** ([execution-domains](/docs/execution-domains)). Each domain owns its own `SwarmRegistry`, scheduler plan, runner task handle, and SQLite WAL file under that repo’s `data_dir`. Domains do not share in-memory PTY buffers or event append paths.

6. The Reality Deck may **request** approve-to-flush, HITL responses, and dispatch via Tauri IPC; orchestration validates against the active `PermissionProfile` before mutating disk or releasing blocked agents.

## Consequences

**Positive**

- One vocabulary for security reviews, `pytxo.toml`, and IPC ACL tables.
- Clear mapping from profiles to existing moats (Signal, Blast, Race) without duplicating trait surfaces.
- Multi-project “productivity max” without interleaved telemetry at the store layer.

**Negative**

- Four profiles × multiple mechanisms increases test matrix size.
- Galaxy HITL and DeepSpace network/env stripping require platform-specific hooks not yet in MVP code.
- Documentation and code must disambiguate **PermissionProfile** from **FidelityTier** and subscription tiers in every public surface.

## Links

- [permission-profile-engine](/docs/permission-profile-engine) — capability matrix and trait boundaries
- [execution-domains](/docs/execution-domains) — hypervisor registry and WAL separation
- [ADR-0001-three-tier-rust-svelte-tauri](/docs/adr-0001-three-tier-rust-svelte-tauri)
- [ADR-0005-worktree-isolation-for-mvp](/docs/adr-0005-worktree-isolation-for-mvp)
- Supersedes: none
