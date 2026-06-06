use serde::{Deserialize, Serialize};

use crate::billing::{CliAdapter, ModelId, ModelRoute, ProviderId};
use crate::moat::PermissionProfile;
use crate::TaskId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSpec {
    pub name: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub permission_profile: Option<PermissionProfile>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub cli_adapter: Option<String>,
    /// Per-agent Signal Core fidelity override ([[closed-loop-fidelity]]); still
    /// capped by the permission profile. Falls back to the global `signal_fidelity`.
    #[serde(default)]
    pub signal_fidelity: Option<crate::moat::FidelityTier>,
}

impl AgentSpec {
    pub fn model_route(&self) -> ModelRoute {
        let model = ModelId::new(
            self.model
                .clone()
                .unwrap_or_else(|| "claude-opus-4-8".to_string()),
        );
        let provider = self
            .cli_adapter
            .as_deref()
            .and_then(CliAdapter::parse)
            .map(|a| match a {
                CliAdapter::Antigravity => ProviderId::Google,
                CliAdapter::ClaudeCode => ProviderId::Anthropic,
                CliAdapter::Generic => model
                    .provider_hint()
                    .map(provider_from_hint)
                    .unwrap_or(ProviderId::Generic),
            })
            .unwrap_or_else(|| {
                model
                    .provider_hint()
                    .map(provider_from_hint)
                    .unwrap_or(ProviderId::Anthropic)
            });
        let cli_adapter = self
            .cli_adapter
            .as_deref()
            .and_then(CliAdapter::parse)
            .unwrap_or(CliAdapter::ClaudeCode);
        ModelRoute {
            model,
            provider,
            cli_adapter,
        }
    }
}

fn provider_from_hint(h: &str) -> ProviderId {
    match h {
        "openai" => ProviderId::Openai,
        "google" => ProviderId::Google,
        "anthropic" => ProviderId::Anthropic,
        _ => ProviderId::Generic,
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub agent: String,
    pub paths: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Modular project root label ([[ADR-0011-modular-project-manifest]]).
    #[serde(default)]
    pub root: Option<String>,
    /// Per-task Signal Core fidelity override ([[closed-loop-fidelity]]); takes
    /// precedence over the agent and global fidelity, still capped by permissions.
    #[serde(default)]
    pub signal_fidelity: Option<crate::moat::FidelityTier>,
}
