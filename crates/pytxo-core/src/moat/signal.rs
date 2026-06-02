use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::PytxoError;

/// Context fidelity tier ([[adaptive-semantic-scaffolding]]).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FidelityTier {
    #[default]
    Low,
    Medium,
    High,
}

impl FidelityTier {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScaffoldStats {
    pub original_bytes: usize,
    pub scaffolded_bytes: usize,
    pub language: Option<String>,
    pub token_reduction_pct: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScaffoldResult {
    pub path: String,
    pub content: String,
    pub stats: ScaffoldStats,
    /// True when no parser matched and raw bytes were returned unchanged.
    pub fallback_raw: bool,
}

/// Signal Core read-path contract: structural skeletons before context egress.
pub trait SignalCore: Send + Sync {
    fn scaffold(
        &self,
        path: &Path,
        source: &str,
        tier: FidelityTier,
    ) -> crate::Result<ScaffoldResult>;

    fn read_scaffolded(&self, path: &Path, tier: FidelityTier) -> crate::Result<ScaffoldResult> {
        let source = std::fs::read_to_string(path).map_err(|e| {
            PytxoError::Io(std::io::Error::new(
                e.kind(),
                format!("signal read {}: {e}", path.display()),
            ))
        })?;
        self.scaffold(path, &source, tier)
    }
}
