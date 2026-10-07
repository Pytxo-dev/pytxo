use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use base64::Engine;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::Value;
use sha2::Sha256;
use tracing::info;

use crate::commerce::{
    CommerceProvider, CommerceStore, PlanKey, ReconcileOutcome, SubscriptionEvent,
    SubscriptionLifecycle,
};
use crate::entitlements::EntitlementStore;

const DODO_SIGNATURE_TOLERANCE_SECS: i64 = 300;
const ACCEPTED_EVENT_TYPES: [&str; 12] = [
    "subscription.active",
    "subscription.renewed",
    "subscription.updated",
    "subscription.plan_changed",
    "subscription.update_payment_method",
    "subscription.past_due",
    "subscription.on_hold",
    "subscription.paused",
    "subscription.unpaused",
    "subscription.cancelled",
    "subscription.failed",
    "subscription.expired",
];

#[derive(Debug, Deserialize)]
pub struct DodoWebhook {
    business_id: String,
    timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    event_type: String,
    data: DodoSubscriptionData,
}

#[derive(Debug, Deserialize)]
struct DodoSubscriptionData {
    subscription_id: String,
    payload_type: String,
    brand_id: String,
    product_id: String,
    customer: DodoCustomer,
    #[serde(default)]
    metadata: HashMap<String, Value>,
    status: DodoSubscriptionStatus,
    next_billing_date: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    past_due_ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    cancel_at_next_billing_date: bool,
}

#[derive(Debug, Deserialize)]
struct DodoCustomer {
    customer_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DodoSubscriptionStatus {
    Pending,
    Active,
    OnHold,
    Paused,
    Cancelled,
    Failed,
    Expired,
    PastDue,
}

#[derive(Clone)]
pub struct DodoProductCatalog {
    business_id: String,
    brand_id: String,
    by_id: HashMap<String, PlanKey>,
}

impl DodoProductCatalog {
    pub fn from_env() -> Option<Self> {
        Self::new(
            std::env::var("DODO_BUSINESS_ID").ok(),
            std::env::var("DODO_BRAND_ID").ok(),
            [
                (std::env::var("DODO_PRODUCT_PRO").ok(), PlanKey::Pro),
                (std::env::var("DODO_PRODUCT_MAX").ok(), PlanKey::Max),
                (std::env::var("DODO_PRODUCT_ULTRA").ok(), PlanKey::Ultra),
            ],
        )
    }

    fn new(
        business_id: Option<String>,
        brand_id: Option<String>,
        entries: [(Option<String>, PlanKey); 3],
    ) -> Option<Self> {
        let business_id = nonempty(business_id.as_deref()?)?.to_string();
        let brand_id = nonempty(brand_id.as_deref()?)?.to_string();
        let mut by_id = HashMap::new();
        for (product_id, plan) in entries {
            let Some(product_id) = product_id.as_deref().and_then(nonempty) else {
                continue;
            };
            if by_id.insert(product_id.to_string(), plan).is_some() {
                return None;
            }
        }
        (!by_id.is_empty()).then_some(Self {
            business_id,
            brand_id,
            by_id,
        })
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Self::new(
            Some("business-test".into()),
            Some("brand-pytxo".into()),
            [
                (Some("product-pro".into()), PlanKey::Pro),
                (Some("product-max".into()), PlanKey::Max),
                (Some("product-ultra".into()), PlanKey::Ultra),
            ],
        )
        .unwrap()
    }
}

/// Verify a Dodo Standard Webhooks signature over the exact request bytes.
pub fn verify_dodo_signature(
    raw_body: &[u8],
    webhook_id: &str,
    webhook_timestamp: &str,
    webhook_signature: &str,
    secret: &str,
) -> bool {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return false;
    };
    verify_dodo_signature_at(
        raw_body,
        webhook_id,
        webhook_timestamp,
        webhook_signature,
        secret,
        duration.as_secs() as i64,
    )
}

fn verify_dodo_signature_at(
    raw_body: &[u8],
    webhook_id: &str,
    webhook_timestamp: &str,
    webhook_signature: &str,
    secret: &str,
    now_unix: i64,
) -> bool {
    if webhook_id.is_empty()
        || webhook_id.contains('.')
        || webhook_signature.is_empty()
        || !secret.starts_with("whsec_")
    {
        return false;
    }
    let Ok(signed_at) = webhook_timestamp.parse::<i64>() else {
        return false;
    };
    if signed_at < 0 || now_unix.abs_diff(signed_at) > DODO_SIGNATURE_TOLERANCE_SECS as u64 {
        return false;
    }
    let Some(key) = decode_base64(&secret["whsec_".len()..]) else {
        return false;
    };
    let mut signed = format!("{webhook_id}.{webhook_timestamp}.").into_bytes();
    signed.extend_from_slice(raw_body);

    webhook_signature
        .split_whitespace()
        .filter_map(|candidate| candidate.strip_prefix("v1,"))
        .filter_map(decode_base64)
        .any(|candidate| {
            let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(&key) else {
                return false;
            };
            mac.update(&signed);
            mac.verify_slice(&candidate).is_ok()
        })
}

fn decode_base64(value: &str) -> Option<Vec<u8>> {
    STANDARD
        .decode(value)
        .or_else(|_| STANDARD_NO_PAD.decode(value))
        .ok()
}

pub async fn handle_dodo_webhook(
    commerce: &CommerceStore,
    entitlements: &EntitlementStore,
    catalog: &DodoProductCatalog,
    event_id: &str,
    body: &DodoWebhook,
) -> Result<ReconcileOutcome, sqlx::Error> {
    let Some(event) = normalize(catalog, event_id, body) else {
        return Ok(ReconcileOutcome::Rejected);
    };
    let outcome = commerce.reconcile(entitlements, &event).await?;
    if outcome == ReconcileOutcome::Applied {
        info!(
            provider = "dodo",
            event_id = %event.event_id,
            subscription_id = %event.subscription_id,
            lifecycle = ?event.lifecycle,
            "commerce entitlement reconciled"
        );
    }
    Ok(outcome)
}

fn normalize(
    catalog: &DodoProductCatalog,
    event_id: &str,
    body: &DodoWebhook,
) -> Option<SubscriptionEvent> {
    if body.business_id != catalog.business_id
        || !ACCEPTED_EVENT_TYPES.contains(&body.event_type.as_str())
        || nonempty(body.data.subscription_id.as_str()).is_none()
        || body.data.payload_type != "Subscription"
        || body.data.brand_id != catalog.brand_id
    {
        return None;
    }
    let customer_id = nonempty(body.data.customer.customer_id.as_str())?;
    let product_id = nonempty(body.data.product_id.as_str())?;
    let lifecycle = lifecycle_for(body.data.status);
    let mapped_plan = catalog.by_id.get(product_id).copied();
    let plan = if matches!(
        lifecycle,
        SubscriptionLifecycle::Active | SubscriptionLifecycle::Grace
    ) {
        Some(mapped_plan?)
    } else {
        mapped_plan
    };
    let user_id = body
        .data
        .metadata
        .get("user_id")
        .and_then(Value::as_str)
        .and_then(nonempty)
        .map(str::to_string);
    let access_until = match lifecycle {
        SubscriptionLifecycle::Grace => body
            .data
            .past_due_ends_at
            .or(body.data.expires_at)
            .or(Some(body.data.next_billing_date)),
        SubscriptionLifecycle::Active if body.data.cancel_at_next_billing_date => {
            body.data.expires_at.or(Some(body.data.next_billing_date))
        }
        _ => None,
    };

    Some(SubscriptionEvent {
        provider: CommerceProvider::Dodo,
        event_id: nonempty(event_id)?.to_string(),
        event_type: body.event_type.clone(),
        occurred_at: body.timestamp,
        subscription_id: body.data.subscription_id.clone(),
        customer_id: customer_id.to_string(),
        establishes_binding: body.event_type == "subscription.active",
        user_id,
        plan,
        lifecycle,
        access_until,
    })
}

fn lifecycle_for(status: DodoSubscriptionStatus) -> SubscriptionLifecycle {
    match status {
        DodoSubscriptionStatus::Active => SubscriptionLifecycle::Active,
        DodoSubscriptionStatus::PastDue => SubscriptionLifecycle::Grace,
        DodoSubscriptionStatus::Pending
        | DodoSubscriptionStatus::OnHold
        | DodoSubscriptionStatus::Paused => SubscriptionLifecycle::Suspended,
        DodoSubscriptionStatus::Cancelled
        | DodoSubscriptionStatus::Failed
        | DodoSubscriptionStatus::Expired => SubscriptionLifecycle::Ended,
    }
}

fn nonempty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> String {
        format!("whsec_{}", STANDARD.encode(b"test-signing-key"))
    }

    fn signature(body: &[u8], id: &str, timestamp: i64, secret: &str) -> String {
        let key = decode_base64(secret.trim_start_matches("whsec_")).unwrap();
        let mut signed = format!("{id}.{timestamp}.").into_bytes();
        signed.extend_from_slice(body);
        let mut mac = Hmac::<Sha256>::new_from_slice(&key).unwrap();
        mac.update(&signed);
        format!("v1,{}", STANDARD.encode(mac.finalize().into_bytes()))
    }

    #[test]
    fn standard_webhook_signature_covers_exact_raw_bytes() {
        let body = br#"{"type":"subscription.active"}"#;
        let timestamp = 1_800_000_000;
        let secret = secret();
        let valid = signature(body, "msg_1", timestamp, &secret);

        assert!(verify_dodo_signature_at(
            body,
            "msg_1",
            "1800000000",
            &valid,
            &secret,
            timestamp
        ));
        assert!(!verify_dodo_signature_at(
            br#"{"type": "subscription.active"}"#,
            "msg_1",
            "1800000000",
            &valid,
            &secret,
            timestamp
        ));
    }

    #[test]
    fn standard_webhook_accepts_any_valid_rotating_signature() {
        let body = b"{}";
        let timestamp = 1_800_000_000;
        let secret = secret();
        let valid = signature(body, "msg_2", timestamp, &secret);
        let header = format!("v1,aW52YWxpZA== {valid}");

        assert!(verify_dodo_signature_at(
            body,
            "msg_2",
            "1800000000",
            &header,
            &secret,
            timestamp
        ));
    }

    #[test]
    fn stale_standard_webhook_is_rejected() {
        let body = b"{}";
        let secret = secret();
        let signature = signature(body, "msg_3", 1, &secret);
        assert!(!verify_dodo_signature_at(
            body, "msg_3", "1", &signature, &secret, 1_000
        ));
    }
}
