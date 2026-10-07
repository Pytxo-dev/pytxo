use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Core,
    Pro,
    Max,
    Ultra,
}

impl Tier {
    pub fn max_agents(self) -> usize {
        match self {
            Self::Core => 3,
            Self::Pro => 64,
            Self::Max => 128,
            Self::Ultra => 256,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Pro => "pro",
            Self::Max => "max",
            Self::Ultra => "ultra",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "pro" => Self::Pro,
            "max" => Self::Max,
            "ultra" => Self::Ultra,
            _ => Self::Core,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntitlementRecord {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clerk_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    pub tier: Tier,
    pub max_agents: usize,
    pub cloud_enabled: bool,
}

#[derive(Clone)]
pub enum EntitlementStore {
    Memory(MemoryStore),
    Postgres(PgPool),
}

#[derive(Clone, Default)]
pub struct MemoryStore {
    inner: Arc<Mutex<MemoryEntitlementState>>,
}

#[derive(Default)]
struct MemoryEntitlementState {
    records: HashMap<String, EntitlementRecord>,
    grants: HashMap<(String, String), EntitlementGrant>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntitlementGrant {
    pub source: String,
    pub grant_id: String,
    pub user_id: String,
    pub tier: Tier,
    pub max_agents: usize,
    pub cloud_enabled: bool,
    pub active: bool,
    pub valid_until: Option<DateTime<Utc>>,
}

impl EntitlementGrant {
    fn active_at(&self, now: DateTime<Utc>) -> bool {
        self.active && self.valid_until.is_none_or(|valid_until| valid_until > now)
    }
}

impl EntitlementStore {
    pub fn memory() -> Self {
        Self::Memory(MemoryStore::default())
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self::Postgres(pool)
    }

    pub async fn get(&self, user_id: &str) -> EntitlementRecord {
        match self {
            Self::Memory(m) => m.get(user_id),
            Self::Postgres(pool) => fetch_postgres(pool, user_id)
                .await
                .unwrap_or_else(|| default_record(user_id)),
        }
    }

    pub async fn upsert(&self, record: EntitlementRecord) -> Result<(), sqlx::Error> {
        match self {
            Self::Memory(m) => {
                m.upsert(record);
                Ok(())
            }
            Self::Postgres(pool) => upsert_postgres(pool, &record).await,
        }
    }

    #[cfg(test)]
    pub async fn apply_grant(&self, grant: EntitlementGrant) -> Result<(), sqlx::Error> {
        match self {
            Self::Memory(store) => {
                store.apply_grant(grant);
                Ok(())
            }
            Self::Postgres(pool) => {
                let mut transaction = pool.begin().await?;
                upsert_grant_in_transaction(&mut transaction, &grant).await?;
                recompute_entitlement_in_transaction(&mut transaction, &grant.user_id).await?;
                transaction.commit().await?;
                Ok(())
            }
        }
    }
}

impl MemoryStore {
    pub fn get(&self, user_id: &str) -> EntitlementRecord {
        let mut state = self.inner.lock().unwrap();
        recompute_memory_entitlement(&mut state, user_id);
        state
            .records
            .get(user_id)
            .cloned()
            .unwrap_or_else(|| default_record(user_id))
    }

    pub fn upsert(&self, record: EntitlementRecord) {
        let grant = EntitlementGrant {
            source: "admin".into(),
            grant_id: record.user_id.clone(),
            user_id: record.user_id.clone(),
            tier: record.tier,
            max_agents: record.max_agents,
            cloud_enabled: record.cloud_enabled,
            active: true,
            valid_until: None,
        };
        let user_id = grant.user_id.clone();
        let mut state = self.inner.lock().unwrap();
        state.records.insert(record.user_id.clone(), record);
        state
            .grants
            .insert((grant.source.clone(), grant.grant_id.clone()), grant);
        recompute_memory_entitlement(&mut state, &user_id);
    }

    #[cfg(test)]
    pub fn apply_grant(&self, grant: EntitlementGrant) {
        let mut state = self.inner.lock().unwrap();
        state
            .records
            .entry(grant.user_id.clone())
            .or_insert_with(|| default_record(&grant.user_id));
        let user_id = grant.user_id.clone();
        state
            .grants
            .insert((grant.source.clone(), grant.grant_id.clone()), grant);
        recompute_memory_entitlement(&mut state, &user_id);
    }
}

fn recompute_memory_entitlement(state: &mut MemoryEntitlementState, user_id: &str) {
    let now = Utc::now();
    let selected = state
        .grants
        .values()
        .filter(|grant| grant.user_id == user_id && grant.active_at(now))
        .max_by_key(|grant| (grant.source == "admin", tier_rank(grant.tier)))
        .cloned();
    let record = state
        .records
        .entry(user_id.to_string())
        .or_insert_with(|| default_record(user_id));
    if let Some(grant) = selected {
        record.tier = grant.tier;
        record.max_agents = grant.max_agents;
        record.cloud_enabled = grant.cloud_enabled;
    } else {
        record.tier = Tier::Core;
        record.max_agents = Tier::Core.max_agents();
        record.cloud_enabled = false;
    }
}

fn tier_rank(tier: Tier) -> u8 {
    match tier {
        Tier::Core => 0,
        Tier::Pro => 1,
        Tier::Max => 2,
        Tier::Ultra => 3,
    }
}

fn default_record(user_id: &str) -> EntitlementRecord {
    EntitlementRecord {
        user_id: user_id.to_string(),
        clerk_user_id: None,
        org_id: None,
        tier: Tier::Core,
        max_agents: Tier::Core.max_agents(),
        cloud_enabled: false,
    }
}

async fn fetch_postgres(pool: &PgPool, user_id: &str) -> Option<EntitlementRecord> {
    sqlx::query_as::<_, EntitlementRow>(
        r#"
        SELECT entitlements.user_id, entitlements.clerk_user_id, entitlements.org_id,
               COALESCE(selected.tier, 'core') AS tier,
               COALESCE(selected.max_agents, 3) AS max_agents,
               COALESCE(selected.cloud_enabled, FALSE) AS cloud_enabled
        FROM entitlements
        LEFT JOIN LATERAL (
            SELECT tier, max_agents, cloud_enabled
            FROM entitlement_grants
            WHERE entitlement_grants.user_id = entitlements.user_id
              AND active = TRUE
              AND (valid_until IS NULL OR valid_until > now())
            ORDER BY CASE WHEN source = 'admin' THEN 1 ELSE 0 END DESC,
            CASE tier
                WHEN 'ultra' THEN 3
                WHEN 'max' THEN 2
                WHEN 'pro' THEN 1
                ELSE 0
            END DESC,
            updated_at DESC
            LIMIT 1
        ) AS selected ON TRUE
        WHERE entitlements.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(|r| r.into_record())
}

async fn upsert_postgres(pool: &PgPool, record: &EntitlementRecord) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO entitlements (user_id, clerk_user_id, org_id, tier, max_agents, cloud_enabled, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        ON CONFLICT (user_id) DO UPDATE SET
            clerk_user_id = EXCLUDED.clerk_user_id,
            org_id = EXCLUDED.org_id,
            updated_at = now()
        "#,
    )
    .bind(&record.user_id)
    .bind(&record.clerk_user_id)
    .bind(&record.org_id)
    .bind(record.tier.as_str())
    .bind(record.max_agents as i32)
    .bind(record.cloud_enabled)
    .execute(&mut *transaction)
    .await?;

    let grant = EntitlementGrant {
        source: "admin".into(),
        grant_id: record.user_id.clone(),
        user_id: record.user_id.clone(),
        tier: record.tier,
        max_agents: record.max_agents,
        cloud_enabled: record.cloud_enabled,
        active: true,
        valid_until: None,
    };
    upsert_grant_in_transaction(&mut transaction, &grant).await?;
    recompute_entitlement_in_transaction(&mut transaction, &record.user_id).await?;
    transaction.commit().await?;
    Ok(())
}

pub(crate) async fn upsert_grant_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    grant: &EntitlementGrant,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO entitlement_grants
            (source, grant_id, user_id, tier, max_agents, cloud_enabled, active, valid_until, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
        ON CONFLICT (source, grant_id) DO UPDATE SET
            user_id = EXCLUDED.user_id,
            tier = EXCLUDED.tier,
            max_agents = EXCLUDED.max_agents,
            cloud_enabled = EXCLUDED.cloud_enabled,
            active = EXCLUDED.active,
            valid_until = EXCLUDED.valid_until,
            updated_at = now()
        "#,
    )
    .bind(&grant.source)
    .bind(&grant.grant_id)
    .bind(&grant.user_id)
    .bind(grant.tier.as_str())
    .bind(grant.max_agents as i32)
    .bind(grant.cloud_enabled)
    .bind(grant.active)
    .bind(grant.valid_until)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Serialize grant projection for one principal. Provider subscriptions have
/// independent row locks, so without this guard two concurrent providers could
/// each project a snapshot that does not yet include the other's committed
/// grant and leave the materialized entitlement row stale.
pub(crate) async fn lock_entitlement_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO entitlements (user_id, tier, max_agents, cloud_enabled, updated_at)
        VALUES ($1, 'core', 3, FALSE, now())
        ON CONFLICT (user_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;
    sqlx::query("SELECT user_id FROM entitlements WHERE user_id = $1 FOR UPDATE")
        .bind(user_id)
        .fetch_one(&mut **transaction)
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct GrantProjectionRow {
    tier: String,
    max_agents: i32,
    cloud_enabled: bool,
}

pub(crate) async fn recompute_entitlement_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    let selected = sqlx::query_as::<_, GrantProjectionRow>(
        r#"
        SELECT tier, max_agents, cloud_enabled
        FROM entitlement_grants
        WHERE user_id = $1
          AND active = TRUE
          AND (valid_until IS NULL OR valid_until > now())
        ORDER BY CASE WHEN source = 'admin' THEN 1 ELSE 0 END DESC,
        CASE tier
            WHEN 'ultra' THEN 3
            WHEN 'max' THEN 2
            WHEN 'pro' THEN 1
            ELSE 0
        END DESC,
        updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&mut **transaction)
    .await?;

    let (tier, max_agents, cloud_enabled) = selected
        .map(|row| (Tier::parse(&row.tier), row.max_agents, row.cloud_enabled))
        .unwrap_or((Tier::Core, Tier::Core.max_agents() as i32, false));
    sqlx::query(
        r#"
        INSERT INTO entitlements (user_id, tier, max_agents, cloud_enabled, updated_at)
        VALUES ($1, $2, $3, $4, now())
        ON CONFLICT (user_id) DO UPDATE SET
            tier = EXCLUDED.tier,
            max_agents = EXCLUDED.max_agents,
            cloud_enabled = EXCLUDED.cloud_enabled,
            updated_at = now()
        "#,
    )
    .bind(user_id)
    .bind(tier.as_str())
    .bind(max_agents)
    .bind(cloud_enabled)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct EntitlementRow {
    user_id: String,
    clerk_user_id: Option<String>,
    org_id: Option<String>,
    tier: String,
    max_agents: i32,
    cloud_enabled: bool,
}

impl EntitlementRow {
    fn into_record(self) -> EntitlementRecord {
        let tier = Tier::parse(&self.tier);
        EntitlementRecord {
            user_id: self.user_id,
            clerk_user_id: self.clerk_user_id,
            org_id: self.org_id,
            tier,
            max_agents: self.max_agents as usize,
            cloud_enabled: self.cloud_enabled,
        }
    }
}

#[derive(Serialize)]
pub struct EntitlementStatusResponse {
    pub tier: String,
    pub max_agents: usize,
    pub cloud_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
}

impl From<EntitlementRecord> for EntitlementStatusResponse {
    fn from(r: EntitlementRecord) -> Self {
        Self {
            tier: r.tier.as_str().to_string(),
            max_agents: r.max_agents,
            cloud_enabled: r.cloud_enabled,
            org_id: r.org_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AdminUpsertBody {
    pub tier: String,
    #[serde(default)]
    pub clerk_user_id: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub max_agents: Option<usize>,
    #[serde(default)]
    pub cloud_enabled: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgPolicyResponse {
    pub org_id: String,
    pub default_permission_profile: Option<String>,
    pub shared_trusted_domains: Vec<String>,
}

#[derive(sqlx::FromRow)]
struct OrgPolicyRow {
    org_id: String,
    default_permission_profile: Option<String>,
    shared_trusted_domains: serde_json::Value,
}

pub async fn get_org_policy(pool: &PgPool, org_id: &str) -> Option<OrgPolicyResponse> {
    let row = sqlx::query_as::<_, OrgPolicyRow>(
        r#"
        SELECT org_id, default_permission_profile, shared_trusted_domains
        FROM org_policies WHERE org_id = $1
        "#,
    )
    .bind(org_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    let domains: Vec<String> =
        serde_json::from_value(row.shared_trusted_domains).unwrap_or_default();
    Some(OrgPolicyResponse {
        org_id: row.org_id,
        default_permission_profile: row.default_permission_profile,
        shared_trusted_domains: domains,
    })
}

#[derive(Debug, Deserialize)]
pub struct OrgPolicyPutBody {
    pub default_permission_profile: Option<String>,
    #[serde(default)]
    pub shared_trusted_domains: Option<Vec<String>>,
}

pub async fn upsert_org_policy(
    pool: &PgPool,
    org_id: &str,
    body: &OrgPolicyPutBody,
) -> Result<(), sqlx::Error> {
    let domains = body
        .shared_trusted_domains
        .as_ref()
        .map(|d| serde_json::to_value(d).unwrap_or(serde_json::json!([])))
        .unwrap_or_else(|| serde_json::json!([]));
    sqlx::query(
        r#"
        INSERT INTO org_policies (org_id, default_permission_profile, shared_trusted_domains, updated_at)
        VALUES ($1, $2, $3, now())
        ON CONFLICT (org_id) DO UPDATE SET
            default_permission_profile = COALESCE(EXCLUDED.default_permission_profile, org_policies.default_permission_profile),
            shared_trusted_domains = CASE
                WHEN $3::jsonb = '[]'::jsonb THEN org_policies.shared_trusted_domains
                ELSE EXCLUDED.shared_trusted_domains
            END,
            updated_at = now()
        "#,
    )
    .bind(org_id)
    .bind(&body.default_permission_profile)
    .bind(domains)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_agent_caps() {
        assert_eq!(Tier::Core.max_agents(), 3);
        assert_eq!(Tier::Pro.max_agents(), 64);
        assert!(Tier::Max.max_agents() > Tier::Pro.max_agents());
    }

    #[tokio::test]
    async fn memory_defaults_to_core() {
        let store = EntitlementStore::memory();
        let ent = store.get("user-1").await;
        assert_eq!(ent.tier, Tier::Core);
        assert_eq!(ent.max_agents, 3);
    }

    #[tokio::test]
    async fn independent_grants_recompute_without_cross_provider_revocation() {
        let store = EntitlementStore::memory();
        store
            .apply_grant(EntitlementGrant {
                source: "paddle".into(),
                grant_id: "sub-paddle".into(),
                user_id: "user-1".into(),
                tier: Tier::Pro,
                max_agents: Tier::Pro.max_agents(),
                cloud_enabled: false,
                active: true,
                valid_until: None,
            })
            .await
            .unwrap();
        store
            .apply_grant(EntitlementGrant {
                source: "dodo".into(),
                grant_id: "sub-dodo".into(),
                user_id: "user-1".into(),
                tier: Tier::Ultra,
                max_agents: Tier::Ultra.max_agents(),
                cloud_enabled: true,
                active: true,
                valid_until: None,
            })
            .await
            .unwrap();
        store
            .apply_grant(EntitlementGrant {
                source: "dodo".into(),
                grant_id: "sub-dodo".into(),
                user_id: "user-1".into(),
                tier: Tier::Ultra,
                max_agents: Tier::Ultra.max_agents(),
                cloud_enabled: true,
                active: false,
                valid_until: None,
            })
            .await
            .unwrap();

        let entitlement = store.get("user-1").await;
        assert_eq!(entitlement.tier, Tier::Pro);
        assert!(!entitlement.cloud_enabled);
    }

    #[tokio::test]
    async fn expired_grace_grant_is_not_returned_as_current_access() {
        let store = EntitlementStore::memory();
        store
            .apply_grant(EntitlementGrant {
                source: "dodo".into(),
                grant_id: "sub-grace".into(),
                user_id: "user-grace".into(),
                tier: Tier::Pro,
                max_agents: Tier::Pro.max_agents(),
                cloud_enabled: false,
                active: true,
                valid_until: Some(Utc::now() - chrono::Duration::seconds(1)),
            })
            .await
            .unwrap();

        assert_eq!(store.get("user-grace").await.tier, Tier::Core);
    }

    #[tokio::test]
    async fn explicit_admin_record_remains_an_override() {
        let store = EntitlementStore::memory();
        store
            .apply_grant(EntitlementGrant {
                source: "dodo".into(),
                grant_id: "sub-paid".into(),
                user_id: "user-override".into(),
                tier: Tier::Ultra,
                max_agents: Tier::Ultra.max_agents(),
                cloud_enabled: true,
                active: true,
                valid_until: None,
            })
            .await
            .unwrap();
        store
            .upsert(EntitlementRecord {
                user_id: "user-override".into(),
                clerk_user_id: None,
                org_id: None,
                tier: Tier::Core,
                max_agents: Tier::Core.max_agents(),
                cloud_enabled: false,
            })
            .await
            .unwrap();

        assert_eq!(store.get("user-override").await.tier, Tier::Core);
    }

    #[test]
    fn status_response_serializes_for_cli() {
        let record = EntitlementRecord {
            user_id: "u1".into(),
            clerk_user_id: None,
            org_id: None,
            tier: Tier::Pro,
            max_agents: 64,
            cloud_enabled: false,
        };
        let resp = EntitlementStatusResponse::from(record);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"tier\":\"pro\""));
        assert!(json.contains("\"max_agents\":64"));
    }
}
