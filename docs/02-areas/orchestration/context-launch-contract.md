---
title: Context launch contract
slug: context-launch-contract
status: active
tags: [orchestration, context, signal-core, agent-interface]
audience: [human, agent]
layer: orchestration
created: 2026-06-05
updated: 2026-06-05
related: [[signal-core]], [[closed-loop-fidelity]], [[ADR-0011-modular-project-manifest]], [[permission-profile-engine]]
---

# Context launch contract

This is the contract between Pytxo and the agent CLI it spawns: **how materialized
code context is delivered at launch**, and what the agent is responsible for. It is
the local answer to "how does Pytxo handle context" — there is no separate chat or
LLM-memory layer in the control plane, only **materialized code context**.

## What Pytxo pushes at launch

Before spawning an agent, the runner calls `prepare_agent_context`
([context.rs](../../../crates/pytxo-runner/src/context.rs)). For an agent whose task
has `paths` (unioned with its `[[agent]].paths`), it:

1. Globs each path under the run's `repo_root`.
2. Scaffolds every matched file through [[signal-core]] at the effective
   [[#Fidelity|fidelity tier]].
3. Writes scaffolded copies under `.pytxo/data/context/{run_id}/{agent_id}/`,
   preserving the relative path of each source file.
4. Writes a `manifest.json` index in that directory.
5. Exposes the directory to the child via environment variables.

If Signal Core is disabled, or the task has no paths, **no context directory is
created** and the environment variables below are absent.

## Environment variables

Set by [`ChildLaunchEnv::with_context_dir`](../../../crates/pytxo-core/src/child_env.rs)
on both PTY and subprocess backends:

| Variable | Meaning |
|----------|---------|
| `PYTXO_CONTEXT_DIR` | Absolute path to this agent's context directory. |
| `PYTXO_SIGNAL_CORE` | `1` when Signal Core context was materialized. |

An agent CLI integration should, at startup, check `PYTXO_SIGNAL_CORE`; when set,
prefer reading from `PYTXO_CONTEXT_DIR` (and its `manifest.json`) over re-reading
raw repository files, so the token savings are realized.

## `manifest.json` schema

A JSON array; one object per materialized file:

```json
[
  {
    "source": "src/lib.rs",
    "scaffolded": "src/lib.rs",
    "token_reduction_pct": 58.3,
    "fallback_raw": false,
    "fidelity": "low",
    "root_id": "api",
    "bytes_scaffolded": 412
  }
]
```

| Field | Type | Meaning |
|-------|------|---------|
| `source` | string | Repo-relative source path (forward slashes). |
| `scaffolded` | string | Relative path of the scaffolded copy inside `PYTXO_CONTEXT_DIR`. |
| `token_reduction_pct` | number | Estimated input-token reduction vs. raw bytes. |
| `fallback_raw` | bool | `true` when the language was unsupported and raw bytes were copied. |
| `fidelity` | string | Tier used to scaffold this entry: `low`, `medium`, or `high`. |
| `root_id` | string? | Modular project root label; omitted for single-root runs. |
| `bytes_scaffolded` | number | Byte length of the scaffolded copy. |

The schema is append-only; consumers must ignore unknown fields. On a failed run,
the closed loop ([[closed-loop-fidelity]]) rewrites the manifest with only the
**implicated** files at `fidelity: "high"`.

## Fidelity

The effective tier is `min(config signal_fidelity, PermissionEngine::max_fidelity)`
([[permission-profile-engine]]): `DeepSpace` caps to `Low`. **Low/Medium** emit AST
skeletons; **High** copies full file bytes. On a failed run, the closed loop
re-materializes at High and re-spawns ([[closed-loop-fidelity]]).

## Agent responsibilities

- Treat `PYTXO_CONTEXT_DIR` as **read-only** pre-fetched context; edits still happen
  in the worktree cwd.
- Use `manifest.json` to map a scaffolded file back to its `source` path.
- For files not in the manifest, read from the worktree directly or via the MCP
  `pytxo_read` / `pytxo_read_scaffolded` tools, which scaffold on demand using the
  same Signal Core.

## Verification

The CI smoke step asserts the contract by running an agent command that prints
`PYTXO_CONTEXT_DIR` and reads its `manifest.json`; see
[tooling/scripts/smoke.ps1](../../../tooling/scripts/smoke.ps1).

Back: [[signal-core]] · [[closed-loop-fidelity]] · [[MOC-home]]
