use serde::{Deserialize, Serialize};

use crate::{AgentId, DomainId, RunId, TaskId};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReservationId(pub String);

impl ReservationId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for ReservationId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct UsageKey {
    pub run_id: RunId,
    pub agent_id: AgentId,
    pub domain_id: DomainId,
    pub task_id: TaskId,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct TokenCounts {
    pub tokens_in: u64,
    pub tokens_out: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbitrageYield {
    pub raw_tokens_in: u64,
    pub sent_tokens_in: u64,
    pub saved_tokens: u64,
    pub reduction_pct: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbitrageSample {
    pub path: String,
    pub original_bytes: usize,
    pub scaffolded_bytes: usize,
    pub raw_tokens_in: u64,
    pub sent_tokens_in: u64,
    pub saved_tokens: u64,
    pub reduction_pct: f64,
    pub fallback_raw: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunUsageTotals {
    pub tokens_in_billed: u64,
    pub tokens_in_sent: u64,
    pub tokens_out: u64,
    pub saved_tokens: u64,
    pub cost_micro_usd: i64,
}
