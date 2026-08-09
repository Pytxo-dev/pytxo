---
title: First mission without pytxo.toml tasks
slug: first-mission
status: active
tags: [guides, mission, getting-started]
audience: [human]
layer: meta
created: 2026-07-27
updated: 2026-07-27
related: [[mission-loop]], [[cli-reference]], [[product-vision]]
---

# First mission without pytxo.toml tasks

You do not need to hand-write `[[task]]` rows to try Pytxo.

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
pytxo mission "Add input validation to the public API and update unit tests"
```

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

Open **Pytxo Desktop -> Flow -> History -> Run Review**. Run Review is the
reviewed-package Apply surface for Orbit and Galaxy. It shows the exact
prepared additions, modifications, and deletions. Before Apply, Pytxo checks
the current preimage of every affected path; unrelated dirty files are
allowed. A changed affected path marks the review stale.

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
