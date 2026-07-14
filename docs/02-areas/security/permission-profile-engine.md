---
title: Permission Profile Engine
slug: permission-profile-engine
status: active
tags: [security, orchestration, policy]
audience: [human, agent]
layer: security
created: 2026-06-02
updated: 2026-07-10
related: [[ADR-0008-local-permission-profile-four-tiers]], [[blast-shield]], [[race-shield]], [[signal-core]], [[execution-domains]]
---

# Permission Profile Engine

The **Permission Profile Engine** is the orchestration-layer policy facade that gates what headless agents may read, write, execute, and reach on the network—before any PTY command or MCP tool runs. It is distinct from:

| Term | Meaning |
|------|---------|
| **Three-tier model** | Presentation / Orchestration / Execution yard ([[three-tier-model]]) |
| **FidelityTier** | Signal Core context density: Low / Medium / High ([[adaptive-semantic-scaffolding]]) |
| **Subscription tier** | Hobbyist / Pro / Max billing ([[tiers-hobbyist-pro-max]]) |
| **PermissionProfile** | Local trust ladder (this document) |

ADR: [[ADR-0008-local-permission-profile-four-tiers]].

## Reference type (`pytxo-core::moat::permission`)

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionProfile {
    /// Tier 1 — DeepSpace: air-gapped process directory
    DeepSpace,
    /// Tier 2 — Orbit: default engineering; CoW bubble, approve-to-flush
    #[default]
    Orbit,
    /// Tier 3 — Galaxy: host tools + HITL for high-risk actions
    Galaxy,
    /// Tier 4 — Supernova: full host user privileges
    Supernova,
}
```

Configure via `permission_profile` in [[pytxo-toml]] (default `orbit`). Optional per-`[[agent]]` override; do not confuse `[[agent]].name` (scheduling profile label) with `PermissionProfile`.

## Capability matrix

| Capability | DeepSpace | Orbit | Galaxy | Supernova |
|------------|-----------|-------|--------|-----------|
| Read repo tree | Agent cwd only | Full repo read | Full repo read | Full repo read |
| Writes | Denied or ephemeral cwd only | CoW / worktree; flush via IPC approve | Physical writes; HITL for destructive / out-of-root | Unrestricted |
| Network egress | Blocked | Default deny (allowlist TBD) | Local service ports (Postgres, Docker API, dev servers) | Full |
| Host env (`.ssh`, global `.env`) | Stripped / blinded | Filtered child env | Partial inherit | Full inherit |
| External script runners | Disabled | Sandboxed shell in isolation bubble | Allowed; HITL hooks | Full shell |
| Docker / local DB / dev servers | No | No (or stub) | Yes + HITL | Yes |
| Max Signal **FidelityTier** | Low | Config default | Config default | Config default |
| Primary moat binding | Signal (read-only scaffold) | Blast + Race + Signal | Race HITL + Blast optional | Race advisory only |

### Enforcement status (MVP vs north star)

| Profile | Shipping today | North star |
|---------|----------------|------------|
| **Orbit** | `permission_profile` in config; worktree or sparse overlay (`prefer_kernel_overlay`); path waves; env strip (`SSH_*`); `spawn_egress_allowed` denies network fetch at spawn; `commit_workspace` → `IsolationBackend::flush` | Full kernel CoW; Tauri IPC approve before physical flush |
| **DeepSpace** | Config + `PermissionEngine::max_fidelity(Low)`; env strip; `may_flush` denied; spawn egress denied; OS isolation hooks (Linux netns, macOS sandbox-exec, Windows WFP opt-in `PYTXO_DEEPSPACE_WFP=1`) | Stronger per-process WFP / AppContainer |
| **Galaxy** | Spawn HITL (`fs.delete`, `git.push`, `git.destructive`, `proc.infrastructure`, `fs.permission`, `proc.docker`, `net.egress`, `net.bind`, `proc.package_install`); MCP proxy HITL (`mcp.tool`); flush HITL; stdin line gates; persisted `HitlQueue` | Full runtime syscall hooks |
| **Supernova** | Skips worktree isolation (cwd = `repo_root`); flush without approval gate; spawn egress allowed | Explicit opt-in + audit logging |

## Policy traits (v2 — trait objects deferred)

Composable traits behind a `PermissionEngine` facade:

| Trait | Responsibility | Primary profiles |
|-------|----------------|------------------|
| `FilesystemPolicy` | `may_read`, `may_write`, `flush_requires_approval` | All |
| `NetworkPolicy` | `egress_allowed(host, port)` | DeepSpace blocks all; Orbit default deny |
| `EnvironmentPolicy` | `sanitize_env(child)` — strip `SSH_*`, blind global `.env` | DeepSpace, Orbit |
| `ProcessPolicy` | `may_spawn`, `allowed_interpreters` | DeepSpace denies externals |
| `HitlGate` | `await_approval(action, ctx) -> Approved \| Denied` | Galaxy required; optional on Supernova |

## Mapping to existing moat traits

Do not duplicate moat surfaces—profiles **select and parameterize** them:

| Moat | Trait / type | Profile binding |
|------|--------------|-----------------|
| **Blast Shield** | `IsolationBackend` | **Orbit** primary: `prepare` / `rollback` / `flush` ([[blast-shield]]). `flush()` is the approve-channel contract; UI calls `commit_workspace` over IPC, orchestration calls `flush`. |
| **Race Shield** | `RaceShield` | All profiles: path claims per [[execution-domains]]. **Galaxy** adds HITL for boundary violations and destructive ops. |
| **Signal Core** | `SignalCore` | Unchanged API; profile may cap max `FidelityTier` on egress (e.g. DeepSpace → Low only). |
| **Sovereign Shield** | `pytxo-sanitize` | All profiles when `sanitize = true` ([[regex-sanitization]], ADR-0006)—redaction is not authorization. |

`IsolationMode` (`worktree` | `overlay`) remains a **mechanism** under Orbit/Galaxy, not a replacement for `PermissionProfile`.

## Implementation ownership

| Crate | Role |
|-------|------|
| `pytxo-core` | `PermissionProfile`, `DomainId`, `PermissionEngine` |
| `pytxo-runner` | Enforce at `run_one_agent`: env, fidelity cap, Supernova cwd, `commit_workspace` |
| `pytxo-orchestrate` | Resolve profile from config; `HypervisorRegistry` per-domain isolation |
| `pytxo-store` | Audit `permission_profile` on `runs` row (future migration) |

Presentation ([[presentation-passive-telemetry]]) never enforces policy—only displays state and sends IPC intents.

## Codename reference

| Profile | Codename | Typical use |
|---------|----------|-------------|
| `deep_space` | DeepSpace | Untrusted codegen, air-gapped review |
| `orbit` | Orbit | Day-to-day engineering (default) |
| `galaxy` | Galaxy | Integration tests against local Docker/DB |
| `supernova` | Supernova | Production deploy scripts, global package managers |

Back: [[MOC-home]] · [[execution-domains]] · [[ADR-0008-local-permission-profile-four-tiers]]
