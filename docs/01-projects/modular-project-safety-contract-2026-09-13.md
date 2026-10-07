---
title: Modular project safety contract
slug: modular-project-safety-contract-2026-09-13
status: draft
tags: [project, orchestration, multi-root, apply]
audience: [human, agent]
layer: orchestration
created: 2026-09-13
updated: 2026-09-13
related: [[modular-projects]], [[execution-domains]], [[commit-layer]], [[ADR-0011-modular-project-manifest]], [[ADR-0034-immutable-review-package-and-durable-apply]]
---

# Modular project safety contract

Attaching folders is project metadata, not a safe modular mission. Current code
can route CLI project-run tasks to labeled roots, create per-root agent
worktrees, namespace path claims by root, and record one run ID plus root IDs.
Desktop Flow rejects `task.root`; combined-candidate verification and reviewed
Apply reject multi-root plans. Therefore Pytxo has multi-root **execution
primitives**, not end-to-end modular projects.

## Required mission contract

A reviewed mission must freeze a root set before dispatch. Each root identity
contains its label, canonical path, repository/non-repository kind, writable or
read-only role, base revision or inventory digest, permission request, current
trust ceiling, execution domain, and isolation mechanism. Tasks own exactly one
writable root and root-relative claims. Cross-root dependencies are explicit
edges whose inputs and produced artifacts are recorded by digest; implicit
ambient reads from sibling roots are not authority.

Each agent receives only its assigned writable sandbox plus declared read-only
dependency snapshots. Effective permission is calculated per root and capped by
that folder's current trust. Race claims are keyed by project, root, and
relative path. Removing, relabeling, moving, or changing trust for a root makes
the mission contract stale.

## Candidate and verification

Preparation produces one immutable child package per writable root and an
umbrella manifest binding the project ID, mission/run ID, ordered root set,
child package digests, dependency edges, ownership, and verification recipe.
Verification commands declare their working root and which candidate roots are
mounted read-only or writable. Cross-project checks run against the complete
isolated candidate set, never a mixture of candidate and live checkout files.
The evidence ledger records command, root identities, inventories, effective
permission, enforcement receipt, result, and package digest.

Before review and again before Apply, Pytxo checks every root's base identity,
affected-path preimages, attached-root membership, trust ceiling, and child
package digest. Any drift invalidates the umbrella candidate. Review groups
changes and checks by root, shows dependency and ownership provenance, and
states the recovery class before authorization.

## Apply and recovery

Independent filesystems cannot honestly provide one atomic transaction. A
cross-root Apply must be an explicit coordinated, reversible commit protocol:
acquire mutation leases for all domains in stable order; reconcile old journals;
preflight every root before the first write; then apply journaled child packages.
If a later root fails, compensate already-applied roots from their durable
backups and re-verify all roots. The umbrella state is only `applied` after all
postconditions pass. It becomes `rolled_back` when every prior state is proven,
or `recovery_required` when any root cannot be proven restored. The UI must call
this coordinated and reversible, never cross-root atomic.

## Implementation order and proof

1. Define versioned root/umbrella candidate and evidence types; supersede the
   single-root ADR with a proposed cross-root recovery ADR rather than editing
   it.
2. Implement preparation and verification without Apply, including root-set,
   dependency, permission, and stale-change failures.
3. Add coordinated leases, journals, compensation, crash reconciliation, and
   injected-failure tests across two real temporary Git repositories.
4. Only then enable Desktop Flow dispatch and review for labeled roots.

Completion requires a native mission that modifies two attached repositories,
verifies the combined candidate, reviews both child packages, applies them, and
demonstrates rollback/recovery under an injected second-root failure. Folder
attachment and the existing CLI scheduling test do not satisfy that proof.

Back: [[MOC-home]] · [[modular-projects]] · [[commit-layer]]
