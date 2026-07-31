# Collision demo fixture

Tiny `pytxo.toml` that forces Race Shield waves: two tasks claim `src/shared.ts`, one task claims a disjoint path.

## Commands (from this directory)

```bash
# Initialize a throwaway git repo if this folder is not already one
git init
git add -A && git commit -m "collision demo fixture" --allow-empty

pytxo trust orbit
pytxo run --config pytxo.toml --dry-run
```

Inspect JSON: `conflicts` lists the overlapping pair; `waves` puts them in different stages while `alone` can share a wave with the first overlap winner.

Smoke run without agent CLIs:

```bash
pytxo run --config pytxo.toml --cmd "echo pytxo-collision-demo"
```

Each agent still gets its own worktree (Blast). The real checkout is not mutated by the echo command.

From the monorepo root:

```bash
pytxo run --config tooling/demos/collision/pytxo.toml --dry-run --repo tooling/demos/collision
```

(Use a git-initialized copy of this folder as `--repo`, or init git here first.)

## Expected dry-run shape

- `conflicts` includes `touch-shared-a` vs `touch-shared-b` on `src/shared.ts`
- At least **two** waves
- `touch-alone` can run alongside one of the shared tasks

Docs: [First three-agent run / collision demo](/docs/getting-started/first-three-agent-run).
