---
title: Permission Profile Engine
slug: permission-profile-engine
status: active
tags: [security, orchestration, policy]
audience: [human, agent]
layer: security
created: 2026-06-02
updated: 2026-07-31
related: [[ADR-0008-local-permission-profile-four-tiers]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[blast-shield]], [[race-shield]], [[signal-core]], [[execution-domains]], [[pytxo-improvement-research]]
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
| Writes | Denied or ephemeral cwd only | Isolated workspace; immutable-package Apply | Isolated workspace; immutable-package Apply + HITL for destructive / out-of-root actions | Unrestricted host-direct writes |
| Network egress | Blocked (fail closed when unavailable) | Spawn-command deny; arbitrary child sockets advisory | Local service and egress tools through HITL; arbitrary child sockets advisory | Full |
| Host env (`.ssh`, global `.env`) | Stripped / blinded | Filtered child env | Partial inherit | Full inherit |
| External script runners | Disabled | Sandboxed shell in isolation bubble | Allowed; HITL hooks | Full shell |
| Docker / local DB / dev servers | No | No (or stub) | Yes + HITL | Yes |
| Max Signal **FidelityTier** | Low | Config default | Config default | Config default |
| Primary moat binding | Signal (read-only scaffold) | Blast + Race + Signal | Race HITL + Blast optional | Race advisory only |

### Enforcement status (MVP vs north star)

| Profile | Shipping today | North star |
|---------|----------------|------------|
| **Orbit** | Worktree or sparse copy-layer; path waves; filtered child env; spawn-command network policy; persisted enforcement receipt; single-root immutable-package Apply | Kernel CoW and syscall-grade network/filesystem boundary |
| **DeepSpace** | Low Signal fidelity; filtered env; non-flushable; spawn egress denied; dispatch fails closed when the required socket-isolation mechanism is unavailable | Strong per-process WFP / AppContainer on Windows and equivalent boundaries everywhere |
| **Galaxy** | Isolated workspace plus single-root immutable-package Apply; spawn/MCP/stdin HITL for high-risk actions; persisted `HitlQueue` and enforcement receipt | Full runtime syscall hooks |
| **Supernova** | Skips workspace isolation (`cwd = repo_root`); writes directly to the host tree; spawn egress allowed | Explicit opt-in + audit logging |

Every run records `requested_profile`, `effective_profile`, execution domain,
and four enforcement surfaces: workspace isolation, host filesystem boundary,
network, and Apply boundary. A surface is labeled `enforced`, `advisory`,
`unavailable`, or `bypassed`; presentation must not upgrade an advisory check
into a sandbox claim.

## Policy traits (v2 — trait objects deferred)

Composable traits behind a `PermissionEngine` facade:

| Trait | Responsibility | Primary profiles |
|-------|----------------|------------------|
| `FilesystemPolicy` | `may_read`, `may_write`, `flush_requires_approval` | All |
| `NetworkPolicy` | `egress_allowed(host, port)` | DeepSpace blocks all; Orbit default deny |
| `EnvironmentPolicy` | `sanitize_env(child)` — strip `SSH_*`, blind global `.env` | DeepSpace, Orbit |
| `HitlGate` | `await_approval(action, ctx) -> Approved \| Denied` | Galaxy required; optional on Supernova |

`ProcessPolicy` (`may_spawn`, `allowed_interpreters`) is **deferred to Phase 75** — remove from docs claims until implemented in crates.

## Mapping to existing moat traits

Do not duplicate moat surfaces—profiles **select and parameterize** them:

| Moat | Trait / type | Profile binding |
|------|--------------|-----------------|
| **Blast Shield** | `IsolationBackend` + prepared review package | Orbit/Galaxy stage exact add/modify/delete bytes for one repository root. Desktop calls `apply_run_changes`; orchestration validates affected paths and applies only the immutable package ([[ADR-0034-immutable-review-package-and-durable-apply]]). |
| **Race Shield** | `RaceShield` | All profiles: path claims per [[execution-domains]]. **Galaxy** adds HITL for boundary violations and destructive ops. |
| **Signal Core** | `SignalCore` | Unchanged API; profile may cap max `FidelityTier` on egress (e.g. DeepSpace → Low only). |
| **Sovereign Shield** | `pytxo-sanitize` | All profiles when `sanitize = true` ([[regex-sanitization]], ADR-0006)—redaction is not authorization. |

`IsolationMode` (`worktree` | `overlay`) remains a **mechanism** under Orbit/Galaxy, not a replacement for `PermissionProfile`.

## Implementation ownership

| Crate | Role |
|-------|------|
| `pytxo-core` | `PermissionProfile`, `DomainId`, `PermissionEngine` |
| `pytxo-runner` | Enforce agent env, fidelity cap, isolation, receipts, immutable package preparation, journaled Apply, and recovery |
| `pytxo-orchestrate` | Resolve profile, keep workspaces through package preparation, and reconcile each execution domain |
| `pytxo-store` | Persist requested/effective enforcement JSON, prepared manifests, Apply lifecycle/errors, and domain change cursors |

Presentation ([[presentation-passive-telemetry]]) never enforces policy—only displays state and sends IPC intents.

## Codename reference

| Profile | Codename | Typical use |
|---------|----------|-------------|
| `deep_space` | DeepSpace | Untrusted codegen, air-gapped review |
| `orbit` | Orbit | Day-to-day engineering (default) |
| `galaxy` | Galaxy | Integration tests against local Docker/DB |
| `supernova` | Supernova | Production deploy scripts, global package managers |

Back: [[MOC-home]] · [[execution-domains]] · [[ADR-0008-local-permission-profile-four-tiers]]
