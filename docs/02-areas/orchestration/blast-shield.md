---
title: Blast Shield
slug: blast-shield
status: active
tags: [orchestration, filesystem, moat]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-08-01
related: [[sparse-overlay-fs]], [[ADR-0003-sparse-overlay-not-ram-cow]], [[ADR-0005-worktree-isolation-for-mvp]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[permission-profile-engine]], [[product-vision]]
---

# Blast Shield

Blast Shield keeps eligible agent writes outside the primary repository until
the operator reviews a prepared package. v1.1 supports this reviewed Apply
path for Orbit and Galaxy in one execution domain and one repository root.

## Shipping model

```text
Physical repository
        ↑ affected-path validation + journaled Apply
Immutable review package
        ↑ exact add / modify / delete bytes and ownership
Worktree or sparse copy-layer workspace
        ↑ agent writes and shell cwd
Headless PTY agent
```

The workspace backend is either a Git worktree or a sparse copy-layer. The
Windows label `projfs-sparse-copy-v2` is not a kernel ProjFS provider, and the
copy-layer is not an mmap CoW filesystem. [[sparse-overlay-fs]] records the
kernel FUSE/ProjFS direction separately.

For a successful Orbit or Galaxy run, Pytxo keeps dependent-wave workspaces
until it writes `.pytxo/data/reviews/<run-id>/`. The package includes exact
target bytes, add/modify/delete kind, before and after SHA-256 digests, byte
count, base revision, package digest, and task/agent ownership. Untracked and
binary additions use the same package format.

Preparation rejects path traversal, protected `.git` and `.pytxo` paths,
symlinks, sockets, FIFOs, devices, unsafe target ancestors, and divergent
ownership. Apply reads only this package; it never rebuilds approved bytes from
a live workspace.

## Apply boundary

Before mutation, Pytxo compares each affected path with its recorded preimage.
Unrelated dirty checkout files do not block Apply. Drift on an affected path
changes the review to `stale`, and Refresh prepares a new package.

Every attempt writes backups, operations, temporary paths, created-directory
ownership, progress, and phase under
`.pytxo/data/apply/<run-id>/<attempt-id>/`. Reconciliation runs before another
run or Apply in the execution domain. A proven rollback returns the package to
`ready`; a proven commit records `applied`; ambiguous evidence becomes
`recovery_required`.

This journal supports automatic process-crash recovery. It is not a
power-loss ACID or cross-filesystem durability guarantee.

## Permission profiles

- **Orbit:** isolated workspaces and reviewed Apply.
- **Galaxy:** isolated workspaces and reviewed Apply, plus HITL for high-risk
  actions.
- **DeepSpace:** non-flushable.
- **Supernova:** host-direct writes with no reviewed Apply step.

`IsolationMode` selects the workspace mechanism. It does not change the active
[[permission-profile-engine|permission profile]].

## Scope

Cross-root transactions, partial-file acceptance, and kernel-grade
filesystem/network isolation are outside v1.1. See
[[ADR-0034-immutable-review-package-and-durable-apply]].
