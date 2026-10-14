---
title: ADR-0042 Landing gate and receipts
slug: ADR-0042-landing-gate-and-receipts
adr_id: ADR-0042
status: accepted
tags: [adr, land, receipts, commit-layer, git]
audience: [human, agent]
layer: orchestration
created: 2026-10-10
updated: 2026-10-10
related: [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0036-effect-contract-commit-boundary]], [[commit-layer]], [[2026-10-10-pytxo-v1-3-design]], [[pytxo-business-strategy-2026-10-10]]
---

# ADR-0042: Landing gate and receipts

## Status

Accepted for 1.3 (Matt, 2026-10-10: "go all in" on the v1.3 design).

## Context

Branch-per-agent workspaces (Orca, Conductor, Superset, vendor apps) are free
and widespread. Their output is N branches whose individual checks pass while
the combination can break. Pytxo's reviewed Apply ([[ADR-0034-immutable-review-package-and-durable-apply]])
only covered work that Pytxo itself ran, and the CLI could not Review or Apply
at all. The product needed one landing boundary for agent output from any
tool, and a record of each decision that can leave the machine safely.

## Decision

1. **`pytxo land` is a separate crate (`pytxo-land`) built on Git, not on the
   run store.** Inputs are commits. The gate combines them with
   `git merge-tree --write-tree` and `git commit-tree` (Git ≥ 2.38), falling
   back to merges in a Pytxo-owned worktree. Every Git command in owned
   worktrees runs with `core.hooksPath` pointed at a non-existent directory
   and signing off. Worktrees exist only when checks need files on disk.
2. **Checks run on the combined commit**, optionally on each input alone.
   Symbol-level overlap from Signal Core (`symbol_spans`) is reported and only
   blocks with `--strict`.
3. **Shadow mode is the default.** `--apply` moves the target only by
   `git update-ref <ref> <new> <expected-old>` or by `git merge --ff-only` in
   the worktree that has the target checked out, after confirming its `HEAD`
   is still the checked base. Git's own atomic ref update and fast-forward
   refusal are the stale guards; Pytxo adds no second locking scheme.
4. **Receipts** (`pytxo.com/receipt/v1`) are in-toto Statement v1 documents
   stored in `<git-common-dir>/pytxo/receipts/`. They are written for every
   landing decision and, best effort, for every committed reviewed Apply.
   `content_digest` is SHA-256 over the canonical predicate; receipts are
   unsigned in 1.3.
5. **Sharing is a separate, redacted payload** (`pytxo.receipt.share/v1`):
   counts, decision, check tallies, agent labels, version and digest. Commands,
   refs, output, file contents and repository names never leave; paths only
   with an explicit flag. Link stores shared payloads after strict schema
   validation (`deny_unknown_fields`, character whitelists).

## Consequences

- One landing vocabulary (ready, landed, blocked, stale) across CLI, CI and
  Desktop; the repository Apply journal remains the recovery authority for
  Pytxo runs.
- Landed history contains Pytxo-made merge commits, unsigned. Repositories
  requiring signed commits must squash or sign after landing.
- Inputs are merged in argument order; another order can produce another
  result when files overlap.
- Gate cost is dominated by Git process start-up (measured by Pybench).
- Follow-ups, not part of this decision: DSSE-signed receipts with a per-install
  key; a GitHub App that runs the gate across open pull requests; mapping
  receipts into the effect-contract evidence model of
  [[ADR-0036-effect-contract-commit-boundary]].

Back: [[MOC-home]] · [[commit-layer]]
