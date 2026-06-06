---
title: ADR-0011 Modular project manifest and multi-root execution
slug: adr-0011-modular-project-manifest
status: accepted
tags: [adr, orchestration, workspace]
audience: [human, agent]
layer: orchestration
created: 2026-06-04
updated: 2026-06-04
adr_id: ADR-0011
related: [[modular-projects]], [[execution-domains]], [[ADR-0005-worktree-isolation-for-mvp]], [[ADR-0008-local-permission-profile-four-tiers]], [[race-shield]]
---

# ADR-0011: Modular project manifest and multi-root execution

## Status

Accepted

## Context

[[modular-projects]] describes a user-facing **project** that may attach **multiple path roots** (folders or repos) to one workspace, inspired by multi-root workspaces in tools like Antigravity. v1 of the control plane only models one [[execution-domains|execution domain]] per canonical `repo_root`. We need a stable manifest format and a runtime model that does not break the single-repo path or the per-domain WAL separation guaranteed by [[execution-domains]].

## Decision

### Manifest

1. A **project manifest** is a TOML file with a `[project]` table and one or more `[[roots]]` entries:

```toml
[project]
id = "acme-platform"
name = "Acme Platform"

[[roots]]
path = "/home/dev/acme-api"
label = "api"
primary = true

[[roots]]
path = "/home/dev/acme-web"
label = "web"

[[roots]]
path = "/home/dev/shared-protos"
label = "protos"
read_only = true
```

2. Manifests are discovered in this order: explicit `--project <file>`, then `~/.pytxo/projects/<id>.toml`, then `.pytxo/project.toml` under the cwd.
3. Exactly one root is `primary = true` (or the first root is treated as primary). The primary root supplies the default cwd and `pytxo.toml` resolution.
4. `read_only` roots may be read and scaffolded ([[signal-core]]) but never receive a Blast Shield flush.

### Runtime model (v1)

5. A project resolves to a **project-scoped execution domain**. `DomainId` for a project is derived from the canonical **primary root** (unchanged hashing), so existing per-repo telemetry keeps working.
6. Additional roots are registered on the domain as **path roots** with labels. Race Shield claims are keyed `(root_id, relative_path)` so two agents cannot collide across the union ([[race-shield]]).
7. `[[task]]` entries may add `root = "label"`; task paths resolve under that root. Tasks without a `root` use the primary root.
8. Worktrees are created **per root per agent** under each root's `worktree_dir`. Non-git roots are read-only and skip worktree creation.
9. WAL rows carry `project_id` and `root_id` columns so the Deck can filter by project and by folder without merging unrelated streams.

### Scope cuts (deferred to v2)

10. Cross-root `depends_on` is allowed only **within one run**; cross-run or cross-project DAG edges are out of scope.
11. A single wave scheduling agents across multiple roots simultaneously is allowed, but **cross-root conflict detection** beyond per-root path claims is deferred.
12. A dedicated project catalog DB is deferred to the hypervisor catalog work ([[execution-domains]] v2); v1 persists project metadata next to the primary root.

## Consequences

**Positive**

- Single-repo users are unaffected: no manifest means `repo_root` behaves exactly as today.
- The manifest is declarative and diffable; roots can be added or removed without touching `pytxo.toml` task definitions.
- Per-root claims and `root_id` telemetry preserve the [[execution-domains]] isolation contract.

**Negative**

- Two roots that are independent git repos produce independent worktree trees; a single logical "approve merge" may span multiple roots and must be presented per root in the Deck.
- Project-scoped `DomainId` reuse of the primary root means moving the primary root changes the domain id.

## Amendment (Phase 17 — runtime closure)

Phase 17 implemented the v1 runtime and resolves the questions previously open in [[modular-projects]]:

- **`task.root` is in the schema.** `TaskConfig.root: Option<String>` flows through `Task` → `ScheduledTask` → the runner, and is persisted as `agents.root_id` in the WAL (`insert_agent_with_root`). Tasks without `root` keep `root_id = NULL` and use the primary root.
- **Race claims are root-scoped.** The runner namespaces each claim with its root label via `root_scoped_claim(root, path)` before `try_claim_paths`, so identical relative paths under different roots never collide while within-root prefix semantics are preserved.
- **Catalog carries `project_id`.** `project run` upserts the project association into the hypervisor catalog (`Catalog::upsert_domain` with `project_id`) so the Deck can group domains by project.
- **Worktrees when roots are different repos:** one worktree per root per agent (decision 8). Submodule consolidation is explicitly **not** pursued; independent repos stay independent.
- **Non-git paths:** allowed **read-only** for assets and scaffolding only; a full Blast overlay on arbitrary directories is deferred to the overlay spike (Phase 22a).
- **Cross-root `depends_on`:** permitted within a single run via explicit edges across `root` labels; the unified single-`run_id` multi-root scheduler lands in Phase 20 (this ADR's decision 11).

## Supersedes

None. Extends [[ADR-0005-worktree-isolation-for-mvp]] and [[ADR-0008-local-permission-profile-four-tiers]].
