# Asynchronous DAG flow engine

Parallel multi-agent edits risk **file-lock deadlocks** under a naive sequential scheduler. Pytxo uses an **asynchronous directed acyclic graph (DAG) flow engine**.

## Construction

Swarm proposals become a task DAG:

- **Vertices** — execution tasks
- **Edges** — file or semantic dependencies

Topological scheduling runs ready tasks concurrently when resources allow.

## Cycle detection

If agents deadlock (A waits on B, B waits on A):

1. Detect the cycle in the dependency graph.
2. Pause one agent (e.g. B).
3. Inject a **mocked intermediate state** into the other’s buffer so the dependency chain can complete.
4. Resume and reconcile real artifacts.

## Complexity

Scheduling cost scales with graph structure; document target workloads in benchmarks ([competitive-benchmarks](/docs/competitive-benchmarks)).

ADR: [ADR-0004-dag-scheduler-over-sequential-locks](/docs/adr-0004-dag-scheduler-over-sequential-locks).
