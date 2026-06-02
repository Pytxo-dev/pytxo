---
title: ADR-0003 Sparse overlay instead of RAM COW
slug: adr-0003-sparse-overlay-not-ram-cow
status: accepted
tags: [adr, filesystem]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
adr_id: ADR-0003
related: [[sparse-overlay-fs]]
---

# ADR-0003: Sparse overlay instead of RAM COW

## Status

Accepted

## Context

Monorepos with large `node_modules` and build artifacts cannot be fully materialized in RAM for every agent workspace.

## Decision

Use **sparse overlay virtual filesystem** (FUSE on Unix, ProjFS on Windows): read-only virtual links for bulk deps; in-memory layer for active source edits only.

Reject designs that copy entire dependency trees into RAM per agent.

## Consequences

**Positive**

- Bounded memory with fast local compiles.

**Negative**

- Platform-specific mount complexity and ops documentation.
- Overlay correctness must be tested per OS.

## Links

- [[sparse-overlay-fs]]
