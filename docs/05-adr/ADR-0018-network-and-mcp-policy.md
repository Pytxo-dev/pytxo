---
title: ADR-0018 Network and MCP policy enforcement
slug: adr-0018-network-and-mcp-policy
status: accepted
tags: [adr, security, orchestration]
audience: [human, agent]
layer: security
created: 2026-06-21
updated: 2026-06-21
adr_id: ADR-0018
related: [[permission-profile-engine]], [[ADR-0016-galaxy-hitl-enforcement]], [[ADR-0008-local-permission-profile-four-tiers]]
---

# ADR-0018: Network and MCP policy enforcement

## Status

Accepted

## Context

[[ADR-0016-galaxy-hitl-enforcement]] wired Galaxy spawn HITL for destructive and network-ish shell commands. Phase 32 completes the **Orbit network deny** path and extends Galaxy HITL to MCP proxy calls, package installs, and out-of-root flushes.

## Decision

1. **Orbit egress deny:** Before agent spawn, `PermissionEngine::spawn_egress_allowed` blocks `curl`, `wget`, `nc`, `ssh`, and similar commands under **Orbit** and **DeepSpace** ([[permission-profile-engine]] `NetworkPolicy`).
2. **Galaxy spawn HITL extensions:** `hitl_gate::classify_risky_command` adds `proc.package_install` (`npm install`, `pip install`, `cargo install`, …). Existing `net.egress` / `net.bind` remain Galaxy HITL triggers.
3. **Galaxy MCP gate:** `mcp_proxy_call` in orchestration calls `gate_mcp_proxy` before forwarding JSON-RPC. `tools/call`, `resources/read`, and `prompts/get` submit `mcp.tool` to the domain `HitlQueue`.
4. **Galaxy flush outside root:** `commit_workspace` submits `fs.write_outside_root` when the workspace cwd is outside the canonical `repo_root` before `blast.flush`.
5. **Doctor:** `network_policy` verifies Orbit denies / Galaxy allows egress probes; `mcp_hitl` documents Galaxy MCP gating when `permission_profile = galaxy`.

## Consequences

- Orbit is no longer a silent superset of Galaxy for network egress at spawn time.
- MCP hub v2 proxy inherits the same human approval surface as shell spawns under Galaxy.
- Additional classifiers extend `hitl_gate.rs` without new IPC surfaces.
