//! Org audit log (Phase 49).

use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;

#[derive(Clone, Debug, Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub org_id: Option<String>,
    pub actor_id: String,
    pub action: String,
    pub detail: Option<Value>,
    pub created_at: String,
}

pub async fn append(
    pool: &PgPool,
    org_id: Option<&str>,
    actor_id: &str,
    action: &str,
    detail: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (org_id, actor_id, action, detail)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(org_id)
    .bind(actor_id)
    .bind(action)
    .bind(detail)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_for_org(pool: &PgPool, org_id: &str, limit: i64) -> Result<Vec<AuditEntry>, sqlx::Error> {
    let rows: Vec<(i64, Option<String>, String, String, Option<Value>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            r#"
            SELECT id, org_id, actor_id, action, detail, created_at
            FROM audit_log
            WHERE org_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|(id, org_id, actor_id, action, detail, created_at)| AuditEntry {
            id,
            org_id,
            actor_id,
            action,
            detail,
            created_at: created_at.to_rfc3339(),
        })
        .collect())
}
