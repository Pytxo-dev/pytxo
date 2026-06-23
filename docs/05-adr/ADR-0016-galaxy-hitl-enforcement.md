---
title: ADR-0016 Galaxy HITL enforcement and persistence
slug: adr-0016-galaxy-hitl-enforcement
status: accepted
tags: [adr, security, orchestration]
audience: [human, agent]
layer: security
created: 2026-06-19
updated: 2026-06-19
adr_id: ADR-0016
related: [[permission-profile-engine]], [[race-shield]], [[ADR-0008-local-permission-profile-four-tiers]]
---

# ADR-0016: Galaxy HITL enforcement and persistence

## Status

Accepted

## Context

[[ADR-0008-local-permission-profile-four-tiers]] defines **Galaxy** as host tools plus human-in-the-loop for high-risk actions. Phase 18 shipped the `HitlQueue`, CLI, Deck panel, and flush gating on `commit_workspace`. Only **blast.flush** was wired at runtime.

## Decision

1. **Spawn gate (Galaxy):** Before agent PTY/subprocess spawn, classify the resolved command via `hitl_gate::classify_risky_command` (`fs.delete`, `git.push`, `proc.docker`, `net.egress`, `net.bind`). When matched, block until `HitlQueue` resolves approve/deny.
2. **Persistence:** Each domain's queue persists pending requests + decisions to `{data_dir}/hitl.json`. `HitlQueue::with_persistence` loads on domain creation and writes on submit/resolve.
3. **Flush gate unchanged:** `commit_workspace` still requires HITL when `flush_requires_approval()` and a queue is present.
4. **Doctor:** `hitl_persistence` check verifies the domain data dir can write `hitl.json`.

## Consequences

- Galaxy is enforceable without relying on manual Deck flush alone.
- Hypervisor restart can reload pending HITL from disk for domains that were previously active.
- Additional risky-op patterns extend `hitl_gate.rs` without new IPC surfaces.
