use serde::Deserialize;
use tracing::info;

use crate::entitlements::{EntitlementStore, Tier};

#[derive(Debug, Deserialize)]
pub struct PaddleWebhook {
    pub event_type: Option<String>,
    pub data: Option<PaddleData>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleData {
    pub custom_data: Option<PaddleCustomData>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleCustomData {
    pub user_id: Option<String>,
    pub tier: Option<String>,
}

/// Verify Paddle Billing webhook signature (`ts=...;h1=...` HMAC-SHA256).
pub fn verify_paddle_signature(raw_body: &str, signature_header: &str, secret: &str) -> bool {
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

pub async fn handle_paddle_webhook(store: &EntitlementStore, body: &PaddleWebhook) -> bool {
    let Some(data) = body.data.as_ref() else {
        return false;
    };
    let custom = data.custom_data.as_ref();
    let user_id = custom
        .and_then(|c| c.user_id.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let tier_str = custom
        .and_then(|c| c.tier.as_deref())
        .unwrap_or("core")
        .to_ascii_lowercase();

    let tier = match tier_str.as_str() {
        "pro" | "pro_cloud" => Tier::Pro,
        "max" | "max_swarm" => Tier::Max,
        "ultra" => Tier::Ultra,
        _ => Tier::Core,
    };

    if body.event_type.as_deref() == Some("subscription.canceled") {
        if store.set_tier(&user_id, Tier::Core).await.is_err() {
            return false;
        }
        info!(user_id = %user_id, "paddle: downgraded to core");
        return true;
    }

    if store.set_tier(&user_id, tier).await.is_err() {
        return false;
    }
    info!(user_id = %user_id, tier = tier.as_str(), "paddle: entitlement updated");
    true
}
