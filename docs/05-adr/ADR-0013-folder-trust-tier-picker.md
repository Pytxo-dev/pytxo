---
title: ADR-0013 Folder trust with tier picker
slug: ADR-0013-folder-trust-tier-picker
status: accepted
tags: [adr, tui, security, trust]
audience: [human, agent]
layer: meta
created: 2026-06-07
related: [[ADR-0008-local-permission-profile-four-tiers]], [[ADR-0012-hypervisor-shell-default-ux]]
---

# ADR-0013: Folder trust with tier picker

## Status

Accepted (v0.3.0)

## Context

Pytxo spawns arbitrary shell commands in git worktrees. Users need a clear consent moment before agents run — similar to IDE “Do you trust this folder?” flows — tied to the existing four-tier `permission_profile` ladder.

## Decision

1. Persist trust per **canonical repo root** in `~/.pytxo/trusted-domains.json`.
2. Hypervisor Shell shows a **tier picker modal** on first launch for untrusted folders.
3. **`/run` and `dispatch` are blocked** until trusted; `/doctor`, `/dry-run`, `/status`, `/help` remain available.
4. Trusted tier **overrides** the default `permission_profile` from `pytxo.toml` for that domain.
5. `/trust` re-opens the picker to change tier.

## Consequences

- New `TrustedDomainStore` in `pytxo-core`.
- TUI boot state machine: trust modal before prompt.
- Orchestrate merges trust record when loading config.

Back: [[index]]
