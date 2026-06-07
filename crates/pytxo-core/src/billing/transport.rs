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
            proxy_base_url: "https://link.pytxo.com/v1".to_string(),
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

            match route.cli_adapter {
                CliAdapter::ClaudeCode => {
                    env.set("ANTHROPIC_BASE_URL", format!("{proxy}/anthropic"));
                    env.set("PYTXO_CLI", "claude");
                }
                CliAdapter::Antigravity => {
                    env.set("AGY_ENDPOINT", format!("{proxy}/agy"));
                    env.set("PYTXO_CLI", "agy");
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
