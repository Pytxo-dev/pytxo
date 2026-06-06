# Closed-loop fidelity feedback

When an agent’s run fails validation (compiler, linter, tests), the scaffolder **raises AST context fidelity** without manual prompt edits.

## Behavior

1. Detect failure class and implicated symbols/files.
2. Bump fidelity for dependencies in the failure chain.
3. Re-issue context to the same or retried agent task.

## Goal

Self-correction with minimal token waste: start lean, expand only where evidence demands it.

Parent: [adaptive-semantic-scaffolding](/docs/adaptive-semantic-scaffolding).
