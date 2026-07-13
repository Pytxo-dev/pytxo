//! Proxy-reported inference usage (Phase 41).

use serde::Deserialize;
use sqlx::PgPool;

#[derive(Clone, Debug, Deserialize)]
pub struct InferenceUsageBody {
    pub domain_id: Option<String>,
    pub run_id: Option<String>,
    pub provider: String,
    pub model: Option<String>,
    pub tokens_in: u64,
    pub tokens_out: u64,
    #[serde(default)]
    pub cost_micro_usd: i64,
}

pub async fn record(
    pool: &PgPool,
    user_id: &str,
    body: &InferenceUsageBody,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO inference_usage (user_id, domain_id, run_id, provider, model, tokens_in, tokens_out, cost_micro_usd)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(user_id)
    .bind(&body.domain_id)
    .bind(&body.run_id)
    .bind(&body.provider)
    .bind(&body.model)
    .bind(body.tokens_in as i64)
    .bind(body.tokens_out as i64)
    .bind(body.cost_micro_usd)
    .execute(pool)
    .await?;
    Ok(())
}
