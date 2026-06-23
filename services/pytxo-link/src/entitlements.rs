use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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
    inner: Arc<Mutex<HashMap<String, EntitlementRecord>>>,
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

    pub async fn set_tier(&self, user_id: &str, tier: Tier) -> Result<(), sqlx::Error> {
        let max_agents = tier.max_agents();
        let cloud_enabled = matches!(tier, Tier::Max | Tier::Ultra);
        self.upsert(EntitlementRecord {
            user_id: user_id.to_string(),
            clerk_user_id: None,
            org_id: None,
            tier,
            max_agents,
            cloud_enabled,
        })
        .await
    }
}

impl MemoryStore {
    pub fn get(&self, user_id: &str) -> EntitlementRecord {
        self.inner
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .unwrap_or_else(|| default_record(user_id))
    }

    pub fn upsert(&self, record: EntitlementRecord) {
        self.inner
            .lock()
            .unwrap()
            .insert(record.user_id.clone(), record);
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
        SELECT user_id, clerk_user_id, org_id, tier, max_agents, cloud_enabled
        FROM entitlements WHERE user_id = $1
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
    sqlx::query(
        r#"
        INSERT INTO entitlements (user_id, clerk_user_id, org_id, tier, max_agents, cloud_enabled, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        ON CONFLICT (user_id) DO UPDATE SET
            clerk_user_id = EXCLUDED.clerk_user_id,
            org_id = EXCLUDED.org_id,
            tier = EXCLUDED.tier,
            max_agents = EXCLUDED.max_agents,
            cloud_enabled = EXCLUDED.cloud_enabled,
            updated_at = now()
        "#,
    )
    .bind(&record.user_id)
    .bind(&record.clerk_user_id)
    .bind(&record.org_id)
    .bind(record.tier.as_str())
    .bind(record.max_agents as i32)
    .bind(record.cloud_enabled)
    .execute(pool)
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
