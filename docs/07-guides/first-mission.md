---
title: First mission without pytxo.toml tasks
slug: first-mission
status: active
tags: [guides, mission, getting-started]
audience: [human]
layer: meta
created: 2026-07-27
updated: 2026-09-05
related: [[mission-loop]], [[cli-reference]], [[product-vision]]
---

# First mission without pytxo.toml tasks

You do not need to hand-write `[[task]]` rows to try Pytxo.
One installed coding agent CLI is enough: isolate its work, inspect recorded
task checks, and review the exact prepared changes before Apply. Multiple
instances and vendors are optional.

## Prerequisites

- `pytxo` on PATH (`npm i -g pytxo` or a release binary)
- At least one coding agent CLI on PATH (Claude Code, Codex, or OpenCode)
- A git repo you trust at Orbit (or higher)

```bash
cd my-project
pytxo trust orbit
pytxo agents
```

## Run a mission

```bash
pytxo mission "Add input validation in src/api.ts and regression tests in test/api.test.ts"
```

Replace the paths with existing paths in your repository. The default local
planner infers ownership from explicit paths and repository structure; a vague
request without safe ownership fails with an actionable error. Automatic skill
routing and screenshot-based analysis are not part of this planner.

In Desktop, use **New work** and choose your ready CLI. Inspect prompts,
paths, dependencies, and per-task verification commands before Run. If no
checks are detected, define `verify` commands on your `pytxo.toml` tasks and
rebuild. Passing per-task checks does not establish combined-candidate success.
Pytxo reruns the commands on the frozen combined source before preparing a
version 3 package for Apply. Run Review displays that separate receipt.

The combined snapshot excludes the paths listed in its receipt, including Git
metadata. Checks requiring Git history fail rather than inspect the primary
repository. Dependencies and build outputs excluded from the snapshot are not
attested; use verification commands that work from the included source.

Pytxo will:

1. Detect available agents
2. Propose tasks, paths, and execution stages (waves)
3. Show predicted path overlaps and isolation mode in plain language (Blast / Race)
4. Ask `Approve mission? [Y/n]` (use `--yes` to skip)
5. Run agents in isolated copies of the repo
6. Run any suggested verify commands
7. Persist the run contract and prepare an immutable review package

Preview only:

```bash
pytxo mission "…" --json
```

## After the run

Open **Pytxo Desktop → Work**, select the run, and open **Run Review**. New
missions begin from **New work**; completed outcomes remain under
**History**. Run Review is the reviewed-package Apply surface for Orbit and Galaxy. It shows the exact
prepared additions, modifications, and deletions. Before Apply, Pytxo checks
the current preimage of every affected path and the included source inventory
bound to verification. Changes to any included file mark the review stale,
including new operator files. Explicitly refresh to preserve those files and
rerun checks against the updated base before reviewing and applying again.

The `hitl` command manages Galaxy approval requests only:

```bash
pytxo hitl list
pytxo hitl approve <id>
```

These commands do not initiate Apply. Orbit has no Galaxy HITL step; Orbit
users review and Apply the prepared package in Run Review.

Reviewed Apply covers Orbit and Galaxy in one execution domain and one
repository root. DeepSpace is non-flushable. Supernova writes directly to the
host tree.

Back: [[mission-loop]] · [[MOC-home]]
