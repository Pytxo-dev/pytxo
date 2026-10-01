use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::commerce::CommerceStore;
use crate::dodo::DodoProductCatalog;
use crate::entitlements::EntitlementStore;
use crate::jwt::JwksValidator;
use crate::paddle::PaddlePriceCatalog;
use crate::routing_admission::RoutingAdmissionConfig;
use crate::runs::{MemoryRunStore, RunStore};

#[derive(Clone)]
pub enum RunLedger {
    Postgres(RunStore),
    Memory(Arc<MemoryRunStore>),
}

impl RunLedger {
    pub fn postgres(pool: PgPool) -> Self {
        Self::Postgres(RunStore::new(pool))
    }

    pub fn memory() -> Self {
        Self::Memory(Arc::new(MemoryRunStore::default()))
    }
}

#[derive(Clone)]
pub struct AppState {
    pub api_key: Option<String>,
    pub admin_key: Option<String>,
    pub require_auth: bool,
    pub jwks: Option<Arc<JwksValidator>>,
    /// Separate, opt-in session audience for the sponsored routing token path.
    pub routing_token_audience: Option<String>,
    /// Stops new Desktop connections, workspace grants and evaluation token
    /// issuance without disabling grant reads, revocation or account recovery.
    pub routing_grant_experiment: bool,
    pub routing_admission: Option<RoutingAdmissionConfig>,
    pub entitlements: EntitlementStore,
    pub db: Option<PgPool>,
    pub runs: RunLedger,
    pub commerce: Option<CommerceStore>,
    pub paddle_webhook_secret: Option<String>,
    pub paddle_prices: Option<PaddlePriceCatalog>,
    pub dodo_webhook_secret: Option<String>,
    pub dodo_products: Option<DodoProductCatalog>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRecord {
    pub domain_id: String,
    pub started: bool,
    pub ended: bool,
    pub usage: Option<UsagePayload>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RunStartBody {
    pub domain_id: String,
    pub run_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RunEndBody {
    pub domain_id: String,
    pub run_id: String,
    pub usage: UsagePayload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsagePayload {
    pub tokens_in_billed: u64,
    pub tokens_in_sent: u64,
    pub tokens_out: u64,
    pub saved_tokens: u64,
    pub cost_micro_usd: i64,
}
