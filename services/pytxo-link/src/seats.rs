//! Org seat counting (Phase 59).

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone, Debug, Deserialize)]
pub struct AdminSeatsPutBody {
    pub seats_total: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgSeatsResponse {
    pub org_id: String,
    pub seats_total: usize,
    pub seats_used: usize,
    pub seats_available: usize,
}

pub async fn count_used(pool: &PgPool, org_id: &str) -> Result<usize, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::bigint
        FROM entitlements
        WHERE org_id = $1
        "#,
    )
    .bind(org_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0.max(0) as usize)
}

pub async fn seats_total(pool: &PgPool, org_id: &str) -> Result<usize, sqlx::Error> {
    let row: Option<(i32,)> = sqlx::query_as(
        r#"
        SELECT seats_total
        FROM org_seats
        WHERE org_id = $1
        "#,
    )
    .bind(org_id)
    .fetch_optional(pool)
    .await?;
    Ok(row
        .map(|(n,)| n.max(0) as usize)
        .unwrap_or(default_seats_total()))
}

pub fn default_seats_total() -> usize {
    std::env::var("LINK_ORG_SEATS_DEFAULT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

pub async fn get(pool: &PgPool, org_id: &str) -> Result<OrgSeatsResponse, sqlx::Error> {
    let seats_total = seats_total(pool, org_id).await?;
    let seats_used = count_used(pool, org_id).await?;
    Ok(OrgSeatsResponse {
        org_id: org_id.to_string(),
        seats_total,
        seats_used,
        seats_available: seats_total.saturating_sub(seats_used),
    })
}

pub async fn upsert_total(
    pool: &PgPool,
    org_id: &str,
    seats_total: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO org_seats (org_id, seats_total, updated_at)
        VALUES ($1, $2, now())
        ON CONFLICT (org_id) DO UPDATE SET
            seats_total = EXCLUDED.seats_total,
            updated_at = now()
        "#,
    )
    .bind(org_id)
    .bind(seats_total)
    .execute(pool)
    .await?;
    Ok(())
}
