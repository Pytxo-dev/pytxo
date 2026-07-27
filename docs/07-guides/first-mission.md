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
3. Show predicted path overlaps and isolation mode
4. Ask `Approve mission? [Y/n]` (use `--yes` to skip)
5. Run agents in isolated changes
6. Run any suggested verify commands
7. Print a mission report and how to apply (flush) accepted work

Preview only:

```bash
pytxo mission "…" --json
```

## After the run

Review the report. Apply accepted changes when verifies passed:

```bash
pytxo hitl list
# or use Pytxo Desktop → Approvals / Run Review → Complete run
```

Orbit never auto-merges. Verification unlocks eligibility; you still approve the flush.

Back: [[mission-loop]] · [[MOC-home]]
