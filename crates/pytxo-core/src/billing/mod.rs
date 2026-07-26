//! Pytxo Ultra — metered billing, arbitrage metering, and managed model routing.

mod config;
mod estimator;
mod link_reconciler;
mod meter;
mod pricing;
pub mod providers;
mod router;
mod transport;
mod types;
mod wallet;

pub use config::{BillingConfig, BillingMode};
pub use estimator::{
    default_token_estimator, ByteHeuristicEstimator, TiktokenEstimator, TokenEstimator,
};
pub use link_reconciler::HttpBillingReconciler;
pub use meter::{UsageMeter, UsageSource};
pub use pricing::StaticPriceTable;
pub use providers::{
    all_byok_key_envs, all_providers, find_custom_provider, get_provider, inject_byok_env,
    key_configured, key_env_configured, list_provider_status, list_static_models,
    load_custom_providers, providers_json_path, resolve_openai_base_url, CustomProviderSpec,
    ProviderSpec, ProviderStatus,
};
pub use router::{
    provider_from_hint, CliAdapter, ConfigModelRouter, ModelId, ModelRoute, ModelRouter, ProviderId,
};
pub use transport::ManagedTransport;
pub use types::{
    ArbitrageSample, ArbitrageYield, ReservationId, RunUsageTotals, TokenCounts, UsageKey,
};
pub use wallet::{BillingReconciler, LocalHybridBilling, NoopBillingReconciler, TokenWallet};
