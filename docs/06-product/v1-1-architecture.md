---
title: What changed architecturally in Pytxo v1.1
slug: v1-1-architecture
status: active
tags: [product, architecture, release]
audience: [human, agent]
layer: orchestration
created: 2026-08-10
updated: 2026-08-10
related: [[ADR-0034-immutable-review-package-and-durable-apply]], [[mission-loop]], [[execution-domains]], [[blast-shield]]
---

# What changed architecturally in Pytxo v1.1

Pytxo v1.1 changed the mission loop from “review a workspace, then reconstruct its changes” to “prepare exact bytes once, review that package, and Apply only that package.” The distinction closes the most important trust gap in the earlier design.

## The reviewed result became an artifact

After a successful Orbit or Galaxy run, Pytxo writes a package under `.pytxo/data/reviews/<run-id>/`. Its digest-covered manifest records the base revision, add/modify/delete kind, before and after digests, byte counts, file modes, ownership, and content-addressed blobs. Workspaces are retained until dependent waves finish and the package is durable.

Run Review reads those immutable blobs rather than recomputing a diff from a live agent workspace. Large text and binary content is served through fixed, digest-covered chunks, so exact review does not require an unbounded desktop payload. Untracked files and deletions use the same contract as ordinary modifications.

## Apply became a recoverable state machine

Apply validates only the affected checkout paths. An unrelated dirty file can coexist with a prepared review; drift on a reviewed path marks the package stale and requires another review.

Before mutation, Pytxo claims the execution domain and writes an Apply journal containing backups, operations, temporary paths, directory ownership, and phase progress. Writes use same-volume temporary files. If the process stops, reconciliation runs before another mutation in that domain. It can confirm an Apply, restore the previous tree and return the package to `ready`, classify source drift as `stale`, or stop at `recovery_required` when evidence cannot prove a safe result.

The persisted lifecycle now covers `preparing`, `ready`, `applying`, `applied`, `stale`, `review_failed`, `recovery_required`, and `discarded`. A structured run finalizer also settles errors and clears only the matching active-run marker, preventing abandoned “running” records.

## Desktop became a view of the same domain model

Flow, Operations, Run Review, the CLI, and MCP now read the same persisted run contract. Domain changes append a cursor and emit events, replacing screen-owned polling loops with event-driven updates plus cursor catch-up. This makes Desktop a presentation layer rather than a second orchestration implementation.

## Exact scope

The guarantee applies to reviewed Orbit and Galaxy runs within one execution domain and one repository root. DeepSpace remains non-flushable; Supernova writes directly. v1.1 does not claim power-loss ACID semantics, cross-filesystem durability, cross-root transactions, partial-file acceptance, or kernel-grade isolation. Those boundaries are important because they make the implemented guarantee testable rather than aspirational.

See [[ADR-0034-immutable-review-package-and-durable-apply]] for the accepted decision.
