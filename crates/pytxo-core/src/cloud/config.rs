use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_sandbox_url")]
    pub sandbox_url: String,
    #[serde(default = "default_cache_url")]
    pub cache_url: String,
    #[serde(default = "default_true")]
    pub cache_enabled: bool,
    #[serde(default = "default_true")]
    pub fallback_local: bool,
}

fn default_sandbox_url() -> String {
    "https://cloud.pytxo.com/v1".to_string()
}

fn default_cache_url() -> String {
    "https://cloud.pytxo.com/v1".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sandbox_url: default_sandbox_url(),
            cache_url: default_cache_url(),
            cache_enabled: true,
            fallback_local: true,
        }
    }
}

impl CloudConfig {
    pub fn ping(&self) -> crate::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        if self.sandbox_url.trim().is_empty() {
            return Err(crate::PytxoError::Other("cloud.sandbox_url empty".into()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpHubConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub allowlist: Vec<String>,
}

impl Default for McpHubConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowlist: Vec::new(),
        }
    }
}
