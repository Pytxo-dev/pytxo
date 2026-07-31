use std::process::Command;

use crate::billing::providers::{all_byok_key_envs, inject_byok_env};
use crate::billing::{BillingMode, CliAdapter, ModelRoute, ProviderId};
use crate::child_env::ChildLaunchEnv;

/// Managed Pytxo Ultra proxy env injection for child CLI processes.
#[derive(Clone, Debug)]
pub struct ManagedTransport {
    pub proxy_base_url: String,
    pub session_token_env: &'static str,
    pub mode: BillingMode,
}

impl Default for ManagedTransport {
    fn default() -> Self {
        Self {
            proxy_base_url: "https://proxy.pytxo.com".to_string(),
            session_token_env: "PYTXO_ULTRA_SESSION",
            mode: BillingMode::Byok,
        }
    }
}

impl ManagedTransport {
    pub fn apply(&self, command: &mut Command, route: &ModelRoute) {
        let mut env = ChildLaunchEnv::new();
        self.inject_into(&mut env, route);
        env.apply_command(command);
    }

    pub fn inject_into(&self, env: &mut ChildLaunchEnv, route: &ModelRoute) {
        if self.mode.is_ultra() {
            for key in all_byok_key_envs() {
                if !key.is_empty() {
                    env.remove(key);
                }
            }
            env.remove("GEMINI_API_KEY");

            let proxy = self.proxy_base_url.trim_end_matches('/');
            env.set("PYTXO_ULTRA", "1");
            env.set("PYTXO_MODEL", route.model.as_str());
            env.set("PYTXO_PROXY_URL", proxy);

            let cli_label = route.ade_id.as_deref().or(match route.cli_adapter {
                CliAdapter::ClaudeCode => Some("claude"),
                CliAdapter::Antigravity => Some("agy"),
                CliAdapter::Generic => None,
            });
            if let Some(label) = cli_label {
                env.set("PYTXO_CLI", label);
            }

            match route.cli_adapter {
                CliAdapter::ClaudeCode => {
                    env.set("ANTHROPIC_BASE_URL", format!("{proxy}/anthropic"));
                }
                CliAdapter::Antigravity => {
                    env.set("AGY_ENDPOINT", format!("{proxy}/agy"));
                }
                CliAdapter::Generic => {}
            }

            match route.provider {
                ProviderId::Openai | ProviderId::Azure => {
                    env.set("OPENAI_BASE_URL", format!("{proxy}/openai"));
                }
                ProviderId::Google => {
                    env.set("GOOGLE_API_BASE", format!("{proxy}/google"));
                }
                ProviderId::Deepseek => {
                    env.set("OPENAI_BASE_URL", format!("{proxy}/deepseek"));
                }
                ProviderId::Openrouter => {
                    env.set("OPENAI_BASE_URL", format!("{proxy}/openrouter"));
                }
                _ => {}
            }
            return;
        }

        inject_byok_env(env, route);
        if let Some(key_env) = &route.api_key_env {
            if let Ok(val) = std::env::var(key_env) {
                if !val.trim().is_empty() {
                    env.set(key_env.clone(), val);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::billing::{ModelId, ModelRoute};

    #[test]
    fn deepseek_byok_maps_one_selected_key_into_the_openai_compatible_contract() {
        const SELECTED: &str = "PYTXO_DEEPSEEK_CONTRACT_KEY";
        std::env::set_var(SELECTED, "contract-test-not-a-secret");

        let route = ModelRoute {
            model: ModelId::new("deepseek-chat"),
            provider: ProviderId::Deepseek,
            cli_adapter: CliAdapter::Generic,
            api_key_env: Some(SELECTED.into()),
            ade_id: Some("codex".into()),
            provider_label: Some("deepseek".into()),
        };
        let mut launch = ChildLaunchEnv::new();
        ManagedTransport::default().inject_into(&mut launch, &route);
        std::env::remove_var(SELECTED);

        assert_eq!(
            launch.vars().get("OPENAI_BASE_URL").map(String::as_str),
            Some("https://api.deepseek.com/v1")
        );
        assert_eq!(
            launch.vars().get("OPENAI_API_KEY").map(String::as_str),
            Some("contract-test-not-a-secret")
        );
        assert_eq!(
            launch.vars().get(SELECTED).map(String::as_str),
            Some("contract-test-not-a-secret")
        );
        assert!(!launch.vars().contains_key("OPENROUTER_API_KEY"));
        assert!(!launch.vars().contains_key("ANTHROPIC_API_KEY"));
    }
}
