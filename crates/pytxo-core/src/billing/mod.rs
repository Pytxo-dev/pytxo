//! Pytxo Ultra — metered billing, arbitrage metering, and managed model routing.

mod config;
mod estimator;
mod link_reconciler;
mod meter;
mod pricing;
mod router;
mod transport;
mod types;
mod wallet;

pub use config::{BillingConfig, BillingMode};
pub use estimator::{ByteHeuristicEstimator, TokenEstimator};
pub use link_reconciler::HttpBillingReconciler;
pub use meter::{UsageMeter, UsageSource};
pub use pricing::StaticPriceTable;
pub use router::{CliAdapter, ConfigModelRouter, ModelId, ModelRoute, ModelRouter, ProviderId};
pub use transport::ManagedTransport;
pub use types::{
    ArbitrageSample, ArbitrageYield, ReservationId, RunUsageTotals, TokenCounts, UsageKey,
};
pub use wallet::{BillingReconciler, LocalHybridBilling, NoopBillingReconciler, TokenWallet};
