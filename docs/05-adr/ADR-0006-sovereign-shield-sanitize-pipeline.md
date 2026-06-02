# ADR-0006: Sovereign Shield sanitize pipeline

## Status

Accepted

## Context

Agent stdout/stderr and MCP tool responses may contain API keys, bearer tokens, and local filesystem paths. Pytxo persists logs in SQLite and surfaces them in the Reality Deck and MCP egress.

## Decision

- Add `pytxo-sanitize` with built-in regex rules (API keys, bearer tokens, home paths).
- Gate via `sanitize = true` in `pytxo.toml` (default on).
- Apply before `append_event` in `pytxo-orchestrate` and before MCP log payloads.

## Consequences

- Redaction is best-effort regex, not semantic secret detection.
- Users can extend rules in a future release via `.pytxo/sanitize-rules.toml`.
