use std::path::Path;

use pytxo_core::{ArbitrageSample, ModelId, ScaffoldResult, TokenEstimator};

pub struct ArbitrageProfiler<'a> {
    estimator: &'a dyn TokenEstimator,
}

impl<'a> ArbitrageProfiler<'a> {
    pub fn new(estimator: &'a dyn TokenEstimator) -> Self {
        Self { estimator }
    }

    pub fn profile_file(
        &self,
        path: &Path,
        scaffold: &ScaffoldResult,
        model: &ModelId,
    ) -> ArbitrageSample {
        let raw_source = std::fs::read_to_string(path).unwrap_or_default();
        let raw_tokens = self.estimator.estimate_tokens(&raw_source, model);
        let sent_tokens = self.estimator.estimate_tokens(&scaffold.content, model);
        let saved_tokens = raw_tokens.saturating_sub(sent_tokens);
        let reduction_pct = if raw_tokens == 0 {
            0.0
        } else {
            (saved_tokens as f64 / raw_tokens as f64) * 100.0
        };

        ArbitrageSample {
            path: scaffold.path.clone(),
            original_bytes: scaffold.stats.original_bytes,
            scaffolded_bytes: scaffold.stats.scaffolded_bytes,
            raw_tokens_in: raw_tokens,
            sent_tokens_in: sent_tokens,
            saved_tokens,
            reduction_pct,
            fallback_raw: scaffold.fallback_raw,
        }
    }
}
