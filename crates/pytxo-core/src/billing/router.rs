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
        if id.contains("claude") || id.contains("opus") || id.contains("sonnet") {
            Some("anthropic")
        } else if id.contains("gpt") {
            Some("openai")
        } else if id.contains("gemini") {
            Some("google")
        } else {
            None
        }
    }

    pub fn heuristic_multiplier(&self) -> f64 {
        match self.provider_hint() {
            Some("anthropic") => 1.0,
            Some("openai") => 1.05,
            Some("google") => 0.95,
            _ => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    #[default]
    Anthropic,
    Openai,
    Google,
    Generic,
}

impl ProviderId {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "anthropic" => Some(Self::Anthropic),
            "openai" => Some(Self::Openai),
            "google" => Some(Self::Google),
            "generic" => Some(Self::Generic),
            _ => None,
        }
    }
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
        }
    }
}
