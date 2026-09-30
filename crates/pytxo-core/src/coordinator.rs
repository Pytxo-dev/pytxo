use serde::{Deserialize, Serialize};

use crate::{ExecutionBackend, IsolationMode, ModelId, ProviderId};

pub const DEFAULT_COORDINATOR_PROVIDER: &str = "deepseek";
pub const DEFAULT_COORDINATOR_MODEL: &str = "deepseek-flash";

/// How the advisory coordinator reaches a model. This is separate from the
/// execution backend used for coding-agent processes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinatorTransport {
    /// Call the selected provider directly. Local providers such as Ollama and
    /// LM Studio use this mode without an API key.
    #[default]
    Direct,
    /// Call the optional Pytxo managed-inference proxy.
    Managed,
}

/// Model profile for planning, route advice, and diagnosis. Enabling model
/// egress remains an explicit caller decision; this value only selects a route.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorConfig {
    #[serde(default = "default_coordinator_provider")]
    pub provider: String,
    #[serde(default = "default_coordinator_model")]
    pub model: String,
    #[serde(default)]
    pub transport: CoordinatorTransport,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            provider: default_coordinator_provider(),
            model: default_coordinator_model(),
            transport: CoordinatorTransport::Direct,
        }
    }
}

impl CoordinatorConfig {
    pub fn profile(&self) -> Result<CoordinatorProfile, String> {
        let provider_label = self.provider.trim();
        if provider_label.is_empty() {
            return Err("coordinator provider cannot be empty".into());
        }
        let model = self.model.trim();
        if model.is_empty() {
            return Err("coordinator model cannot be empty".into());
        }

        let provider = ProviderId::parse(provider_label)
            .or_else(|| {
                crate::billing::providers::find_custom_provider(provider_label)
                    .map(|_| ProviderId::Generic)
            })
            .ok_or_else(|| format!("unknown coordinator provider: {provider_label}"))?;

        if self.transport == CoordinatorTransport::Managed
            && matches!(
                provider,
                ProviderId::Ollama | ProviderId::Lmstudio | ProviderId::Generic
            )
        {
            return Err(format!(
                "coordinator provider {provider_label} is not available through managed transport"
            ));
        }

        Ok(CoordinatorProfile {
            provider,
            provider_label: provider_label.to_string(),
            model: ModelId::new(model),
            transport: self.transport,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoordinatorProfile {
    pub provider: ProviderId,
    pub provider_label: String,
    pub model: ModelId,
    pub transport: CoordinatorTransport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinatorIntent {
    Plan,
    Route,
    Diagnose,
}

/// A provider/model pair visible in a deterministic catalog snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteModel {
    pub provider: String,
    pub model: String,
}

/// Advisory route output. Permission profile, approval, verification, mission
/// state, and Apply authority are intentionally absent: a model cannot propose
/// or widen those Core-owned controls through this contract.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteProposal {
    pub model: RouteModel,
    pub harness: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub plugins: Vec<String>,
    pub execution_backend: ExecutionBackend,
    pub isolation: IsolationMode,
}

/// Eligible catalog supplied by Core after deterministic discovery, policy,
/// and entitlement filtering. The coordinator never invents availability.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RoutingCatalogSnapshot {
    pub models: Vec<RouteModel>,
    pub harnesses: Vec<String>,
    pub skills: Vec<String>,
    pub tools: Vec<String>,
    pub plugins: Vec<String>,
    pub execution_backends: Vec<ExecutionBackend>,
    pub isolation_modes: Vec<IsolationMode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteConstraintViolation {
    ModelUnavailable(RouteModel),
    HarnessUnavailable(String),
    SkillUnavailable(String),
    ToolUnavailable(String),
    PluginUnavailable(String),
    ExecutionBackendUnavailable(ExecutionBackend),
    IsolationUnavailable(IsolationMode),
}

/// Accept advisory output only when every selected component is still present
/// in the Core-owned eligible snapshot. Mission, permission, verification,
/// approval, and Apply checks remain separate Core gates.
pub fn validate_route_proposal(
    proposal: RouteProposal,
    catalog: &RoutingCatalogSnapshot,
) -> Result<RouteProposal, Vec<RouteConstraintViolation>> {
    let mut violations = Vec::new();
    if !catalog.models.contains(&proposal.model) {
        violations.push(RouteConstraintViolation::ModelUnavailable(
            proposal.model.clone(),
        ));
    }
    if !catalog.harnesses.contains(&proposal.harness) {
        violations.push(RouteConstraintViolation::HarnessUnavailable(
            proposal.harness.clone(),
        ));
    }
    for skill in &proposal.skills {
        if !catalog.skills.contains(skill) {
            violations.push(RouteConstraintViolation::SkillUnavailable(skill.clone()));
        }
    }
    for tool in &proposal.tools {
        if !catalog.tools.contains(tool) {
            violations.push(RouteConstraintViolation::ToolUnavailable(tool.clone()));
        }
    }
    for plugin in &proposal.plugins {
        if !catalog.plugins.contains(plugin) {
            violations.push(RouteConstraintViolation::PluginUnavailable(plugin.clone()));
        }
    }
    if !catalog
        .execution_backends
        .contains(&proposal.execution_backend)
    {
        violations.push(RouteConstraintViolation::ExecutionBackendUnavailable(
            proposal.execution_backend,
        ));
    }
    if !catalog.isolation_modes.contains(&proposal.isolation) {
        violations.push(RouteConstraintViolation::IsolationUnavailable(
            proposal.isolation,
        ));
    }

    if violations.is_empty() {
        Ok(proposal)
    } else {
        Err(violations)
    }
}

fn default_coordinator_provider() -> String {
    DEFAULT_COORDINATOR_PROVIDER.to_string()
}

fn default_coordinator_model() -> String {
    DEFAULT_COORDINATOR_MODEL.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_is_current_deepseek_flash() {
        let profile = CoordinatorConfig::default().profile().unwrap();
        assert_eq!(profile.provider, ProviderId::Deepseek);
        assert_eq!(profile.model.as_str(), "deepseek-flash");
        assert_eq!(profile.transport, CoordinatorTransport::Direct);
    }

    #[test]
    fn local_provider_is_direct_and_replaceable() {
        let config = CoordinatorConfig {
            provider: "ollama".into(),
            model: "qwen3:8b".into(),
            transport: CoordinatorTransport::Direct,
        };
        let profile = config.profile().unwrap();
        assert_eq!(profile.provider, ProviderId::Ollama);
        assert_eq!(profile.model.as_str(), "qwen3:8b");
    }

    #[test]
    fn managed_transport_rejects_local_provider() {
        let error = CoordinatorConfig {
            provider: "lmstudio".into(),
            model: "local-model".into(),
            transport: CoordinatorTransport::Managed,
        }
        .profile()
        .unwrap_err();
        assert!(error.contains("not available through managed transport"));
    }

    #[test]
    fn route_validation_fails_closed_on_invented_components() {
        let catalog = RoutingCatalogSnapshot {
            models: vec![RouteModel {
                provider: "deepseek".into(),
                model: "deepseek-flash".into(),
            }],
            harnesses: vec!["codex".into()],
            skills: vec!["rust".into()],
            tools: vec!["pytxo_read".into()],
            plugins: vec![],
            execution_backends: vec![ExecutionBackend::Pty],
            isolation_modes: vec![IsolationMode::Worktree],
        };
        let proposal = RouteProposal {
            model: catalog.models[0].clone(),
            harness: "invented-harness".into(),
            skills: vec!["rust".into()],
            tools: vec!["unknown-tool".into()],
            plugins: vec![],
            execution_backend: ExecutionBackend::Pty,
            isolation: IsolationMode::Worktree,
        };

        let violations = validate_route_proposal(proposal, &catalog).unwrap_err();
        assert_eq!(violations.len(), 2);
        assert!(matches!(
            &violations[0],
            RouteConstraintViolation::HarnessUnavailable(name) if name == "invented-harness"
        ));
    }
}
