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
