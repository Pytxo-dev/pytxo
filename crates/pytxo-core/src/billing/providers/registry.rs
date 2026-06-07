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
    if spec.api_key_env.is_empty() {
        return true;
    }
    std::env::var(spec.api_key_env)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub fn list_static_models(id: ProviderId) -> Vec<String> {
    get_provider(id)
        .map(|p| p.static_models.iter().map(|s| (*s).to_string()).collect())
        .unwrap_or_default()
}

/// Inject OpenAI-compatible BYOK env for generic child CLIs.
pub fn inject_byok_env(env: &mut ChildLaunchEnv, route: &crate::billing::ModelRoute) {
    if route.cli_adapter != CliAdapter::Generic {
        return;
    }
    let Some(spec) = get_provider(route.provider) else {
        return;
    };
    if let Some(base) = spec.openai_base_url {
        env.set("OPENAI_BASE_URL", base);
    }
    if !spec.api_key_env.is_empty() {
        if let Ok(key) = std::env::var(spec.api_key_env) {
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
        openai_base_url: Some("https://api.deepseek.com"),
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
        openai_base_url: None,
        models_url: None,
        openai_compatible: false,
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
        id: ProviderId::Ollama,
        display_name: "Ollama (local)",
        api_key_env: "",
        openai_base_url: Some("http://127.0.0.1:11434/v1"),
        models_url: Some("http://127.0.0.1:11434/api/tags"),
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
