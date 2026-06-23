//! Postgres run ledger for `/v1/runs/*` (Phase 40, ADR-0021).

use sqlx::PgPool;

use crate::state::{RunEndBody, RunStartBody, UsagePayload};

#[derive(Clone)]
pub struct RunStore {
    pool: PgPool,
}

impl RunStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Idempotent start: duplicate `run_id` returns OK without error.
    pub async fn start(&self, body: &RunStartBody) -> Result<(), sqlx::Error> {
        let idem = format!("{}:{}", body.domain_id, body.run_id);
        sqlx::query(
            r#"
            INSERT INTO runs (run_id, domain_id, idempotency_key, status)
            VALUES ($1, $2, $3, 'started')
            ON CONFLICT (run_id) DO NOTHING
            "#,
        )
        .bind(&body.run_id)
        .bind(&body.domain_id)
        .bind(&idem)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Idempotent end: upserts usage; safe to retry.
    pub async fn end(&self, body: &RunEndBody) -> Result<(), sqlx::Error> {
        let usage = serde_json::to_value(&body.usage).unwrap_or(serde_json::json!({}));
        sqlx::query(
            r#"
            INSERT INTO runs (run_id, domain_id, idempotency_key, status, ended_at, usage_json)
            VALUES ($1, $2, $3, 'completed', now(), $4)
            ON CONFLICT (run_id) DO UPDATE SET
                status = 'completed',
                ended_at = COALESCE(runs.ended_at, now()),
                usage_json = EXCLUDED.usage_json
            "#,
        )
        .bind(&body.run_id)
        .bind(&body.domain_id)
        .bind(format!("{}:{}", body.domain_id, body.run_id))
        .bind(usage)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_usage(&self, run_id: &str) -> Result<Option<UsagePayload>, sqlx::Error> {
        let row: Option<(serde_json::Value,)> =
            sqlx::query_as("SELECT usage_json FROM runs WHERE run_id = $1")
                .bind(run_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.and_then(|(v,)| serde_json::from_value(v).ok()))
    }
}

/// In-memory fallback when `DATABASE_URL` is unset (local dev only).
#[derive(Default)]
pub struct MemoryRunStore {
    inner: std::sync::Mutex<std::collections::HashMap<String, crate::state::RunRecord>>,
}

impl MemoryRunStore {
    pub fn start(&self, body: &RunStartBody) {
        let mut m = self.inner.lock().unwrap();
        m.entry(body.run_id.clone()).or_insert(crate::state::RunRecord {
            domain_id: body.domain_id.clone(),
            started: true,
            ended: false,
            usage: None,
        });
    }

    pub fn end(&self, body: &RunEndBody) {
        let mut m = self.inner.lock().unwrap();
        let entry = m.entry(body.run_id.clone()).or_insert(crate::state::RunRecord {
            domain_id: body.domain_id.clone(),
            started: false,
            ended: false,
            usage: None,
        });
        entry.ended = true;
        entry.usage = Some(body.usage.clone());
    }
}
