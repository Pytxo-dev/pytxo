use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;

use crate::commerce::{
    CommerceProvider, CommerceStore, PlanKey, ReconcileOutcome, SubscriptionEvent,
    SubscriptionLifecycle,
};
use crate::entitlements::EntitlementStore;

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
    pub occurred_at: Option<DateTime<Utc>>,
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
    by_id: HashMap<String, PlanKey>,
}

impl PaddlePriceCatalog {
    pub fn from_env() -> Option<Self> {
        Self::new([
            (std::env::var("PADDLE_PRICE_PRO").ok(), PlanKey::Pro),
            (std::env::var("PADDLE_PRICE_MAX").ok(), PlanKey::Max),
            (std::env::var("PADDLE_PRICE_ULTRA").ok(), PlanKey::Ultra),
        ])
    }

    fn new(entries: [(Option<String>, PlanKey); 3]) -> Option<Self> {
        let mut by_id = HashMap::new();
        for (price_id, plan) in entries {
            let Some(price_id) = price_id.filter(|value| !value.trim().is_empty()) else {
                continue;
            };
            if by_id.insert(price_id, plan).is_some() {
                return None;
            }
        }
        (!by_id.is_empty()).then_some(Self { by_id })
    }

    fn plan_for_items(&self, items: Option<&[PaddleItem]>) -> Option<PlanKey> {
        let items = items.filter(|items| !items.is_empty())?;
        let mut selected = None;
        for item in items {
            let price_id = item.price.as_ref()?.id.as_deref()?;
            let plan = *self.by_id.get(price_id)?;
            if selected.is_some_and(|selected| selected != plan) {
                return None;
            }
            selected = Some(plan);
        }
        selected
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Self::new([
            (Some("pri-pro".into()), PlanKey::Pro),
            (Some("pri-max".into()), PlanKey::Max),
            (Some("pri-ultra".into()), PlanKey::Ultra),
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

fn verify_paddle_signature_at(
    raw_body: &str,
    signature_header: &str,
    secret: &str,
    now_unix: i64,
) -> bool {
    if signature_header.is_empty() || secret.is_empty() {
        return false;
    }
    let parts: HashMap<&str, &str> = signature_header
        .split(';')
        .filter_map(|part| {
            let mut split = part.splitn(2, '=');
            Some((split.next()?.trim(), split.next()?.trim()))
        })
        .collect();
    let (Some(ts), Some(h1)) = (parts.get("ts"), parts.get("h1")) else {
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
    let payload = format!("{ts}:{raw_body}");
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
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
    commerce: &CommerceStore,
    entitlements: &EntitlementStore,
    prices: &PaddlePriceCatalog,
    body: &PaddleWebhook,
) -> Result<ReconcileOutcome, sqlx::Error> {
    let Some(event_type) = body.event_type.as_deref() else {
        return Ok(ReconcileOutcome::Rejected);
    };
    if body.event_id.trim().is_empty() || !ACCEPTED_EVENT_TYPES.contains(&event_type) {
        return Ok(ReconcileOutcome::Rejected);
    }
    let Some(data) = body.data.as_ref() else {
        return Ok(ReconcileOutcome::Rejected);
    };
    let Some(subscription_id) = data.id.as_deref().filter(|value| !value.trim().is_empty()) else {
        return Ok(ReconcileOutcome::Rejected);
    };
    let Some(customer_id) = data
        .customer_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(ReconcileOutcome::Rejected);
    };
    let user_id = data
        .custom_data
        .as_ref()
        .and_then(|custom| custom.user_id.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let canceled = event_type == "subscription.canceled";
    let plan = if canceled {
        None
    } else {
        let Some(plan) = prices.plan_for_items(data.items.as_deref()) else {
            return Ok(ReconcileOutcome::Rejected);
        };
        Some(plan)
    };
    let event = SubscriptionEvent {
        provider: CommerceProvider::Paddle,
        event_id: body.event_id.clone(),
        event_type: event_type.to_string(),
        occurred_at: body.occurred_at.unwrap_or_else(Utc::now),
        subscription_id: subscription_id.to_string(),
        customer_id: customer_id.to_string(),
        establishes_binding: event_type == "subscription.created",
        user_id,
        plan,
        lifecycle: if canceled {
            SubscriptionLifecycle::Ended
        } else {
            SubscriptionLifecycle::Active
        },
        access_until: None,
    };

    let outcome = commerce.reconcile(entitlements, &event).await?;
    if outcome == ReconcileOutcome::Applied {
        info!(
            provider = "paddle",
            event_id = %body.event_id,
            subscription_id,
            lifecycle = ?event.lifecycle,
            "commerce entitlement reconciled"
        );
    }
    Ok(outcome)
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
