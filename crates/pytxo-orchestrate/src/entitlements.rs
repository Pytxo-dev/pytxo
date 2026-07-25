use pytxo_core::{HttpBillingReconciler, PermissionProfile, PytxoConfig, PytxoError};
use serde::Deserialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const ENTITLEMENTS_CACHE_TTL: Duration = Duration::from_secs(300);

struct CachedEntitlements {
    value: EntitlementStatus,
    fetched_at: Instant,
}

static ENTITLEMENTS_CACHE: Mutex<Option<CachedEntitlements>> = Mutex::new(None);

/// Process-local session bearer (Desktop keyring hydrate / CLI env). Prefer this over
/// putting `PYTXO_ULTRA_SESSION` in the process environment where agent children inherit it.
static RUNTIME_SESSION_TOKEN: Mutex<Option<String>> = Mutex::new(None);

/// Drop cached Link entitlements so the next fetch reflects sign-in / sign-out.
pub fn invalidate_entitlements_cache() {
    if let Ok(mut guard) = ENTITLEMENTS_CACHE.lock() {
        *guard = None;
    }
}

/// Install or clear the in-process Ultra/Clerk session token (does not touch env).
pub fn set_runtime_session_token(token: Option<String>) {
    if let Ok(mut guard) = RUNTIME_SESSION_TOKEN.lock() {
        *guard = token.filter(|t| !t.is_empty());
    }
}

/// Session token for Link HTTP: runtime store first, then `PYTXO_ULTRA_SESSION` (CLI).
pub fn runtime_session_token() -> Option<String> {
    if let Ok(guard) = RUNTIME_SESSION_TOKEN.lock() {
        if let Some(t) = guard.as_ref() {
            if !t.is_empty() {
                return Some(t.clone());
            }
        }
    }
    std::env::var("PYTXO_ULTRA_SESSION")
        .ok()
        .filter(|t| !t.is_empty())
}

#[derive(Clone, Debug, Deserialize)]
pub struct EntitlementStatus {
    pub tier: String,
    pub max_agents: usize,
    pub cloud_enabled: bool,
    #[serde(default)]
    pub org_id: Option<String>,
    /// Org-wide permission profile ceiling from Link (when session present).
    #[serde(default)]
    pub permission_ceiling: Option<PermissionProfile>,
}

/// Effective tier limits for the current config (local defaults + optional Link fetch).
pub fn effective_entitlements(cfg: &PytxoConfig) -> Result<EntitlementStatus, PytxoError> {
    let proxy = cfg.billing.proxy_url.trim();
    // Fetch remote entitlements when Link reconcile is on (Ultra default) OR when a
    // Clerk/Desktop session token is present — so BYOK users who sign in still sync tier.
    let should_fetch_link =
        !proxy.is_empty() && (cfg.billing.link_reconcile_enabled() || ultra_session_present());

    if should_fetch_link {
        if let Ok(guard) = ENTITLEMENTS_CACHE.lock() {
            if let Some(cached) = guard.as_ref() {
                if cached.fetched_at.elapsed() < ENTITLEMENTS_CACHE_TTL {
                    return Ok(cached.value.clone());
                }
            }
        }
        match fetch_link_entitlements(proxy) {
            Ok(remote) => {
                if let Ok(mut guard) = ENTITLEMENTS_CACHE.lock() {
                    *guard = Some(CachedEntitlements {
                        value: remote.clone(),
                        fetched_at: Instant::now(),
                    });
                }
                return Ok(remote);
            }
            Err(e) if ultra_session_present() && !cfg.billing.link_reconcile_enabled() => {
                // Signed-in BYOK: fall through to local defaults if Link is unreachable.
                eprintln!("link entitlements (session present): {e}");
            }
            Err(e) => return Err(e),
        }
    }

    let tier = if cfg.billing_mode().is_ultra() {
        "ultra".to_string()
    } else if cfg.billing.link_reconcile_enabled() {
        "pro".to_string()
    } else {
        "core".to_string()
    };
    let max_agents = cfg.tier_max_agents;
    let cloud_enabled = false;
    let org_id = None;
    let mut permission_ceiling = None;

    if ultra_session_present() {
        if let Ok(ceiling) = fetch_org_policy_ceiling(cfg) {
            permission_ceiling = Some(ceiling);
        }
    }

    Ok(EntitlementStatus {
        tier,
        max_agents,
        cloud_enabled,
        org_id,
        permission_ceiling,
    })
}

/// Cap per-agent profiles against the org ceiling when present.
pub fn effective_agent_profiles(
    cfg: &PytxoConfig,
    entitlements: &EntitlementStatus,
) -> std::collections::HashMap<String, PermissionProfile> {
    cfg.agent_profile_map()
        .into_iter()
        .map(|(name, profile)| {
            let capped = entitlements
                .permission_ceiling
                .map(|ceiling| apply_permission_ceiling(profile, ceiling))
                .unwrap_or(profile);
            (name, capped)
        })
        .collect()
}

pub fn apply_permission_ceiling(
    profile: PermissionProfile,
    ceiling: PermissionProfile,
) -> PermissionProfile {
    if profile_rank(profile) > profile_rank(ceiling) {
        ceiling
    } else {
        profile
    }
}

/// Effective permission profile after optional Link org policy ceiling.
pub fn effective_permission_profile(
    cfg: &PytxoConfig,
    entitlements: &EntitlementStatus,
) -> PermissionProfile {
    let base = cfg.permission_profile;
    entitlements
        .permission_ceiling
        .map(|ceiling| apply_permission_ceiling(base, ceiling))
        .unwrap_or(base)
}

fn profile_rank(p: PermissionProfile) -> u8 {
    match p {
        PermissionProfile::DeepSpace => 0,
        PermissionProfile::Orbit => 1,
        PermissionProfile::Galaxy => 2,
        PermissionProfile::Supernova => 3,
    }
}

fn ultra_session_present() -> bool {
    runtime_session_token().is_some()
}

#[derive(Deserialize)]
struct LinkEntitlementResponse {
    tier: String,
    max_agents: usize,
    #[serde(default)]
    cloud_enabled: bool,
    #[serde(default)]
    org_id: Option<String>,
}

#[derive(Deserialize)]
struct OrgPolicyResponse {
    #[serde(default)]
    default_permission_profile: Option<String>,
}

fn fetch_link_entitlements(base_url: &str) -> Result<EntitlementStatus, PytxoError> {
    let reconciler = HttpBillingReconciler::new(base_url);
    let endpoint = reconciler.endpoint("v1/entitlements/status");
    reconciler.ping()?;

    #[cfg(feature = "link-http")]
    {
        let mut req = ureq::get(&endpoint);
        if let Some(token) = runtime_session_token() {
            req = req.set("Authorization", &format!("Bearer {token}"));
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

        let mut permission_ceiling = None;
        if ultra_session_present() {
            if let Some(ref org_id) = body.org_id {
                permission_ceiling = fetch_org_policy_ceiling_for(base_url, org_id).ok();
            }
        }

        std::env::set_var("PYTXO_LINK_TIER", &body.tier);

        Ok(EntitlementStatus {
            tier: body.tier,
            max_agents: body.max_agents,
            cloud_enabled: body.cloud_enabled,
            org_id: body.org_id,
            permission_ceiling,
        })
    }

    #[cfg(not(feature = "link-http"))]
    {
        let _ = endpoint;
        Err(PytxoError::Other(
            "link-http feature required for remote entitlements".into(),
        ))
    }
}

fn fetch_org_policy_ceiling(cfg: &PytxoConfig) -> Result<PermissionProfile, PytxoError> {
    let org_id = std::env::var("PYTXO_ORG_ID")
        .map_err(|_| PytxoError::Other("PYTXO_ORG_ID required for org policy fetch".into()))?;
    fetch_org_policy_ceiling_for(&cfg.billing.proxy_url, &org_id)
}

fn fetch_org_policy_ceiling_for(
    base_url: &str,
    org_id: &str,
) -> Result<PermissionProfile, PytxoError> {
    let reconciler = HttpBillingReconciler::new(base_url);
    let endpoint = reconciler.endpoint(&format!("v1/orgs/{org_id}/policy"));
    reconciler.ping()?;

    #[cfg(feature = "link-http")]
    {
        let mut req = ureq::get(&endpoint);
        if let Some(token) = runtime_session_token() {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }
        let resp = req
            .call()
            .map_err(|e| PytxoError::Other(format!("org policy fetch: {e}")))?;
        if resp.status() != 200 {
            return Err(PytxoError::Other(format!(
                "org policy status {}",
                resp.status()
            )));
        }
        let body: OrgPolicyResponse = resp
            .into_json()
            .map_err(|e| PytxoError::Other(format!("org policy json: {e}")))?;
        let profile_str = body.default_permission_profile.ok_or_else(|| {
            PytxoError::Other("org policy missing default_permission_profile".into())
        })?;
        PermissionProfile::parse(&profile_str).ok_or_else(|| {
            PytxoError::Other(format!("unknown org permission profile: {profile_str}"))
        })
    }

    #[cfg(not(feature = "link-http"))]
    {
        let _ = endpoint;
        Err(PytxoError::Other(
            "link-http feature required for org policy".into(),
        ))
    }
}

#[derive(Deserialize)]
struct LinkWalletResponse {
    balance_microcredits: i64,
}

/// Fetch Ultra wallet balance from Link (`GET /v1/wallet/balance`).
pub fn fetch_link_wallet_balance(base_url: &str) -> Result<i64, PytxoError> {
    let reconciler = HttpBillingReconciler::new(base_url);
    let endpoint = reconciler.endpoint("v1/wallet/balance");
    reconciler.ping()?;

    #[cfg(feature = "link-http")]
    {
        let mut req = ureq::get(&endpoint);
        if let Some(token) = runtime_session_token() {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }
        let resp = req
            .call()
            .map_err(|e| PytxoError::Other(format!("wallet fetch: {e}")))?;
        if resp.status() != 200 {
            return Err(PytxoError::Other(format!(
                "wallet balance status {}",
                resp.status()
            )));
        }
        let body: LinkWalletResponse = resp
            .into_json()
            .map_err(|e| PytxoError::Other(format!("wallet json: {e}")))?;
        Ok(body.balance_microcredits)
    }

    #[cfg(not(feature = "link-http"))]
    {
        let _ = endpoint;
        Err(PytxoError::Other(
            "link-http feature required for remote wallet".into(),
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

    #[test]
    fn ceiling_caps_supernova_to_galaxy() {
        assert_eq!(
            apply_permission_ceiling(PermissionProfile::Supernova, PermissionProfile::Galaxy),
            PermissionProfile::Galaxy
        );
    }

    #[test]
    fn ceiling_leaves_lower_profiles() {
        assert_eq!(
            apply_permission_ceiling(PermissionProfile::Orbit, PermissionProfile::Galaxy),
            PermissionProfile::Orbit
        );
    }

    #[test]
    fn ultra_config_enables_link_reconcile_by_default() {
        let mut cfg = PytxoConfig::default();
        cfg.billing.mode = pytxo_core::BillingMode::Ultra;
        assert!(cfg.billing.link_reconcile_enabled());
    }

    #[test]
    fn link_reconcile_with_unreachable_url_fails_loud() {
        let mut cfg = PytxoConfig::default();
        cfg.billing.mode = pytxo_core::BillingMode::Ultra;
        cfg.billing.proxy_url = "http://127.0.0.1:1".into();
        #[cfg(feature = "link-http")]
        {
            assert!(effective_entitlements(&cfg).is_err());
        }
        #[cfg(not(feature = "link-http"))]
        {
            let _ = cfg;
        }
    }
}
