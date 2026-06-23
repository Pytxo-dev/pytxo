use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillingMode {
    #[default]
    Byok,
    Ultra,
}

impl BillingMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "byok" => Some(Self::Byok),
            "ultra" => Some(Self::Ultra),
            _ => None,
        }
    }

    pub fn is_ultra(self) -> bool {
        matches!(self, Self::Ultra)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingConfig {
    #[serde(default)]
    pub mode: BillingMode,
    #[serde(default = "default_proxy_url")]
    pub proxy_url: String,
    #[serde(default = "default_reserve_microcredits")]
    pub reserve_microcredits: i64,
    #[serde(default = "default_initial_balance")]
    pub initial_balance_microcredits: i64,
    /// When true (or unset under Ultra), POST run start/end to Pytxo Link.
    #[serde(default)]
    pub link_reconcile: Option<bool>,
}

fn default_proxy_url() -> String {
    "https://link.pytxo.com".to_string()
}

fn default_reserve_microcredits() -> i64 {
    500_000
}

fn default_initial_balance() -> i64 {
    10_000_000
}

impl BillingConfig {
    /// Effective Link reconcile flag: explicit `link_reconcile` or `true` when `mode = ultra`.
    pub fn link_reconcile_enabled(&self) -> bool {
        self.link_reconcile.unwrap_or_else(|| self.mode.is_ultra())
    }
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self {
            mode: BillingMode::Byok,
            proxy_url: default_proxy_url(),
            reserve_microcredits: default_reserve_microcredits(),
            initial_balance_microcredits: default_initial_balance(),
            link_reconcile: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ultra_defaults_link_reconcile_on() {
        let cfg = BillingConfig {
            mode: BillingMode::Ultra,
            ..BillingConfig::default()
        };
        assert!(cfg.link_reconcile_enabled());
    }

    #[test]
    fn byok_defaults_link_reconcile_off() {
        let cfg = BillingConfig::default();
        assert!(!cfg.link_reconcile_enabled());
    }

    #[test]
    fn explicit_false_disables_ultra_default() {
        let cfg = BillingConfig {
            mode: BillingMode::Ultra,
            link_reconcile: Some(false),
            ..BillingConfig::default()
        };
        assert!(!cfg.link_reconcile_enabled());
    }
}
