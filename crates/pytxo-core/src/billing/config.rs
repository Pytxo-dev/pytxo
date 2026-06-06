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
    /// When true with `mode = ultra`, call [`HttpBillingReconciler`] (HTTP stub until Link ships).
    #[serde(default)]
    pub link_reconcile: bool,
}

fn default_proxy_url() -> String {
    "https://link.pytxo.com/v1".to_string()
}

fn default_reserve_microcredits() -> i64 {
    500_000
}

fn default_initial_balance() -> i64 {
    10_000_000
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self {
            mode: BillingMode::Byok,
            proxy_url: default_proxy_url(),
            reserve_microcredits: default_reserve_microcredits(),
            initial_balance_microcredits: default_initial_balance(),
            link_reconcile: false,
        }
    }
}
