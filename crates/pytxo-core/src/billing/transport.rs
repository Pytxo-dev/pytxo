use std::process::Command;

use crate::billing::{BillingMode, CliAdapter, ModelRoute, ProviderId};
use crate::child_env::ChildLaunchEnv;

const BYOK_KEYS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "GOOGLE_API_KEY",
    "GEMINI_API_KEY",
    "AZURE_OPENAI_API_KEY",
];

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
        if !self.mode.is_ultra() {
            return;
        }

        for key in BYOK_KEYS {
            env.remove(key);
        }

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
            ProviderId::Openai => {
                env.set("OPENAI_BASE_URL", format!("{proxy}/openai"));
            }
            ProviderId::Google => {
                env.set("GOOGLE_API_BASE", format!("{proxy}/google"));
            }
            ProviderId::Anthropic | ProviderId::Generic => {}
        }
    }
}
