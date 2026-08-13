---
title: Pytxo v1.1 reviewed mission control
slug: pytxo-v1-1-trustworthy-mission-control
status: active
tags: [project, release, desktop, mission-loop]
audience: [human, agent]
layer: orchestration
created: 2026-07-31
updated: 2026-08-01
related: [[mission-loop]], [[product-vision]], [[pytxo-improvement-research]], [[desktop-visual-system]], [[ADR-0034-immutable-review-package-and-durable-apply]]
---

# Pytxo v1.1 reviewed mission control

## Release outcome

Pytxo v1.1 makes the prepared run package the boundary between agent execution
and repository mutation. The target is narrow: Orbit and Galaxy, one execution
domain, one repository root.

```mermaid
flowchart LR
  M["Mission"] --> P["Owned task plan"]
  P --> E["Isolated waves"]
  E --> K["Immutable review package"]
  K --> R["Run Review in Flow"]
  R --> A["Journaled Apply"]
  A --> O["Applied or recovered state"]
```

After every successful eligible run, Pytxo inventories Git worktrees or
copy-layer workspaces and writes `.pytxo/data/reviews/<run-id>/`. Target bytes
are stored as content-addressed blobs. The manifest records add, modify, or
delete kind, path, before and after digests, byte count, owner, blob digest,
base revision, preparation time, and package digest. Apply consumes the stored
blobs. It does not recompute the result from a workspace after approval.

The package builder includes untracked and binary additions. It rejects
traversal, `.git`, `.pytxo`, symlinks, special files, unsafe target ancestors,
and divergent ownership. Workspaces stay available until dependent waves
finish and the package is durable.

## Apply and recovery

Apply checks the current preimage of each affected path. Unrelated dirty files
are allowed. A changed affected path marks the review `stale`; Refresh prepares
a new package for review.

Each attempt records backups, operations, temporary paths, created-directory
ownership, progress, and phase in
`.pytxo/data/apply/<run-id>/<attempt-id>/`. The runner persists and file-syncs
that evidence before mutation. Reconciliation runs before another run or Apply
in the same execution domain.

A proven rollback returns the existing package to `ready`, so an ordinary
failure can be retried. A proven commit records `applied`. Source drift becomes
`stale`; evidence that cannot prove the tree is restored or committed becomes
`recovery_required` and blocks further mutation. Discard removes staged blobs
and retained workspaces after confirmation while preserving the manifest and
audit history.

DeepSpace remains non-flushable. Supernova continues to write directly to the
host tree and has no reviewed Apply step.

## Desktop and product evidence

Flow is the mission home. Compose, Active, History, and Run Review share one
surface, while old Runs links redirect into Flow. Run Review shows the package
digest, full base revision, exact additions/edits/deletions, ownership DAG,
permission profile, enforcement receipt, and Apply attempt history.

Desktop combines immediate Tauri mutation events with a durable change cursor.
Full snapshots remain for initial load, reconnect, cursor reset, and an
infrequent integrity check. Structural Focus is contextual. The legacy Deck is
absent from the normal production bundle and requires development flags.

The 52-second Remotion cut uses current Desktop captures and no burned
subtitles. The silent 1920x1080 master, poster, contact sheets, transcript, and
provisional SRT are complete. The SRT must be retimed against the approved
continuous narration before publication. The narrated master stays blocked
until approved voice, music, interface cues, and the Pixabay certificate exist
locally.

## Limits

v1.1 does not claim cross-root transactions, partial-file acceptance,
cross-filesystem durability, power-loss ACID behavior, or kernel isolation.
Orbit and Galaxy enforcement receipts continue to distinguish enforced,
advisory, unavailable, and bypassed controls.

The executable release checklist is in the root `PLAN.md`.
