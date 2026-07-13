//! Ultra wallet balance from inference spend (Phase 52).

use serde::Serialize;
use sqlx::PgPool;

#[derive(Clone, Debug, Serialize)]
pub struct WalletBalanceResponse {
    pub balance_microcredits: i64,
    pub spent_microcredits: i64,
}

pub fn initial_microcredits() -> i64 {
    std::env::var("LINK_WALLET_INITIAL_MICROCREDITS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000_000)
}

pub async fn balance_for_user(
    pool: &PgPool,
    user_id: &str,
) -> Result<WalletBalanceResponse, sqlx::Error> {
    let spent: (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(cost_micro_usd), 0) FROM inference_usage WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .inspect_err(|e| tracing::warn!(?e, %user_id, "wallet balance query failed"))?;
    let initial = initial_microcredits();
    Ok(WalletBalanceResponse {
        balance_microcredits: initial.saturating_sub(spent.0),
        spent_microcredits: spent.0,
    })
}

pub fn balance_memory(spent_microcredits: i64) -> WalletBalanceResponse {
    let initial = initial_microcredits();
    WalletBalanceResponse {
        balance_microcredits: initial.saturating_sub(spent_microcredits),
        spent_microcredits,
    }
}
