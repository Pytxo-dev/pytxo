use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[cfg(test)]
use crate::entitlements::EntitlementRecord;
use crate::entitlements::{EntitlementStore, Tier};

const PADDLE_SIGNATURE_TOLERANCE_SECS: i64 = 300;
const ACCEPTED_EVENT_TYPES: [&str; 3] = [
    "subscription.created",
    "subscription.updated",
    "subscription.canceled",
];

#[derive(Debug, Deserialize)]
pub struct PaddleWebhook {
    pub event_id: String,
    pub event_type: Option<String>,
    pub data: Option<PaddleData>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleData {
    pub id: Option<String>,
    pub customer_id: Option<String>,
    pub items: Option<Vec<PaddleItem>>,
    pub custom_data: Option<PaddleCustomData>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleItem {
    pub price: Option<PaddlePrice>,
}

#[derive(Debug, Deserialize)]
pub struct PaddlePrice {
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleCustomData {
    pub user_id: Option<String>,
}

#[derive(Clone)]
pub struct PaddlePriceCatalog {
    by_id: HashMap<String, Tier>,
}

impl PaddlePriceCatalog {
    pub fn from_env() -> Option<Self> {
        Self::new([
            (std::env::var("PADDLE_PRICE_PRO").ok(), Tier::Pro),
            (std::env::var("PADDLE_PRICE_MAX").ok(), Tier::Max),
            (std::env::var("PADDLE_PRICE_ULTRA").ok(), Tier::Ultra),
        ])
    }

    fn new(entries: [(Option<String>, Tier); 3]) -> Option<Self> {
        let mut by_id = HashMap::new();
        for (price_id, tier) in entries {
            let Some(price_id) = price_id.filter(|value| !value.trim().is_empty()) else {
                continue;
            };
            if by_id.insert(price_id, tier).is_some() {
                return None;
            }
        }
        (!by_id.is_empty()).then_some(Self { by_id })
    }

    fn tier_for_items(&self, items: Option<&[PaddleItem]>) -> Option<Tier> {
        let items = items.filter(|items| !items.is_empty())?;
        let mut selected = None;
        for item in items {
            let price_id = item.price.as_ref()?.id.as_deref()?;
            let tier = *self.by_id.get(price_id)?;
            if selected.is_some_and(|selected| selected != tier) {
                return None;
            }
            selected = Some(tier);
        }
        selected
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Self::new([
            (Some("pri-pro".into()), Tier::Pro),
            (Some("pri-max".into()), Tier::Max),
            (Some("pri-ultra".into()), Tier::Ultra),
        ])
        .unwrap()
    }
}

/// Verify Paddle Billing webhook signature (`ts=...;h1=...` HMAC-SHA256).
pub fn verify_paddle_signature(raw_body: &str, signature_header: &str, secret: &str) -> bool {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return false;
    };
    verify_paddle_signature_at(
        raw_body,
        signature_header,
        secret,
        duration.as_secs() as i64,
    )
}

#[derive(Clone)]
pub enum PaddleEventStore {
    Postgres(PgPool),
    #[cfg(test)]
    Memory(Arc<Mutex<PaddleMemoryStore>>),
}

#[cfg(test)]
#[derive(Default)]
pub struct PaddleMemoryStore {
    events: HashSet<String>,
    subscriptions: HashMap<String, PaddleSubscriptionBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PaddleSubscriptionBinding {
    subscription_id: String,
    customer_id: String,
    user_id: String,
}

impl PaddleEventStore {
    pub fn postgres(pool: PgPool) -> Self {
        Self::Postgres(pool)
    }

    #[cfg(test)]
    pub fn memory() -> Self {
        Self::Memory(Arc::new(Mutex::new(PaddleMemoryStore::default())))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum PaddleWebhookOutcome {
    Applied,
    Duplicate,
    Rejected,
}

enum StoreOutcome {
    Applied(String),
    Duplicate,
    Rejected,
}

fn verify_paddle_signature_at(
    raw_body: &str,
    signature_header: &str,
    secret: &str,
    now_unix: i64,
) -> bool {
    if signature_header.is_empty() || secret.is_empty() {
        return false;
    }
    let parts: std::collections::HashMap<&str, &str> = signature_header
        .split(';')
        .filter_map(|p| {
            let mut split = p.splitn(2, '=');
            Some((split.next()?.trim(), split.next()?.trim()))
        })
        .collect();
    let Some(ts) = parts.get("ts") else {
        return false;
    };
    let Some(h1) = parts.get("h1") else {
        return false;
    };
    let Ok(signed_at) = ts.parse::<i64>() else {
        return false;
    };
    if signed_at < 0 || now_unix.abs_diff(signed_at) > PADDLE_SIGNATURE_TOLERANCE_SECS as u64 {
        return false;
    }
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let payload = format!("{ts}:{raw_body}");
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let expected = hex::encode(result.into_bytes());
    constant_time_eq(h1, &expected)
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

pub async fn handle_paddle_webhook(
    events: &PaddleEventStore,
    _entitlements: &EntitlementStore,
    prices: &PaddlePriceCatalog,
    body: &PaddleWebhook,
) -> Result<PaddleWebhookOutcome, sqlx::Error> {
    let Some(event_type) = body.event_type.as_deref() else {
        return Ok(PaddleWebhookOutcome::Rejected);
    };
    if body.event_id.trim().is_empty() || !ACCEPTED_EVENT_TYPES.contains(&event_type) {
        return Ok(PaddleWebhookOutcome::Rejected);
    }
    let Some(data) = body.data.as_ref() else {
        return Ok(PaddleWebhookOutcome::Rejected);
    };
    let Some(subscription_id) = data.id.as_deref().filter(|value| !value.trim().is_empty()) else {
        return Ok(PaddleWebhookOutcome::Rejected);
    };
    let Some(customer_id) = data
        .customer_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(PaddleWebhookOutcome::Rejected);
    };
    let custom = data.custom_data.as_ref();
    let claimed_user_id = custom
        .and_then(|custom| custom.user_id.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if event_type == "subscription.created" && claimed_user_id.is_none() {
        return Ok(PaddleWebhookOutcome::Rejected);
    }
    let tier = if event_type == "subscription.canceled" {
        Tier::Core
    } else {
        let Some(tier) = prices.tier_for_items(data.items.as_deref()) else {
            return Ok(PaddleWebhookOutcome::Rejected);
        };
        tier
    };

    let store_outcome = match events {
        PaddleEventStore::Postgres(pool) => {
            apply_postgres_event(
                pool,
                &body.event_id,
                event_type,
                subscription_id,
                customer_id,
                claimed_user_id,
                tier,
            )
            .await?
        }
        #[cfg(test)]
        PaddleEventStore::Memory(store) => {
            let (user_id, inserted_binding) = {
                let mut store = store.lock().unwrap();
                if store.events.contains(&body.event_id) {
                    return Ok(PaddleWebhookOutcome::Duplicate);
                }
                let binding = match event_type {
                    "subscription.created" => {
                        let claimed_user_id = claimed_user_id.expect("validated above");
                        let proposed = PaddleSubscriptionBinding {
                            subscription_id: subscription_id.to_string(),
                            customer_id: customer_id.to_string(),
                            user_id: claimed_user_id.to_string(),
                        };
                        if let Some(existing) = store.subscriptions.get(subscription_id) {
                            if existing != &proposed {
                                return Ok(PaddleWebhookOutcome::Rejected);
                            }
                            (existing.clone(), false)
                        } else {
                            store
                                .subscriptions
                                .insert(subscription_id.to_string(), proposed.clone());
                            (proposed, true)
                        }
                    }
                    _ => {
                        let Some(existing) = store.subscriptions.get(subscription_id).cloned()
                        else {
                            return Ok(PaddleWebhookOutcome::Rejected);
                        };
                        if existing.customer_id != customer_id
                            || claimed_user_id.is_some_and(|user_id| user_id != existing.user_id)
                        {
                            return Ok(PaddleWebhookOutcome::Rejected);
                        }
                        (existing, false)
                    }
                };
                store.events.insert(body.event_id.clone());
                (binding.0.user_id, binding.1)
            };
            let record = EntitlementRecord {
                user_id: user_id.clone(),
                clerk_user_id: None,
                org_id: None,
                tier,
                max_agents: tier.max_agents(),
                cloud_enabled: matches!(tier, Tier::Max | Tier::Ultra),
            };
            if let Err(error) = _entitlements.upsert(record).await {
                let mut store = store.lock().unwrap();
                store.events.remove(&body.event_id);
                if inserted_binding {
                    store.subscriptions.remove(subscription_id);
                }
                return Err(error);
            }
            StoreOutcome::Applied(user_id)
        }
    };

    let user_id = match store_outcome {
        StoreOutcome::Applied(user_id) => user_id,
        StoreOutcome::Duplicate => return Ok(PaddleWebhookOutcome::Duplicate),
        StoreOutcome::Rejected => return Ok(PaddleWebhookOutcome::Rejected),
    };

    if event_type == "subscription.canceled" {
        info!(user_id = %user_id, event_id = %body.event_id, "paddle: downgraded to core");
    } else {
        info!(user_id = %user_id, event_id = %body.event_id, tier = tier.as_str(), "paddle: entitlement updated");
    }
    Ok(PaddleWebhookOutcome::Applied)
}

async fn apply_postgres_event(
    pool: &PgPool,
    event_id: &str,
    event_type: &str,
    subscription_id: &str,
    customer_id: &str,
    claimed_user_id: Option<&str>,
    tier: Tier,
) -> Result<StoreOutcome, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let inserted = sqlx::query(
        r#"
        INSERT INTO paddle_webhook_events (event_id, event_type)
        VALUES ($1, $2)
        ON CONFLICT (event_id) DO NOTHING
        "#,
    )
    .bind(event_id)
    .bind(event_type)
    .execute(&mut *transaction)
    .await?
    .rows_affected();
    if inserted == 0 {
        transaction.rollback().await?;
        return Ok(StoreOutcome::Duplicate);
    }

    let binding = if event_type == "subscription.created" {
        let claimed_user_id = claimed_user_id.expect("validated before transaction");
        sqlx::query(
            r#"
            INSERT INTO paddle_subscriptions (subscription_id, customer_id, user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (subscription_id) DO NOTHING
            "#,
        )
        .bind(subscription_id)
        .bind(customer_id)
        .bind(claimed_user_id)
        .execute(&mut *transaction)
        .await?;
        fetch_binding(&mut transaction, subscription_id).await?
    } else {
        fetch_binding(&mut transaction, subscription_id).await?
    };

    let Some(binding) = binding else {
        transaction.rollback().await?;
        return Ok(StoreOutcome::Rejected);
    };
    if binding.customer_id != customer_id
        || claimed_user_id.is_some_and(|user_id| user_id != binding.user_id)
    {
        transaction.rollback().await?;
        return Ok(StoreOutcome::Rejected);
    }

    sqlx::query("UPDATE paddle_subscriptions SET updated_at = now() WHERE subscription_id = $1")
        .bind(subscription_id)
        .execute(&mut *transaction)
        .await?;
    upsert_entitlement(&mut transaction, &binding.user_id, tier).await?;
    transaction.commit().await?;
    Ok(StoreOutcome::Applied(binding.user_id))
}

async fn fetch_binding(
    transaction: &mut Transaction<'_, Postgres>,
    subscription_id: &str,
) -> Result<Option<PaddleSubscriptionBinding>, sqlx::Error> {
    sqlx::query_as::<_, PaddleSubscriptionRow>(
        r#"
        SELECT subscription_id, customer_id, user_id
        FROM paddle_subscriptions
        WHERE subscription_id = $1
        FOR UPDATE
        "#,
    )
    .bind(subscription_id)
    .fetch_optional(&mut **transaction)
    .await
    .map(|row| row.map(Into::into))
}

#[derive(sqlx::FromRow)]
struct PaddleSubscriptionRow {
    subscription_id: String,
    customer_id: String,
    user_id: String,
}

impl From<PaddleSubscriptionRow> for PaddleSubscriptionBinding {
    fn from(row: PaddleSubscriptionRow) -> Self {
        Self {
            subscription_id: row.subscription_id,
            customer_id: row.customer_id,
            user_id: row.user_id,
        }
    }
}

async fn upsert_entitlement(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: &str,
    tier: Tier,
) -> Result<(), sqlx::Error> {
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
    .bind(tier.max_agents() as i32)
    .bind(matches!(tier, Tier::Max | Tier::Ultra))
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    fn signature(body: &str, secret: &str, timestamp: i64) -> String {
        let payload = format!("{timestamp}:{body}");
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        format!(
            "ts={timestamp};h1={}",
            hex::encode(mac.finalize().into_bytes())
        )
    }

    #[test]
    fn stale_signature_is_rejected() {
        let body = r#"{"event_id":"evt-stale"}"#;
        let secret = "webhook-secret";
        let header = signature(body, secret, 1);

        assert!(!verify_paddle_signature(body, &header, secret));
    }
}
