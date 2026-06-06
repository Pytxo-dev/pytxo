use crate::billing::ModelId;

pub trait TokenEstimator: Send + Sync {
    fn estimate_tokens(&self, text: &str, model: &ModelId) -> u64;
}

/// Default estimator: ~4 chars per token with optional per-model multiplier.
#[derive(Clone, Debug, Default)]
pub struct ByteHeuristicEstimator;

impl TokenEstimator for ByteHeuristicEstimator {
    fn estimate_tokens(&self, text: &str, model: &ModelId) -> u64 {
        let chars = text.chars().count() as u64;
        let base = chars.div_ceil(4);
        let mult = model.heuristic_multiplier();
        ((base as f64) * mult).ceil() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_zero() {
        let est = ByteHeuristicEstimator;
        assert_eq!(est.estimate_tokens("", &ModelId::default()), 0);
    }

    #[test]
    fn scales_with_length() {
        let est = ByteHeuristicEstimator;
        let model = ModelId::new("gpt-5.5-instant");
        assert!(est.estimate_tokens("a".repeat(100).as_str(), &model) >= 25);
    }
}
