# ADR-0004: DAG scheduler over sequential locks

## Status

Accepted

## Context

Multi-agent swarms on the same repo create file dependency cycles and deadlocks under naive global file locks.

## Decision

Schedule work with an **asynchronous DAG flow engine**: topological execution, cycle detection, and controlled mock-state injection to break dependency cycles.

Do not rely on a single sequential lock queue for parallel swarms.

## Consequences

**Positive**

- Higher parallelism with explicit dependency semantics.

**Negative**

- Mock injection requires reconciliation and test coverage.
- Operators need visibility into DAG state (Reality Deck / WAL).

## Links

- [dag-flow-engine](/docs/dag-flow-engine)
