---
title: Providers & BYOK
---

# Providers & BYOK

Pytxo v0.3.0 ships a **multi-provider registry** for bring-your-own-key (BYOK) runs. Use direct provider APIs, a single **OpenRouter** key, or local **Ollama**, without storing secrets in `pytxo.toml`.

## Check configured providers

```bash
pytxo providers              # registry + whether each API key env is set (boolean only)
```

## Environment variables

| Provider | API key env | OpenAI-compat base (generic CLI) |
|----------|-------------|----------------------------------|
| Anthropic | `ANTHROPIC_API_KEY` | — (native Claude adapters) |
| OpenAI | `OPENAI_API_KEY` | default OpenAI |
| Google | `GOOGLE_API_KEY` | — |
| DeepSeek | `DEEPSEEK_API_KEY` | `https://api.deepseek.com` |
| Groq | `GROQ_API_KEY` | `https://api.groq.com/openai/v1` |
| Mistral | `MISTRAL_API_KEY` | Mistral API |
| OpenRouter | `OPENROUTER_API_KEY` | `https://openrouter.ai/api/v1` |
| Together | `TOGETHER_API_KEY` | Together API |
| Fireworks | `FIREWORKS_API_KEY` | Fireworks API |
| Cohere | `COHERE_API_KEY` | Cohere API |
| xAI | `XAI_API_KEY` | xAI API |
| Ollama | _(none)_ | `http://127.0.0.1:11434/v1` |
| Azure | `AZURE_OPENAI_API_KEY` | your deployment URL |

Ultra managed mode strips registered BYOK keys from child environments.

## `pytxo.toml` agent routing

Per-agent model and provider override the default router:

```toml
[[agent]]
name = "builder"
model = "deepseek-chat"
provider = "deepseek"
cli_adapter = "generic"
api_key_env = "DEEPSEEK_API_KEY"   # optional override
```

OpenRouter gateway (one key, many models):

```toml
[[agent]]
name = "router"
model = "anthropic/claude-3.5-sonnet"
provider = "openrouter"
cli_adapter = "generic"
```

```bash
export OPENROUTER_API_KEY=sk-or-...
pytxo models search claude --provider openrouter
pytxo run --agents 1 --cmd "your-openai-compat-cli"
```

Native ADE CLIs (Claude Code, Antigravity, etc.) keep their own env contracts; generic `--cmd` shells benefit most from OpenAI-compat injection.

See also: [Models CLI](/docs/reference/models), [`pytxo.toml`](/docs/reference/pytxo-toml).
