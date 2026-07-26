use crate::billing::{CliAdapter, ProviderId};
use crate::child_env::ChildLaunchEnv;

#[derive(Clone, Debug)]
pub struct ProviderSpec {
    pub id: ProviderId,
    pub display_name: &'static str,
    pub api_key_env: &'static str,
    pub openai_base_url: Option<&'static str>,
    pub models_url: Option<&'static str>,
    pub openai_compatible: bool,
    pub static_models: &'static [&'static str],
}

/// Owned provider row for CLI/Desktop status (builtins + `providers.json`).
#[derive(Clone, Debug, serde::Serialize)]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub api_key_env: String,
    pub key_configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openai_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models_url: Option<String>,
    pub openai_compatible: bool,
    pub builtin: bool,
}

pub fn all_providers() -> &'static [ProviderSpec] {
    PROVIDERS
}

pub fn get_provider(id: ProviderId) -> Option<&'static ProviderSpec> {
    PROVIDERS.iter().find(|p| p.id == id)
}

pub fn all_byok_key_envs() -> Vec<&'static str> {
    PROVIDERS.iter().map(|p| p.api_key_env).collect()
}

pub fn key_configured(spec: &ProviderSpec) -> bool {
    key_env_configured(spec.api_key_env)
}

pub fn key_env_configured(api_key_env: &str) -> bool {
    if api_key_env.is_empty() {
        return true;
    }
    std::env::var(api_key_env)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub fn list_static_models(id: ProviderId) -> Vec<String> {
    get_provider(id)
        .map(|p| p.static_models.iter().map(|s| (*s).to_string()).collect())
        .unwrap_or_default()
}

/// Builtin + custom (`~/.pytxo/providers.json`) status rows for `pytxo providers` / Desktop.
pub fn list_provider_status() -> Vec<ProviderStatus> {
    let mut rows: Vec<ProviderStatus> = PROVIDERS
        .iter()
        .map(|p| ProviderStatus {
            id: p.id.as_str().to_string(),
            name: p.display_name.to_string(),
            api_key_env: p.api_key_env.to_string(),
            key_configured: key_configured(p),
            openai_base_url: p.openai_base_url.map(|s| s.to_string()),
            models_url: p.models_url.map(|s| s.to_string()),
            openai_compatible: p.openai_compatible,
            builtin: true,
        })
        .collect();

    for custom in crate::billing::providers::load_custom_providers() {
        if let Some(existing) = rows.iter_mut().find(|r| r.id == custom.id) {
            existing.name = custom.display_name.clone();
            existing.api_key_env = custom.api_key_env.clone();
            existing.key_configured = key_env_configured(&custom.api_key_env);
            existing.openai_base_url = custom.openai_base_url.clone();
            existing.models_url = custom.models_url.clone();
            existing.openai_compatible = custom.openai_compatible;
            existing.builtin = false;
            continue;
        }
        rows.push(ProviderStatus {
            id: custom.id.clone(),
            name: custom.display_name,
            api_key_env: custom.api_key_env.clone(),
            key_configured: key_env_configured(&custom.api_key_env),
            openai_base_url: custom.openai_base_url,
            models_url: custom.models_url,
            openai_compatible: custom.openai_compatible,
            builtin: false,
        });
    }
    rows
}

/// Resolve OpenAI-compat base URL for a route (registry, Azure endpoint env, or custom JSON).
pub fn resolve_openai_base_url(
    provider: ProviderId,
    provider_label: Option<&str>,
) -> Option<String> {
    if let Some(label) = provider_label {
        if let Some(custom) = crate::billing::providers::find_custom_provider(label) {
            if let Some(base) = custom.openai_base_url {
                return Some(base);
            }
        }
    }
    if provider == ProviderId::Azure {
        if let Ok(endpoint) = std::env::var("AZURE_OPENAI_ENDPOINT") {
            let trimmed = endpoint.trim().trim_end_matches('/');
            if !trimmed.is_empty() {
                // Accept either resource root or full .../openai/v1 style URL.
                if trimmed.contains("/openai") {
                    return Some(trimmed.to_string());
                }
                return Some(format!("{trimmed}/openai/v1"));
            }
        }
    }
    get_provider(provider).and_then(|p| p.openai_base_url.map(|s| s.to_string()))
}

/// Inject OpenAI-compatible BYOK env for generic child CLIs.
pub fn inject_byok_env(env: &mut ChildLaunchEnv, route: &crate::billing::ModelRoute) {
    if route.cli_adapter != CliAdapter::Generic {
        return;
    }

    let label = route.provider_label.as_deref();
    if let Some(base) = resolve_openai_base_url(route.provider, label) {
        env.set("OPENAI_BASE_URL", base);
    }

    let key_env = route
        .api_key_env
        .clone()
        .or_else(|| {
            label
                .and_then(crate::billing::providers::find_custom_provider)
                .map(|c| c.api_key_env)
        })
        .or_else(|| get_provider(route.provider).map(|p| p.api_key_env.to_string()))
        .unwrap_or_default();

    if !key_env.is_empty() {
        if let Ok(key) = std::env::var(&key_env) {
            if !key.trim().is_empty() {
                env.set("OPENAI_API_KEY", key);
            }
        }
    }
}

static PROVIDERS: &[ProviderSpec] = &[
    ProviderSpec {
        id: ProviderId::Anthropic,
        display_name: "Anthropic",
        api_key_env: "ANTHROPIC_API_KEY",
        openai_base_url: None,
        models_url: None,
        openai_compatible: false,
        static_models: &["claude-opus-4-8", "claude-sonnet-4-6", "claude-haiku-4-5"],
    },
    ProviderSpec {
        id: ProviderId::Openai,
        display_name: "OpenAI",
        api_key_env: "OPENAI_API_KEY",
        openai_base_url: Some("https://api.openai.com/v1"),
        models_url: Some("https://api.openai.com/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Google,
        display_name: "Google",
        api_key_env: "GOOGLE_API_KEY",
        openai_base_url: None,
        models_url: None,
        openai_compatible: false,
        static_models: &["gemini-2.5-pro", "gemini-2.5-flash"],
    },
    ProviderSpec {
        id: ProviderId::Deepseek,
        display_name: "DeepSeek",
        api_key_env: "DEEPSEEK_API_KEY",
        openai_base_url: Some("https://api.deepseek.com/v1"),
        models_url: Some("https://api.deepseek.com/models"),
        openai_compatible: true,
        static_models: &["deepseek-chat", "deepseek-reasoner"],
    },
    ProviderSpec {
        id: ProviderId::Mistral,
        display_name: "Mistral",
        api_key_env: "MISTRAL_API_KEY",
        openai_base_url: Some("https://api.mistral.ai/v1"),
        models_url: Some("https://api.mistral.ai/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Groq,
        display_name: "Groq",
        api_key_env: "GROQ_API_KEY",
        openai_base_url: Some("https://api.groq.com/openai/v1"),
        models_url: Some("https://api.groq.com/openai/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Openrouter,
        display_name: "OpenRouter",
        api_key_env: "OPENROUTER_API_KEY",
        openai_base_url: Some("https://openrouter.ai/api/v1"),
        models_url: Some("https://openrouter.ai/api/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Azure,
        display_name: "Azure OpenAI",
        api_key_env: "AZURE_OPENAI_API_KEY",
        // Base comes from AZURE_OPENAI_ENDPOINT at inject time.
        openai_base_url: None,
        models_url: None,
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Together,
        display_name: "Together",
        api_key_env: "TOGETHER_API_KEY",
        openai_base_url: Some("https://api.together.xyz/v1"),
        models_url: Some("https://api.together.xyz/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Fireworks,
        display_name: "Fireworks",
        api_key_env: "FIREWORKS_API_KEY",
        openai_base_url: Some("https://api.fireworks.ai/inference/v1"),
        models_url: Some("https://api.fireworks.ai/inference/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Cohere,
        display_name: "Cohere",
        api_key_env: "COHERE_API_KEY",
        openai_base_url: Some("https://api.cohere.com/compatibility/v1"),
        models_url: Some("https://api.cohere.com/compatibility/v1/models"),
        openai_compatible: true,
        static_models: &["command-r-plus", "command-r"],
    },
    ProviderSpec {
        id: ProviderId::Xai,
        display_name: "xAI",
        api_key_env: "XAI_API_KEY",
        openai_base_url: Some("https://api.x.ai/v1"),
        models_url: Some("https://api.x.ai/v1/models"),
        openai_compatible: true,
        static_models: &["grok-3", "grok-3-mini"],
    },
    ProviderSpec {
        id: ProviderId::Cerebras,
        display_name: "Cerebras",
        api_key_env: "CEREBRAS_API_KEY",
        openai_base_url: Some("https://api.cerebras.ai/v1"),
        models_url: Some("https://api.cerebras.ai/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Sambanova,
        display_name: "SambaNova",
        api_key_env: "SAMBANOVA_API_KEY",
        openai_base_url: Some("https://api.sambanova.ai/v1"),
        models_url: Some("https://api.sambanova.ai/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Hyperbolic,
        display_name: "Hyperbolic",
        api_key_env: "HYPERBOLIC_API_KEY",
        openai_base_url: Some("https://api.hyperbolic.xyz/v1"),
        models_url: Some("https://api.hyperbolic.xyz/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Perplexity,
        display_name: "Perplexity",
        api_key_env: "PERPLEXITY_API_KEY",
        openai_base_url: Some("https://api.perplexity.ai"),
        models_url: None,
        openai_compatible: true,
        static_models: &["sonar", "sonar-pro"],
    },
    ProviderSpec {
        id: ProviderId::Nvidia,
        display_name: "NVIDIA NIM",
        api_key_env: "NVIDIA_API_KEY",
        openai_base_url: Some("https://integrate.api.nvidia.com/v1"),
        models_url: Some("https://integrate.api.nvidia.com/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Ollama,
        display_name: "Ollama (local)",
        api_key_env: "",
        openai_base_url: Some("http://127.0.0.1:11434/v1"),
        models_url: Some("http://127.0.0.1:11434/api/tags"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Lmstudio,
        display_name: "LM Studio (local)",
        api_key_env: "",
        openai_base_url: Some("http://127.0.0.1:1234/v1"),
        models_url: Some("http://127.0.0.1:1234/v1/models"),
        openai_compatible: true,
        static_models: &[],
    },
    ProviderSpec {
        id: ProviderId::Generic,
        display_name: "Generic",
        api_key_env: "",
        openai_base_url: None,
        models_url: None,
        openai_compatible: false,
        static_models: &[],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builtins_present() {
        assert!(get_provider(ProviderId::Cerebras).is_some());
        assert!(get_provider(ProviderId::Lmstudio).is_some());
        assert!(get_provider(ProviderId::Cohere)
            .unwrap()
            .openai_compatible);
    }

    #[test]
    fn status_includes_builtins() {
        let rows = list_provider_status();
        assert!(rows.iter().any(|r| r.id == "openrouter"));
        assert!(rows.iter().any(|r| r.id == "cerebras"));
    }
}
