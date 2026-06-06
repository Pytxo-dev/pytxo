---
title: Local regex sanitization (Sovereign Shield)
slug: regex-sanitization
status: active
tags: [security, compliance]
audience: [human, agent]
layer: security
created: 2026-06-02
updated: 2026-06-02
related: [[pytxo-link-signing]]
---

# Local regex sanitization (Sovereign Shield)

Before files or `stdout`/`stderr` are packaged for agents or sent to [[hybrid-execution|cloud]], a **multi-threaded Rust scanner** sanitizes strings locally.

## Patterns (examples)

- API keys and bearer tokens
- Absolute home-directory paths
- Designated sensitive schema identifiers

## Policy

Matched regions are **zeroed or redacted** in the outbound buffer—fail-safe for enterprise defaults.

## Scope

Part of **Sovereign Shield** alongside [[pytxo-link-signing]]. Never bypass for convenience in examples or tests committed to the repo.
