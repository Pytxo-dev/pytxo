---
title: ADR-0034 Immutable review package and durable Apply
slug: adr-0034-immutable-review-package-and-durable-apply
status: accepted
tags: [adr, orchestration, blast-shield, recovery]
audience: [human, agent]
layer: orchestration
created: 2026-08-01
updated: 2026-08-01
adr_id: ADR-0034
related: [[mission-loop]], [[blast-shield]], [[permission-profile-engine]], [[ADR-0033-reviewed-run-atomic-apply]]
---

# ADR-0034: Immutable review package and durable Apply

## Status

Accepted

## Context

[[ADR-0033-reviewed-run-atomic-apply]] made the run the unit of review and
Apply, but its change set was rebuilt from live workspaces after approval.
Eligible workspaces could also be removed before their output was staged.
That left a gap between the bytes shown in Run Review and the bytes later
applied. Its in-process rollback did not settle an interrupted Apply.

## Decision

1. After every successful Orbit or Galaxy run, Pytxo prepares an immutable
   package under `.pytxo/data/reviews/<run-id>/`. Content-addressed blobs store
   the exact target bytes for additions and modifications; deletions have no
   target blob. The manifest is metadata. It records the base revision,
   preparation time, package digest, add/modify/delete kind, path, before and
   after SHA-256 digests, byte count, task/agent ownership, and any target blob
   digest. The manifest references blobs by digest rather than containing the
   target bytes.
2. Preparation includes untracked and binary additions from Git worktrees and
   copy-layer workspaces. It rejects traversal, `.git`, `.pytxo`, symlinks,
   sockets, FIFOs, devices, non-regular targets, and divergent ownership.
   Dependency-inherited identical output is deduplicated.
3. Apply reads only the prepared package. It never rebuilds an approved change
   set from an agent workspace. Review-eligible workspaces remain available
   until dependent waves finish and the package is durably written.
4. Apply validates the current preimage of every affected path. Unrelated dirty
   checkout files are allowed. Drift on an affected path changes the review to
   `stale`; the operator must prepare and review a new package.
5. Every Apply attempt writes a journal under
   `.pytxo/data/apply/<run-id>/<attempt-id>/`. Backups, directory ownership,
   operations, temporary paths, completion progress, and phase are persisted
   and file-synced before the corresponding mutation. Replacement uses a
   same-volume temporary file.
6. Journal reconciliation runs before another run or Apply in the execution
   domain. A proven rollback returns the package to `ready`; a proven commit
   records `applied`; source drift becomes `stale`; evidence that cannot prove
   either result becomes `recovery_required`.
7. The persisted lifecycle is `pending -> preparing -> ready -> applying ->
   applied`, with `stale`, `review_failed`, `recovery_required`, and
   `discarded` branches. An ordinary failure with a confirmed rollback remains
   retryable. Discard removes staged blobs and retained workspaces after
   confirmation while keeping manifest and audit history.
8. The v1.1 boundary is one execution domain and one repository root. Reviewed
   Apply is available only to Orbit and Galaxy. DeepSpace is non-flushable;
   Supernova writes directly to the host tree and has no reviewed Apply step.

## Consequences

Run Review and Apply now refer to the same bytes. Process crashes can be
reconciled from durable local evidence, and unrelated checkout work does not
invalidate a package.

The package, backups, and journal consume local disk until Apply or discard.
Ambiguous recovery stops further mutation and requires manual repair.

This is not a claim of power-loss ACID behavior, cross-filesystem durability,
kernel isolation, cross-root transactions, or partial-file acceptance.

## Links

- Supersedes: [[ADR-0033-reviewed-run-atomic-apply]]
- Related: [[mission-loop]], [[blast-shield]], [[permission-profile-engine]]
