---
title: Models catalog
---

# Models catalog

`pytxo models` lists and searches provider model IDs with a 24-hour local cache under `~/.pytxo/model-cache/`.

## Commands

```bash
pytxo models list                          # all cached / static lists
pytxo models list --provider deepseek      # one provider
pytxo models list --provider openrouter --refresh

pytxo models search deepseek               # fuzzy match id + name
pytxo models search claude --provider openrouter

pytxo models refresh --provider groq       # force cache bust
```

## Hypervisor Shell

```text
/models list --provider deepseek
/models search llama --provider openrouter
/models refresh --provider openai
```

List endpoints use provider HTTP APIs where available; Anthropic and Google use curated static lists. Ollama uses `GET /api/tags` on localhost.

## Requirements

- Network access for live fetches (except static/Ollama).
- Matching API key in the environment for authenticated providers (`pytxo providers` shows which keys are set).

## Examples

```bash
export DEEPSEEK_API_KEY=...
pytxo models search chat --provider deepseek

export OPENROUTER_API_KEY=...
pytxo models list --provider openrouter | head
```

See [Providers & BYOK](/docs/reference/providers-byok) for key setup.
