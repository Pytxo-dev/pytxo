# Local regex sanitization (Sovereign Shield)

Before files or `stdout`/`stderr` are packaged for agents or sent to [cloud](/docs/hybrid-execution), a **multi-threaded Rust scanner** sanitizes strings locally.

## Patterns (examples)

- API keys and bearer tokens
- Absolute home-directory paths
- Designated sensitive schema identifiers

## Policy

Matched regions are **zeroed or redacted** in the outbound buffer—fail-safe for enterprise defaults.

## Scope

Part of **Sovereign Shield** alongside [pytxo-link-signing](/docs/pytxo-link-signing). Never bypass for convenience in examples or tests committed to the repo.
