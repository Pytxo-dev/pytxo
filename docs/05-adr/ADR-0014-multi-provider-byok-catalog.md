---
title: ADR-0014 Multi-provider BYOK catalog
slug: ADR-0014-multi-provider-byok-catalog
status: accepted
tags: [adr, billing, byok, providers]
audience: [human, agent]
layer: meta
created: 2026-06-07
related: [[ADR-0009-ultra-managed-metering]], [[ADR-0012-hypervisor-shell-default-ux]]
---

# ADR-0014: Multi-provider BYOK catalog

## Status

Accepted (v0.3.0)

## Context

BYOK today assumes a handful of env vars (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, …) and three `CliAdapter` values. Users want DeepSeek, Groq, Mistral, OpenRouter, and other providers with model discovery — without Pytxo becoming a key vault.

## Decision

1. **`ProviderRegistry`** in `pytxo-core` — spec per provider (api_key_env, base_url, models endpoint).
2. **Expand `ProviderId`** — DeepSeek, Mistral, Groq, OpenRouter, Azure, Together, Fireworks, Cohere, xAI, Ollama, plus existing four.
3. **`pytxo-catalog` crate** — fetch/cache/search models per provider; OpenRouter as unified gateway.
4. **Keys stay in OS env** — `pytxo providers` shows set/missing only; optional `api_key_env` on `[[agent]]`.
5. **Fix routing** — per-agent `model` / `provider` / `cli_adapter` from `pytxo.toml` applied at runtime via `RunContext.agent_specs`.
6. **Generic CLI env injection** — OpenAI-compatible base URLs for BYOK generic adapters.

## Consequences

- New crate `pytxo-catalog`; CLI `pytxo models` and shell `/models`.
- Ultra proxy paths for new providers remain stubbed; BYOK direct is primary.
- No secrets in `pytxo.toml` values.

Back: [[index]]
