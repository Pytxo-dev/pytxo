use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::entitlements::EntitlementStore;
use crate::jwt::JwksValidator;
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
    pub entitlements: EntitlementStore,
    pub db: Option<PgPool>,
    pub runs: RunLedger,
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
