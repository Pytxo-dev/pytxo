use pytxo_core::{HttpBillingReconciler, PytxoConfig, PytxoError};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct EntitlementStatus {
    pub tier: String,
    pub max_agents: usize,
    pub cloud_enabled: bool,
}

/// Effective tier limits for the current config (local defaults + optional Link fetch).
pub fn effective_entitlements(cfg: &PytxoConfig) -> Result<EntitlementStatus, PytxoError> {
    let mut tier = if cfg.billing_mode().is_ultra() {
        "ultra".to_string()
    } else if cfg.billing.link_reconcile {
        "pro".to_string()
    } else {
        "core".to_string()
    };
    let mut max_agents = cfg.tier_max_agents;
    let mut cloud_enabled = false;

    if cfg.billing.link_reconcile && !cfg.billing.proxy_url.trim().is_empty() {
        if let Ok(remote) = fetch_link_entitlements(&cfg.billing.proxy_url) {
            tier = remote.tier;
            max_agents = remote.max_agents;
            cloud_enabled = remote.cloud_enabled;
        }
    }

    Ok(EntitlementStatus {
        tier,
        max_agents,
        cloud_enabled,
    })
}

#[derive(Deserialize)]
struct LinkEntitlementResponse {
    tier: String,
    max_agents: usize,
    #[serde(default)]
    cloud_enabled: bool,
}

fn fetch_link_entitlements(base_url: &str) -> Result<EntitlementStatus, PytxoError> {
    let reconciler = HttpBillingReconciler::new(base_url);
    let endpoint = reconciler.endpoint("v1/entitlements/status");
    reconciler.ping()?;

    #[cfg(feature = "link-http")]
    {
        let mut req = ureq::get(&endpoint);
        if let Ok(token) = std::env::var("PYTXO_ULTRA_SESSION") {
            if !token.is_empty() {
                req = req.set("Authorization", &format!("Bearer {token}"));
            }
        }
        let resp = req
            .call()
            .map_err(|e| PytxoError::Other(format!("entitlements fetch: {e}")))?;
        if resp.status() != 200 {
            return Err(PytxoError::Other(format!(
                "entitlements status {}",
                resp.status()
            )));
        }
        let body: LinkEntitlementResponse = resp
            .into_json()
            .map_err(|e| PytxoError::Other(format!("entitlements json: {e}")))?;
        return Ok(EntitlementStatus {
            tier: body.tier,
            max_agents: body.max_agents,
            cloud_enabled: body.cloud_enabled,
        });
    }

    #[cfg(not(feature = "link-http"))]
    {
        let _ = endpoint;
        Err(PytxoError::Other(
            "link-http feature required for remote entitlements".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_defaults_without_link() {
        let cfg = PytxoConfig::default();
        let ent = effective_entitlements(&cfg).unwrap();
        assert_eq!(ent.tier, "core");
        assert_eq!(ent.max_agents, 3);
        assert!(!ent.cloud_enabled);
    }
}
