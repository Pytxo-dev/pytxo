---
title: pytxo.toml reference
slug: pytxo-toml
status: active
tags: [reference, config]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[MOC-home]], [[permission-profile-engine]], [[modular-projects]]
---

# pytxo.toml reference

## Top-level

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_agents` | usize | `3` | Max parallel agents per wave |
| `worktree_dir` | path | `.pytxo/worktrees` | Worktree root |
| `data_dir` | path | `.pytxo/data` | SQLite + state |
| `fail_fast` | bool | `true` | Fail run if any agent exits non-zero |
| `permission_profile` | string | `orbit` | Local capability: `deep_space`, `orbit`, `galaxy`, `supernova` ([[permission-profile-engine]]) |
| `sanitize` | bool | `true` | Sovereign Shield redaction before WAL/MCP |
| `signal_core` | bool | `true` | Materialize scaffolded context for task paths |
| `signal_fidelity` | string | `low` | `low` \| `medium` \| `high` — **FidelityTier** (context), not permission profile |
| `isolation` | string | `worktree` | Blast mechanism: `worktree` \| `overlay` (overlay copy-layer POC with `overlay-fuse` feature; otherwise delegates to worktrees) |
| `dag_explicit_deps` | bool | `false` | Force DAG scheduling mode |

## `[[agent]]`

| Key | Description |
|-----|-------------|
| `name` | Agent **scheduling profile** name (referenced by `[[task]].agent`) — not `PermissionProfile` |
| `paths` | Optional owned globs (scheduling / future enforcement) |
| `permission_profile` | Optional override of top-level `permission_profile` for this agent |
| `model` | Model id for billing router / BYOK |
| `provider` | Provider id (`deepseek`, `openrouter`, `openai`, …) — see [[providers-byok]] |
| `cli_adapter` | CLI adapter (`claude`, `generic`, …) |
| `api_key_env` | Optional env var name for BYOK API key |

Trusted folder tier ([[ADR-0013-folder-trust-tier-picker]]) overrides top-level `permission_profile` when the repo is trusted.

## `[[task]]`

| Key | Description |
|-----|-------------|
| `id` | Task identifier |
| `agent` | Agent profile name |
| `paths` | Paths/globs for conflict preflight and Signal Core context |
| `depends_on` | Optional task ids that must finish first |
| `root` | Optional modular project root label ([[modular-projects]], [[ADR-0011-modular-project-manifest]]) |
| `signal_fidelity` | Optional per-task fidelity override (`low` / `medium` / `high`) |

Tasks with overlapping `paths` are scheduled in different **waves**.

Top-level `subprocess_stdin = true` pumps the Race Shield stdin queue into subprocess children at **spawn time** ([[race-shield]]). PTY backend uses a continuous drain loop.

## `[billing]`

Ultra-tier managed metering ([[ADR-0009-ultra-managed-metering]], [[pytxo-link-service]]).

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `mode` | string | `byok` | `byok` \| `ultra` |
| `proxy_url` | string | `https://link.pytxo.com` | Pytxo Link base URL (entitlements + run ledger) |
| `inference_proxy_url` | string | `https://proxy.pytxo.com` | Ultra managed-inference proxy (ADR-0024) |
| `link_reconcile` | bool | `true` when `mode = ultra`, else `false` | POST run start/end envelopes to Link (HTTP when `link-http` enabled on CLI) |
| `reserve_microcredits` | i64 | `500000` | Wallet reserve per run |
| `initial_balance_microcredits` | i64 | `10000000` | Seed balance for local Ultra dev |

Set `PYTXO_ULTRA_SESSION` for `Authorization: Bearer` on reconcile POSTs. Local reference Link service: [`services/pytxo-link`](../../services/pytxo-link/).

**Ultra default provider:** set `provider = "deepseek"` and `model = "deepseek-chat"` on `[[agent]]` rows; keys live on `pytxo-proxy` (`DEEPSEEK_API_KEY`), not on the client.

## `[cloud]`

Max Swarm remote sandbox + Pro context cache ([[cloud-sandbox-service]]).

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `false` | Enable cloud HTTP clients |
| `sandbox_url` | string | `https://cloud.pytxo.com/v1` | Cloud sandbox API base |
| `cache_enabled` | bool | `true` | Read-through / write-through scaffold cache |
| `fallback_local` | bool | `true` | Fall back to local PTY when cloud unreachable |

Set `PYTXO_CLOUD_SESSION` for `Authorization: Bearer`. Use `execution_backend = "cloud"` or `pytxo run --execution cloud`.

## `[mcp_hub]`

MCP hub v2 — bidirectional proxy to child agent MCP servers ([[mcp-router]]).

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `true` | Register child MCP sessions during runs |
| `allowlist` | string[] | `[]` | Empty = all commands; else match substrings in run `cmd` |

## Example

See [`pytxo.toml.example`](../../pytxo.toml.example) at repo root.

## CLI

```bash
pytxo run --config pytxo.toml --dry-run
pytxo run --config pytxo.toml --cmd "claude"
```

If no config tasks are defined, `pytxo run --agents N` uses synthetic disjoint paths.

## Modular projects

Per-repo `pytxo.toml` configures agents and tasks for the **primary** root. Multi-root workspaces use a **project manifest** (see [[modular-projects]]) with `[[roots]]`; `pytxo project run` schedules one coordinated run and tasks use `root = "label"` to target other roots.
