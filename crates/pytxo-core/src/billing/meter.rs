use crate::billing::{ArbitrageSample, ModelId, RunUsageTotals, TokenCounts, UsageKey};
use crate::{Result, RunId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageSource {
    Estimate,
    Provider,
    Reconcile,
}

impl UsageSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Estimate => "estimate",
            Self::Provider => "provider",
            Self::Reconcile => "reconcile",
        }
    }
}

pub trait UsageMeter: Send + Sync {
    fn record_context_arbitrage(&self, key: &UsageKey, samples: &[ArbitrageSample]) -> Result<()>;

    fn record_provider_usage(
        &self,
        key: &UsageKey,
        actual: &TokenCounts,
        model: &ModelId,
        cost_micro_usd: i64,
        source: UsageSource,
    ) -> Result<()>;

    fn run_totals(&self, run_id: &RunId) -> Result<RunUsageTotals>;
}
