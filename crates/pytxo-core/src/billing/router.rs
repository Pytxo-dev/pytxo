use serde::{Deserialize, Serialize};

use crate::PytxoConfig;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn provider_hint(&self) -> Option<&str> {
        let id = self.0.to_ascii_lowercase();
        if id.contains("claude")
            || id.contains("opus")
            || id.contains("sonnet")
            || id.contains("haiku")
        {
            Some("anthropic")
        } else if id.contains("gpt") || id.contains("o1") || id.contains("o3") {
            Some("openai")
        } else if id.contains("gemini") {
            Some("google")
        } else if id.contains("deepseek") {
            Some("deepseek")
        } else if id.contains("mistral") || id.contains("mixtral") {
            Some("mistral")
        } else if id.contains("llama") || id.contains("groq") {
            Some("groq")
        } else if id.contains("grok") {
            Some("xai")
        } else if id.contains("command-r") {
            Some("cohere")
        } else if id.contains("qwen") {
            Some("together")
        } else {
            None
        }
    }

    pub fn heuristic_multiplier(&self) -> f64 {
        match self.provider_hint() {
            Some("anthropic") => 1.0,
            Some("openai") => 1.05,
            Some("google") => 0.95,
            Some("deepseek") => 0.85,
            Some("groq") => 0.8,
            _ => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    #[default]
    Anthropic,
    Openai,
    Google,
    Deepseek,
    Mistral,
    Groq,
    Openrouter,
    Azure,
    Together,
    Fireworks,
    Cohere,
    Xai,
    Ollama,
    Generic,
}

impl ProviderId {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "anthropic" => Some(Self::Anthropic),
            "openai" => Some(Self::Openai),
            "google" | "gemini" => Some(Self::Google),
            "deepseek" => Some(Self::Deepseek),
            "mistral" => Some(Self::Mistral),
            "groq" => Some(Self::Groq),
            "openrouter" | "open_router" => Some(Self::Openrouter),
            "azure" | "azure_openai" => Some(Self::Azure),
            "together" => Some(Self::Together),
            "fireworks" => Some(Self::Fireworks),
            "cohere" => Some(Self::Cohere),
            "xai" | "x.ai" => Some(Self::Xai),
            "ollama" => Some(Self::Ollama),
            "generic" => Some(Self::Generic),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::Openai => "openai",
            Self::Google => "google",
            Self::Deepseek => "deepseek",
            Self::Mistral => "mistral",
            Self::Groq => "groq",
            Self::Openrouter => "openrouter",
            Self::Azure => "azure",
            Self::Together => "together",
            Self::Fireworks => "fireworks",
            Self::Cohere => "cohere",
            Self::Xai => "xai",
            Self::Ollama => "ollama",
            Self::Generic => "generic",
        }
    }
}

pub fn provider_from_hint(h: &str) -> ProviderId {
    ProviderId::parse(h).unwrap_or(ProviderId::Generic)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliAdapter {
    #[default]
    Generic,
    ClaudeCode,
    Antigravity,
}

impl CliAdapter {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "generic" => Some(Self::Generic),
            "claude_code" | "claude" => Some(Self::ClaudeCode),
            "antigravity" | "agy" => Some(Self::Antigravity),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelRoute {
    pub model: ModelId,
    pub provider: ProviderId,
    pub cli_adapter: CliAdapter,
    pub api_key_env: Option<String>,
}

pub trait ModelRouter: Send + Sync {
    fn route(&self, agent_name: &str, cfg: &PytxoConfig) -> ModelRoute;
}

#[derive(Clone, Debug, Default)]
pub struct ConfigModelRouter;

impl ModelRouter for ConfigModelRouter {
    fn route(&self, agent_name: &str, cfg: &PytxoConfig) -> ModelRoute {
        if let Some(agent) = cfg.agent.iter().find(|a| a.name == agent_name) {
            return agent.model_route();
        }
        ModelRoute {
            model: ModelId::new("claude-opus-4-8"),
            provider: ProviderId::Anthropic,
            cli_adapter: CliAdapter::ClaudeCode,
            api_key_env: None,
        }
    }
}
